# eth-fork-node

`eth-fork-node` is a lightweight, high-performance Rust CLI tool that forks the Ethereum Mainnet at a specific block, allowing for local transaction execution, automated block fetch/replay, and deep EVM inspection.

Built on top of [revm](https://github.com/bluealloy/revm) and [alloy](https://github.com/alloy-rs/alloy), `eth-fork-node` operates similarly to Foundry's Anvil or Hardhat Network but focuses explicitly on granular EVM instruction inspection, such as accurately tracking and profiling cryptographic precompile invocations during transaction execution.

## Features

- **Mainnet Forking**: Forks Ethereum state dynamically from an upstream RPC provider using a lazy-fetch caching mechanism.
- **Local JSON-RPC Server**: Exposes a localized `:8545` barebones RPC server supporting core methods (`eth_call`, `eth_sendRawTransaction`, `eth_getBalance`, `eth_getTransactionCount`, `eth_getCode`, `eth_getStorageAt`).
- **Batch Block Fetching & Execution**: Automatically downloads sequences of mainnet blocks into local JSON files and replays all the encapsulated transactions sequentially over the simulated state.
- **Precompile Introspection**: Seamlessly wires `revm-inspector` hooks into the execution pipeline using `inspect_tx_commit`. It intercepts sub-calls (`STATICCALL`, `CALL`, etc.) to count and log usages of complex precompiles (e.g. `sha256`, `modexp`, `blake2f`, `point_eval`, `identity`, `ripemd160`).
- **State Overlays**: Mutates local state via an memory-based Overlay DB without polluting or requiring a full archive node sync.

## Prerequisites

- Rust 1.80+ installed
- An active Ethereum RPC URL (e.g., Alchemy, Infura, etc.)

## Usage

### 1. Starting the Fork Node

You can boot the RPC server by specifying the upstream provider and the block to fork from:

```bash
cargo run --release -- \
    --rpc-url "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY" \
    --fork-block 24625511 \
    --port 8545
```

Once running, you can connect your tooling (like Foundry `cast` or Ethers.js) directly to `http://localhost:8545`.

### 2. Fetching Blocks

To fetch a chunk of consecutive blocks and save them locally for offline replay:

```bash
cargo run --release -- \
    --rpc-url "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY" \
    --fork-block 24625511 \
    --fetch-blocks 50 \
    --blocks-dir ./blocks
```
This will fetch blocks `24625512` to `24625561` and store them in the `./blocks` directory.

### 3. Replaying Blocks

To replay the previously fetched blocks sequentially against the in-memory state:

```bash
cargo run --release -- \
    --rpc-url "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY" \
    --fork-block 24625511 \
    --run-blocks ./blocks
```
The node will execute every transaction within the block batch, incrementing block indices and updating the base fees automatically.

## Precompile Testing Script

The repository includes an automated integration script (`run_inspector_test.sh`) to verify that the node correctly hooks into the EVM and audits `STATICCALL`s to precompiles.

The script:
1. Boots the fork node in the background.
2. Uses Foundry's `cast` to compile and deploy `src/PrecompileTester.sol`.
3. Invokes the `testAll()` function on the deployed contract, which internally triggers six different cryptographic precompiles.
4. The node prints out a usage statistic block confirming exact interception counts.

**Run the test**:
```bash
./run_inspector_test.sh
```

## Architecture

* `src/provider.rs`: Asynchronous upstream RPC fetching layer powered by `alloy`.
* `src/cache.rs`: In-memory caching for retrieved upstream data (`AccountInfo`, block hashes, etc).
* `src/overlay_db.rs`: A layered mutable database preserving local execution diffs.
* `src/fork_db.rs`: Combines the Provider, Cache, and Overlay into a unified `revm::DatabaseRef`.
* `src/executor.rs`: Configures and manages the `revm` engine (`execute`, `inspect_tx_commit`). Handles `TxEnvelope` decomposition into execution frames.
* `src/inspector.rs`: Implements `revm_inspector::Inspector`. Hooks into EVM call execution frames natively to extract targeted addresses and increment precompile counters.

## License

MIT
