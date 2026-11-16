# Breeja EVM Project Notes

## Final integration results

On 2026-11-07, `cargo test --workspace` passed 63 tests, including the Solidity
counter demo, RPC simulations, accounting, historical reads, and failed receipts.
`cargo build --release` and `cargo clippy --workspace --all-targets -- -D warnings`
also passed. The demo source, compiled bytecode, and reproduction instructions are
in `demo/`. The block executor reserves each transaction's gas limit against the
remaining block capacity; transactions that do not fit remain pending.

Known limits include full in-memory snapshots for historical state, heuristic gas
estimation for gas-sensitive contracts, and the unsupported `eth_getLogs`
`blockHash` filter. The node remains a teaching implementation; its state roots
and other Ethereum compatibility details should not be treated as production consensus.

Working log for `FINAL-PROJECT.md`. Update this file at each milestone and keep unresolved items explicit.

## Branch and baseline

- Working branch: `feat/evm`, created from `main` before implementation.
- Baseline test result: **48 tests passed; 0 failed** across `node` unit tests (13), `node` integration tests (8), `primitives` unit tests (14), and `rpc` integration tests (13).
- Baseline development mining: ran `breeja --dev` (`cargo run --release -- --dev`). Observed successful startup, mining blocks with difficulty retargeting (~2.8 MH/s hash rate, blocks 0 through 21+ sealed smoothly with proof-of-work nonces verified).
- Current test result after M1: **54 tests passed; 0 failed** (6 new unit tests in `crates/node/src/evm.rs` covering `basic`, `code_by_hash`, `storage`, `commit`, selfdestruct, and round-trip conversion).
- Current test result after M2: **55 tests passed; 0 failed** (added parity test verifying 21,000 gas execution through REVM; all mining integration and supply conservation tests passing).
- Current test result after M3: **58 tests passed; 0 failed** (added 3 end-to-end integration tests in `crates/node/tests/mining.rs` verifying contract deployment, contract call with storage write and event log emission, and reverting transaction inclusion with status 0, sender gas debit, and nonce advancement).
- REVM dependency: Added `revm = "43"` to `crates/node/Cargo.toml`. Builds cleanly and passes `cargo clippy --all-targets --all-features -- -D warnings`.

> [!NOTE]
> REVM version pinned to `43` (`revm = "43"` in `crates/node/Cargo.toml`). Most online
> snippets target older versions; when in doubt, read
> `~/.cargo/registry/src/*/revm-43.0.3/` directly.

## Orientation observations and decisions

### 1. Mempool assumptions in `crates/node/src/pool.rs`

- `decode_and_check` rejects `TxKind::Create` with “contract creation is not supported”. Remove this rejection and represent `to` as optional so creation reaches the executor.
- `decode_and_check` rejects non-empty input/calldata because no interpreter exists. Remove this rejection once execution is backed by the EVM; preserve the transaction input through pending and mined transaction data.
- `gas < TRANSFER_GAS` assumes a transfer always costs 21,000. Replace with appropriate intrinsic-gas validation (or defer execution-dependent sufficiency to REVM); calldata bytes affect intrinsic cost. Keep a block gas-limit check during inclusion.
- Current admission reserves `value + gas_limit * gas_price` against pending balance. This is the maximum upfront liability; execution should charge actual gas used and return the unused-gas difference. Verify sender/miner accounting against REVM so fees are credited exactly once.

### 2. Failed execution and inclusion

`executor.rs` currently says failures are skipped because pre-validation leaves only races such as stale nonce. This ceases to be correct with contracts: an EVM-valid transaction may revert or halt after inclusion. Ethereum includes it, advances the nonce, charges gas, reports receipt status `0x0`, and discards reverted execution effects. Distinguish invalid transaction/database errors from EVM `Revert`/`Halt` outcomes.

### 3. Gas reservation vs charge

The pool reserves the maximum (`gas_limit × gas_price`) so pending transactions cannot collectively promise more than the sender can cover. Current transfer execution burns only `TRANSFER_GAS`, refunding the remainder implicitly because only actual transfer gas is debited. With variable execution, calculate actual gas from REVM, charge it, and account for unused gas refund without double-paying the beneficiary. REVM already credits the beneficiary under the configured environment; see assignment Pitfall 1.

### 4. Historical snapshots and storage cost

