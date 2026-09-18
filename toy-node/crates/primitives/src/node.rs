//! This module holds this core type for the blockchain node
//!
use std::collections::HashMap;

use alloy::primitives::{Address, B256, Bytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    nonce: u64,
    balance: U256,
    storage_root: B256,
    code_hash: B256,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub gas_limit: U256,
    pub gas_used: U256,
    pub hash: B256,
    pub miner: Address,
    pub mix_hash: B256,
    pub nonce: u64,
    pub number: u64,
    pub parent_hash: B256,
    pub size: u64,
    pub state_root: B256,
    pub transactions: Vec<B256>,
    pub transaction_root: B256,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub block_hash: B256,
    pub block_number: u64,
    pub chain_id: u64,
    pub from: Address,
    pub gas: U256,
    pub gas_price: U256, // Note: gas price would const
    pub hash: B256,
    pub input: Bytes, // 0xtransfer, 0xto, 0xamount,
    pub nonce: u64,
    pub to: Address,
    pub value: U256,
    pub transaction_index: u64,
    pub v: u8,
    pub r: U256,
    pub s: U256,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockState {
    accounts: HashMap<Address, Account>,
    blocks: Vec<Block>,
    transactions: HashMap<B256, Transaction>, // TODO: CLASS:: TransactionReceipt
}

impl Account {
    pub fn increase_nonce(&mut self) -> u64 {
        let passed_nonce = self.nonce;
        self.nonce += 1;

        passed_nonce
    }

    pub fn transfer(&mut self, amount: U256) -> Result<U256, anyhow::Error> {
        match self.balance.checked_sub(amount) {
            Some(_) => Ok(self.balance),
            None => anyhow::bail!("Insufficent balance for transfer"),
        }
    }

    pub fn receive(&mut self, amount: U256) -> Result<U256, anyhow::Error> {
        match self.balance.checked_add(amount) {
            Some(_) => Ok(self.balance),
            None => anyhow::bail!("Checked balance add error"),
        }
    }
}

impl BlockState {
    pub fn new_with_gensis() {
        todo!()
    }
}
