use anyhow::Result;
use hyper::{Request, body::Bytes};
use hyper_util::rt::TokioIo;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::net::UnixStream;

#[derive(Debug, Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<String>,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    jsonrpc: String,
    result: String,
    id: u64,
}

const SECRET_KEY: &[u8] = b"super_secret_key";
const SOCKET_PATH: &str = "/tmp/benchmark.sock";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting benchmark client...");

    // Generate JWT token
    let claims = Claims {
        sub: "benchmark_user".to_owned(),
        exp: 10000000000, // far future
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY))?;

    let sizes = ["100mb", "300mb", "500mb"];

    for size in sizes {
        run_benchmark_for_size(size, &token).await?;
        // Small delay between runs to let things settle
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn run_benchmark_for_size(size: &str, token: &str) -> Result<()> {
    let stream = UnixStream::connect(SOCKET_PATH).await?;
    let io = TokioIo::new(stream);
    
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            tracing::error!("Connection failed: {:?}", err);
        }
    });

    let request_body = serde_json::to_string(&RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "get_data".to_string(),
        params: vec![size.to_string()],
        id: 1,
    })?;

    let req = Request::builder()
        .method("POST")
        .uri("http://localhost/") 
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(http_body_util::Full::new(Bytes::from(request_body)))?;

    // START TIMER
    let start = Instant::now();

    let res = sender.send_request(req).await?;

    let body_bytes = http_body_util::BodyExt::collect(res.into_body()).await?.to_bytes();
    
    let response: RpcResponse = serde_json::from_slice(&body_bytes)?;
    
    // STOP TIMER (after deserialization is complete)
    let duration = start.elapsed();

    // Verify data length roughly matches (base64 overhead ~33%)
    // 100MB * 1.33 = ~133MB
    let received_len = response.result.len();
    tracing::info!("Size: {}, RTT: {:?}, Received payload len: {} bytes", size, duration, received_len);

    Ok(())
}
