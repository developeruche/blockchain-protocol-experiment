use crate::framing::{pack_header, unpack_header, HEADER_SIZE, MSG_TYPE_EXECUTION_WITNESS_BY_BLOCK_NUMBER, MSG_TYPE_EXECUTION_WITNESS_BY_BLOCK_HASH, MSG_TYPE_REQUEST, MAX_PAYLOAD_SIZE};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn run_benchmarks(sizes_mb: &[u32]) -> anyhow::Result<()> {
    println!("--- WARMUP ---");
    let _ = test_witness_protocol(1).await;
    println!("--- WARMUP COMPLETE ---\n");

    for &size in sizes_mb {
        println!("==== Payload Size: {} MB ====", size);
        test_witness_protocol(size).await?;
        println!();
    }
    Ok(())
}

async fn test_witness_protocol(size_mb: u32) -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8005").await?;
    
    let req_payload = size_mb.to_be_bytes();
    let req_header = pack_header(MSG_TYPE_REQUEST, req_payload.len() as u64);
    
    stream.write_all(&req_header).await?;
    stream.write_all(&req_payload).await?;

    let mut header_buf = [0u8; HEADER_SIZE];
    stream.read_exact(&mut header_buf).await?;
    let (msg_type, payload_len) = unpack_header(&header_buf);

    if msg_type != MSG_TYPE_EXECUTION_WITNESS_BY_BLOCK_NUMBER && msg_type != MSG_TYPE_EXECUTION_WITNESS_BY_BLOCK_HASH {
        return Err(anyhow::anyhow!("Unexpected response message type: {}", msg_type));
    }

    if payload_len > MAX_PAYLOAD_SIZE {
        return Err(anyhow::anyhow!("Server responded with payload length {} exceeding MAX_PAYLOAD_SIZE {}", payload_len, MAX_PAYLOAD_SIZE));
    }

    // Timer starts as soon as we begin draining the execution witness payload
    let start = Instant::now();
    let mut buf = [0u8; 8192];
    let mut first_byte = None;
    let mut total_read = 0u64;

    while total_read < payload_len {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        if first_byte.is_none() {
            first_byte = Some(start.elapsed());
        }
        total_read += n as u64;
    }
    
    let elapsed = start.elapsed();
    print_results("Wire TCP", size_mb, first_byte.unwrap_or_default(), elapsed, total_read as usize);
    Ok(())
}

fn print_results(
    name: &str,
    _size_mb: u32,
    ttfb: std::time::Duration,
    total_time: std::time::Duration,
    bytes: usize,
) {
    let mb = bytes as f64 / 1_000_000.0;
    let throughput = mb / total_time.as_secs_f64();
    println!(
        "{}: TTFB = {:>6.2?} | Total = {:>6.2?} | Throughput = {:>7.2} MB/s",
        name, ttfb, total_time, throughput
    );
}
