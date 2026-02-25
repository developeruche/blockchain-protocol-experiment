import * as net from 'net';
import { performance } from 'perf_hooks';

// Protocol Message Types
const MSG_TYPE_REQUEST = 0x00;
const MSG_TYPE_EXECUTION_WITNESS_BY_BLOCK_NUMBER = 0x01;
const MSG_TYPE_EXECUTION_WITNESS_BY_BLOCK_HASH = 0x02;

const HEADER_SIZE = 9; // 1 byte type + 8 bytes length
const MAX_PAYLOAD_SIZE = 5n * 1024n * 1024n * 1024n; // 5 GB

function packHeader(msgType: number, length: bigint): Buffer {
    const buf = Buffer.alloc(HEADER_SIZE);
    buf.writeUInt8(msgType, 0);
    buf.writeBigUInt64BE(length, 1);
    return buf;
}

function unpackHeader(buf: Buffer): { msgType: number; payloadLen: bigint } {
    const msgType = buf.readUInt8(0);
    const payloadLen = buf.readBigUInt64BE(1);
    return { msgType, payloadLen };
}

async function testWitnessProtocol(sizeMb: number): Promise<void> {
    return new Promise((resolve, reject) => {
        const client = new net.Socket();

        let headerBuf = Buffer.alloc(0);
        let payloadLen: bigint | null = null;
        let msgType: number | null = null;
        let totalRead = 0n;

        let startTime: number;
        let firstByteTime: number | null = null;

        client.connect(8005, '127.0.0.1', () => {
            // 1. Send Request
            const reqPayload = Buffer.alloc(4);
            reqPayload.writeUInt32BE(sizeMb, 0); // Requested size in MB

            const reqHeader = packHeader(MSG_TYPE_REQUEST, BigInt(reqPayload.length));
            client.write(reqHeader);
            client.write(reqPayload);
        });

        client.on('data', (rawChunk) => {
            const chunk = rawChunk as Buffer;
            // Unpack exactly the header first
            if (payloadLen === null) {
                headerBuf = Buffer.concat([headerBuf, chunk]);
                if (headerBuf.length >= HEADER_SIZE) {
                    const header = headerBuf.subarray(0, HEADER_SIZE);
                    const unpacked = unpackHeader(header);
                    msgType = unpacked.msgType;
                    payloadLen = unpacked.payloadLen;

                    if (msgType !== MSG_TYPE_EXECUTION_WITNESS_BY_BLOCK_NUMBER && msgType !== MSG_TYPE_EXECUTION_WITNESS_BY_BLOCK_HASH) {
                        client.destroy(new Error(`Unexpected response message type: ${msgType}`));
                        return;
                    }

                    if (payloadLen > MAX_PAYLOAD_SIZE) {
                        client.destroy(new Error(`Server responded with payload length ${payloadLen} exceeding MAX_PAYLOAD_SIZE ${MAX_PAYLOAD_SIZE}`));
                        return;
                    }

                    // Timer starts as soon as we begin draining the execution witness payload
                    startTime = performance.now();

                    const remainingChunk = headerBuf.subarray(HEADER_SIZE);
                    if (remainingChunk.length > 0) {
                        if (firstByteTime === null) {
                            firstByteTime = performance.now() - startTime;
                        }
                        totalRead += BigInt(remainingChunk.length);
                    }
                }
            } else {
                if (firstByteTime === null) {
                    firstByteTime = performance.now() - startTime;
                }
                totalRead += BigInt(chunk.length);
            }

            if (payloadLen !== null && totalRead >= payloadLen) {
                const totalTime = performance.now() - startTime;
                printResults("Wire TCP (TS)", sizeMb, firstByteTime ?? 0, totalTime, Number(totalRead));
                client.destroy();
                resolve();
            }
        });

        client.on('error', (err: Error) => {
            reject(err);
        });
    });
}

function printResults(name: string, sizeMb: number, ttfbMs: number, totalTimeMs: number, bytes: number) {
    const mb = bytes / 1_000_000.0;
    const throughput = mb / (totalTimeMs / 1000.0);

    // Format TTFB
    let ttfbStr = "";
    if (ttfbMs < 1) {
        ttfbStr = `${(ttfbMs * 1000).toFixed(2)}µs`;
    } else {
        ttfbStr = `${ttfbMs.toFixed(2)}ms`;
    }

    // Format Total Time
    let totalTimeStr = "";
    if (totalTimeMs < 1) {
        totalTimeStr = `${(totalTimeMs * 1000).toFixed(2)}µs`;
    } else if (totalTimeMs < 1000) {
        totalTimeStr = `${totalTimeMs.toFixed(2)}ms`;
    } else {
        totalTimeStr = `${(totalTimeMs / 1000).toFixed(2)}s`;
    }

    console.log(`${name}: TTFB = ${ttfbStr.padStart(8)} | Total = ${totalTimeStr.padStart(8)} | Throughput = ${throughput.toFixed(2).padStart(7)} MB/s`);
}

async function runBenchmarks(sizesMb: number[]) {
    console.log("--- WARMUP ---");
    try {
        await testWitnessProtocol(1);
    } catch (e) {
        console.error("Warmup failed:", e);
        return;
    }
    console.log("--- WARMUP COMPLETE ---\n");

    for (const size of sizesMb) {
        console.log(`==== Payload Size: ${size} MB ====`);
        try {
            await testWitnessProtocol(size);
        } catch (e) {
            console.error(`Benchmark failed for size ${size}:`, e);
            break;
        }
        console.log();
    }
}

const defaultSizes = [8, 20, 100, 300, 500];

async function main() {
    console.log(`Running TS TCP benchmarks for sizes: [${defaultSizes.join(", ")}]`);
    await runBenchmarks(defaultSizes);
}

// Execute if run directly
if (require.main === module) {
    main().catch(console.error);
}
