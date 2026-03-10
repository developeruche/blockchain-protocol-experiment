use alloy::providers::Provider;
use alloy::primitives::{Address, B256, U256, Bytes};
use alloy::rpc::types::BlockId;

pub struct UpstreamProvider<P> {
    inner: P,
    fork_block: BlockId,
}

impl<P: Provider> UpstreamProvider<P> {
    pub fn new(inner: P, fork_block: u64) -> Self {
        Self { inner, fork_block: BlockId::from(fork_block) }
    }

    pub async fn get_chain_id(&self) -> eyre::Result<u64> {
        let id = self.inner.get_chain_id().await?;
        Ok(id)
    }

    pub async fn get_balance(&self, address: Address) -> eyre::Result<U256> {
        let balance = self.inner.get_balance(address).block_id(self.fork_block).await?;
        Ok(balance)
    }

    pub async fn get_transaction_count(&self, address: Address) -> eyre::Result<u64> {
        let nonce = self.inner.get_transaction_count(address).block_id(self.fork_block).await?;
        Ok(nonce)
    }

    pub async fn get_code(&self, address: Address) -> eyre::Result<Bytes> {
        let code = self.inner.get_code_at(address).block_id(self.fork_block).await?;
        Ok(code)
    }

    pub async fn get_storage_at(&self, address: Address, slot: U256) -> eyre::Result<B256> {
        let storage = self.inner.get_storage_at(address, slot).block_id(self.fork_block).await?;
        Ok(B256::from(storage))
    }

    pub async fn get_block_by_number(&self, number: u64) -> eyre::Result<Option<alloy::rpc::types::Block>> {
        let block = self.inner.get_block_by_number(number.into()).await?;
        Ok(block)
    }
}
