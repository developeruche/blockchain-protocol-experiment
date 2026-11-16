use std::collections::HashMap;

use alloy::primitives::{Address, B256, TxKind, U256};
use primitives::{
    constants::{BLOCK_GAS_LIMIT, BLOCK_REWARD, CHAIN_ID, GAS_PRICE},
    node::{Account, PendingTransaction, Transaction, TransactionReceipt, hash_list_root},
};
use revm::{
    Context, DatabaseCommit, ExecuteEvm, MainBuilder, MainContext,
    context::{BlockEnv, CfgEnv, TxEnv},
    primitives::hardfork::SpecId,
};

use crate::evm::BreejaDb;

/// What executing a batch of transactions produced.
pub struct Execution {
    /// Account state after the batch, ready to become the new head state.
    pub accounts: HashMap<Address, Account>,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<TransactionReceipt>,
    pub gas_used: u64,
    pub transaction_root: B256,
    pub receipts_root: B256,
    pub state_root: B256,
}

/// Applies one transaction using REVM as the execution engine.
///
/// In Milestone 2, this replaces the manual balance/nonce/fee manipulation with
/// execution through REVM. REVM automatically validates the sender's nonce and balance,
/// deducts the value and fee from the sender, increments the nonce, credits the
/// recipient, and credits the transaction fee to `beneficiary` (the miner).
///
/// # Pitfalls Avoided
/// - **Pitfall 1 (Double miner payment)**: REVM credits the fee to `beneficiary` directly.
///   We do NOT credit the fee manually here.
/// - **Pitfall 2 (Base fee burn)**: `basefee` is set to 0 in `BlockEnv`, so no ETH is burned.
/// - **Pitfall 3 (Double nonce increment)**: REVM increments the sender's nonce; we do NOT
///   call `increase_nonce` manually.
/// - **Pitfall 4 (Deprecated gas_used)**: We use `tx_gas_used()` on `ExecutionResult`.
/// - **Pitfall 5 (SpecId)**: We configure `SpecId::SHANGHAI`.
/// - **Pitfall 6 (State lock & borrow checker)**: The EVM borrow on `BreejaDb` is dropped
///   before committing state changes.
pub fn apply_transfer(
    accounts: &mut HashMap<Address, Account>,
    tx: &PendingTransaction,
    miner: Address,
) -> Result<u64, anyhow::Error> {
    execute_transaction(accounts, tx, 0, miner, B256::ZERO).map(|res| res.gas_used)
}

/// Result of executing a single transaction through REVM.
pub struct TxExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub created_address: Option<Address>,
    pub logs: Vec<primitives::node::Log>,
    pub output: alloy::primitives::Bytes,
}

/// Runs an unsigned RPC simulation against a private state snapshot. The result
/// is returned to the caller and all state changes are discarded.
pub fn simulate_call(
    accounts: &HashMap<Address, Account>, request: &primitives::params::CallRequest,
    block_number: u64, miner: Address, gas_limit: u64,
) -> Result<TxExecutionResult, anyhow::Error> {
    use alloy::primitives::Bytes;
    let mut db = BreejaDb::from_accounts(accounts);
    let mut cfg = CfgEnv::new_with_spec(SpecId::SHANGHAI).with_chain_id(CHAIN_ID);
    cfg.disable_nonce_check = true;
    cfg.disable_balance_check = true;
    let block_env = BlockEnv { number: U256::from(block_number), beneficiary: miner,
        timestamp: U256::ZERO, gas_limit: BLOCK_GAS_LIMIT, basefee: 0,
        difficulty: U256::ZERO, prevrandao: Some(B256::ZERO), ..Default::default() };
    let tx_env = TxEnv { tx_type: 0, caller: request.from.unwrap_or(Address::ZERO),
        gas_limit, gas_price: request.gas_price.unwrap_or(U256::from(GAS_PRICE)).to::<u128>(),
        kind: request.to.map(TxKind::Call).unwrap_or(TxKind::Create),
        value: request.value.unwrap_or(U256::ZERO), data: request.calldata(), nonce: 0,
        chain_id: Some(CHAIN_ID), ..Default::default() };
    let mut evm = Context::mainnet().with_db(&mut db).with_block(block_env)
        .with_cfg(cfg).build_mainnet();
    let outcome = evm.transact(tx_env).map_err(|e| anyhow::anyhow!("EVM simulation error: {e:?}"))?;
    let result = outcome.result;
    let answer = TxExecutionResult { success: result.is_success(), gas_used: result.tx_gas_used(),
        created_address: result.created_address(), output: result.output().cloned().unwrap_or(Bytes::new()),
        logs: result.logs().iter().map(|l| primitives::node::Log { address:l.address,
            topics:l.data.topics().to_vec(), data:l.data.data.clone() }).collect() };
    // Deliberately do not commit outcome.state.
    Ok(answer)
}

