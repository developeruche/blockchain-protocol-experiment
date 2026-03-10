use alloy::primitives::B256;
use alloy::primitives::U256;
use revm::state::AccountInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedAccount {
    pub balance: U256,
    pub nonce: u64,
    pub code_hash: B256,
}

impl Into<AccountInfo> for CachedAccount {
    fn into(self) -> AccountInfo {
        AccountInfo {
            balance: self.balance,
            nonce: self.nonce,
            code_hash: self.code_hash,
            code: None,
            ..Default::default()
        }
    }
}
