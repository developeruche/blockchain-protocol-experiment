# Automate

A production-grade Rust automation engine designed to monitor on-chain/off-chain trigger conditions and seamlessly submit transactions to the `AutomateRegistry` smart contract locally or on testnets/mainnets. 

The workspace is organized into multiple independent crates:
- `bin/automate` - The main binary runner
- `crates/primitives` - Shared global types, data structures, and ABIs (bindings)
- `crates/tasks` - The actual `ExecutorTask` and `TriggerTask` loops that perform evaluations

## Running the Integration Test

To verify the Trigger and Executor tasks evaluate and mine correctly on an Ethereum node (e.g., Anvil), you can run the integration test via the CLI using a test configuration file. 

### Prerequisites
1. Ensure you have [Foundry/Anvil](https://getfoundry.sh/) installed.
2. Open a terminal and start Anvil inside the `contract` directory:
   ```bash
   cd contract
   anvil
   ```
3. Open a separate terminal and deploy the smart contracts to your local Anvil instance:
   ```bash
   cd contract
   PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 forge script script/DeployAnvil.s.sol --rpc-url http://127.0.0.1:8545 --broadcast
   ```
   *Note: This script will print a generated `config.test.toml` that has the robust identifiers matching the newly deployed smart contracts.*

### Execute the Rust Engine

You can start the engine by running the `automate` binary at the workspace root, making sure to inject the corresponding deployer `PRIVATE_KEY` so the executor is authenticated to call the `automate_contract`:

```bash
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 cargo run --bin automate -- --config config.test.toml
```

If everything is configured correctly, after the configured timeout completes (e.g. 2000ms), you should see output resembling:
```
[INFO] automate: Setting up automation `0xfa9c5a0839d519f000b2e9be188b8ebc85a884041df25bd0d9fc085fae2f51cc`
[INFO] trigger: Trigger fired
[INFO] executor: Executing JOB_ID natively: 0xfa9c5a0839d519f000b2e9be188b8ebc85a884041df25bd0d9fc085fae2f51cc
[INFO] executor: Execution successful: tx 0xcb7ad8df0c7bbd8f3e8b26d0a917de05eb2a5b794e1...
```