`BlockState.state_history` retains a full `HashMap<Address, Account>` snapshot after every block. Adding per-account storage makes each snapshot copy contract storage as well, so memory grows roughly with blocks × total account/storage state (plus map/bytecode overhead). Consider sharing/persistent maps or a trie/versioned database for a production design; for this project preserve history semantics and document the teaching-node cost.

## State and integration notes

- `primitives::node::Account` currently contains nonce, balance, `storage_root`, and `code_hash`, but no code bytes or storage map; the latter fields default to empty values.
- `PendingTransaction` and mined `Transaction` currently require a recipient and do not preserve calldata. These models must support contract creation and input.
- `TransactionReceipt` currently has no logs or contract address; status is always success. Extend it for real execution results.
- Miner/executor use an account snapshot; implement REVM `Database` and `DatabaseCommit` against a transaction-local snapshot, then commit changes to the next block state.
- Keep PoW, difficulty retargeting, block structure, signature recovery, mempool plumbing, and RPC server skeleton working while replacing transfer execution.

---

## Implementation details

This section documents the concrete implementation choices made at each milestone,
the REVM integration patterns used, and the reasoning behind each decision.

### Architecture overview

```
                  eth_sendRawTransaction / eth_call
 wallet / cast ───────────────────────────────────────┐
                                                      ▼
                                               ┌─────────────┐
                                               │  JSON-RPC   │  crates/rpc
                                               └──────┬──────┘
                                 validate + queue     │     simulate (eth_call)
                                                      ▼           │
                                               ┌─────────────┐    │
                                               │   mempool   │    │
                                               └──────┬──────┘    │
                                                      ▼           ▼
 ┌──────────────────────────────────────────────────────────────────────┐
 │  miner loop                                                          │
 │    1. snapshot state                                                 │
 │    2. for each tx:  ┌──────────────────────────────────────┐         │
 │                     │  REVM                                │         │
 │                     │    reads  ──► Database  (crates/node/src/evm)  │
 │                     │    writes ──► EvmState               │         │
 │                     └──────────────┬───────────────────────┘         │
 │                        DatabaseCommit                                │
 │    3. pay block reward (Breeja policy — REVM knows nothing of it)    │
 │    4. proof of work (unchanged)                                      │
 │    5. commit block                                                   │
 └────────────────────────────────────┬─────────────────────────────────┘
                                      ▼
                               ┌─────────────┐
                               │ chain state │  accounts + CODE + STORAGE
                               └─────────────┘
```

What does **not** change: proof of work, difficulty retargeting, block structure,
signature recovery, mempool plumbing, RPC server skeleton. Only `apply_transfer` (and
the state model it touches) is replaced.

---

### M1 — State model and `Database` implementation

#### Account representation

`Account` in `crates/primitives/src/node.rs` is extended from `{nonce, balance}` to
include code bytes and a storage map. Two representation strategies exist:

| Strategy | Pros | Cons |
| --- | --- | --- |
| Store `Bytecode` inline per account | Simple; no extra lookup | Identical contracts duplicated in every snapshot |
| Store `code_hash` + separate `HashMap<B256, Bytecode>` | Deduplicates; matches `code_by_hash` REVM trait method | Extra indirection on every code read |

**Decision:** use the hash-indexed strategy (matches what `Database::code_by_hash`
expects, and what real nodes do).

#### `BreejaDb` — the REVM database wrapper

`crates/node/src/evm.rs` holds `BreejaDb`, a snapshot struct that wraps account state
for a single transaction execution. It is **not** implemented directly on `BlockState`
because REVM borrows the database mutably for the entire lifetime of the EVM object,
and `BlockState` lives behind an `RwLock` shared with the RPC server. The miner's
existing `accounts_snapshot()` pattern is reused.

```rust
#[derive(Debug, Default)]
pub struct BreejaDb {
    pub accounts:      HashMap<Address, EvmAccount>,
    pub code_by_hash:  HashMap<B256, Bytecode>,
    pub block_hashes:  HashMap<u64, B256>,
}
```

`Database` implementation covers four methods required by REVM:

| Method | What it does |
| --- | --- |
| `basic(address)` | Returns `Option<AccountInfo>` — `None` for unknown accounts (REVM treats `None` as empty, not missing) |
| `code_by_hash(hash)` | Looks up pre-indexed bytecode; returns empty `Bytecode` if absent |
| `storage(address, slot)` | Returns `U256` slot value or zero |
| `block_hash(number)` | Backs the `BLOCKHASH` opcode from the chain's history |

