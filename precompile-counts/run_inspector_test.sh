#!/bin/bash

# Clean up any lingering node
lsof -ti :8545 | xargs kill -9 2>/dev/null || true

echo "Starting eth-fork-node in background..."
RUST_LOG=eth_fork_node=debug cargo run --release -- --rpc-url https://eth-mainnet.g.alchemy.com/v2/diWtHiEFpWCZ8xCz8a6fm --fork-block 24625511 > node.log 2>&1 &
NODE_PID=$!

trap "kill -9 $NODE_PID 2>/dev/null || true" EXIT

echo "Waiting for node to start..."
sleep 8

echo "Creating Forge project..."
mkdir -p /tmp/precompile_test
cd /tmp/precompile_test
if [ ! -f "foundry.toml" ]; then
  forge init --no-git
fi

cat << 'EOF' > src/PrecompileTester.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract PrecompileTester {
    function testAll() external {
        // sha256 (0x02)
        sha256("test");
        
        // ripemd160 (0x03)
        ripemd160("test");
        
        // identity (0x04)
        address identity = address(0x04);
        (bool s1, ) = identity.staticcall("test");
        require(s1, "id fail");
        
        // modexp (0x05)
        address modexp = address(0x05);
        (bool s2, ) = modexp.staticcall(abi.encode(32, 32, 32, 2, 2, 3)); 
        require(s2, "modexp fail");
        
        // blake2f (0x09)
        address blake2f = address(0x09);
        bytes memory blake2fPayload = new bytes(213);
        blake2fPayload[3] = 0x01; // Set rounds to 1
        blake2f.staticcall(blake2fPayload);
        
        // point_eval (0x0a)
        address point_eval = address(0x0a);
        bytes memory pointEvalPayload = new bytes(192);
        point_eval.staticcall(pointEvalPayload);
    }
}
EOF

echo "Building contract..."
forge build

BYTECODE=$(jq -r '.bytecode.object' out/PrecompileTester.sol/PrecompileTester.json)

echo "Deploying contract..."
export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Send the transaction synchronously so it polls for eth_getTransactionReceipt
RECEIPT=$(cast send --rpc-url http://127.0.0.1:8545 --private-key $PRIVATE_KEY --legacy --create $BYTECODE --json)
CONTRACT_ADDR=$(echo "$RECEIPT" | jq -r '.contractAddress')

echo "Deployed to: $CONTRACT_ADDR"

if [ "$CONTRACT_ADDR" != "null" ] && [ -n "$CONTRACT_ADDR" ]; then
    echo "Executing testAll()..."
    cast send $CONTRACT_ADDR "testAll()" --rpc-url http://127.0.0.1:8545 --private-key $PRIVATE_KEY --legacy
else
    echo "Failed to get contract address!"
fi

sleep 1
echo "--- NODE OUTPUT ---"
cat /Users/gregg/Documents/projects/RESEARCH/blockchain-protocol-experiment/precompile-counts/node.log
