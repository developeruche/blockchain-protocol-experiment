#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PodError {
    InvalidTransaction,
    InvalidSignature,
    TransactionProcessingFailed(String),
}