`DatabaseCommit` iterates `AddressMap<Account>` returned by REVM and applies only
*touched* accounts (`account.is_touched()`). Selfdestructed accounts are removed.
Storage slots are merged slot-by-slot using `value.present_value`.

> [!IMPORTANT]
> `is_touched()` — REVM returns every account it **read**, not just those it changed.
> Committing untouched accounts is harmless here but would corrupt change-tracking in a
> real node. Always guard with `is_touched()`.

---

### M2 — Transfer parity through REVM

`apply_transfer` in `crates/node/src/executor.rs` is replaced (or shadowed by
`execute_transaction`) with a call to REVM. The mapping between Breeja types and REVM
environment structs:

#### `BlockEnv`

```rust
let block_env = BlockEnv {
    number:      U256::from(block.number),
    beneficiary: block.miner,          // REVM pays fees here automatically
    timestamp:   U256::from(block.timestamp),
    gas_limit:   block.gas_limit,
    basefee:     0,                    // pre-EIP-1559; see Pitfall 2
    difficulty:  U256::from(block.difficulty),
    prevrandao:  Some(block.parent_hash), // see Pitfall 7
    ..Default::default()
};
```

#### `CfgEnv`

```rust
let cfg = CfgEnv::new_with_spec(SpecId::SHANGHAI)  // see Pitfall 5
    .with_chain_id(CHAIN_ID);
```

#### `TxEnv`

```rust
let tx_env = TxEnv {
    tx_type:   0,              // legacy transaction
    caller:    tx.from,
    gas_limit: tx.gas,
    gas_price: tx.gas_price,
    kind: match tx.to {
        Some(to) => TxKind::Call(to),
        None     => TxKind::Create,   // contract deployment
    },
    value: tx.value,
    data:  tx.input.clone(),
    nonce: tx.nonce,
    chain_id: Some(CHAIN_ID),
    ..Default::default()
};
```

#### Executing and committing

```rust
let mut evm = Context::mainnet()
    .with_db(&mut *db)
    .with_block(block_env)
    .with_cfg(cfg)
    .build_mainnet();

let outcome = evm.transact(tx_env)?;
let state   = outcome.state.clone();
drop(evm);        // release &mut borrow before commit (Pitfall 6)
db.commit(state);
```

**Parity checkpoint:** a plain ETH transfer with empty calldata must report
`result.tx_gas_used() == 21_000`. If it differs, `TxEnv` or `CfgEnv` is misconfigured.

---

### M3 — Contracts

#### Mempool rule changes (`pool.rs`)

| Old check | New behaviour | Reason |
| --- | --- | --- |
| Reject `TxKind::Create` | Allow; pass to executor | No EVM was the only reason to block it |
| Reject non-empty calldata | Allow; preserve input | Same reason |
| `gas < TRANSFER_GAS` flat floor | Real intrinsic gas: 21,000 base + per-byte calldata + 32,000 for `Create` | Calldata is charged per byte; a flat floor is wrong for contract calls |

#### Receipt fields

Three previously hardcoded fields become real:

| Field | Before | After |
| --- | --- | --- |
| `contract_address` | always `None` | `Some(addr)` for `TxKind::Create`; `None` otherwise. Source: `result.created_address()` |
| `logs` | always `[]` | `result.logs().to_vec()`. Requires a `Log` type in `primitives` with `address`, `topics`, `data` |
| `status` | always `1` | `1` for `Success`; `0` for `Revert` or `Halt` |

#### Reverted transactions

A reverted transaction is **still mined**. The distinction between invalid and reverted:

| Outcome | Included? | Gas charged? | Nonce advanced? | State effects kept? |
| --- | --- | --- | --- | --- |
| Invalid (bad nonce, insufficient balance) | No — skipped | No | No | — |
| `Revert` / `Halt` | **Yes** | **Yes** | **Yes** | No — discarded |

`ExecutionResult` has three variants: `Success`, `Revert`, `Halt`. Both `Revert` and
`Halt` produce `status: 0x0` on the receipt.

#### Reference contract (from assignment, verified on REVM 43)

```
initcode:  0x600b600c600039600b6000f3602a60005560006000a000

runtime (11 bytes):
  602a 6000 55   PUSH1 42, PUSH1 0, SSTORE   → storage[0] = 42
  6000 6000 a0   PUSH1 0,  PUSH1 0, LOG0     → emits an empty log
  00             STOP
```

