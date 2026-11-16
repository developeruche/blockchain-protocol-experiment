# Breeja Node Architecture (Before EVM Integration)

This guide describes the current implementation in this repository. Breeja is a teaching Ethereum-style proof-of-work node: it accepts signed legacy ETH transfers, mines blocks locally, stores account and block history in memory, and exposes a JSON-RPC server. Contract execution is not implemented yet.

## 1. The major pieces

```text
wallet / cast
    │ signed legacy transaction bytes
    ▼
JSON-RPC server (crates/rpc)
    │ eth_sendRawTransaction
    ▼
Chain::submit_raw_transaction (crates/node/chain.rs)
    │ decode, validate, reserve pending funds
    ▼
mempool (BlockState)
    │ snapshot account state + candidate transactions
    ▼
executor (crates/node/executor.rs)
    │ apply transfer, charge fee, pay block reward
    ▼
miner + proof of work (miner.rs, pow.rs)
    │ seal header, then commit state/block/receipts
    ▼
in-memory chain state (crates/primitives/src/node.rs)
    ▲
    └── JSON-RPC reads blocks, receipts, balances, and nonces
```

The central shared object is `Chain`, which wraps `BlockState` in a read/write lock. RPC handlers read it and add transactions; the miner executes snapshots and writes the resulting block back.

## 2. Startup and shutdown

The binary entry point is `bin/breeja/src/main.rs`. It parses options, configures logging and mining parameters, creates a genesis state, starts the miner on a dedicated OS thread, and starts the async JSON-RPC server.

```rust
let genesis = genesis();
let chain = Chain::new(genesis.clone(), config);
let miner_thread = miner::spawn(chain.clone());
let (server, _address) =
    start_server(RPCContext::new(chain.clone()), &options.rpc_address).await?;

tokio::signal::ctrl_c().await?;
chain.shutdown();
let _ = server.stop();
let _ = miner_thread.join();
```

`Chain::new` creates the genesis block and initializes shared state. The miner is kept off Tokio's worker threads because proof-of-work searching is CPU-intensive. On Ctrl-C, shutdown is signaled, the RPC server is stopped, and the miner thread is joined.

Genesis currently pre-funds four public development accounts. This is for local learning and testing; their keys are deliberately listed in the README and are not for real funds.

## 3. Shared chain state and concurrency

`Chain` is a cloneable handle to shared state and configuration:

```rust
pub struct Chain {
    state: Arc<RwLock<BlockState>>,
    config: Arc<ChainConfig>,
    shutdown: Arc<AtomicBool>,
    mempool_generation: Arc<AtomicU64>,
}
```

- `Arc` lets the miner and RPC server own handles to the same chain.
- `RwLock` permits concurrent reads while ensuring writes such as transaction admission and block commits are exclusive.
- `shutdown` lets the main thread stop the miner.
- `mempool_generation` increments when a transaction arrives. The miner watches it to abandon an outdated proof-of-work candidate and rebuild with the new transaction.

The miner deliberately avoids holding the state lock during execution and hashing. It takes copies of the account map, parent block, and candidate transactions while briefly locked, works on those local values, then reacquires a write lock only to commit. This keeps RPC queries responsive while mining.

## 4. Transactions: decode, validate, queue

`eth_sendRawTransaction` receives signed hex bytes. The handler parses them and calls `Chain::submit_raw_transaction`, which delegates to `pool::admit`:

```rust
pub fn submit_raw_transaction(&self, raw: &Bytes) -> Result<B256, anyhow::Error> {
    let hash = pool::admit(&mut self.write(), raw)?;
    self.mempool_generation.fetch_add(1, Ordering::Release);
    Ok(hash)
}
```

The pool decodes the EIP-2718 envelope, accepts only legacy transactions, recovers the sender from the signature, checks the chain ID, and checks nonce, gas price, and available funds. In this pre-EVM version it also rejects contract creation and any calldata, and assumes the minimum gas is 21,000.

Admission reserves the maximum possible cost, not just the value:

```rust
let max_cost = tx.value + U256::from(tx.gas) * U256::from(tx.gas_price);
let available = state.get_pending_balance(&tx.from);
if available < max_cost {
    anyhow::bail!("insufficient funds ...");
}
```

This prevents multiple pending transactions from spending the same balance. `get_pending_nonce` counts queued transactions from an address; `get_pending_balance` subtracts their reserved value and gas costs. The accepted transaction is stored in the mempool and its hash is returned immediately. It is not mined yet, so its receipt is still `null`.

## 5. Execution and accounting

`execute_block` receives an account snapshot and a batch from the mempool. Today its only transaction operation is `apply_transfer`:

