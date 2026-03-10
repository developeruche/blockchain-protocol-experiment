pub mod cli;
pub mod provider;
pub mod cache;
pub mod overlay_db;
pub mod fork_db;
pub mod block_env;
pub mod executor;
pub mod rpc_server;
pub mod inspector;
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
    if let Some(end_fetch_block) = cli.fetch_blocks {
        if end_fetch_block <= cli.fork_block {
            tracing::error!("Fetch end block must be between fork_block + 1 and infinity");
            return Ok(());
        }
        tracing::info!("Fetching blocks from {} to {} (interval: {})", cli.fork_block + 1, end_fetch_block, cli.fetch_interval);
        std::fs::create_dir_all(&cli.blocks_dir)?;
        
        let mut current_block = cli.fork_block + 1;
        
        while current_block <= end_fetch_block {
            let chunk_end = std::cmp::min(current_block + cli.fetch_interval - 1, end_fetch_block);
            let file_path = format!("{}/n{}-{}.json", cli.blocks_dir, current_block, chunk_end);
            
            let mut blocks = Vec::new();
            for target_block in current_block..=chunk_end {
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
            
            current_block = chunk_end + 1;
        }
        
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
    if let Some(dir_path) = cli.run_blocks {
        tracing::info!("Running batched blocks from directory: {}", dir_path);
        
        let mut files: Vec<_> = std::fs::read_dir(&dir_path)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
            .collect();
            
        // Sort files to ensure chronological sequence based on filename
        files.sort();
        
        for file_path in files {
            let contents = std::fs::read_to_string(&file_path)?;
            let blocks: Vec<alloy::rpc::types::Block> = serde_json::from_str(&contents)?;
            
            tracing::info!("Loaded {} blocks from {:?} to execute sequentially.", blocks.len(), file_path);
            
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
                    let total_txs = txs.len();
                    for (i, tx) in txs.into_iter().enumerate() {
                        tracing::info!("Executing tx {}/{} - Hash: {:?}", i + 1, total_txs, tx.inner.tx_hash());
                        if let Err(e) = exec.execute_alloy_transaction(tx) {
                            tracing::error!("Failed to execute transaction: {:?}", e);
                        }
                        
                        // Interim precompile stats print
                        if (i + 1) % 50 == 0 {
                            tracing::info!("--- INTERIM PRECOMPILE STATS (Tx {}) ---", i + 1);
                            let counts = &exec.inspector.counts;
                            if counts.is_empty() {
                                tracing::info!("No target precompiles executed yet.");
                            } else {
                                for (name, count) in counts {
                                    tracing::info!("{}: {} calls", name, count);
                                }
                            }
                            tracing::info!("----------------------------------------");
                        }
                    }
                } else {
                    tracing::error!("Block {} did not contain full transaction objects in JSON", number);
                }
            }
        }
        
        tracing::info!("Batch execution completed successfully.");
        
        tracing::info!("--- PRECOMPILE USAGE STATISTICS ---");
        let exec = executor.read().await;
        let counts = &exec.inspector.counts;
        if counts.is_empty() {
            tracing::info!("No target precompiles were executed during this batch.");
        } else {
            for (name, count) in counts {
                tracing::info!("{}: {} calls", name, count);
            }
        }
        tracing::info!("-----------------------------------");
        
        return Ok(());
    }

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cli.port));
    tracing::info!("Starting RPC server on {}", addr);
    rpc_server::start_server(addr, executor).await?;

    Ok(())
}
