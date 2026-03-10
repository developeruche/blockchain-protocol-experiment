use std::sync::Arc;
use tokio::runtime::Handle;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;
use revm::state::{AccountInfo, Account};
use revm::bytecode::Bytecode;
use revm::{DatabaseRef, DatabaseCommit};
use crate::provider::UpstreamProvider;
use crate::cache::ForkCache;
use crate::overlay_db::{OverlayDb, LocalAccount};

pub struct ForkDb<P> {
    provider: Arc<UpstreamProvider<P>>,
    cache: Arc<ForkCache>,
    overlay: Arc<OverlayDb>,
    rt: Handle,
}

impl<P: Provider> ForkDb<P> {
    pub fn new(
        provider: Arc<UpstreamProvider<P>>,
        cache: Arc<ForkCache>,
        overlay: Arc<OverlayDb>,
    ) -> Self {
        Self {
            provider,
            cache,
            overlay,
            rt: Handle::current(),
        }
    }

    pub fn insert_account_info(&self, address: Address, info: AccountInfo) {
        let mut local = self.overlay.accounts.get(&address).map(|a| a.clone()).unwrap_or_default();
        local.info = info;
        self.overlay.accounts.insert(address, local);
    }

    fn fetch_account(&self, address: Address) -> Result<AccountInfo, eyre::Report> {
        let provider = self.provider.clone();
        
        let (balance, nonce, code) = tokio::task::block_in_place(|| {
            self.rt.block_on(async {
                let p = provider;
                let balance = p.get_balance(address).await?;
                let nonce = p.get_transaction_count(address).await?;
                let code = p.get_code(address).await?;
                Ok::<_, eyre::Report>((balance, nonce, code))
            })
        })?;

        let code_hash = if code.is_empty() {
            revm_primitives::KECCAK_EMPTY
        } else {
            alloy::primitives::keccak256(&code)
        };

        if !code.is_empty() {
            let bytecode = Bytecode::new_raw(code.clone());
            self.cache.contracts.insert(code_hash, bytecode);
        }

        let info = AccountInfo {
            balance,
            nonce,
            code_hash,
            code: None,
            ..Default::default()
        };

        self.cache.accounts.insert(address, crate::types::CachedAccount {
            balance,
            nonce,
            code_hash,
        });

        Ok(info)
    }

    fn fetch_storage(&self, address: Address, index: U256) -> Result<U256, eyre::Report> {
        let provider = self.provider.clone();
        
        let value = tokio::task::block_in_place(|| {
            self.rt.block_on(async {
                provider.get_storage_at(address, index).await
            })
        })?;

        let val_u256 = U256::from_be_bytes(value.0);
        self.cache.storage.insert((address, index), val_u256);
        Ok(val_u256)
    }
}

impl<P: Provider> DatabaseRef for ForkDb<P> {
    type Error = std::convert::Infallible;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(local) = self.overlay.accounts.get(&address) {
            let info: AccountInfo = local.value().info.clone();
            return Ok(Some(info));
        }

        if let Some(cached) = self.cache.accounts.get(&address) {
            let info: AccountInfo = cached.value().clone().into();
            return Ok(Some(info));
        }

        let info = match self.fetch_account(address) {
            Ok(info) => info,
            Err(e) => {
                tracing::error!("fetch account failed for {:?}: {:?}", address, e);
                AccountInfo::default()
            }
        };
        Ok(Some(info))
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if let Some(local) = self.overlay.contracts.get(&code_hash) {
            let code: Bytecode = local.value().clone();
            return Ok(code);
        }

        if let Some(cached) = self.cache.contracts.get(&code_hash) {
            let code: Bytecode = cached.value().clone();
            return Ok(code);
        }

        Ok(Bytecode::default())
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        if let Some(local) = self.overlay.accounts.get(&address) {
            if let Some(val) = local.value().storage.get(&index) {
                return Ok(*val);
            }
        }

        if let Some(cached) = self.cache.storage.get(&(address, index)) {
            let val: U256 = *cached.value();
            return Ok(val);
        }

        let storage_val = match self.fetch_storage(address, index) {
            Ok(val) => val,
            Err(e) => {
                tracing::error!("fetch storage failed for {:?} at {:?}: {:?}", address, index, e);
                U256::ZERO
            }
        };
        Ok(storage_val)
    }
    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        let provider = self.provider.clone();
        
        let block = tokio::task::block_in_place(|| {
            self.rt.block_on(async {
                provider.get_block_by_number(number).await
            })
        });

        match block {
            Ok(Some(b)) => Ok(b.header.hash),
            Ok(None) => Ok(B256::ZERO),
            Err(e) => {
                tracing::error!("fetch block failed for number {}: {:?}", number, e);
                Ok(B256::ZERO)
            }
        }
    }
}

impl<P: Provider> revm::Database for ForkDb<P> {
    type Error = std::convert::Infallible;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.basic_ref(address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.code_by_hash_ref(code_hash)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.storage_ref(address, index)
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.block_hash_ref(number)
    }
}

impl<P: Provider> DatabaseCommit for ForkDb<P> {
    fn commit(&mut self, changes: std::collections::HashMap<Address, Account, alloy::primitives::map::FbBuildHasher<20>>) {
        for (address, account) in changes {
            let mut local_acct = self.overlay.accounts.entry(address).or_insert_with(LocalAccount::default);
            local_acct.info = account.info.clone();
            
            for (slot, value) in account.storage {
                local_acct.storage.insert(slot, value.present_value);
            }
            if let Some(code) = account.info.code {
                if !code.is_empty() {
                    self.overlay.contracts.insert(account.info.code_hash, code);
                }
            }
        }
    }
}
