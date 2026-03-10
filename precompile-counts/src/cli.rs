use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "A local Ethereum fork node using REVM", long_about = None)]
pub struct Cli {
    /// Upstream HTTP RPC URL to fork from
    #[arg(long, env = "RPC_URL")]
    pub rpc_url: String,

    /// Block number to fork from
    #[arg(long, env = "FORK_BLOCK")]
    pub fork_block: u64,

    /// RPC HTTP Server port
    #[arg(long, default_value_t = 8545)]
    pub port: u16,

    /// Chain ID (optional, defaults to the upstream chain ID if not provided)
    #[arg(long)]
    pub chain_id: Option<u64>,
}
