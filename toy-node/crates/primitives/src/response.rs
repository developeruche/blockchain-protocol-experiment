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
    pub chain_id: String,
    pub from: String,
    pub gas: String,
    pub gas_price: String, // Note: gas price would const
    
}
