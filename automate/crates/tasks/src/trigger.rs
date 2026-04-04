use alloy::primitives::{Address, Bytes, B256};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use primitives::config::{TriggerStrategy, TopicFilter};
use futures_util::stream::StreamExt;
use primitives::shared::{AutomationError, ErrorBus, Metrics, ProviderPool, SharedState, TaskMessage};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use tokio::time::{sleep, Duration};
use std::time::Instant;
use tracing::{error, info, warn, info_span, Instrument};

pub struct TriggerTask {
    pub automation_id: String,
    pub strategy: TriggerStrategy,
    pub calldata: Bytes,
    pub tx: mpsc::Sender<TaskMessage>,
    pub error_bus: ErrorBus,
    pub shared_state: SharedState,
    pub shutdown: watch::Receiver<bool>,
    pub provider_pool: Arc<ProviderPool>,
    pub metrics: Arc<Metrics>,
}

impl TriggerTask {
    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        let id = self.automation_id.clone();
        tokio::spawn(
            async move {
                if let Err(e) = self.run().await {
                    error!("Trigger task failed: {:?}", e);
                }
            }
            .instrument(info_span!("trigger", automation_id = %id)),
        )
    }

    async fn run(mut self) -> Result<(), AutomationError> {
        info!("Starting trigger task");

        match self.strategy.clone() {
            TriggerStrategy::Timeout { duration } => self.run_timeout(duration).await,
            TriggerStrategy::Interval { period } => self.run_interval(period).await,
            TriggerStrategy::EventLog {
                contract,
                event_sig,
                topic_filters,
            } => self.run_event_log(contract, event_sig, topic_filters).await,
        }
    }

    async fn run_timeout(&mut self, duration: Duration) -> Result<(), AutomationError> {
        tokio::select! {
            _ = sleep(duration) => {
                self.fire_trigger().await?;
            }
            _ = self.shutdown.changed() => {
                info!("Shutdown signal received, canceling timeout trigger.");
            }
        }
        Ok(())
    }

    async fn run_interval(&mut self, period: Duration) -> Result<(), AutomationError> {
        let mut interval = tokio::time::interval(period);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.fire_trigger().await?;
                }
                _ = self.shutdown.changed() => {
                    info!("Shutdown signal received, stopping interval trigger.");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn run_event_log(
        &mut self,
        contract: Address,
        event_sig: B256,
        topic_filters: Vec<TopicFilter>,
    ) -> Result<(), AutomationError> {
        let mut filter = Filter::new()
            .address(contract)
            .event_signature(event_sig);

        for tf in &topic_filters {
            match tf.position {
                1 => filter = filter.event_signature(event_sig).topic1(tf.value),
                2 => filter = filter.event_signature(event_sig).topic2(tf.value),
                3 => filter = filter.event_signature(event_sig).topic3(tf.value),
                _ => {}
            }
        }

        loop {
            let sub_result = self.provider_pool.ws.subscribe_logs(&filter).await;
            match sub_result {
                Ok(sub) => {
                    info!("Successfully subscribed to event logs");
                    let mut stream = sub.into_stream();

                    loop {
                        tokio::select! {
                            Some(log) = stream.next() => {
                                if self.topic_filters_match(&log, &topic_filters) {
                                    if let Err(e) = self.fire_trigger().await {
                                        warn!("Failed to fire trigger: {}", e);
                                    }
                                }
                            }
                            _ = self.shutdown.changed() => {
                                info!("Shutdown signal received, stopping event log trigger.");
                                return Ok(());
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to subscribe to logs, retrying in 5s: {}", e);
                    tokio::select! {
                        _ = sleep(Duration::from_secs(5)) => {}
                        _ = self.shutdown.changed() => return Ok(())
                    }
                }
            }
        }
    }

    fn topic_filters_match(&self, log: &Log, filters: &[TopicFilter]) -> bool {
        let topics = log.topics();
        for filter in filters {
            if topics.len() <= filter.position as usize {
                return false;
            }
            if topics[filter.position as usize] != filter.value {
                return false;
            }
        }
        true
    }

    async fn fire_trigger(&self) -> Result<(), AutomationError> {
        let msg = TaskMessage {
            automation_id: self.automation_id.clone(),
            calldata: self.calldata.clone(),
            triggered_at: Instant::now(),
        };

        if let Err(e) = self.tx.send(msg).await {
            let err = AutomationError::TriggerFailed {
                automation_id: self.automation_id.clone(),
                message: format!("Channel send error: {}", e),
            };
            let _ = self.error_bus.send(err.clone());
            return Err(err);
        }
        
        info!("Trigger fired");
        self.metrics
            .trigger_fires_total
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}
