use alloy::primitives::U256;
use alloy::providers::Provider;
use revm::context::BlockEnv;
use std::sync::Arc;

use crate::provider::UpstreamProvider;

pub struct BlockEnvironment {
    pub inner: BlockEnv,
}

impl BlockEnvironment {
    pub async fn new<P: Provider>(provider: Arc<UpstreamProvider<P>>, fork_block: u64) -> eyre::Result<Self> {
        let block = provider.get_block_by_number(fork_block)
            .await?
            .ok_or_else(|| eyre::eyre!("Block not found"))?;

        let block_header = block.header;
        
        let inner = BlockEnv {
            number: U256::from(fork_block),
            timestamp: U256::from(block_header.timestamp),
            beneficiary: block_header.beneficiary,
            difficulty: block_header.difficulty,
            gas_limit: block_header.gas_limit.try_into().unwrap_or(u64::MAX),
            basefee: block_header.base_fee_per_gas.unwrap_or_default().try_into().unwrap_or(0),
            ..Default::default()
        };

        Ok(Self { inner })
    }

    pub fn increment_block(&mut self) -> u64 {
        self.inner.number += U256::from(1);
        self.inner.timestamp += U256::from(12);
        self.inner.number.try_into().unwrap_or(0)
    }
}