```rust
let fee = U256::from(TRANSFER_GAS) * U256::from(tx.gas_price);
let total = tx.value.checked_add(fee).ok_or_else(...)?;
let sender = accounts.entry(tx.from).or_default();
sender.debit(total)?;
sender.increase_nonce();
accounts.entry(tx.to).or_default().credit(tx.value)?;
accounts.entry(miner).or_default().credit(fee)?;
```

The sender pays the ETH value plus a fee, the recipient receives the value, and the miner receives the fee. The transaction consumes the sender's nonce. Because transfer execution has no contract code, every accepted transfer burns exactly 21,000 gas. The block reward is credited separately after transactions; currently it is the only newly issued ETH.

The executor creates mined transaction and receipt records. A receipt includes gas used, cumulative gas used, effective gas price, and success status. In the current implementation all included transfers succeed. Execution errors are logged and skipped rather than included. That policy must change when the EVM is added, because valid EVM transactions can revert or halt after inclusion and still consume gas.

## 6. Mining and proof of work

The miner loop in `crates/node/src/miner.rs` roughly does the following:

1. Take a brief snapshot of the current parent, accounts, mempool generation, and candidate batch.
2. Execute the batch and calculate transaction, receipt, and state roots.
3. Build a block header with a timestamp, gas limit, miner address, and roots.
4. Search for a nonce whose header hash meets the configured leading-zero difficulty.
5. If a new transaction arrived or shutdown began, abandon the search and rebuild.
6. If the parent is still the chain head, commit the sealed block and execution results.
7. Adjust the next block's difficulty toward the target block time.

The core proof-of-work search is intentionally small:

```rust
for nonce in 0..=u64::MAX {
    header.nonce = nonce;
    let hash = header.hash();
    if meets_difficulty(&hash, difficulty) {
        return Some(Seal { nonce, hash, ... });
    }
}
```

`BlockHeader::hash` hashes the RLP-encoded header. `pow::verify` recomputes that hash and checks the difficulty condition. The chain in this exercise is mined by one local thread; peer-to-peer networking and validation of blocks from other nodes are outside the current architecture.

After sealing, the miner stamps the now-known block hash into transaction and receipt records, then calls `BlockState::commit_block`. The commit removes mined transactions from the mempool, replaces current accounts, indexes the block and its transactions/receipts, and stores a historical state snapshot.

## 7. State model and roots

The main data structures live in `crates/primitives/src/node.rs`. An account currently holds:

```rust
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: B256,
    pub code_hash: B256,
}
```

`storage_root` and `code_hash` exist as fields, but there are no code bytes or contract storage maps yet. `BlockState` stores current accounts, blocks, full per-block account snapshots, and maps of transactions and receipts, as well as the mempool.

This teaching implementation computes roots by sorting addresses and hashing the encoded accounts; transaction and receipt roots hash ordered lists of hashes. These are deterministic commitments to the data, but they are not Ethereum Merkle-Patricia trie roots and cannot provide inclusion proofs. Full state snapshots make historical balance and nonce queries straightforward, at the cost of copying account state for every block. Adding contract storage will increase that memory cost.

## 8. JSON-RPC layer

`crates/rpc/src/lib.rs` registers JSON-RPC method names and binds the server. Each handler receives `RPCContext`, which contains a clone of `Chain`.

Current methods include chain ID, block number, block lookup by number/hash, transaction and receipt lookup, balance, transaction count, raw transaction submission, `eth_call`, and `eth_estimateGas`.

Historical `eth_getBalance` resolves a block tag and reads that block's saved snapshot. For the `pending` tag it reports the head balance minus pending reservations. `eth_getTransactionCount` similarly returns the mined nonce or a pending nonce including queued transactions.

`eth_call` and `eth_estimateGas` are placeholders constrained by the lack of an EVM. A call with no calldata returns `0x`; anything that needs contract execution returns an error. Gas estimation returns 21,000 for a plain transfer and rejects calls requiring code.

## 9. Current limits and EVM integration boundary

The current node already has transaction signature recovery, chain IDs, accounts, a mempool, block production, proof of work, snapshots, receipts, and JSON-RPC. The main execution boundary is `execute_block` / `apply_transfer`. Integrating REVM will require widening the account and transaction models, adding a database adapter over a miner-owned state snapshot, applying REVM's state changes, and making gas, receipts, logs, reverts, and RPC behavior reflect actual execution.

Several assumptions are transfer-specific today: required recipient, empty transaction input, constant transfer gas, no logs or contract address in receipts, and skipped execution failures. These are useful orientation points before starting the project milestones.
