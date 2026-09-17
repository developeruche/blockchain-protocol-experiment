use std::error::Error;

use jsonrpsee::{
    RpcModule,
    core::RpcResult,
    server::{ServerBuilder, ServerHandle},
    types::Params,
};
use serde_json::{Value, json};

const RPC_ADDRESS: &str = "127.0.0.1:8545";
const DUMMY_BLOCK_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const DUMMY_TX_HASH: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";

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

fn chain_id_handler() -> &'static str {
    "0x1"
}

fn block_number_handler() -> &'static str {
    "0x0"
}

fn get_block_by_number_handler(params: Params<'_>) -> RpcResult<Value> {
    let (block_number, _full_transactions): (String, bool) = params.parse()?;

    Ok(dummy_block(Value::String(block_number)))
}

fn get_block_by_hash_handler(params: Params<'_>) -> RpcResult<Value> {
    let (block_hash, _full_transactions): (String, bool) = params.parse()?;

    let mut block = dummy_block(Value::String("0x0".to_owned()));
    block["hash"] = Value::String(block_hash);

    Ok(block)
}

fn get_transaction_by_hash_handler(params: Params<'_>) -> RpcResult<Value> {
    let tx_hash: String = params.one()?;

    if tx_hash != DUMMY_TX_HASH {
        return Ok(Value::Null);
    }

    Ok(json!({
         "hash": DUMMY_TX_HASH,
         "blockHash": DUMMY_BLOCK_HASH,
         "blockNumber": "0x0",
         "from": "0x0000000000000000000000000000000000000000",
         "to": "0x0000000000000000000000000000000000000000",
         "value": "0x0"
    }))
}

fn get_transaction_receipt_handler(params: Params<'_>) -> RpcResult<Value> {
    let tx_hash: String = params.one()?;

    if tx_hash != DUMMY_TX_HASH {
        return Ok(Value::Null);
    }

    Ok(json!({
        "transactionHash": tx_hash,
        "status": "0x1"
    }))
}

fn get_balance_handler(params: Params<'_>) -> RpcResult<&'static str> {
    let (_address, _block_tag): (String, String) = params.parse()?;

    Ok("0x0")
}

fn dummy_block(number: Value) -> Value {
    json!({
        "number": number,
        "hash": DUMMY_BLOCK_HASH,
        "transactions": [],
    })
}
