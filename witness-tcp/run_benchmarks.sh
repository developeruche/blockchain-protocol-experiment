#!/bin/bash
set -e

# Build the release binary
echo "Building release binary..."
cargo build --release

# Start the server in the background
echo "Starting protocol servers..."
./target/release/witness-tcp server &
SERVER_PID=$!

sleep 1

# Run the client tests and capture into a file
echo "Running client benchmarks..."
./target/release/witness-tcp client > benchmark_output.txt
cat benchmark_output.txt

# Kill the server
echo "Cleaning up..."
kill $SERVER_PID
echo "Done."
