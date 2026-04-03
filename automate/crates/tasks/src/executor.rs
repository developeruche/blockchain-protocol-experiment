use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use chrono::Utc;
use primitives::contract::AutomateContract;
use primitives::shared::{
    AutomationError, ErrorBus, ExecutionRecord, ExecutionStatus, Metrics, PersistenceStore,
    SharedState, TaskMessage,
};
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{error, info, info_span, warn, Instrument};

#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub max_fee_per_gas: u128,
    pub max_priority_fee: u128,
    pub gas_limit: u64,
    pub max_retries: u32,
    pub base_backoff_ms: u64,
}

pub struct NonceManager {
    current: Option<u64>,
}

impl std::fmt::Debug for NonceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonceManager")
            .field("current", &self.current)
            .finish()
    }
}

impl NonceManager {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub async fn next<T, P>(&mut self, provider: &P, address: Address) -> Result<u64, AutomationError>
    where
        T: alloy::transports::Transport + Clone,
        P: Provider<T, alloy::network::Ethereum>,
    {
        if let Some(nonce) = self.current {
            self.current = Some(nonce + 1);
            return Ok(nonce);
        }

        let nonce = provider
            .get_transaction_count(address)
            .await
            .map_err(|e| AutomationError::Provider(e.to_string()))?;
        self.current = Some(nonce + 1);
        Ok(nonce)
    }

    pub fn reset(&mut self) {
        self.current = None;
    }
}

pub struct ExecutorTask<P> {
    pub automation_id: String,
    pub rx: mpsc::Receiver<TaskMessage>,
    pub contract: AutomateContract<P>,
    pub nonce_manager: Arc<Mutex<NonceManager>>,
    pub config: ExecutorConfig,
    pub sender_address: Address,
    pub error_bus: ErrorBus,
    pub shared_state: SharedState,
    pub shutdown: watch::Receiver<bool>,
    pub metrics: Arc<Metrics>,
    pub persistence: Arc<dyn PersistenceStore>,
}

impl<P: Clone + Send + Sync + 'static> ExecutorTask<P> {
    pub fn spawn<T>(self) -> tokio::task::JoinHandle<()>
    where
        T: alloy::transports::Transport + Clone + Send + Sync + 'static,
        P: Provider<T, alloy::network::Ethereum>,
    {
        let id = self.automation_id.clone();
        tokio::spawn(
            async move {
                if let Err(e) = self.run::<T>().await {
                    error!("Executor task failed: {:?}", e);
                }
            }
            .instrument(info_span!("executor", automation_id = %id)),
        )
    }

    async fn run<T>(mut self) -> Result<(), AutomationError>
    where
        T: alloy::transports::Transport + Clone + Send + Sync + 'static,
        P: Provider<T, alloy::network::Ethereum>,
    {
        info!("Starting executor task");

        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    self.submit_with_retry::<T>(msg).await;
                }
                _ = self.shutdown.changed() => {
                    info!("Shutdown signal received, stopping executor.");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn submit_with_retry<T>(&mut self, msg: TaskMessage)
    where
        T: alloy::transports::Transport + Clone + Send + Sync + 'static,
        P: Provider<T, alloy::network::Ethereum>,
    {
        let mut attempt = 0;
        let start_time = Utc::now();
        
        loop {
            match self.try_submit::<T>(&msg).await {
                Ok(hash) => {
                    info!("Execution successful: tx {}", hash);
                    self.metrics.executions_total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    
                    let mut state = self.shared_state.write().await;
                    state.execution_count += 1;
                    
                    let record = ExecutionRecord {
                        automation_id: self.automation_id.clone(),
                        tx_hash: hash.to_string(),
                        status: ExecutionStatus::Success,
                        triggered_at: start_time, 
                        executed_at: Utc::now(),
                        gas_used: None, 
                    };
                    let _ = self.persistence.record_execution(record).await;
                    break;
                }
                Err(e) => {
                    warn!("Execution attempt {} failed: {}", attempt + 1, e);
                    if attempt < self.config.max_retries {
                        let delay = self.config.base_backoff_ms * 2u64.pow(attempt);
                        sleep(Duration::from_millis(delay)).await;
                        attempt += 1;
                        
                        let err_str = e.to_string();
                        if err_str.contains("nonce too low") || err_str.contains("nonce") {
                            let mut nm = self.nonce_manager.lock().await;
                            nm.reset();
                        }
                    } else {
                        error!("Execution failed after {} attempts: {}", attempt + 1, e);
                        self.metrics.errors_total.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let _ = self.error_bus.send(AutomationError::ExecutionFailed {
                            automation_id: self.automation_id.clone(),
                            message: e.to_string(),
                        });
                        
                        let record = ExecutionRecord {
                            automation_id: self.automation_id.clone(),
                            tx_hash: "".to_string(),
                            status: ExecutionStatus::Failed,
                            triggered_at: start_time,
                            executed_at: Utc::now(),
                            gas_used: None, 
                        };
                        let _ = self.persistence.record_execution(record).await;
                        break;
                    }
                }
            }
        }
    }

    async fn try_submit<T>(&self, _msg: &TaskMessage) -> Result<B256, AutomationError>
    where
        T: alloy::transports::Transport + Clone + Send + Sync + 'static,
        P: Provider<T, alloy::network::Ethereum>,
    {
        let job_id: B256 = self.automation_id.parse().unwrap_or_else(|_| {
            error!("Failed to parse automation_id {} as B256 hex string!", self.automation_id);
            B256::ZERO
        });
        
        info!("Executing JOB_ID natively: {}", job_id);
        
        let contract_instance = primitives::contract::AutomateRegistry::new(self.contract.address, self.contract.http_provider.clone());

        // 1. Estimate gas implicitly or request builder
        let call_builder = contract_instance.execute(job_id)
            .gas(self.config.gas_limit)
            .max_fee_per_gas(self.config.max_fee_per_gas)
            .max_priority_fee_per_gas(self.config.max_priority_fee);

        // 2. Nonce
        let mut nm = self.nonce_manager.lock().await;
        // need to extract the underlying Provider via &*
        let provider = &self.contract.http_provider;
        let nonce = nm.next(provider, self.sender_address).await?;
        drop(nm); // Explicitly drop to unblock other tasks

        // 3. Set nonce
        let call_builder = call_builder.nonce(nonce);
            
        // 4. Send
        let pending_tx = call_builder.send().await
            .map_err(|e| AutomationError::ExecutionFailed {
                 automation_id: self.automation_id.clone(),
                 message: e.to_string(),
            })?;
            
        // 5. Wait for receipt
        let receipt = pending_tx.get_receipt().await
             .map_err(|e| AutomationError::ExecutionFailed {
                 automation_id: self.automation_id.clone(),
                 message: e.to_string(),
             })?;
             
        if !receipt.status() {
            return Err(AutomationError::ExecutionFailed {
                automation_id: self.automation_id.clone(),
                message: "Transaction reverted".to_string(),
            });
        }

        Ok(receipt.transaction_hash)
    }
}
