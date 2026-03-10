use dashmap::DashMap;
use alloy::primitives::{Address, B256, U256};
use revm::bytecode::Bytecode;
use crate::types::CachedAccount;

pub struct ForkCache {
    pub accounts: DashMap<Address, CachedAccount>,
    pub storage: DashMap<(Address, U256), U256>,
    pub contracts: DashMap<B256, Bytecode>,
}

impl ForkCache {
    pub fn new() -> Self {
        Self {
            accounts: DashMap::new(),
            storage: DashMap::new(),
            contracts: DashMap::new(),
        }
    }
}
