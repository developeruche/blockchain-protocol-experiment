#!/bin/bash
set -e

# Build the release binary
echo "Building release binary..."
cargo build --release

# Start the server in the background
echo "Starting protocol servers on ports 8001-8004..."
./target/release/server-experiment server &
SERVER_PID=$!

# Give servers a moment to bind to ports
sleep 2

# Run the client tests
echo "Running client benchmarks..."
# The sizes are default (8,20,100,300,500)
./target/release/server-experiment client

# Kill the server
echo "Cleaning up..."
kill $SERVER_PID
echo "Done."