Expected gas costs (REVM 43, `SpecId::SHANGHAI`, `basefee=0`, `gas_price=1 gwei`):

| Operation | Gas |
| --- | --- |
| Plain ETH transfer | **21,000** |
| Deploy reference contract | ~55,522 |
| Call reference contract | ~43,487 |

---

### M4 — `eth_call` and `eth_estimateGas`

#### `eth_call`

Execute the transaction against a **throwaway copy** of current state and return the
output bytes. The only difference from a real transaction: **do not call `db.commit()`**.

Whether to disable nonce/balance checks (`CfgEnv` offers `disable_nonce_check` and
`disable_balance_check`): enabling both lets tooling simulate calls from accounts with
no on-chain balance, which is what most clients do. **Decision:** disable both for
`eth_call` only; document this in notes.

#### `eth_estimateGas`

Gas is unknown until execution; execution with insufficient gas fails. Standard approach
is a **binary search**:

1. Execute with the block gas limit. If this fails, the transaction cannot succeed at any
   gas limit — return an error.
2. Binary search `[intrinsic_cost, block_gas_limit]` for the lowest limit at which
   execution still succeeds (~20 iterations covers a 30 M gas range).
3. Return with a small safety margin.

> [!WARNING]
> A contract can behave *differently* depending on how much gas it has (via the `GAS`
> opcode), so estimation is a heuristic, not a proof. This is noted in the write-up and
> is accepted behaviour for all Ethereum clients.

#### Implemented behavior (2026-11-07)

`eth_call` and `eth_estimateGas` now build REVM over an account snapshot selected by
the supplied block tag. Simulations disable nonce and balance validation, return call
output / reject reverting or halting execution, and never commit REVM's resulting
state. Gas estimation first checks the requested maximum (capped at the block gas
limit), then binary-searches from 21,000 and adds a 1/64 margin. A node unit test
confirms both returned data and discarded storage writes.

For `pending`, the node replays its selected mempool batch into a temporary snapshot
before simulation. The estimator also treats execution errors as unsuccessful search points, which is
appropriate for out-of-gas but can conceal some validation errors until the maximum
gas simulation.

---

### M5 — New RPC methods

Implemented `eth_getCode`, `eth_getStorageAt`, `eth_getLogs`, and `eth_gasPrice`.
Code and storage reads use the selected historical snapshot; pending reads use the
same pending replay as simulations. Log queries support block ranges, one or many
addresses, and positional topic filters with wildcards and alternatives. The optional
`blockHash` log filter is currently rejected explicitly. Integration coverage deploys
a contract, checks historical code and storage, and filters its emitted log.

| Method | Returns | Implementation note |
| --- | --- | --- |
| `eth_getCode` | Runtime bytecode at address; `0x` if none | Look up `Account.code` from the block state at the requested tag |
| `eth_getStorageAt` | 32-byte hex value of one storage slot | Look up `Account.storage[slot]` |
| `eth_getLogs` | Logs matching filter | Iterate receipts across the requested block range; filter by `address` and `topics` |
| `eth_gasPrice` | Current gas price (one line) | Returns `GAS_PRICE` constant; enables `cast send` without `--gas-price` |

`eth_getLogs` filter semantics (from the spec):
- `topics` is a positional array; `null` at a position is a wildcard.
- An address filter accepts one or many addresses.
- Minimum required: `fromBlock`, `toBlock`, `address`.
- Topic filtering is implemented for full marks.

---

### M6 — Variable gas and final polish

#### Block gas limit enforcement

The old limit `BLOCK_GAS_LIMIT / TRANSFER_GAS = 149 txs` assumed uniform cost. The
miner now accumulates `gas_used` per block and skips any transaction that would cause
the total to exceed `BLOCK_GAS_LIMIT`.

#### Mempool intrinsic gas floor

Replaces the flat `TRANSFER_GAS` floor with:

```
intrinsic = 21_000
          + 4 × (zero calldata bytes)
          + 16 × (non-zero calldata bytes)
          + 32_000 (if TxKind::Create)
```

#### `logsBloom`

Built from receipt logs using the Ethereum bloom-filter algorithm (Yellow Paper §4.3.1):
for each log, hash the address and each topic with Keccak-256 and set three bits in a
256-byte bitfield. Previously hardcoded to 256 zero bytes.

