use std::collections::HashMap;
use alloy::primitives::{Address, B256, U256};
use revm::state::AccountInfo;
use revm::bytecode::Bytecode;
use dashmap::DashMap;


#[derive(Clone, Default, Debug)]
pub struct LocalAccount {
    pub info: AccountInfo,
    pub storage: HashMap<U256, U256>,
}

#[derive(Clone, Default)]
pub struct OverlayDb {
    pub accounts: DashMap<Address, LocalAccount>,
    pub contracts: DashMap<B256, Bytecode>,
}

impl OverlayDb {
    pub fn new() -> Self {
        Self::default()
    }
}
