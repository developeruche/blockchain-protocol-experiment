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

    /// Fetch blocks sequentially from fork-block up to this end block and save them to JSON
    #[arg(long, env = "FETCH_BLOCKS")]
    pub fetch_blocks: Option<u64>,

    /// The provider backend to use for fetching blocks (e.g. "rpc" or "xatu")
    #[arg(long, default_value = "rpc")]
    pub provider: String,

    /// The maximum number of blocks to write sequentially per file
    #[arg(long, default_value_t = 50)]
    pub fetch_interval: u64,

    /// The directory to save/load back block batch files
    #[arg(long, env = "BLOCKS_DIR", default_value = "blocks")]
    pub blocks_dir: String,

    /// Execute all batched block JSON files from the specified directory locally
    #[arg(long, env = "RUN_BLOCKS")]
    pub run_blocks: Option<String>,
}
