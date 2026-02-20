use clap::{Parser, Subcommand};
use tracing::{info, Level};

mod client;
mod payload;
mod pb;
mod server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the servers
    Server,
    /// Run the benchmarking client
    Client {
        #[arg(short, long, default_value = "8,20,100,300,500")]
        sizes_mb: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Server => {
            info!("Starting all test servers...");
            server::run_all().await?;
        }
        Commands::Client { sizes_mb } => {
            let sizes: Vec<u32> = sizes_mb
                .split(',')
                .map(|s| s.trim().parse().unwrap())
                .collect();
            info!("Running benchmarks for sizes: {:?}", sizes);
            client::run_benchmarks(&sizes).await?;
        }
    }

    Ok(())
}
