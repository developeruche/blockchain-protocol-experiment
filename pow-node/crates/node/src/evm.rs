//! REVM integration: the database wrapper that lets REVM read and write Breeja's
//! account state during transaction execution.
//!
//! # Design
//!
//! REVM does not take state directly — it takes a [`Database`] trait object and
//! calls it during execution whenever it needs to read an account, fetch bytecode,
//! or look up a storage slot. When execution finishes, [`DatabaseCommit`] writes
//! the resulting `AddressMap<Account>` back to our in-memory state.
//!
//! [`BreejaDb`] is a *snapshot* of chain state for one block execution. The miner
//! builds it from `BlockState::accounts_snapshot()`, executes transactions against
//! it one by one, then hands the mutated snapshot back to `BlockState::commit_block`.
//! This avoids holding the `RwLock` across execution (which could block RPC
//! requests for the entire duration of a contract call).
//!
//! # Code-hash indexing
//!
//! REVM's `Database::code_by_hash` method looks up bytecode by its Keccak-256
//! hash. We maintain a separate `code_by_hash` map so that identical contracts
//! deployed to different addresses share one copy, matching what real nodes do.

use std::collections::HashMap;
use std::convert::Infallible;

use alloy::primitives::{Address, B256, U256};
use primitives::node::{Account, StorageKey, StorageValue};
use revm::{
    Database, DatabaseCommit,
    primitives::AddressMap,
    state::{Account as RevmAccount, AccountInfo, Bytecode},
};

/// A snapshot of chain account state that REVM can read and write through the
/// [`Database`] / [`DatabaseCommit`] traits.
///
/// Build one per block from `BlockState::accounts_snapshot()`. After executing
/// all transactions, the mutated accounts map becomes the new head state.
#[derive(Debug, Default, Clone)]
pub struct BreejaDb {
    /// All known accounts, keyed by address.
    pub accounts: HashMap<Address, EvmAccount>,
    /// Runtime bytecode indexed by its Keccak-256 hash.
    /// Populated on `DatabaseCommit` so `code_by_hash` lookups hit the cache.
    pub code_by_hash: HashMap<B256, Bytecode>,
    /// Recent block hashes for the `BLOCKHASH` opcode. The key is the block
    /// number; the value is `keccak256(rlp(header))` as stored in `Block::hash`.
    pub block_hashes: HashMap<u64, B256>,
}

/// An account as held inside `BreejaDb`.
///
/// Mirrors `primitives::node::Account` but uses REVM's `Bytecode` type so the
/// code is immediately usable by the interpreter without re-parsing.
#[derive(Debug, Clone, Default)]
pub struct EvmAccount {
    pub nonce: u64,
    pub balance: U256,
    /// Runtime bytecode. `None` for EOAs.
    pub code: Option<Bytecode>,
    /// Storage slots that have been written. Absent slots read as zero.
    pub storage: HashMap<StorageKey, StorageValue>,
}

impl BreejaDb {
    /// Builds a snapshot from the primitives account map, populating
    /// `code_by_hash` for every contract account found.
    pub fn from_accounts(accounts: &HashMap<Address, Account>) -> Self {
        let mut db = BreejaDb::default();
        for (addr, acct) in accounts {
            let code = acct.code.as_ref().map(|bytes| {
                // `Bytecode::new_raw` wraps bytes without re-hashing; the hash
                // is derived lazily via `hash_slow()` when REVM needs it.
                Bytecode::new_raw(bytes.clone())
            });
            if let Some(ref bc) = code {
                db.code_by_hash.insert(bc.hash_slow(), bc.clone());
            }
            db.accounts.insert(
                *addr,
                EvmAccount {
                    nonce: acct.nonce,
                    balance: acct.balance,
                    code,
                    storage: acct.storage.clone(),
                },
            );
        }
        db
    }

    /// Converts the snapshot back into the primitives account map so it can be
    /// committed to `BlockState` after execution.
    pub fn into_accounts(self) -> HashMap<Address, Account> {
        self.accounts
            .into_iter()
            .map(|(addr, evm_acct)| {
                let (code_bytes, code_hash) = match evm_acct.code {
                    Some(ref bc) if !bc.is_empty() => {
                        let hash = bc.hash_slow();
                        (Some(bc.original_bytes().clone()), hash)
                    }
                    _ => (None, B256::ZERO),
                };
                let acct = Account {
                    nonce: evm_acct.nonce,
                    balance: evm_acct.balance,
                    code_hash,
                    // storage_root is set lazily; left ZERO for the teaching node.
                    storage_root: B256::ZERO,
                    code: code_bytes,
                    storage: evm_acct.storage,
                };
                (addr, acct)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// REVM trait implementations
// ---------------------------------------------------------------------------

impl Database for BreejaDb {
    /// Reading from an in-memory map cannot fail. A database backed by disk
    /// would use a real error type here.
    type Error = Infallible;

    /// Returns account metadata for REVM: balance, nonce, code hash, and an
    /// inline copy of the bytecode.
    ///
    /// Returning `None` for an unknown address tells REVM to treat the account
    /// as empty (nonce 0, balance 0, no code) — identical to Ethereum semantics
    /// for accounts that have never been touched.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self.accounts.get(&address).map(|a| {
            let code = a.code.clone().unwrap_or_default();
            AccountInfo {
                balance: a.balance,
                nonce: a.nonce,
                code_hash: code.hash_slow(),
                code: Some(code),
                ..Default::default()
            }
        }))
    }

