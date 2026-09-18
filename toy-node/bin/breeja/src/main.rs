use alloy::primitives::U256;
use primitives::{constants::DEV_ADDRESS, node::BreejaGenesis};
use rpc::{RPCContext, start_server};

#[tokio::main]
async fn main() {
    let breeja_gen = BreejaGenesis {
        accounts: vec![(DEV_ADDRESS, U256::from(21_000_000))],
    };
    let rpc_context = RPCContext::new(breeja_gen);
    let server_handle = start_server(rpc_context).await.unwrap();
    server_handle.stopped().await;
}