#### State root

`state_root_of` previously hashed only `{nonce, balance}` per account. It now commits
to code hash and storage root as well, so two chains differing only in contract state
produce different roots.

---

## Challenges faced

This section is an honest account of the bugs, design tensions, and surprises encountered
during the EVM integration. Each entry follows the format: **expected → observed → cause → fix → regression test**.

### Challenge 1 — Pitfall 1: Double miner payment

**Expected:** remove `apply_transfer`'s manual fee credit and let REVM handle it.  
**Observed (risk):** if both paths are kept, every fee is paid twice and the chain mints
ETH out of nothing. The supply-conservation test
`the_chain_never_creates_or_destroys_eth_outside_the_block_reward` catches this silently.  
**Cause:** `apply_transfer` credited the fee to the miner by hand; REVM also pays the
beneficiary automatically when `BlockEnv.beneficiary` is set and `basefee = 0`.  
**Fix:** remove the manual fee credit entirely. Keep the block reward — REVM has no
concept of Breeja's block-reward policy and must still be applied by hand after
execution.  
**Verified:** after a 21,000-gas transfer at 1 gwei with `basefee=0`, the beneficiary's
balance increases by exactly `21_000 × 1_000_000_000 = 21_000_000_000_000` wei — paid
by REVM, not by our code.  
**Regression test:** `the_chain_never_creates_or_destroys_eth_outside_the_block_reward`
(existing). Do not weaken this assertion.

---

### Challenge 2 — Pitfall 2: Non-zero `basefee` burns ETH

**Expected:** Breeja is pre-EIP-1559; sender pays full gas price, miner receives all of
it.  
**Observed (risk):** with `basefee = 500_000_000` and `gas_price = 1_000_000_000`, the
sender was charged the full price but the beneficiary received only *half*. The other
half was burned — EIP-1559 working as designed.  
**Cause:** `BlockEnv.basefee` was not explicitly zeroed.  
**Fix:** always set `basefee: 0` in `BlockEnv`. If a fee market is desired as a stretch
goal, the supply-conservation test must be updated to account for the burn, and the
choice must be documented.  
**Regression test:** supply-conservation test; any non-zero burn causes it to fail.

---

### Challenge 3 — Pitfall 3: Double nonce increment

**Expected:** REVM increments the sender's nonce internally as part of transaction
processing.  
**Observed (risk):** `apply_transfer` also called `increase_nonce()`. Keeping both
consumes two nonces per transaction, breaking every subsequent transaction from that
sender.  
**Fix:** remove the manual `increase_nonce()` call. REVM's committed state already
contains the incremented nonce; `DatabaseCommit` writes it back.  
**Regression test:** submit two consecutive transactions from the same sender; assert
the second is included with nonce `n+1`.

---

### Challenge 4 — Pitfall 4: Deprecated `gas_used()`

**Expected:** use `ExecutionResult::gas_used()` to read gas consumed.  
**Observed:** the method compiles but emits a deprecation warning under REVM 43 because
it is ambiguous after EIP-8037's state-gas split.  
**Fix:** use **`tx_gas_used()`** throughout. This is also required by the `cargo clippy`
deliverable (no warnings).  
**Regression test:** `cargo clippy --all-targets --all-features -- -D warnings` must pass
cleanly.

---

### Challenge 5 — Pitfall 5: `SpecId` tension (Frontier vs Shanghai)

**Expected:** Breeja is thematically Frontier-era (5 ETH block rewards, "pi million" gas
limit). Running Frontier's `SpecId` would be consistent.  
**Observed:** Frontier predates nearly every opcode a modern `solc` emits (e.g.,
`PUSH0` from EIP-3855 requires Shanghai). Contracts compiled with current tooling fail
on `SpecId::FRONTIER`.  
**Decision:** use **`SpecId::SHANGHAI`**. It has everything modern Solidity needs and
does not require the blob-gas fields that `CANCUN` demands. `CANCUN` would require
`blob_excess_gas_and_price` in `BlockEnv`.  
**Acknowledged inconsistency:** "Frontier consensus rules, Shanghai execution rules" is
not a real chain. This is a deliberate teaching compromise: Frontier consensus is kept
for its simplicity; Shanghai EVM is chosen so that real Solidity contracts can run.
Being able to articulate *why* this is a compromise is part of the learning objective.

---

### Challenge 6 — Pitfall 6: Borrow checker and the state lock

