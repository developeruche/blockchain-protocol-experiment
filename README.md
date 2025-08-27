# Pod Network

### Under active development, doing this for fun!!

An opinionated implementation of the **Pod** consensus protocol, based on the research paper ["Pod: An Optimal-Latency, Censorship-Free, and Accountable Generalized Consensus Layer"](https://arxiv.org/abs/2501.14931).

## Overview

Pod is a novel consensus protocol designed to achieve **optimal latency** of one round-trip for transaction processing while maintaining censorship resistance and accountability. Unlike traditional consensus protocols that require inter-replica communication, Pod eliminates this overhead by having clients send transactions directly to all replicas, which process transactions independently and maintain local logs.

### Key Features

- **Optimal Latency**: Achieves 2δ confirmation time (one network trip for writing, one for reading)
- **Censorship Resistance**: Byzantine fault tolerance against malicious replicas
- **No Inter-Replica Communication**: Eliminates traditional consensus overhead
- **Accountability**: Safety violations are detectable and attributable
- **EVM Compatibility**: Supports Ethereum Virtual Machine transactions

## Architecture

The Pod Network implementation consists of several key components:

### Core Components

- **`pod-core`**: Core consensus primitives and protocol implementation
- **`tx-processor`**: Transaction processing and execution engine
- **`replica`**: Replica node implementation
- **`client`**: Client implementation for transaction submission and reading
- **`smart-contracts`**: Solidity SDK and verification contracts

### Protocol Flow

1. **Write Phase**: Clients send transactions directly to all replicas
2. **Processing**: Each replica independently assigns timestamps and sequence numbers
3. **Logging**: Replicas append transactions to their local logs with metadata
4. **Read Phase**: Clients retrieve logs from replicas and extract transaction traces
5. **Consensus**: Clients compute consensus view using timestamp analysis

## Getting Started

### Prerequisites

- Rust 1.80+ (2024 edition)
- Foundry (for smart contracts)

### Building

```bash
# Clone the repository
git clone <repository-url>
cd pod-network

# Build all components
cargo build

# Build specific components
cargo build -p pod-core
cargo build -p replica
cargo build -p client
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p pod-core
```

### Smart Contracts

The smart contract components use Foundry for development:

```bash
cd smart-contracts

# Install dependencies
forge install

# Build contracts
forge build

# Run tests
forge test
```

## Protocol Parameters

Pod uses several key parameters for fault tolerance and performance:

- **n**: Total number of replicas
- **α (alpha)**: Minimum number of honest replicas needed
- **β (beta)**: Maximum number of Byzantine replicas tolerated
- **γ (gamma)**: Additional safety parameter

The protocol maintains safety with `n = 2β + α + γ` replicas.

## Usage

### Running a Replica

```bash
cargo run --bin replica -- --id <replica_id> --port <port>
```

### Running a Client

```bash
cargo run --bin client -- --replicas <replica_list>
```

### Transaction Submission

Clients can submit transactions which will be processed by all replicas:

```rust
use pod_core::primitives::{PodTransaction, PodClient};

// Create client
let mut client = PodClient::new(replica_pks, n, beta, gamma, alpha);

// Submit transaction
let tx = PodTransaction::new(/* transaction data */);
// Client handles broadcasting to all replicas

// Read results
let pod_ds = client.read();
for trace in pod_ds.tx_trace {
    println!("Transaction: {:?}", trace.transaction);
    println!("Confirmation round: {:?}", trace.r_conf);
}
```

## Key Data Structures

### PodTransaction
Represents a transaction in the Pod protocol, convertible to EVM transactions.

### PodVote
A replica's vote on a transaction, including timestamp, sequence number, and signature.

### PodTransactionTrace
Contains transaction metadata including minimum round (`r_min`), maximum round (`r_max`), and confirmation round (`r_conf`).

### PodDS (Data Structure)
The client's view of the system state, containing transaction traces and performance metrics.

## Security Properties

Pod provides several important security guarantees:

- **Liveness**: Transactions from honest clients are eventually confirmed
- **Safety**: Confirmed transactions maintain consistency across honest clients  
- **Censorship Resistance**: Byzantine replicas cannot prevent transaction inclusion
- **Accountability**: Equivocation and other safety violations are detectable

## Performance Characteristics

- **Latency**: 2δ optimal confirmation time
- **Throughput**: Scales with replica processing capacity
- **Network Efficiency**: No inter-replica communication overhead
- **Gas Optimization**: Smart contracts use Yul for performance-critical operations

## Research Background

This implementation is based on the Pod protocol described in the paper:

> Alpos, O., David, B., Mitrovski, J., Sofikitis, O., & Zindros, D. (2025). 
> Pod: An Optimal-Latency, Censorship-Free, and Accountable Generalized Consensus Layer. 
> arXiv preprint arXiv:2501.14931.

The protocol addresses fundamental limitations of existing blockchain systems by eliminating inter-replica communication while maintaining strong security properties.

## Applications

Pod is particularly well-suited for:

- **Single-shot Auctions**: Natural fit for the protocol's properties
- **High-frequency Trading**: Benefits from optimal latency
- **Decentralized Finance**: Censorship resistance and accountability
- **Real-time Applications**: Minimal confirmation delays

## Contributing

We welcome contributions! Please see our contributing guidelines and ensure all tests pass before submitting pull requests.

## License

This project is licensed under [LICENSE] - see the LICENSE file for details.

## Citation

If you use this implementation in research or educational pourpose, please cite the original paper:

```bibtex
@article{alpos2025pod,
  title={Pod: An Optimal-Latency, Censorship-Free, and Accountable Generalized Consensus Layer},
  author={Alpos, Orestis and David, Bernardo and Mitrovski, Jakov and Sofikitis, Odysseas and Zindros, Dionysis},
  journal={arXiv preprint arXiv:2501.14931},
  year={2025}
}
```
