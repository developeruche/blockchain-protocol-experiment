use alloy::primitives::B256;
use alloy::providers::Provider;
use revm::{Context, MainBuilder, MainContext, ExecuteEvm, ExecuteCommitEvm, InspectCommitEvm};
use revm::context_interface::result::ExecutionResult;
use revm::context::TxEnv;

use crate::fork_db::ForkDb;
use crate::block_env::BlockEnvironment;

pub struct Executor<P> {
    pub db: ForkDb<P>,
    pub block_env: BlockEnvironment,
    pub chain_id: u64,
    pub inspector: crate::inspector::PrecompileCounter,
}

impl<P: Provider> Executor<P> {
    pub fn new(db: ForkDb<P>, block_env: BlockEnvironment, chain_id: u64) -> Self {
        Self { db, block_env, chain_id, inspector: Default::default() }
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

    pub fn send_transaction_batch(&mut self, tx: TxEnv) -> eyre::Result<(ExecutionResult, B256)> {
        let ctx = Context::mainnet()
            .with_db(&mut self.db)
            .modify_cfg_chained(|cfg| {
                cfg.chain_id = self.chain_id;
                cfg.disable_base_fee = true;
                cfg.disable_block_gas_limit = true;
                cfg.disable_balance_check = true;
            })
            .modify_block_chained(|b| *b = self.block_env.inner.clone());

        let mut evm = ctx.build_mainnet_with_inspector(&mut self.inspector);
        
        let result = evm.transact_commit(tx).map_err(|e| eyre::eyre!("EVM error: {:?}", e))?;
        
        let tx_hash = B256::ZERO; 
        
        Ok((result, tx_hash))
    }

    pub fn execute_alloy_transaction(&mut self, tx: alloy::rpc::types::Transaction) -> eyre::Result<ExecutionResult> {
        use alloy::consensus::Transaction as _;

        let mut env = TxEnv::default();
        let caller = tx.inner.signer();
        env.caller = caller;

        // Force nonce to match transaction to bypass caching lag
        use revm::Database;
        let mut acct_info = self.db.basic(caller).unwrap_or_default().unwrap_or_default();
        acct_info.nonce = tx.nonce();
        acct_info.balance = alloy::primitives::U256::MAX;
        self.db.insert_account_info(caller, acct_info);
        
        env.gas_limit = tx.gas_limit();
        env.gas_price = tx.max_fee_per_gas(); // Use max_fee_per_gas directly to support EIP-1559 flawlessly
        env.gas_priority_fee = tx.max_priority_fee_per_gas();
        
        env.kind = tx.kind();

        env.value = tx.value();
        env.data = tx.input().clone();
        env.nonce = tx.nonce();
        env.chain_id = tx.chain_id();

        env.access_list = tx.access_list().cloned().unwrap_or_default().into();

        if let Some(blob_hashes) = tx.blob_versioned_hashes() {
            env.blob_hashes = blob_hashes.to_vec();
        }
        if let Some(fee) = tx.max_fee_per_blob_gas() {
            env.max_fee_per_blob_gas = fee;
        }

        let (result, _) = self.send_transaction_batch(env)?;
        Ok(result)
    }
}