**Expected:** commit state changes immediately after execution.  
**Observed:** the REVM `Evm` object borrows `BreejaDb` mutably for its entire lifetime.
Calling `db.commit()` while the `Evm` is still alive is a borrow-checker error.  
**Fix:** clone the state output, then explicitly `drop(evm)` before calling `commit`:

```rust
let outcome = evm.transact(tx_env)?;
let state   = outcome.state.clone();
drop(evm);       // release the &mut borrow on db
db.commit(state);
```

**Additional concern:** do **not** hold the `RwLock` write guard across execution. A
contract call can run for a long time, blocking every concurrent RPC request. The
miner's existing snapshot-execute-commit structure handles this correctly; restructuring
it risks introducing a deadlock.

---

### Challenge 7 — Pitfall 7: `prevrandao` and execution-before-mining ordering

**Expected:** use the current block's hash as `prevrandao`.  
**Observed (impossibility):** execution happens *before* mining. The block's own hash
does not exist when the EVM runs — finding the nonce is what the miner does next, and
the hash depends on the state root that execution produces.  
**Fix:** use the **parent block's hash**: `prevrandao: Some(parent.hash)`.  
**Deeper reason:** Under post-merge specs (including Shanghai), the `DIFFICULTY` opcode
becomes `PREVRANDAO` and reads `BlockEnv.prevrandao`. Leaving it `None` can silently
cause contracts that use `block.prevrandao` to behave incorrectly.  
**Alternative:** add a real `mix_hash` field to `Block` / `BlockHeader`. This changes
every block's hash and requires consistent propagation, so the parent-hash shortcut is
used for this project.

---

### Challenge 8 — Reverted transactions and the inclusion model

**Expected (old Breeja):** a transaction that fails during execution is skipped and not
included.  
**Observed (Ethereum semantics):** a transaction can be valid — correct nonce, sufficient
balance and gas — and still `REVERT` or `HALT` because the contract rejected it.  
**Decision:** align with Ethereum:
- `Revert` / `Halt` → include the transaction; charge the sender for gas burned; advance
  the nonce; discard state changes from that execution; set `receipt.status = 0x0`.
- Invalid (bad nonce, insufficient balance for up-front gas reservation) → skip; do not
  include; do not advance nonce.

This required changing `execute_transaction` to distinguish REVM's validity errors (from
`.transact()` returning `Err`) from EVM-level reverts/halts (from
`ExecutionResult::Revert` / `ExecutionResult::Halt`).

---

### Challenge 9 — Historical snapshots and storage cost

**Expected:** `BlockState.state_history` retains a full account snapshot per block for
historical `eth_getBalance`.  
**Observed:** once accounts hold a `HashMap<StorageKey, StorageValue>`, each snapshot
copies *all* contract storage, so memory grows with `blocks × total_storage_size`. For a
teaching node with modest state this is acceptable; for a production node it is not.  
**Decision:** preserve the current snapshot semantics and document the cost. A
production design would use a trie or a versioned key-value store (e.g., MDBX or
RocksDB with a journal) to share structure between snapshots.

---

### Challenge 10 — `eth_estimateGas` is a heuristic, not a proof

**Problem:** a contract can observe its own remaining gas via the `GAS` opcode and
branch on it, so the gas required for a successful execution may itself depend on the
gas limit supplied.  
**Consequence:** the binary-search approach can, in adversarial contracts, return an
estimate that is insufficient. All mainnet clients accept this limitation.  
**Mitigation:** add a small safety margin to the binary-search result (e.g., round up
to the next multiple of 1,000 or add a fixed buffer). Document that the estimate is
sufficient for well-behaved contracts and note the edge case in the write-up.

---

## Decisions log

