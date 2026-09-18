use jsonrpsee::{core::RpcResult, types::Params};
use primitives::{
    constants::{DUMMY_BLOCK_HASH, DUMMY_TX_HASH},
    response::{BlockResponse, CHAIN_ID, TransactionReceiptResponse, TransactionResponse},
};
use serde_json::{Value, json};

pub fn chain_id_handler() -> &'static str {
    CHAIN_ID
}

pub fn block_number_handler() -> &'static str {
    "0x0"
}

pub fn get_block_by_number_handler(params: Params<'_>) -> RpcResult<Value> {
    let (block_number, _full_transactions): (String, bool) = params.parse()?;

    Ok(dummy_block(Value::String(block_number)))
}

pub fn get_block_by_hash_handler(params: Params<'_>) -> RpcResult<Value> {
    let (block_hash, _full_transactions): (String, bool) = params.parse()?;

    let mut block = dummy_block(Value::String("0x0".to_owned()));
    block["hash"] = Value::String(block_hash);

    Ok(block)
}

pub fn get_transaction_by_hash_handler(params: Params<'_>) -> RpcResult<Value> {
    let tx_hash: String = params.one()?;

    if tx_hash != DUMMY_TX_HASH {
        return Ok(Value::Null);
    }

    let default_tx_response = TransactionResponse::default();

    Ok(serde_json::to_value(default_tx_response).unwrap())
}

pub fn get_transaction_receipt_handler(params: Params<'_>) -> RpcResult<Value> {
    let tx_hash: String = params.one()?;

    if tx_hash != DUMMY_TX_HASH {
        return Ok(Value::Null);
    }

    let default_tx_receipt = TransactionReceiptResponse::default();

    Ok(serde_json::to_value(default_tx_receipt).unwrap())
}

pub fn get_balance_handler(params: Params<'_>) -> RpcResult<&'static str> {
    let (_address, _block_tag): (String, String) = params.parse()?;

    Ok("0x0")
}

pub fn eth_getTransactionCount(params: Params<'_>) -> RpcResult<&'static str> {
    let (_address, _block_tag): (String, String) = params.parse()?;

    Ok("0x0")
}

pub fn eth_sendRawTransaction(params: Params<'_>) -> RpcResult<Value> {
    let raw_txHash: String = params.one()?;

    if raw_txHash != DUMMY_TX_HASH {
        return Ok(Value::Null);
    }

    Ok(json!({
        "transactionHash": raw_txHash,
        "Status": "0x1",
    }
    ))
}

pub fn eth_call(params: Params<'_>) -> RpcResult<&'static str> {
    let (_call_data, _block_tag): (Value, String) = params.parse()?;

    Ok("0x0")
}

pub fn eth_estimateGas(params: Params<'_>) -> RpcResult<&'static str> {
    let _call_data: Value = params.one()?;

    Ok("0x5208")
}

fn dummy_block(number: Value) -> Value {
    let mut block = BlockResponse::default();
    block.number = number.to_string();
    serde_json::to_value(block).unwrap()
}
