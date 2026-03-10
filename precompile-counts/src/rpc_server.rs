use alloy::primitives::{Address, Bytes, U256, B256, TxKind};
use jsonrpsee::server::Server;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::core::RpcResult;
use std::sync::Arc;
use alloy::providers::Provider;
use revm::Database;

use crate::executor::Executor;
use alloy::rpc::types::{TransactionRequest, BlockId};
use revm::context::TxEnv;
use revm::context_interface::result::ExecutionResult;

#[rpc(server, namespace = "eth")]
pub trait EthRpc {
    #[method(name = "chainId")]
    async fn chain_id(&self) -> RpcResult<U256>;

    #[method(name = "blockNumber")]
    async fn block_number(&self) -> RpcResult<U256>;

    #[method(name = "getBalance")]
    async fn get_balance(&self, address: Address, block: Option<BlockId>) -> RpcResult<U256>;

    #[method(name = "getTransactionCount")]
    async fn get_transaction_count(&self, address: Address, block: Option<BlockId>) -> RpcResult<U256>;

    #[method(name = "getCode")]
    async fn get_code(&self, address: Address, block: Option<BlockId>) -> RpcResult<Bytes>;
    
    #[method(name = "getStorageAt")]
    async fn get_storage_at(&self, address: Address, slot: U256, block: Option<BlockId>) -> RpcResult<B256>;

    #[method(name = "call")]
    async fn call(&self, request: TransactionRequest, block: Option<BlockId>) -> RpcResult<Bytes>;

    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, tx: Bytes) -> RpcResult<B256>;
}

pub struct EthRpcImpl<P> {
    executor: Arc<tokio::sync::RwLock<Executor<P>>>,
}

fn req_to_tx_env(req: TransactionRequest) -> TxEnv {
    let mut tx = TxEnv::default();
    tx.caller = req.from.unwrap_or_default();
    tx.kind = req.to.unwrap_or(TxKind::Create);
    tx.data = req.input.into_input().unwrap_or_default();
    tx.value = req.value.unwrap_or_default();
    tx
}

#[async_trait::async_trait]
impl<P: Provider + Send + Sync + 'static> EthRpcServer for EthRpcImpl<P> {
    async fn chain_id(&self) -> RpcResult<U256> {
        let exec = self.executor.read().await;
        Ok(U256::from(exec.chain_id))
    }

    async fn block_number(&self) -> RpcResult<U256> {
        let exec = self.executor.read().await;
        Ok(exec.block_env.inner.number)
    }

    async fn get_balance(&self, address: Address, _block: Option<BlockId>) -> RpcResult<U256> {
        let mut exec = self.executor.write().await;
        let balance = exec.db.basic(address)
            .map_err(|e| jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, e.to_string(), None::<()>))?
            .map(|a| a.balance)
            .unwrap_or_default();
        Ok(balance)
    }

    async fn get_transaction_count(&self, address: Address, _block: Option<BlockId>) -> RpcResult<U256> {
        let mut exec = self.executor.write().await;
        let nonce = exec.db.basic(address)
            .map_err(|e| jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, e.to_string(), None::<()>))?
            .map(|a| a.nonce)
            .unwrap_or_default();
        Ok(U256::from(nonce))
    }

    async fn get_code(&self, address: Address, _block: Option<BlockId>) -> RpcResult<Bytes> {
        let mut exec = self.executor.write().await;
        let code = exec.db.basic(address)
            .map_err(|e| jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, e.to_string(), None::<()>))?
            .and_then(|a| a.code)
            .map(|c| Bytes::from(c.bytes().clone()))
            .unwrap_or_default();
        Ok(code)
    }

    async fn get_storage_at(&self, address: Address, slot: U256, _block: Option<BlockId>) -> RpcResult<B256> {
        let mut exec = self.executor.write().await;
        let val = exec.db.storage(address, slot.into())
            .map_err(|e| jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, e.to_string(), None::<()>))?;
        Ok(val.into())
    }

    async fn call(&self, request: TransactionRequest, _block: Option<BlockId>) -> RpcResult<Bytes> {
        let mut exec = self.executor.write().await;
        let tx_env = req_to_tx_env(request);
        
        let result = exec.call(tx_env)
            .map_err(|e| jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, e.to_string(), None::<()>))?;
            
        match result {
            ExecutionResult::Success { output, .. } => Ok(output.into_data().into()),
            ExecutionResult::Revert { output, .. } => Err(jsonrpsee::types::error::ErrorObjectOwned::owned(3, "execution reverted".to_string(), Some(output))),
            ExecutionResult::Halt { reason, .. } => Err(jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, format!("halted: {:?}", reason), None::<()>)),
        }
    }

    async fn send_raw_transaction(&self, tx: Bytes) -> RpcResult<B256> {
        use alloy::consensus::{TxEnvelope, Transaction};
        use alloy::consensus::transaction::SignerRecoverable;
        use alloy::rlp::Decodable;

        let hash = alloy::primitives::keccak256(&tx);

        let mut data = tx.as_ref();
        let envelope = TxEnvelope::decode(&mut data)
            .map_err(|e| jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, format!("Failed to decode RLP: {}", e), None::<()>))?;

        let caller = envelope.recover_signer()
            .map_err(|e| jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, format!("Failed to recover signer: {}", e), None::<()>))?;

        let mut tx_env = TxEnv::default();
        tx_env.caller = caller;
        tx_env.data = envelope.input().to_vec().into();
        tx_env.value = envelope.value();
        tx_env.nonce = envelope.nonce();
        tx_env.gas_limit = envelope.gas_limit();
        tx_env.gas_price = envelope.max_fee_per_gas();
        tx_env.gas_priority_fee = envelope.max_priority_fee_per_gas();

        if let Some(to) = envelope.to() {
            tx_env.kind = TxKind::Call(to);
        } else {
            tx_env.kind = TxKind::Create;
        }

        let mut exec = self.executor.write().await;
        exec.send_transaction(tx_env)
            .map_err(|e| jsonrpsee::types::error::ErrorObjectOwned::owned(-32000, format!("Execution failed: {}", e), None::<()>))?;

        Ok(hash)
    }
}

pub async fn start_server<P: Provider + Send + Sync + 'static>(addr: std::net::SocketAddr, executor: Arc<tokio::sync::RwLock<Executor<P>>>) -> eyre::Result<()> {
    let rpc_impl = EthRpcImpl { executor };
    let server = Server::builder().build(addr).await?;
    let handle = server.start(rpc_impl.into_rpc());
    handle.stopped().await;
    Ok(())
}
