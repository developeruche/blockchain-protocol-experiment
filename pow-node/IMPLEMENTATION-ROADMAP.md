# Breeja EVM: Components and Change Roadmap

This is the implementation map for `FINAL-PROJECT.md`. We are using the existing **REVM** implementation linked by the assignment; we are integrating an EVM into Breeja, not writing an interpreter. The assignment's Appendix A targets **REVM 43.0.3** and recommends `revm = "43"` in `crates/node/Cargo.toml`.

## What REVM provides, and what Breeja must provide

REVM executes Ethereum transactions and handles EVM rules such as opcodes, gas metering, calls, contract creation, logs, and revert behavior. It does not automatically know how Breeja stores its accounts, blocks, or history. Breeja must:

1. Expose a transaction-local view of account balance, nonce, bytecode, storage, and block hashes through REVM's `Database` trait.
2. Translate Breeja's pending transactions and block fields into REVM transaction and block environments.
3. Apply REVM's returned state changes through `DatabaseCommit`.
4. Convert REVM's result into Breeja transactions, receipts, logs, and RPC responses.

```text
PendingTransaction + candidate Block
                │ map to TxEnv / BlockEnv
                ▼
         REVM executes EVM rules
          ▲                 │
          │ Database reads  │ execution outcome + changed state
          │                 ▼
     BreejaDb snapshot ◄─ DatabaseCommit
          │
          ▼
   block execution + receipt data → PoW sealing → chain commit
```

REVM's `Database` is a pull interface: the EVM requests an account, code, storage slot, or block hash when execution needs it. `DatabaseCommit` applies the resulting changes after execution. This lets the EVM access state dynamically without holding Breeja's shared `BlockState` lock throughout execution.

## Components and expected changes

| Component | Current role | Change for REVM integration |
| --- | --- | --- |
| `crates/node/Cargo.toml` | Node crate dependencies | Add `revm = "43"` and compile the reference integration against the resolved version. |
| `crates/primitives/src/node.rs` | Accounts, pending/mined transactions, receipts, blocks, `BlockState` | Store or index contract bytecode and storage; allow optional `to` for creation; retain calldata; add receipt contract address and logs; preserve storage/code in snapshots. |
| `crates/node/src/evm.rs` (new) | No EVM adapter exists | Define a snapshot database implementing REVM `Database` and `DatabaseCommit`. Track account data, code-by-hash, storage, and block hashes. |
| `crates/node/src/pool.rs` | Signed transaction parsing and mempool admission | Admit contract-creation and calldata transactions; replace the fixed 21,000 gas floor with correct intrinsic gas checks; keep nonce, chain ID, price, and maximum-funds checks. |
| `crates/node/src/executor.rs` | Applies plain transfers, creates receipts and roots | Build REVM environments, execute transactions, commit REVM state changes, account for gas/refunds, create receipts/logs, and include mined reverts/halts as failed transactions. Keep block reward separate. |
| `crates/node/src/miner.rs` | Selects transactions, executes candidate, mines and commits | Preserve snapshot/lock discipline; pass accurate block context into execution; select a batch that fits actual block gas usage; commit updated state and receipts. |
| `crates/node/src/chain.rs` and `BlockState` methods | Shared chain and pending/history queries | Add read access for code/storage/logs where needed; ensure pending reservations and transaction removal remain correct after execution. |
| `crates/rpc/src/handlers.rs` | JSON-RPC reads and transfer submit/call/estimate | Implement real `eth_call`, `eth_estimateGas`, `eth_getCode`, `eth_getStorageAt`, and `eth_getLogs`; translate execution and log data to Ethereum JSON-RPC shapes. |
| `crates/rpc/src/lib.rs` | Registers RPC methods | Register new RPC methods for code, storage, and logs. |
| `crates/node/src/testing.rs`, node/RPC tests | Signing helpers and behavior tests | Add helpers for deployment and contract calls; cover database, execution, revert, gas, logs, and RPC behavior. |
| `pow-node/EVM-NOTES.md` | Assignment decisions and project log | Keep baseline results, design decisions, debugging notes, limitations, and final reflection current. |

## Milestone-by-milestone plan

### M0 — Branch and baseline

**Purpose:** preserve a known-good starting point before changing execution.

- Branch is `feat/evm`, based on `main`.
- Run `cargo test --workspace`; assignment expects 48 existing tests. Record the actual command output and test count in `EVM-NOTES.md`.
- Run `cargo run --release -- --dev` and note several mined blocks and their relevant output.
- Add REVM 43 to the node crate and get it compiling before modifying node behavior.

### M1 — State model and REVM database adapter

**Purpose:** make account state readable and writable by REVM while preserving Breeja's snapshot model.

- Decide how to represent code and storage. Direct code bytes on accounts are simple; a code-hash map deduplicates bytecode and aligns with REVM's `code_by_hash` lookup.
- Add code and storage accessors to chain state.
- Implement `Database::basic`, `code_by_hash`, `storage`, and `block_hash` on a separate snapshot type (suggested: `crates/node/src/evm.rs`).
- Implement `DatabaseCommit` so touched accounts' balances, nonces, code and storage are persisted, and self-destructed accounts are handled.
- Add a database read test and a commit test for balance, nonce, code, and a storage slot.

The snapshot adapter matters because REVM needs mutable database access while executing. Holding the `RwLock` on shared `BlockState` through execution would block JSON-RPC readers. The miner already clones an account snapshot for this reason.

### M2 — Run ordinary transfers through REVM

**Purpose:** keep existing transfer behavior while replacing hand-written transfer execution.

