use alloy::primitives::{Address, address};

pub const DIFFICULTY_LEADING_ZEROS: u8 = 7; // We get a new block every 15 secs
pub const DEV_DIFFICULTY_LEADING_ZEROS: u8 = 3; // On dev mode, we get a new block every 2 secs
pub const RPC_ADDRESS: &str = "127.0.0.1:8545";
pub const DUMMY_BLOCK_HASH: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
pub const DUMMY_TX_HASH: &str =
    "0x1111111111111111111111111111111111111111111111111111111111111111";
pub const DEV_ADDRESS: Address = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
