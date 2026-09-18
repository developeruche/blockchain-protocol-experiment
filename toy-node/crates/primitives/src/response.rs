use serde::{Deserialize, Serialize};

pub const CHAIN_ID: &str = "0x1";

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockResponse {
    pub gas_limit: String,
    pub gas_used: String,
    pub hash: String,
    pub miner: String,
    pub mix_hash: String,
    pub nonce: String,
    pub number: String,
    pub parent_hash: String,
    pub size: String,
    pub state_root: String,
    pub transactions: Vec<String>,
    pub transaction_root: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub block_hash: String,
    pub block_number: String,
    pub chain_id: String, //quantity
    pub from: String,
    pub gas: String,
    pub gas_price: String, // Note: gas price would const
    pub hash: String,
    pub input: String, // 0xtransfer, 0xto, 0xamount,
    pub nonce: String,
    pub to: String,
    pub value: String,
    pub transaction_index: String,
    pub v: String,
    pub r: String,
    pub s: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceiptResponse {
    pub block_hash: String,
    pub block_number: String,
    pub contract_address: String,
    pub cummulative_gas_used: String,
    pub effective_gas_price: String,
    pub from: String,
    pub gas_used: String,
    pub logs: Vec<String>,
    pub status: String,
    pub to: String,
    pub transaction_hash: String,
    pub transaction_index: String,
}