/// Executes a single transaction against the account state using REVM.
pub fn execute_transaction(
    accounts: &mut HashMap<Address, Account>,
    tx: &PendingTransaction,
    block_number: u64,
    miner: Address,
    parent_hash: B256,
) -> Result<TxExecutionResult, anyhow::Error> {
    let mut db = BreejaDb::from_accounts(accounts);

    let block_env = BlockEnv {
        number: U256::from(block_number),
        beneficiary: miner,
        timestamp: U256::ZERO,
        gas_limit: BLOCK_GAS_LIMIT,
        basefee: 0,
        difficulty: U256::ZERO,
        prevrandao: Some(parent_hash),
        ..Default::default()
    };

    let cfg = CfgEnv::new_with_spec(SpecId::SHANGHAI).with_chain_id(CHAIN_ID);

    let kind = match tx.to {
        Some(to) => TxKind::Call(to),
        None => TxKind::Create,
    };

    let tx_env = TxEnv {
        tx_type: 0, // legacy transaction
        caller: tx.from,
        gas_limit: tx.gas,
        gas_price: tx.gas_price,
        kind,
        value: tx.value,
        data: tx.input.clone(),
        nonce: tx.nonce,
        chain_id: Some(CHAIN_ID),
        ..Default::default()
    };

    let mut evm = Context::mainnet()
        .with_db(&mut db)
        .with_block(block_env)
        .with_cfg(cfg)
        .build_mainnet();

    let outcome = evm
        .transact(tx_env)
        .map_err(|e| anyhow::anyhow!("EVM validation/db error: {e:?}"))?;

    let result = outcome.result;
    let gas_used = result.tx_gas_used();
    let success = result.is_success();
    let created_address = result.created_address();
    let output = result.output().cloned().unwrap_or_default();
    let logs = result
        .logs()
        .iter()
        .map(|l| primitives::node::Log {
            address: l.address,
            topics: l.data.topics().to_vec(),
            data: l.data.data.clone(),
        })
        .collect();

    // Drop EVM to release borrow on db before committing changes.
    let state = outcome.state;
    drop(evm);
    db.commit(state);

    *accounts = db.into_accounts();

    Ok(TxExecutionResult {
        success,
        gas_used,
        created_address,
        logs,
        output,
    })
}

/// Pays the miner for sealing a block.
///
/// This is the only way new ETH enters the supply, and it is what makes mining
/// worth the electricity. Breeja pays Frontier's 5 ETH; Ethereum reduced this
/// twice before dropping it entirely at the merge.
pub fn pay_block_reward(
    accounts: &mut HashMap<Address, Account>,
    miner: Address,
) -> Result<(), anyhow::Error> {
    accounts.entry(miner).or_default().credit(BLOCK_REWARD)?;
    Ok(())
}

/// Executes a batch of pending transactions, producing the state and receipts the
/// next block will commit to.
///
/// In Milestone 3, transactions that revert or halt during EVM execution are STILL included
/// in the block:
/// - The receipt status is set to `0` (failure/reverted).
/// - The sender is charged for the gas consumed, and their nonce advances.
/// - Any state changes made during execution are committed per EVM rules (gas is charged, but
///   reverted contract changes are undone by REVM internally).
///
/// Only transactions with fundamental validation errors (e.g. invalid nonce or insufficient
/// upfront balance to cover gas_limit * gas_price) return an `Err` from `transact` and are dropped.
pub fn execute_block(
    mut accounts: HashMap<Address, Account>,
    batch: &[PendingTransaction],
    block_number: u64,
    miner: Address,
) -> Execution {
    let mut transactions = Vec::new();
    let mut receipts = Vec::new();
    let mut gas_used = 0u64;

    for tx in batch {
        // Reserve the transaction's gas limit before executing it. Transactions
        // that cannot fit remain in the pool for a later block.
        if tx.gas > BLOCK_GAS_LIMIT.saturating_sub(gas_used) {
            continue;
        }
        match execute_transaction(&mut accounts, tx, block_number, miner, B256::ZERO) {
            Ok(tx_res) => {
                let index = transactions.len() as u64;
                gas_used += tx_res.gas_used;

                transactions.push(Transaction {
                    hash: tx.hash,
                    block_hash: B256::ZERO,
                    block_number,
                    transaction_index: index,
                    chain_id: tx.chain_id,
                    from: tx.from,
                    to: tx.to,
                    value: tx.value,
                    nonce: tx.nonce,
                    gas: tx.gas,
                    gas_price: tx.gas_price,
                    input: tx.input.clone(),
                    v: tx.v,
                    r: tx.r,
                    s: tx.s,
                });

                receipts.push(TransactionReceipt {
                    transaction_hash: tx.hash,
                    transaction_index: index,
                    block_hash: B256::ZERO,
                    block_number,
                    from: tx.from,
                    to: tx.to,
                    gas_used: tx_res.gas_used,
                    cumulative_gas_used: gas_used,
                    effective_gas_price: tx.gas_price,
                    status: if tx_res.success { 1 } else { 0 },
                    contract_address: tx_res.created_address,
                    logs: tx_res.logs,
                });
            }
            Err(error) => {
                tracing::warn!(tx = %tx.hash, %error, "dropping invalid transaction that cannot execute");
            }
        }
    }

    // The miner's reward is paid after the transactions, so a miner cannot spend
    // it inside the very block that grants it.
    if let Err(error) = pay_block_reward(&mut accounts, miner) {
        tracing::error!(%error, "failed to pay block reward");
    }

    let tx_hashes: Vec<B256> = transactions.iter().map(|tx| tx.hash).collect();
    let receipt_hashes: Vec<B256> = receipts.iter().map(|r| r.transaction_hash).collect();

    Execution {
        transaction_root: hash_list_root(&tx_hashes),
        receipts_root: hash_list_root(&receipt_hashes),
        state_root: primitives::node::state_root_of(&accounts),
        accounts,
        transactions,
        receipts,
        gas_used,
    }
}