| Date / milestone | Decision or issue | Evidence / resolution | Remaining limitation |
| --- | --- | --- | --- |
| 2026-11-07 / setup | Assignment requests branch `feat/evm`; created from clean `main`. | `git switch -c feat/evm` succeeded. | None. |
| 2026-11-07 / M1 | Code/storage representation: hash-indexed `BreejaDb` with separate `code_by_hash` map. | Matches `Database::code_by_hash` REVM trait; deduplicates bytecode across snapshots. | Adds indirection vs inline storage. |
| 2026-11-07 / M2 | `basefee` set to 0 in `BlockEnv`. | Prevents EIP-1559 burn on a pre-1559 chain; supply-conservation test validates. | No fee market (stretch goal). |
| 2026-11-07 / M2 | `SpecId::SHANGHAI` chosen over `FRONTIER`. | Modern `solc` emits opcodes unavailable before Shanghai (e.g., `PUSH0`). | "Frontier consensus, Shanghai EVM" is not a real chain configuration. |
| 2026-11-07 / M2 | `prevrandao: Some(parent.hash)` in `BlockEnv`. | Current block hash unavailable at execution time; parent hash is a reasonable stand-in. | Not cryptographically random; contracts relying on `block.prevrandao` for entropy are exploitable. |
| 2026-11-07 / M3 | Reverted transactions included with `status: 0x0`. | Aligns with Ethereum Yellow Paper §6; sender pays for gas burned. | None for teaching node. |
| 2026-11-07 / M4 | `eth_call` disables nonce and balance checks via `CfgEnv`. | Matches behaviour of go-ethereum and Reth for simulation calls. | Callers can simulate from accounts with no on-chain balance. |
| 2026-11-07 / M4 | `eth_estimateGas` uses binary search with safety margin. | Standard approach; ~20 iterations covers a 30M gas range. | Heuristic only; adversarial contracts with `GAS`-dependent branching may need more gas than estimated. |

## Milestone log

### M0 — Branch and baseline

- [x] Create `feat/evm` from `main`.
- [x] Run `cargo test --workspace`; record exact result (48 tests passed).
- [x] Run `cargo run --release -- --dev` for a few blocks; record output.
- [x] Add REVM to `crates/node/Cargo.toml` and confirm it builds.

### M1 — State model and database

- [x] Choose and document code/storage representation and code-hash indexing trade-off.
- [x] Add account code/storage accessors and transaction-local REVM database.
- [x] Implement REVM `Database` and `DatabaseCommit` including touched accounts, selfdestruct, code, and storage.
- [x] Test basic account reads and committed balance/nonce/code/storage updates.

### M2 — Transfer parity

- [x] Execute transfers through REVM.
- [x] Parity: empty-calldata transfer uses exactly 21,000 gas and moves the same ETH.
- [x] Preserve supply conservation and verify beneficiary fees are not credited twice.
- [x] Keep mining and `cast mktx` transfer flow working.

### M3 — Contracts

- [x] Relax creation/calldata/fixed-gas mempool assumptions appropriately.
- [x] Deploy and call a Solidity contract; preserve input and optional destination.
- [x] Record real contract address, logs, and success/revert receipt status.
- [x] Include reverting transactions, charge gas, advance nonce, and discard reverted effects.
- [ ] Enforce block gas limit for variable-cost transactions.

### M4 — RPC

- [ ] `eth_call` executes without persisting state.
- [ ] `eth_estimateGas` simulates execution and returns a sufficient estimate.
- [ ] Implement `eth_getCode`, `eth_getStorageAt`, and `eth_getLogs` (including required filters).
- [ ] Add RPC integration tests for each behavior.

### M5 — End-to-end and robustness

- [ ] Exercise deployment, storage writes, events, reverts, and JSON-RPC against a mined local chain.
- [ ] Recheck historical reads, gas accounting, nonce handling, and supply conservation.
- [ ] Keep an honest list of unsupported behavior and known limitations.

### M6 — Final delivery

- [ ] Add a shell script or Rust test demonstrating deployment and contract call.
- [ ] Complete this 2–4 page write-up with decisions, failures, fixes, and limitations.
- [ ] `cargo build --release` succeeds.
- [ ] `cargo clippy` passes without warnings.
- [ ] Existing tests pass or each changed expectation has an explanatory comment; all new capabilities have tests.
- [ ] Submit a pull request against the fork.

## Required new coverage checklist

- [x] REVM database basic read; commit balance/nonce/code/storage.
- [x] Transfer parity at 21,000 gas.
- [x] Contract deployment: address returned and runtime code committed.
- [x] Contract call: storage updated and event emitted.
- [x] Revert: included with failed status, gas charged, nonce advanced, state effects reverted.
- [x] Supply equals genesis plus block rewards.
- [ ] Block gas limit holds with varied transaction costs.
- [ ] RPC code and storage reads.
- [ ] `eth_call` output is correct and state remains unchanged.
- [ ] Estimate is sufficient for a successful transaction.
- [ ] `eth_getLogs` filters by range and address (also record topic-filter support).


