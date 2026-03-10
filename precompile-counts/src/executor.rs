use alloy::primitives::B256;
use alloy::providers::Provider;
use revm::{Context, MainBuilder, MainContext, ExecuteEvm, ExecuteCommitEvm};
use revm::context_interface::result::ExecutionResult;
use revm::context::TxEnv;

use crate::fork_db::ForkDb;
use crate::block_env::BlockEnvironment;

pub struct Executor<P> {
    pub db: ForkDb<P>,
    pub block_env: BlockEnvironment,
    pub chain_id: u64,
}

impl<P: Provider> Executor<P> {
    pub fn new(db: ForkDb<P>, block_env: BlockEnvironment, chain_id: u64) -> Self {
        Self { db, block_env, chain_id }
    }

    pub fn call(&mut self, tx: TxEnv) -> eyre::Result<ExecutionResult> {
        let ctx = Context::mainnet()
            .with_db(&mut self.db)
            .modify_cfg_chained(|cfg| {
                cfg.chain_id = self.chain_id;
                cfg.disable_base_fee = true;
                cfg.disable_block_gas_limit = true;
                cfg.disable_balance_check = true;
            })
            .modify_block_chained(|b| *b = self.block_env.inner.clone());

        let mut evm = ctx.build_mainnet();
        
        let result = evm.transact(tx).map_err(|e| eyre::eyre!("EVM error: {:?}", e))?;
        Ok(result.result)
    }

    pub fn send_transaction(&mut self, tx: TxEnv) -> eyre::Result<(ExecutionResult, B256)> {
        let ctx = Context::mainnet()
            .with_db(&mut self.db)
            .modify_cfg_chained(|cfg| cfg.chain_id = self.chain_id)
            .modify_block_chained(|b| *b = self.block_env.inner.clone());

        let mut evm = ctx.build_mainnet();
        
        let result = evm.transact_commit(tx).map_err(|e| eyre::eyre!("EVM error: {:?}", e))?;
        
        let _new_block = self.block_env.increment_block();

        let tx_hash = B256::ZERO; 
        
        Ok((result, tx_hash))
    }
}
