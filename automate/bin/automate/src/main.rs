use alloy::network::EthereumWallet;
use alloy::providers::{ProviderBuilder, WsConnect};
use alloy::signers::local::PrivateKeySigner;
use clap::Parser;
use primitives::config::ConfigLoader;
use primitives::contract::AutomateContract;
use tasks::executor::{ExecutorConfig, ExecutorTask, NonceManager};
use primitives::shared::{Metrics, NoOpPersistence, ProviderPool, SharedState, State};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex, RwLock};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use tasks::trigger::TriggerTask;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.example.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    info!("Starting Automation Engine...");

    let args = Args::parse();
    let config = match ConfigLoader::load(&args.config) {
        Ok(c) => c,
        Err(errors) => {
            error!("Failed to load config:");
            for e in errors {
                error!(" - {}", e);
            }
            std::process::exit(1);
        }
    };

    info!("Loaded config with {} automations", config.automations.len());

    let pk_var = &config.global.private_key_env;
    let pk_hex = std::env::var(pk_var).unwrap_or_else(|_| {
        warn!("Private key env var {} not found. Using dummy key for testing.", pk_var);
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
    });

    let signer: PrivateKeySigner = pk_hex.parse().unwrap_or_else(|e| {
        error!("Failed to parse private key: {}", e);
        std::process::exit(1);
    });
    let sender_address = signer.address();
    info!("Signer address: {}", sender_address);

    let wallet = EthereumWallet::from(signer);

    let exec_http_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(config.global.rpc_http.parse()?);

    let read_http_provider = ProviderBuilder::new()
        .on_http(config.global.rpc_http.parse()?)
        .boxed();
    
    let ws = WsConnect::new(&config.global.rpc_ws);
    let read_ws_provider = ProviderBuilder::new()
        .on_ws(ws)
        .await?
        .boxed();

    let provider_pool = Arc::new(ProviderPool {
        http: read_http_provider,
        ws: read_ws_provider,
    });

    let shared_state: SharedState = Arc::new(RwLock::new(State {
        running: true,
        execution_count: 0,
    }));

    let metrics = Arc::new(Metrics::default());
    let persistence = Arc::new(NoOpPersistence);
    
    let (error_bus_tx, mut error_bus_rx) = tokio::sync::broadcast::channel(256);
    
    let contract = AutomateContract::new(config.global.automate_contract, exec_http_provider);
    let nonce_manager = Arc::new(Mutex::new(NonceManager::new()));

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut handles = Vec::new();

    for automation in &config.automations {
        info!("Setting up automation `{}`", automation.id);

        let (tx, rx) = mpsc::channel(100);
        let calldata = alloy::primitives::Bytes::from_static(b"");

        let trigger = TriggerTask {
            automation_id: automation.id.clone(),
            strategy: automation.trigger.clone(),
            calldata,
            tx,
            error_bus: error_bus_tx.clone(),
            shared_state: shared_state.clone(),
            shutdown: shutdown_rx.clone(),
            provider_pool: provider_pool.clone(),
            metrics: metrics.clone(),
        };

        let executor_config = ExecutorConfig {
            max_retries: 3,
            gas_limit: config.global.gas_limit,
            max_fee_per_gas: config.global.max_fee_per_gas,
            max_priority_fee: config.global.max_priority_fee,
            base_backoff_ms: 1000,
        };

        let executor = ExecutorTask {
            automation_id: automation.id.clone(),
            rx,
            contract: contract.clone(),
            nonce_manager: nonce_manager.clone(),
            config: executor_config,
            sender_address,
            error_bus: error_bus_tx.clone(),
            shared_state: shared_state.clone(),
            metrics: metrics.clone(),
            shutdown: shutdown_rx.clone(),
            persistence: persistence.clone(),
        };

        let th = trigger.spawn();
        let eh = executor.spawn::<alloy::transports::http::Http<reqwest::Client>>();

        info!("Spawned tasks for automation `{}`", automation.id);
        handles.push(th);
        handles.push(eh);
    }

    let error_listener_handle = tokio::spawn(async move {
        loop {
            match error_bus_rx.recv().await {
                Ok(err) => {
                    error!("Global Error Bus: {}", err);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Error bus lagged by {} messages", n);
                }
            }
        }
    });
    handles.push(error_listener_handle);

    match tokio::signal::ctrl_c().await {
        Ok(()) => info!("Received Ctrl-C, initiating shutdown..."),
        Err(err) => error!("Unable to listen for shutdown signal: {}", err),
    }

    let _ = shutdown_tx.send(true);

    info!("Waiting for tasks to finish (up to 5s)...");
    let join_all = futures_util::future::join_all(handles);
    
    match tokio::time::timeout(std::time::Duration::from_secs(5), join_all).await {
        Ok(_) => info!("All tasks finished gracefully."),
        Err(_) => warn!("Forcing shutdown after timeout."),
    }

    Ok(())
}

