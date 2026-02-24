use crate::framing::{pack_header, unpack_header, HEADER_SIZE, MSG_TYPE_EXECUTION_WITNESS, MSG_TYPE_REQUEST, MAX_PAYLOAD_SIZE};
use crate::payload::{get_chunk, CHUNK_SIZE};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn run_server() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8005").await?;
    tracing::info!("TCP Wire Protocol Server listening on 127.0.0.1:8005");

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut header_buf = [0u8; HEADER_SIZE];
            if socket.read_exact(&mut header_buf).await.is_ok() {
                let (msg_type, payload_len) = unpack_header(&header_buf);
                if msg_type == MSG_TYPE_REQUEST {
                    // Extract requested size in MB from payload
                    let mut payload_buf = vec![0u8; payload_len as usize];
                    if socket.read_exact(&mut payload_buf).await.is_ok() {
                        let size_mb = u32::from_be_bytes(payload_buf[..4].try_into().unwrap());
                        let bytes_to_send = (size_mb as u64) * 1024 * 1024;
                        
                        if bytes_to_send > MAX_PAYLOAD_SIZE {
                            tracing::error!("Requested payload size {} exceeds MAX_PAYLOAD_SIZE {}", bytes_to_send, MAX_PAYLOAD_SIZE);
                            return; // Terminate connection
                        }
                        
                        // Send Execution Witness Header
                        let resp_header = pack_header(MSG_TYPE_EXECUTION_WITNESS, bytes_to_send);
                        if socket.write_all(&resp_header).await.is_ok() {
                            // Stream the chunked payload
                            let chunk = get_chunk();
                            let mut sent = 0;
                            while sent < bytes_to_send {
                                let to_send = std::cmp::min(CHUNK_SIZE as u64, bytes_to_send - sent);
                                if socket.write_all(&chunk[..(to_send as usize)]).await.is_err() {
                                    break;
                                }
                                sent += to_send;
                            }
                        }
                    }
                }
            }
        });
    }
}
