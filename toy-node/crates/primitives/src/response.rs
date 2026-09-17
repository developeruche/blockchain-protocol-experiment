use serde::{Deserialize, Serialize};

pub const CHAIN_ID: &str = "0x1";


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Block {
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