#[cfg(test)]
mod tests {
    use primitives::constants::{DEV_ADDRESS, DEV_ADDRESS_1, GAS_PRICE, TRANSFER_GAS};

    use super::*;

    fn one_eth() -> U256 {
        U256::from(10u64).pow(U256::from(18))
    }

    fn funded(balance: U256) -> HashMap<Address, Account> {
        let mut accounts = HashMap::new();
        accounts.insert(DEV_ADDRESS, Account::new(balance));
        accounts
    }

    fn transfer(value: U256, nonce: u64) -> PendingTransaction {
        PendingTransaction {
            hash: B256::repeat_byte(0xab),
            from: DEV_ADDRESS,
            to: Some(DEV_ADDRESS_1),
            value,
            nonce,
            gas: TRANSFER_GAS,
            gas_price: GAS_PRICE,
            chain_id: 1,
            input: alloy::primitives::Bytes::new(),
            v: 0,
            r: U256::ZERO,
            s: U256::ZERO,
        }
    }

    #[test]
    fn a_transfer_moves_value_charges_a_fee_and_bumps_the_nonce() {
        let miner = Address::repeat_byte(0x99);
        let mut accounts = funded(one_eth() * U256::from(10u64));
        let fee = U256::from(TRANSFER_GAS) * U256::from(GAS_PRICE);

        let gas = apply_transfer(&mut accounts, &transfer(one_eth(), 0), miner).unwrap();
        assert_eq!(gas, TRANSFER_GAS);

        let sender = &accounts[&DEV_ADDRESS];
        assert_eq!(
            sender.balance,
            one_eth() * U256::from(10u64) - one_eth() - fee
        );
        assert_eq!(sender.nonce, 1);
        assert_eq!(accounts[&DEV_ADDRESS_1].balance, one_eth());
        assert_eq!(accounts[&miner].balance, fee);
    }

    #[test]
    fn no_eth_is_created_or_destroyed_by_a_transfer() {
        let miner = Address::repeat_byte(0x99);
        let start = one_eth() * U256::from(10u64);
        let mut accounts = funded(start);

        apply_transfer(&mut accounts, &transfer(one_eth(), 0), miner).unwrap();

        let total: U256 = accounts
            .values()
            .map(|a| a.balance)
            .fold(U256::ZERO, |a, b| a + b);
        assert_eq!(total, start);
    }

    #[test]
    fn a_transfer_the_sender_cannot_afford_is_rejected() {
        let miner = Address::repeat_byte(0x99);
        // Enough for the value but not the fee on top.
        let mut accounts = funded(one_eth());
        assert!(apply_transfer(&mut accounts, &transfer(one_eth(), 0), miner).is_err());
    }

    #[test]
    fn a_transfer_with_the_wrong_nonce_is_rejected() {
        let miner = Address::repeat_byte(0x99);
        let mut accounts = funded(one_eth() * U256::from(10u64));
        assert!(apply_transfer(&mut accounts, &transfer(one_eth(), 7), miner).is_err());
    }

