//! This module provides an implementation of the Ethereum Virtual Machine (EVM) for processing transactions.
use crate::PodTransactionProcessor;
use pod_core::primitives::{errors::PodError, pod::PodTransaction};
use revm::{
    ExecuteCommitEvm, MainBuilder, MainContext,
    context::{BlockEnv, CfgEnv, Context, Evm, TxEnv},
    database::{CacheDB, EmptyDBTyped},
    database_interface::EmptyDB,
    handler::{EthFrame, EthPrecompiles, instructions::EthInstructions},
    interpreter::interpreter::EthInterpreter,
};
use std::convert::Infallible;

pub type PodEvm = Evm<
    Context<BlockEnv, TxEnv, CfgEnv, CacheDB<EmptyDBTyped<Infallible>>>,
    (),
    EthInstructions<
        EthInterpreter,
        Context<BlockEnv, TxEnv, CfgEnv, CacheDB<EmptyDBTyped<Infallible>>>,
    >,
    EthPrecompiles,
    EthFrame,
>;

#[derive(Debug)]
pub struct EvmTransactionProcessor {
    pub evm: PodEvm,
}

impl EvmTransactionProcessor {
    pub fn new() -> Self {
        let ctx = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());
        Self {
            evm: ctx.build_mainnet(),
        }
    }
}

impl PodTransactionProcessor for EvmTransactionProcessor {
    fn process_transaction(&mut self, transaction: &PodTransaction) -> Result<(), PodError> {
        self.evm
            .transact_commit(transaction.to_evm_tx())
            .map_err(|e| PodError::TransactionProcessingFailed(e.to_string()))?;

        Ok(())
    }
}
