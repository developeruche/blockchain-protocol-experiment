use rpc::start_server;



#[tokio::main]
async fn main() {
    let server_handle = start_server().await.unwrap();
    server_handle.stopped().await;
}
