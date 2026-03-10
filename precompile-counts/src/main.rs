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

    // Block Fetching Mode
    if let Some(num_blocks) = cli.fetch_blocks {
        tracing::info!("Fetching {} blocks starting from {}", num_blocks, cli.fork_block);
        std::fs::create_dir_all(&cli.blocks_dir)?;
        
        let end_block = cli.fork_block + num_blocks;
        let file_path = format!("{}/n{}-{}.json", cli.blocks_dir, cli.fork_block + 1, end_block);
        
        let mut blocks = Vec::new();
        for i in 1..=num_blocks {
            let target_block = cli.fork_block + i;
            tracing::info!("Fetching block {}...", target_block);
            if let Some(block) = provider.get_full_block_by_number(target_block).await? {
                blocks.push(block);
            } else {
                tracing::warn!("Block {} not found on upstream provider!", target_block);
                break;
            }
        }
        
        let json = serde_json::to_string_pretty(&blocks)?;
        std::fs::write(&file_path, json)?;
        tracing::info!("Successfully saved {} blocks to {}", blocks.len(), file_path);
        
        return Ok(());
    }

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

    // Batch Execution Mode
    if let Some(file_path) = cli.run_blocks {
        tracing::info!("Running batched blocks from file: {}", file_path);
        let contents = std::fs::read_to_string(&file_path)?;
        let blocks: Vec<alloy::rpc::types::Block> = serde_json::from_str(&contents)?;
        
        tracing::info!("Loaded {} blocks to execute sequentially.", blocks.len());
        
        for block in blocks {
            let number = block.header.number;
            tracing::info!("Executing {} transactions in block {}", block.transactions.len(), number);
            
            let mut exec = executor.write().await;
            
            // Sync the executor's block env context with the current batch block
            exec.block_env.inner.number = alloy::primitives::U256::from(number);
            exec.block_env.inner.timestamp = alloy::primitives::U256::from(block.header.timestamp);
            exec.block_env.inner.beneficiary = block.header.beneficiary;
            exec.block_env.inner.basefee = block.header.base_fee_per_gas.unwrap_or_default() as u64;
            
            if let alloy::rpc::types::BlockTransactions::Full(txs) = block.transactions {
                for tx in txs {
                    if let Err(e) = exec.execute_alloy_transaction(tx) {
                        tracing::error!("Failed to execute transaction: {:?}", e);
                    }
                }
            } else {
                tracing::error!("Block {} did not contain full transaction objects in JSON", number);
            }
        }
        
        tracing::info!("Batch execution completed successfully.");
        return Ok(());
    }

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cli.port));
    tracing::info!("Starting RPC server on {}", addr);
    rpc_server::start_server(addr, executor).await?;

    Ok(())
}
