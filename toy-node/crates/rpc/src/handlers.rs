use jsonrpsee::{core::RpcResult, types::Params};
use primitives::{
    constants::{DEV_ADDRESS, DUMMY_BLOCK_HASH, DUMMY_TX_HASH, ZERO_HASH},
    response::{BlockResponse, CHAIN_ID, TransactionReceiptResponse, TransactionResponse},
};
use serde_json::{Value, json};

use crate::RPCContext;

pub fn chain_id_handler() -> &'static str {
    CHAIN_ID
}

pub fn block_number_handler(rpc_context: &RPCContext) -> String {
    rpc_context.block_state.get_block_number().to_string()
}

pub fn get_block_by_number_handler(
    params: Params<'_>,
    rpc_context: &RPCContext,
) -> RpcResult<Value> {
    let (block_number, _full_transactions): (String, bool) = params.parse()?;

    let blk_number: u64 = u64::from_str_radix(&block_number, 16).unwrap();
    let block = rpc_context.block_state.get_block(blk_number as usize);

    Ok(serde_json::to_value(block).unwrap())
}

pub fn get_block_by_hash_handler(params: Params<'_>) -> RpcResult<Value> {
    let (block_hash, _full_transactions): (String, bool) = params.parse()?;

    if block_hash != DUMMY_BLOCK_HASH {
        return Ok(Value::Null);
    }

    Ok(serde_json::to_value(dummy_block()).unwrap())
}

pub fn get_transaction_by_hash_handler(params: Params<'_>) -> RpcResult<Value> {
    let tx_hash: String = params.one()?;

    if tx_hash != DUMMY_TX_HASH {
        return Ok(Value::Null);
    }

    Ok(serde_json::to_value(dummy_transaction()).unwrap())
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

fn dummy_block() -> BlockResponse {
    BlockResponse {
        gas_limit: "0x1c9c380".to_owned(),
        gas_used: "0x5208".to_owned(),
        hash: DUMMY_BLOCK_HASH.to_owned(),
        miner: DEV_ADDRESS.to_string(),
        mix_hash: ZERO_HASH.to_owned(),
        nonce: "0x0000000000000000".to_owned(),
        number: "0x0".to_owned(),
        parent_hash: ZERO_HASH.to_owned(),
        size: "0x220".to_owned(),
        state_root: ZERO_HASH.to_owned(),
        // The dummy transaction below is the one transaction in this block.
        transactions: vec![DUMMY_TX_HASH.to_owned()],
        transaction_root: ZERO_HASH.to_owned(),
    }
}

fn dummy_transaction() -> TransactionResponse {
    TransactionResponse {
        block_hash: DUMMY_BLOCK_HASH.to_owned(),
        block_number: "0x0".to_owned(),
        chain_id: CHAIN_ID.to_owned(),
        from: DEV_ADDRESS.to_string(),
        gas: "0x5208".to_owned(),
        gas_price: "0x3b9aca00".to_owned(),
        hash: DUMMY_TX_HASH.to_owned(),
        input: "0x".to_owned(),
        nonce: "0x0".to_owned(),
        to: DEV_ADDRESS.to_string(),
        value: "0x2386f26fc10000".to_owned(),
        transaction_index: "0x0".to_owned(),
        v: "0x1b".to_owned(),
        r: "0x1b5e176d927f8e9ab405058b2d2457392da3e20f328b16ddabcebc33eaac5fea".to_owned(),
        s: "0x4ba69724e8f69de52f0125ad8b3c5c2cef33019bac3249e2c0a2192766d1721c".to_owned(),
    }
}
