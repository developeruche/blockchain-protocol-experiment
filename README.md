# Blockchain Protocol Experiments

This workspace contains extensive research and experimental implementations for highly optimized, high-throughput network protocols specifically designed for blockchain execution environments (e.g., Ethereum execution nodes, provers, sidecars).

## Overview

The primary focus of this research is identifying and implementing the lowest-latency, highest-throughput transport mechanism for transferring massive payloads—specifically **Execution Witnesses**, which can frequently exceed 100MB and approach 500MB per block. 

By eliminating the overhead of standard HTTP and RPC frameworks on local or high-speed trusted networks, these experiments aim to push gigabytes per second across loopback and LAN interfaces with minimal CPU context switching and zero memory bloat.

## Sub-Projects

### 1. [witness-tcp](./witness-tcp)
A complete specification and implementation of the **Lightweight Execution Witness TCP Wire Protocol**.

*   **What it is:** A bare-bones, connection-oriented raw TCP protocol using a simple 9-byte header framing (`[1 Byte Type] [8 Byte Length]`). 
*   **Why it exists:** Designed to outperform HTTP/2, gRPC, QUIC, and UDS by fully utilizing the OS kernel's native TCP loopback optimizations and zero-copy buffers.
*   **Results:** Reaches **> 6.5 GB/s** throughput with parsing latencies of **< 2µs** on Apple Silicon (M3 Max).
*   **Features:** Includes an EIP-style formal specification (`spec.md`), a highly optimized Rust native chunked streaming server/client, and an equivalent TypeScript `client.ts` implementation operating via the Node.js `net` module.

### 2. [server-experiment](./server-experiment) & [witness-transport](./witness-transport)
The initial benchmarking suites that drove the creation of `witness-tcp`. 

*   These modules contain comprehensive test harnesses that pit raw TCP, HTTP/2 (`hyper`), gRPC (`tonic`), QUIC (`quinn`), and Unix Domain Sockets (IPC) against each other varying payload sizes from 8MB up to 500MB without allocating more than a single 1MB memory footprint.

## Usage

To run the benchmarking suite for the final TCP Wire Protocol implementation:

```bash
cd witness-tcp
./run_benchmarks.sh
```

To run the alternative TypeScript client against the Rust server:

```bash
cd witness-tcp
npm install
npx tsc
node client.js
```
