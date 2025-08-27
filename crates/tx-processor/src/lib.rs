//! This library provides a set of tools for processing transactions in a blockchain network.
use pod_core::primitives::{errors::PodError, pod::PodTransaction};

pub mod evm;

pub trait PodTransactionProcessor {
    /// Processes a transaction and returns a result indicating success or failure.
    fn process_transaction(&mut self, transaction: &PodTransaction) -> Result<(), PodError>;
}
