# Lightweight Execution Witness TCP Wire Protocol

This repository implements a dead-simple, highly optimized Raw TCP wire protocol defined for transferring large Execution Witnesses. 

The protocol operates directly over standard TCP with a basic length-prefixed framing header, completely eliminating the overhead of higher multiplexing or crypto architectures (HTTP2/gRPC/QUIC) during localhost node and sidecar communications.

## Framing Format

*   `0x01` (1 byte): Message Type (Execution Witness)
*   `Length` (8 bytes): Unsigned 64-bit integer (`u64`) in Big-Endian.
*   `Payload`: Raw bytes corresponding to the specified `Length`.

See [`spec.md`](./spec.md) for the protocol specification and pseudo-code logic.

## Benchmarks (Apple Silicon M3 Max)

Testing was done transferring payload arrays directly out of static memory across the TCP loopback proxy `127.0.0.1` minimizing OS memory fragmentation. The benchmark bypasses connection handshake limits and purely benchmarks data transmission and framing parsing latency.

To reproduce internally:
```bash
./run_benchmarks.sh
```

### Results
```text
==== Payload Size: 8 MB ====
Wire TCP: TTFB = 2.75µs | Total = 3.90ms | Throughput = 2151.55 MB/s

==== Payload Size: 20 MB ====
Wire TCP: TTFB = 3.71µs | Total = 6.15ms | Throughput = 3411.97 MB/s

==== Payload Size: 100 MB ====
Wire TCP: TTFB = 7.42µs | Total = 23.41ms | Throughput = 4479.96 MB/s

==== Payload Size: 300 MB ====
Wire TCP: TTFB = 1.00µs | Total = 62.79ms | Throughput = 5009.60 MB/s

==== Payload Size: 500 MB ====
Wire TCP: TTFB = 1.58µs | Total = 92.66ms | Throughput = 5658.37 MB/s
```

*Note on TTFB (Time To First Byte): Because a single packet containing the header and first data frame hits the TCP window concurrently, the parsing latency for the client evaluating the framing is on the order of `~1 to 7 µs` (microseconds). The overall throughput peaks roughly at `~5.6 GB/s` where kernel context switching and TCP loop limit optimizations stabilize on the M3 Max loopback architecture.*
