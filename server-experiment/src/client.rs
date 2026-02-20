use crate::pb::benchmark::benchmark_service_client::BenchmarkServiceClient;
use crate::pb::benchmark::DownloadRequest;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Request, body::Incoming};
use hyper_util::rt::TokioIo;
use quinn::{ClientConfig, Endpoint};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UnixStream};

pub async fn run_benchmarks(sizes_mb: &[u32]) -> anyhow::Result<()> {
    println!("--- WARMUP ---");
    let _ = test_tcp(1).await;
    let _ = test_ipc(1).await;
    let _ = test_h2(1).await;
    let _ = test_grpc(1).await;
    let _ = test_quic(1).await;
    println!("--- WARMUP COMPLETE ---\n");

    for &size in sizes_mb {
        println!("==== Payload Size: {} MB ====", size);
        test_tcp(size).await?;
        test_ipc(size).await?;
        test_h2(size).await?;
        test_grpc(size).await?;
        test_quic(size).await?;
        println!();
    }
    Ok(())
}

async fn test_tcp(size_mb: u32) -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8001").await?;
    let start = Instant::now();
    stream.write_all(&size_mb.to_be_bytes()).await?;

    let mut buf = [0u8; 8192];
    let mut first_byte = None;
    let mut total_read = 0;
    let bytes_to_read = (size_mb as usize) * 1024 * 1024;

    while total_read < bytes_to_read {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        if first_byte.is_none() {
            first_byte = Some(start.elapsed());
        }
        total_read += n;
    }
    let elapsed = start.elapsed();
    print_results("Raw TCP", size_mb, first_byte.unwrap_or_default(), elapsed, total_read);
    Ok(())
}

async fn test_ipc(size_mb: u32) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect("/tmp/benchmark.sock").await?;
    let start = Instant::now();
    stream.write_all(&size_mb.to_be_bytes()).await?;

    let mut buf = [0u8; 8192];
    let mut first_byte = None;
    let mut total_read = 0;
    let bytes_to_read = (size_mb as usize) * 1024 * 1024;

    while total_read < bytes_to_read {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        if first_byte.is_none() {
            first_byte = Some(start.elapsed());
        }
        total_read += n;
    }
    let elapsed = start.elapsed();
    print_results("IPC    ", size_mb, first_byte.unwrap_or_default(), elapsed, total_read);
    Ok(())
}

async fn test_h2(size_mb: u32) -> anyhow::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8002").await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http2::handshake(hyper_util::rt::TokioExecutor::new(), io).await?;
    tokio::spawn(async move {
        if let Err(err) = conn.await {
            tracing::error!("Connection failed: {:?}", err);
        }
    });

    let start = Instant::now();
    let req = Request::builder()
        .uri(format!("http://127.0.0.1:8002/download/{}", size_mb))
        .body(http_body_util::Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;

    let mut first_byte = None;
    let mut total_read = 0;

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(data) = frame.data_ref() {
            if first_byte.is_none() {
                first_byte = Some(start.elapsed());
            }
            total_read += data.len();
        }
    }
    let elapsed = start.elapsed();
    print_results("HTTP/2 ", size_mb, first_byte.unwrap_or_default(), elapsed, total_read);
    Ok(())
}

async fn test_grpc(size_mb: u32) -> anyhow::Result<()> {
    let mut client = BenchmarkServiceClient::connect("http://127.0.0.1:8003").await?;

    let start = Instant::now();
    let req = tonic::Request::new(DownloadRequest { size_mb });
    let mut stream = client.download_data(req).await?.into_inner();

    let mut first_byte = None;
    let mut total_read = 0;
    while let Some(chunk) = stream.message().await? {
        if first_byte.is_none() {
            first_byte = Some(start.elapsed());
        }
        total_read += chunk.data.len();
    }

    let elapsed = start.elapsed();
    print_results("gRPC   ", size_mb, first_byte.unwrap_or_default(), elapsed, total_read);
    Ok(())
}

struct SkipServerVerification;
impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

async fn test_quic(size_mb: u32) -> anyhow::Result<()> {
    let mut crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();
    crypto.alpn_protocols = vec![b"hq-benchmark".to_vec()];

    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(ClientConfig::new(Arc::new(crypto)));

    let conn = endpoint.connect("127.0.0.1:8004".parse()?, "localhost")?.await?;

    let start = Instant::now();
    let (mut tx, mut rx) = conn.open_bi().await?;
    tx.write_all(&size_mb.to_be_bytes()).await?;

    let mut buf = [0u8; 8192];
    let mut first_byte = None;
    let mut total_read = 0;
    let bytes_to_read = (size_mb as usize) * 1024 * 1024;

    while total_read < bytes_to_read {
        let n = rx.read(&mut buf).await?.unwrap_or(0);
        if n == 0 {
            break;
        }
        if first_byte.is_none() {
            first_byte = Some(start.elapsed());
        }
        total_read += n;
    }
    let elapsed = start.elapsed();
    print_results("QUIC   ", size_mb, first_byte.unwrap_or_default(), elapsed, total_read);
    Ok(())
}

fn print_results(
    name: &str,
    size_mb: u32,
    ttfb: std::time::Duration,
    total_time: std::time::Duration,
    bytes: usize,
) {
    // Mathematically accurate MB/s (base 10 - 1,000,000 bytes).
    // The previous implementation used 1,048,576 bytes which is technically MiB/s.
    let mb = bytes as f64 / 1_000_000.0;
    let throughput = mb / total_time.as_secs_f64();
    println!(
        "{}: TTFB = {:>6.2?} | Total = {:>6.2?} | Throughput = {:>7.2} MB/s",
        name, ttfb, total_time, throughput
    );
}
