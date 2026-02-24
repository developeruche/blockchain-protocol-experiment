# Lightweight TCP Wire Protocol for Execution Witnesses

## Abstract

This specification defines a lightweight, connection-oriented TCP wire protocol designed exclusively for high-throughput transfer of Execution Witnesses. The protocol minimizes application-layer overhead by employing an ultra-simple length-prefixed framing mechanism over raw TCP, avoiding the penalization of complex multiplexing or framing headers found in HTTP/2, gRPC, or QUIC.

## Motivation

Benchmarking has demonstrated that Raw TCP significantly outperforms higher-level protocols (HTTP/2, gRPC, QUIC, and even IPC Unix Domain Sockets) in terms of raw throughput for massive payloads on modern OS loopback interfaces. For transferring Execution Witnesses which frequently exceed 100MB and sometimes approach 500MB maximizing single-stream throughput and minimizing Time To First Byte (TTFB) is critical. 

Existing RPC and streaming frameworks introduce unnecessary serialization and flow-control overhead. This specification standardizes a bare-bones TCP wire protocol that retains the performance benefits of raw sockets while providing just enough framing to allow a receiver to safely parse discrete messages.

## Specification

### Connection

The protocol operates over standard TCP. For local communication (e.g., sidecars, local full nodes), `127.0.0.1` or `::1` SHOULD be used to leverage kernel loopback optimizations.

### Message Framing

A single message frame consists of a 1-byte Message Type identifier, an 8-byte Length Prefix, followed by the raw Payload.

| Field | Length | Type | Description |
| --- | --- | --- | --- |
| `message_type` | 1 byte | `u8` | The operation code identifying the payload. |
| `payload_length` | 8 bytes | `u64` | The size of the `payload` in bytes, encoded as Big-Endian. |
| `payload` | Variable | `bytes` | The raw binary data of the message. |

#### Message Types

Currently, only one message type is defined. Future iterations of the protocol MAY introduce new message types.

*   `0x01`: **Execution Witness**. The payload contains the raw byte-encoded execution witness.

### Protocol Limits

*   `MAX_PAYLOAD_SIZE`: `5,368,709,120` bytes (5 GB). Any payload length exceeding this value MUST result in immediate connection termination.

### Serialization Format

All multi-byte integer fields (i.e., `payload_length`) MUST be serialized in **Network Byte Order (Big-Endian)**.

### Pseudo-Code Implementation

The following Python-like pseudo-code illustrates the protocol's encoding and decoding logic.

#### Encoding (Client/Sender)

```python
import struct
import socket

# Message types
MSG_TYPE_EXECUTION_WITNESS = 0x01

def send_message(sock: socket.socket, msg_type: int, payload: bytes):
    """
    Encodes and sends a framed message over a TCP socket.
    """
    payload_length = len(payload)
    
    # Pack the 1-byte message type and 8-byte big-endian payload length
    header = struct.pack(">B Q", msg_type, payload_length)
    
    # Send header followed by payload
    sock.sendall(header)
    sock.sendall(payload)

def send_execution_witness(sock: socket.socket, witness_data: bytes):
    send_message(sock, MSG_TYPE_EXECUTION_WITNESS, witness_data)
```

#### Decoding (Server/Receiver)

```python
import struct
import socket

def read_exactly(sock: socket.socket, num_bytes: int) -> bytes:
    """Read exactly num_bytes from the socket."""
    buf = bytearray(num_bytes)
    view = memoryview(buf)
    while num_bytes > 0:
        n = sock.recv_into(view, num_bytes)
        if n == 0:
            raise EOFError("Socket closed prematurely")
        view = view[n:]
        num_bytes -= n
    return bytes(buf)

def receive_message(sock: socket.socket) -> tuple[int, bytes]:
    """
    Reads and decodes a framed message from a TCP socket.
    Returns a tuple of (msg_type, payload).
    """
    # Read the 9-byte header (1 byte type + 8 bytes length)
    header = read_exactly(sock, 9)
    
    # Unpack big-endian unsigned char (B) and unsigned long long (Q)
    msg_type, payload_length = struct.unpack(">B Q", header)
    
    # Enforce MAXIMUM PAYLOAD SIZE
    if payload_length > (5 * 1024 * 1024 * 1024):
        raise ValueError("Payload length exceeds MAX_PAYLOAD_SIZE of 5GB")
    
    # Read the exact payload length
    payload = read_exactly(sock, payload_length)
    
    return msg_type, payload
```

## Rationale

*   **1-byte Message Type:** Allows for up to 256 distinct message types, which is more than sufficient for the specific domain of execution node and prover communication.
*   **8-byte Payload Length:** A `u64` allows for extremely large payloads. Even though an execution witness is rarely larger than a gigabyte, a `u64` prevents any overflow vulnerabilities and future-proofs the protocol without significant overhead (only 4 bytes larger than a `u32`).
*   **Big-Endian:** Standard network byte order ensures cross-platform compatibility regardless of the CPU architecture (e.g., ARM vs x86).
*   **No Checksum:** TCP already provides a checksum at the transport layer. For localhost loopback transport, adding an application-layer checksum introduces CPU overhead with zero practical benefit regarding data corruption.
*   **No Multiplexing:** To achieve maximum throughput, head-of-line blocking is accepted. If parallel streams are required, multiple TCP connections SHOULD be opened.

## Backwards Compatibility

This is the initial V1 specification. The `message_type` field serves as a multiplexer for future protocol evolution.

## Security Considerations

*   **Denial of Service (DoS):** Because the protocol reads the `payload_length` directly and attempts to read that many bytes, a malicious actor(EL or Witness server) could send a massive `payload_length` (e.g., `0xFFFFFFFFFFFFFFFF`), causing the receiver to allocate an impossible amount of memory (OOM). Implementations MUST enforce a `MAX_PAYLOAD_SIZE` limit of 5 GB and terminate the connection if the requested length exceeds it. Sending a response indicating the error is optional but the underlying TCP connection MUST be forcefully closed to drop the malicious payload.
*   **Timeouts:** Receivers SHOULD implement read and write timeouts to prevent stalled connections from exhausting file descriptors.

## Benchmark Results

The following reference benchmark was executed using the initial Rust implementation located in this repository on an **Apple Silicon M3 Max** running via the macOS loopback interface (`127.0.0.1`).

| Payload Size | Time to First Byte (TTFB) | Total Transfer Time | Throughput |
| :--- | :--- | :--- | :--- |
| **8 MB** | 2.75 µs | 3.90 ms | 2151.55 MB/s |
| **20 MB** | 3.71 µs | 6.15 ms | 3411.97 MB/s |
| **100 MB** | 7.42 µs | 23.41 ms | 4479.96 MB/s |
| **300 MB** | 1.00 µs | 62.79 ms | 5009.60 MB/s |
| **500 MB** | 1.58 µs | 92.66 ms | 5658.37 MB/s |

*TTFB parsing latency is consistently under 10 microseconds thanks to the zero-indirection 9-byte header framing.*

