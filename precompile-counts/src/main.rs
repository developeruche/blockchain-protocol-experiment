pub mod cli;
pub mod provider;
pub mod cache;
pub mod overlay_db;
pub mod fork_db;
pub mod block_env;
pub mod executor;
pub mod rpc_server;
pub mod types;


use std::sync::Arc;
use clap::Parser;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("eth_fork_node=debug".parse()?))
        .init();

    tracing::info!("Starting eth-fork-node");

    let cli = cli::Cli::parse();

    let url = cli.rpc_url.parse()?;
    let alloy_provider = alloy::providers::ProviderBuilder::new().connect_http(url);
    let provider = Arc::new(provider::UpstreamProvider::new(alloy_provider, cli.fork_block));

    let chain_id = if let Some(id) = cli.chain_id {
        id
    } else {
        provider.get_chain_id().await?
    };
    tracing::info!("Chain ID: {}, Fork Block: {}", chain_id, cli.fork_block);

    let cache = Arc::new(cache::ForkCache::new());
    let overlay = Arc::new(overlay_db::OverlayDb::new());

    // Pre-fund the standard Anvil test account for local testing
    let test_addr: alloy::primitives::Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse().unwrap();
    let mut test_acct = overlay_db::LocalAccount::default();
    test_acct.info.balance = alloy::primitives::U256::MAX;
    overlay.accounts.insert(test_addr, test_acct);

    let fork_db = fork_db::ForkDb::new(provider.clone(), cache.clone(), overlay.clone());
    let block_env = block_env::BlockEnvironment::new(provider.clone(), cli.fork_block).await?;

    let executor = Arc::new(tokio::sync::RwLock::new(
        executor::Executor::new(fork_db, block_env, chain_id)
    ));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cli.port));
    tracing::info!("Starting RPC server on {}", addr);
    rpc_server::start_server(addr, executor).await?;

    Ok(())
}