    #[test]
    fn executing_a_block_pays_the_reward_and_records_receipts() {
        let miner = Address::repeat_byte(0x99);
        let accounts = funded(one_eth() * U256::from(10u64));
        let batch = vec![transfer(one_eth(), 0)];

        let execution = execute_block(accounts, &batch, 1, miner);

        assert_eq!(execution.transactions.len(), 1);
        assert_eq!(execution.receipts.len(), 1);
        assert_eq!(execution.receipts[0].status, 1);
        assert_eq!(execution.gas_used, TRANSFER_GAS);
        assert_eq!(execution.receipts[0].cumulative_gas_used, TRANSFER_GAS);

        let fee = U256::from(TRANSFER_GAS) * U256::from(GAS_PRICE);
        assert_eq!(execution.accounts[&miner].balance, BLOCK_REWARD + fee);

        // Non-empty roots, because this block committed to a transaction.
        assert_ne!(execution.transaction_root, B256::ZERO);
        assert_ne!(execution.state_root, B256::ZERO);
    }

    #[test]
    fn an_empty_block_still_pays_the_miner_but_commits_to_no_transactions() {
        let miner = Address::repeat_byte(0x99);
        let execution = execute_block(funded(one_eth()), &[], 1, miner);

        assert!(execution.transactions.is_empty());
        assert_eq!(execution.gas_used, 0);
        assert_eq!(execution.transaction_root, B256::ZERO);
        assert_eq!(execution.accounts[&miner].balance, BLOCK_REWARD);
    }

    #[test]
    fn a_transfer_that_no_longer_applies_is_skipped_not_mined() {
        let miner = Address::repeat_byte(0x99);
        let accounts = funded(one_eth() * U256::from(10u64));
        // Two transactions claiming the same nonce: only the first can apply.
        let batch = vec![transfer(one_eth(), 0), transfer(one_eth(), 0)];

        let execution = execute_block(accounts, &batch, 1, miner);
        assert_eq!(execution.transactions.len(), 1);
    }

    /// Milestone 2 parity test:
    /// A plain ETH transfer executed through REVM costs exactly 21,000 gas
    /// and moves exactly the same ETH as apply_transfer did.
    #[test]
    fn transfer_through_revm_costs_exactly_21000_gas_parity() {
        let miner = Address::repeat_byte(0x99);
        let mut accounts = funded(one_eth() * U256::from(10u64));
        let tx = transfer(one_eth(), 0);

        let res = execute_transaction(&mut accounts, &tx, 1, miner, B256::ZERO).unwrap();
        assert!(res.success);
        assert_eq!(
            res.gas_used, 21_000,
            "plain ETH transfer through REVM must cost exactly 21,000 gas"
        );
        assert_eq!(res.created_address, None);

        let fee = U256::from(21_000u64) * U256::from(GAS_PRICE);
        assert_eq!(
            accounts[&DEV_ADDRESS].balance,
            one_eth() * U256::from(10u64) - one_eth() - fee
        );
        assert_eq!(accounts[&DEV_ADDRESS].nonce, 1);
        assert_eq!(accounts[&DEV_ADDRESS_1].balance, one_eth());
        assert_eq!(accounts[&miner].balance, fee);
    }

    #[test]
    fn simulation_returns_contract_data_and_discards_storage_changes() {
        let contract = Address::repeat_byte(0x44);
        // SSTORE slot 0 = 42, then return the 32-byte word 42.
        let mut accounts = funded(one_eth() * U256::from(10u64));
        accounts.insert(contract, Account { code: Some(alloy::primitives::Bytes::from_static(&[
            0x60,0x2a,0x60,0x00,0x55, 0x60,0x2a,0x60,0x00,0x52,
            0x60,0x20,0x60,0x00,0xf3,
        ])), ..Default::default() });
        let request = primitives::params::CallRequest {
            from: Some(DEV_ADDRESS), to: Some(contract), gas: Some(U256::from(100_000)),
            ..Default::default()
        };
        let result = simulate_call(&accounts, &request, 0, Address::ZERO, 100_000).unwrap();
        assert!(result.success);
        assert_eq!(result.output.len(), 32);
        assert_eq!(U256::from_be_slice(&result.output), U256::from(42));
        assert_eq!(accounts[&contract].storage.get(&U256::ZERO), None,
            "simulation must not commit its SSTORE");
    }

    #[test]
    fn block_skips_transaction_whose_gas_limit_exceeds_remaining_capacity() {
        let accounts = funded(one_eth() * U256::from(100u64));
        let mut first = transfer(U256::ZERO, 0);
        first.gas = BLOCK_GAS_LIMIT;
        let mut second = transfer(U256::ZERO, 1);
        second.gas = BLOCK_GAS_LIMIT;
        let execution = execute_block(accounts, &[first, second], 1, Address::repeat_byte(0x99));
        assert_eq!(execution.transactions.len(), 1);
        assert!(execution.gas_used <= BLOCK_GAS_LIMIT);
    }
}
