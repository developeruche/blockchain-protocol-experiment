use crate::payload::{get_chunk, CHUNK_SIZE};
use crate::pb::benchmark::benchmark_service_server::{BenchmarkService, BenchmarkServiceServer};
use crate::pb::benchmark::{DataChunk, DownloadRequest};
use bytes::Bytes;
use http_body_util::StreamBody;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::{Request, Response};
use quinn::{Endpoint, ServerConfig};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UnixListener};

pub async fn run_all() -> anyhow::Result<()> {
    tokio::spawn(run_tcp_server());
    tokio::spawn(run_ipc_server());
    tokio::spawn(run_h2_server());
    tokio::spawn(run_grpc_server());
    tokio::spawn(run_quic_server());
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    Ok(())
}

async fn run_tcp_server() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8001").await?;
    tracing::info!("TCP listening on 8001");
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 4];
            if socket.read_exact(&mut buf).await.is_ok() {
                let size_mb = u32::from_be_bytes(buf);
                let bytes_to_send = (size_mb as usize) * 1024 * 1024;
                let chunk = get_chunk();
                let mut sent = 0;
                while sent < bytes_to_send {
                    let to_send = std::cmp::min(CHUNK_SIZE, bytes_to_send - sent);
                    if socket.write_all(&chunk[..to_send]).await.is_err() {
                        break;
                    }
                    sent += to_send;
                }
            }
        });
    }
}

async fn run_ipc_server() -> anyhow::Result<()> {
    let sock_path = "/tmp/benchmark.sock";
    let _ = std::fs::remove_file(sock_path);
    let listener = UnixListener::bind(sock_path)?;
    tracing::info!("IPC listening on {}", sock_path);
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 4];
            if socket.read_exact(&mut buf).await.is_ok() {
                let size_mb = u32::from_be_bytes(buf);
                let bytes_to_send = (size_mb as usize) * 1024 * 1024;
                let chunk = get_chunk();
                let mut sent = 0;
                while sent < bytes_to_send {
                    let to_send = std::cmp::min(CHUNK_SIZE, bytes_to_send - sent);
                    if socket.write_all(&chunk[..to_send]).await.is_err() {
                        break;
                    }
                    sent += to_send;
                }
            }
        });
    }
}

async fn handle_h2(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<StreamBody<impl futures::Stream<Item = Result<hyper::body::Frame<Bytes>, Infallible>>>>, Infallible> {
    let path = req.uri().path();
    let size_mb: usize = path.trim_start_matches("/download/").parse().unwrap_or(8);
    let bytes_to_send = size_mb * 1024 * 1024;

    let stream = async_stream::stream! {
        let mut sent = 0;
        let b = Bytes::from_static(get_chunk());
        while sent < bytes_to_send {
            let to_send = std::cmp::min(CHUNK_SIZE, bytes_to_send - sent);
            yield Ok::<_, Infallible>(hyper::body::Frame::data(b.slice(0..to_send)));
            sent += to_send;
        }
    };

    Ok(Response::new(StreamBody::new(stream)))
}

async fn run_h2_server() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8002").await?;
    tracing::info!("HTTP/2 listening on 8002");
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let io = hyper_util::rt::TokioIo::new(stream);
            if let Err(err) = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(io, service_fn(handle_h2))
                .await
            {
                tracing::error!("Error serving h2 connection: {:?}", err);
            }
        });
    }
}

#[derive(Default)]
pub struct MyBenchmarkService {}

#[tonic::async_trait]
impl BenchmarkService for MyBenchmarkService {
    type DownloadDataStream = std::pin::Pin<
        Box<dyn tokio_stream::Stream<Item = Result<DataChunk, tonic::Status>> + Send + 'static>,
    >;

    async fn download_data(
        &self,
        request: tonic::Request<DownloadRequest>,
    ) -> Result<tonic::Response<Self::DownloadDataStream>, tonic::Status> {
        let req = request.into_inner();
        let bytes_to_send = (req.size_mb as usize) * 1024 * 1024;

        let stream = async_stream::try_stream! {
            let b = Bytes::from_static(get_chunk());
            let mut sent = 0;
            while sent < bytes_to_send {
                let to_send = std::cmp::min(CHUNK_SIZE, bytes_to_send - sent);
                let data = b.slice(0..to_send);
                // gRPC uses Vec<u8> by default unless instructed otherwise in build.rs. Provide `.to_vec()`
                yield DataChunk { data: data.to_vec() };
                sent += to_send;
            }
        };

        Ok(tonic::Response::new(Box::pin(stream)))
    }
}

async fn run_grpc_server() -> anyhow::Result<()> {
    let addr = "127.0.0.1:8003".parse()?;
    tracing::info!("gRPC listening on 8003");
    tonic::transport::Server::builder()
        .add_service(BenchmarkServiceServer::new(MyBenchmarkService::default()))
        .serve(addr)
        .await?;
    Ok(())
}

async fn run_quic_server() -> anyhow::Result<()> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = cert.serialize_der()?;
    let priv_key = cert.serialize_private_key_der();

    let key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der)];

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)?;
    server_crypto.alpn_protocols = vec![b"hq-benchmark".to_vec()];

    let server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
    let endpoint = Endpoint::server(server_config, "127.0.0.1:8004".parse()?)?;
    tracing::info!("QUIC listening on 8004");

    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(async move {
            let connection = match conn.await {
                Ok(c) => c,
                Err(_) => return,
            };
            while let Ok(mut stream) = connection.accept_bi().await {
                tokio::spawn(async move {
                    let (mut tx, mut rx) = stream;
                    let mut buf = [0u8; 4];
                    if rx.read_exact(&mut buf).await.is_ok() {
                        let size_mb = u32::from_be_bytes(buf);
                        let bytes_to_send = (size_mb as usize) * 1024 * 1024;
                        let chunk = get_chunk();
                        let mut sent = 0;
                        while sent < bytes_to_send {
                            let to_send = std::cmp::min(CHUNK_SIZE, bytes_to_send - sent);
                            if tx.write_all(&chunk[..to_send]).await.is_err() {
                                break;
                            }
                            sent += to_send;
                        }
                    }
                });
            }
        });
    }
    Ok(())
}