- Map each `PendingTransaction` to REVM `TxEnv` and the candidate Breeja block to `BlockEnv`.
- Configure chain ID, legacy transaction type, hardfork (`SpecId::SHANGHAI` per the assignment reference), zero base fee, beneficiary, timestamp, difficulty, gas limit, and parent-derived `prevrandao` deliberately.
- Convert REVM execution result into Breeja transaction/receipt data and apply state changes.
- Add the parity test: a plain transfer uses exactly 21,000 gas and transfers the same value as before.
- Preserve supply conservation. REVM pays the beneficiary transaction fees; remove Breeja's manual per-transaction miner fee credit or the miner is paid twice. Do not remove the block reward.
- Avoid double-incrementing nonce: REVM handles transaction nonce changes.

### M3 — Contract creation, calls, logs, and reverts

**Purpose:** enable real contract execution end-to-end.

- Change transaction data models so `to` can be absent for creation and calldata survives from pool admission through mined transaction records.
- Relax the pool's creation/calldata rejections and fixed transfer gas assumption. Ensure each transaction has sufficient intrinsic gas and the block respects `BLOCK_GAS_LIMIT`.
- For creation, pass `TxKind::Create` and record `ExecutionResult::created_address()` in the receipt; commit runtime code from REVM's state changes.
- Return logs from REVM in receipts, adding a primitives log representation that can serialize to RPC form.
- Include REVM `Revert` and `Halt` results in blocks with status `0x0`, actual gas charged, nonce advanced, and reverted contract-side effects discarded. They are not pre-execution validity errors.
- Test deployment, code persistence, storage mutation, emitted log, reverting call, and varied gas block limits.

### M4 — `eth_call` and `eth_estimateGas`

**Purpose:** simulate execution against state without corrupting chain state.

- Build an execution path that can run a REVM transaction against a cloned database snapshot.
- For `eth_call`, return REVM's output and do not call `DatabaseCommit` on persistent chain state.
- For `eth_estimateGas`, execute simulations to find a sufficient gas limit; it is not a constant once calldata and code run.
- Test that a call returns expected data and leaves account/storage state unchanged; test that the estimate is enough for a submitted transaction to succeed.

### M5 — Contract state and log RPC

**Purpose:** expose state produced by contracts through JSON-RPC.

- `eth_getCode`: return deployed runtime bytecode or `0x` for no code.
- `eth_getStorageAt`: return a storage word as 32-byte hex for an address, slot, and block tag.
- `eth_getLogs`: match logs by block range and address, and implement the filter behavior required by the assignment (including topic filters, or document any unsupported part honestly).
- Register methods in `crates/rpc/src/lib.rs`, implement handlers and response types, and exercise them through the existing real-server RPC test harness.

### M6 — Integration, quality, and write-up

**Purpose:** make the complete feature reviewable and deliver the requested artifacts.

- Run all existing and new tests; explain any changed old expectation in a nearby test comment.
- Verify `cargo build --release`, then `cargo clippy` with no warnings.
- Verify conservation, nonce handling, fee accounting, failed receipts, historical reads, and block gas bounds.
- Add a shell demo or Rust test that deploys a Solidity contract and calls it on the locally mined chain.
- Complete `EVM-NOTES.md` (2–4 pages): what changed, what broke, decisions and reasons, and known limitations.
- Submit a PR against the fork.

## Key REVM execution shape

The assignment's Appendix A gives a version-43 reference. The essential construction is:

```rust
let mut evm = Context::mainnet()
    .with_db(&mut *db)
    .with_block(block_env)
    .with_cfg(cfg)
    .build_mainnet();

let outcome = evm.transact(tx_env)?;
let result = outcome.result;
let gas_used = result.tx_gas_used();
let created = result.created_address();
let logs = result.logs().to_vec();
let state_changes = outcome.state.clone();
drop(evm);            // release its mutable borrow of db
db.commit(state_changes);
```

For `eth_call`, use the same simulation path but do not commit the returned state. The final code must handle errors rather than using `expect`; this excerpt shows the shape, not a complete production error path.

## Pitfalls to actively guard against

- **Fee credited twice:** REVM credits `BlockEnv.beneficiary`; remove manual fee credit in `apply_transfer` once REVM executes transactions.
- **Base fee burns ETH:** configure `basefee = 0` for Breeja's pre-EIP-1559 model unless implementing a fee market.
- **Nonce incremented twice:** REVM increments transaction nonce; remove duplicate manual handling.
- **Wrong gas measure:** use `tx_gas_used()` as specified by the assignment, not a deprecated/incorrect accessor.
- **Hardfork mismatch:** use the assignment's stated Shanghai configuration and be explicit about any change.
- **Lock/borrow lifetime:** REVM holds a mutable borrow of its database while alive; drop it before committing through the database wrapper.
- **Wrong block context:** populate beneficiary, timestamp, number, gas limit, difficulty and `prevrandao` consistently.
- **Revert misclassified as invalid:** a valid transaction that reverts is included and charged; only validation/database failures should abort processing as errors.

## Acceptance checklist

- [ ] REVM 43 integration compiles; release build succeeds.
- [ ] `cargo clippy` has no warnings.
- [ ] Existing tests pass, or changed expectations have comments explaining why.
- [ ] Tests cover database read/commit, transfer parity, deployment, calls, storage, logs, revert, gas, and supply conservation.
- [ ] RPC tests cover call, estimate, code, storage, and logs.
- [ ] End-to-end Solidity deployment/call demo exists.
- [ ] `EVM-NOTES.md` includes actual baseline results, decisions, failures, and limitations.