    /// Looks up bytecode by hash. Called by REVM when a contract calls another
    /// contract and needs to load its code.
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(self
            .code_by_hash
            .get(&code_hash)
            .cloned()
            .unwrap_or_default())
    }

    /// Returns the value of a single storage slot.
    ///
    /// Absent slots return zero, which is correct: Ethereum defines unwritten
    /// storage as zero, and SLOAD on a fresh slot yields 0.
    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        Ok(self
            .accounts
            .get(&address)
            .and_then(|a| a.storage.get(&index).copied())
            .unwrap_or_default())
    }

    /// Backs the `BLOCKHASH` opcode.
    ///
    /// The EVM can request the hash of any of the 256 most-recent blocks.
    /// Wire the chain's `block_hashes` map here so contracts that read recent
    /// block hashes get real answers instead of zeros.
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        Ok(self
            .block_hashes
            .get(&number)
            .copied()
            .unwrap_or_default())
    }
}

impl DatabaseCommit for BreejaDb {
    /// Applies REVM's output state back to our account map.
    ///
    /// REVM hands back *every* account it read, not just those it modified.
    /// Only touched accounts are merged; untouched ones are skipped to avoid
    /// inflating the state unnecessarily.
    fn commit(&mut self, changes: AddressMap<RevmAccount>) {
        for (address, account) in changes {
            // REVM marks accounts it only read (not modified) as untouched.
            // Committing them would be harmless but wastes work.
            if !account.is_touched() {
                continue;
            }

            // SELFDESTRUCT removes the account from state entirely.
            if account.is_selfdestructed() {
                self.accounts.remove(&address);
                continue;
            }

            let entry = self.accounts.entry(address).or_default();
            entry.balance = account.info.balance;
            entry.nonce = account.info.nonce;

            // Index the bytecode by hash so future `code_by_hash` lookups hit
            // the cache without re-scanning all accounts.
            if let Some(code) = account.info.code.clone()
                && !code.is_empty()
            {
                self.code_by_hash
                    .insert(account.info.code_hash, code.clone());
                entry.code = Some(code);
            }

            // Merge storage: `present_value` is the slot's value after execution.
            for (slot, value) in account.storage {
                entry.storage.insert(slot, value.present_value);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests (M1 acceptance criteria)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use primitives::node::Account;
    use revm::{
        Database, DatabaseCommit,
        primitives::AddressMap,
        state::{
            Account as RevmAccount, AccountInfo, AccountStatus, Bytecode, EvmStorageSlot,
            TransactionId,
        },
    };

    use super::*;

    fn one_eth() -> U256 {
        U256::from(10u64).pow(U256::from(18))
    }

    // -----------------------------------------------------------------------
    // Database::basic
    // -----------------------------------------------------------------------

    /// M1 acceptance: `Database::basic` returns the correct balance and nonce
    /// for an account that exists in the snapshot.
    #[test]
    fn basic_returns_balance_and_nonce_for_funded_account() {
        let addr = Address::repeat_byte(0x11);
        let mut accounts = HashMap::new();
        accounts.insert(
            addr,
            Account {
                nonce: 3,
                balance: one_eth() * U256::from(5u64),
                ..Default::default()
            },
        );

        let mut db = BreejaDb::from_accounts(&accounts);
        let info = db.basic(addr).unwrap().expect("account should exist");

        assert_eq!(info.nonce, 3, "nonce must round-trip through BreejaDb");
        assert_eq!(
            info.balance,
            one_eth() * U256::from(5u64),
            "balance must round-trip through BreejaDb"
        );
    }

    /// M1 acceptance: `Database::basic` returns `None` for an address that has
    /// never been seen. REVM treats `None` as an empty account (nonce 0,
    /// balance 0, no code) — equivalent to Ethereum's behaviour for untouched
    /// addresses.
    #[test]
    fn basic_returns_none_for_unknown_address() {
        let mut db = BreejaDb::default();
        let unknown = Address::repeat_byte(0xde);
        let result = db.basic(unknown).unwrap();
        assert!(
            result.is_none(),
            "unknown address must return None so REVM treats it as empty, not missing"
        );
    }

    // -----------------------------------------------------------------------
    // DatabaseCommit
    // -----------------------------------------------------------------------

    /// M1 acceptance: `DatabaseCommit` persists a balance change, a nonce
    /// change, new code, and a storage write, all in a single commit call.
    #[test]
    fn commit_persists_balance_nonce_code_and_storage() {
        let addr = Address::repeat_byte(0x22);
        let mut db = BreejaDb::default();

        // Build a hand-crafted REVM change set.
        let bytecode = Bytecode::new_raw(alloy::primitives::Bytes::from_static(&[0x60, 0x00, 0x56]));
        let code_hash = bytecode.hash_slow();

        let mut account = RevmAccount::default();
        account.info = AccountInfo {
            balance: one_eth() * U256::from(7u64),
            nonce: 5,
            code_hash,
            code: Some(bytecode.clone()),
            ..Default::default()
        };
        account.storage.insert(
            U256::from(0u64),
            EvmStorageSlot::new_changed(U256::ZERO, U256::from(42u64), TransactionId::ZERO),
        );
        account.status = AccountStatus::Touched;
        account.mark_touch();

        let mut changes = AddressMap::default();
        changes.insert(addr, account);

        db.commit(changes);

        // Balance and nonce
        let entry = db.accounts.get(&addr).expect("account should exist after commit");
        assert_eq!(entry.balance, one_eth() * U256::from(7u64));
        assert_eq!(entry.nonce, 5);

        // Code is stored inline and also indexed by hash
        assert!(entry.code.is_some(), "code should be stored after commit");
        assert!(
            db.code_by_hash.contains_key(&code_hash),
            "code_by_hash must be populated so future code_by_hash lookups succeed"
        );

        // Storage slot
        assert_eq!(
            entry.storage.get(&U256::from(0u64)).copied().unwrap_or_default(),
            U256::from(42u64),
            "storage slot 0 must hold the committed value"
        );
    }

    /// Untouched accounts are ignored by `DatabaseCommit` — REVM returns
    /// every account it read, but only touched ones have changed.
    #[test]
    fn commit_skips_untouched_accounts() {
        let addr = Address::repeat_byte(0x33);
        let mut db = BreejaDb::default();

        let mut account = RevmAccount::default();
        account.info = AccountInfo {
            balance: one_eth(),
            nonce: 1,
            code_hash: B256::ZERO,
            code: None,
            ..Default::default()
        };
        account.status = AccountStatus::default();

        let mut changes = AddressMap::default();
        changes.insert(addr, account);

        db.commit(changes);

        assert!(
            !db.accounts.contains_key(&addr),
            "untouched account must not be inserted into the snapshot"
        );
    }

    /// A selfdestructed account is removed from the snapshot.
    #[test]
    fn commit_removes_selfdestructed_accounts() {
        let addr = Address::repeat_byte(0x44);
        let mut db = BreejaDb::default();

        // Pre-populate so there is something to remove.
        db.accounts.insert(
            addr,
            EvmAccount {
                nonce: 1,
                balance: one_eth(),
                ..Default::default()
            },
        );

        let mut account = RevmAccount::default();
        account.status = AccountStatus::SelfDestructed;
        account.mark_touch();

        let mut changes = AddressMap::default();
        changes.insert(addr, account);

        db.commit(changes);

        assert!(
            !db.accounts.contains_key(&addr),
            "selfdestructed account must be removed from the snapshot"
        );
    }

    // -----------------------------------------------------------------------
    // Round-trip: from_accounts → into_accounts
    // -----------------------------------------------------------------------

    /// A snapshot built from a primitives account map and converted back must
    /// preserve nonce, balance, and storage.
    #[test]
    fn round_trip_preserves_account_fields() {
        let addr = Address::repeat_byte(0x55);
        let mut slot_map = HashMap::new();
        slot_map.insert(U256::from(1u64), U256::from(99u64));

        let mut accounts = HashMap::new();
        accounts.insert(
            addr,
            Account {
                nonce: 7,
                balance: one_eth(),
                storage: slot_map,
                ..Default::default()
            },
        );

        let db = BreejaDb::from_accounts(&accounts);
        let recovered = db.into_accounts();

        let acct = recovered.get(&addr).expect("address must survive the round-trip");
        assert_eq!(acct.nonce, 7);
        assert_eq!(acct.balance, one_eth());
        assert_eq!(
            acct.storage.get(&U256::from(1u64)).copied().unwrap_or_default(),
            U256::from(99u64)
        );
    }
}
