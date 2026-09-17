mod handlers;


use std::error::Error;

use jsonrpsee::{
    RpcModule,
    server::{ServerBuilder, ServerHandle}
};
use primitives::constants::RPC_ADDRESS;

use crate::handlers::{block_number_handler, chain_id_handler, get_balance_handler, get_block_by_hash_handler, get_block_by_number_handler, get_transaction_by_hash_handler, get_transaction_receipt_handler};


/// Binds the JSON-RPC server to port 8545, registers its methods, and starts it.
pub async fn start_server() -> Result<ServerHandle, Box<dyn Error + Send + Sync>> {
    let server = ServerBuilder::default().build(RPC_ADDRESS).await?;
    let mut module = RpcModule::new(());

    module.register_method("eth_chainId", |_params, _context, _extensions| {
        chain_id_handler()
    })?;

    module.register_method("eth_blockNumber", |_params, _context, _extensions| {
        block_number_handler()
    })?;

    module.register_method("eth_getBlockByNumber", |params, _context, _extensions| {
        get_block_by_number_handler(params)
    })?;

    module.register_method("eth_getBlockByHash", |params, _context, _extensions| {
        get_block_by_hash_handler(params)
    })?;

    module.register_method(
        "eth_getTransactionByHash",
        |params, _context, _extensions| get_transaction_by_hash_handler(params),
    )?;

    module.register_method(
        "eth_getTransactionReceipt",
        |params, _context, _extensions| get_transaction_receipt_handler(params),
    )?;

    module.register_method("eth_getBalance", |params, _context, _extensions| {
        get_balance_handler(params)
    })?;

    Ok(server.start(module))
}







