use alloy::primitives::Bytes;
use alloy::providers::RootProvider;
use alloy::transports::BoxTransport;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Default)]
pub struct State {
    pub running: bool,
    pub execution_count: u64,
}

pub type SharedState = Arc<RwLock<State>>;

pub struct ProviderPool {
    pub http: RootProvider<BoxTransport>,
    pub ws: RootProvider<BoxTransport>,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum AutomationError {
    #[error("trigger failed for {automation_id}: {message}")]
    TriggerFailed {
        automation_id: String,
        message: String,
    },
    #[error("execution failed for {automation_id}: {message}")]
    ExecutionFailed {
        automation_id: String,
        message: String,
    },
    #[error("provider error: {0}")]
    Provider(String),
}

pub type ErrorBus = broadcast::Sender<AutomationError>;

#[derive(Debug, Clone)]
pub struct TaskMessage {
    pub automation_id: String,
    pub calldata: Bytes,
    pub triggered_at: std::time::Instant,
}

use std::sync::atomic::AtomicU64;

#[derive(Debug, Default)]
pub struct Metrics {
    pub executions_total: AtomicU64,
    pub trigger_fires_total: AtomicU64,
    pub errors_total: AtomicU64,
}

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Reverted,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub automation_id: String,
    pub tx_hash: String,
    pub status: ExecutionStatus,
    pub triggered_at: DateTime<Utc>,
    pub executed_at: DateTime<Utc>,
    pub gas_used: Option<u64>,
}

#[async_trait::async_trait]
pub trait PersistenceStore: Send + Sync {
    async fn record_execution(&self, record: ExecutionRecord) -> Result<(), anyhow::Error>;
}

pub struct NoOpPersistence;

#[async_trait::async_trait]
impl PersistenceStore for NoOpPersistence {
    async fn record_execution(&self, _record: ExecutionRecord) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
