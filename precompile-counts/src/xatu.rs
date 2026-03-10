use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::RowAccessor;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde_json::json;

const XATU_BASE_URL: &str = "https://data.ethpandaops.io/xatu/mainnet/databases/default";

pub async fn fetch_blocks_from_xatu(
    start_block: u64,
    end_block: u64,
    interval: u64,
    blocks_dir: &str,
) -> eyre::Result<()> {
    tracing::info!("Fetching blocks {} to {} using Xatu Parquet Provider", start_block, end_block);
    
    if !Path::new(blocks_dir).exists() {
        fs::create_dir_all(blocks_dir)?;
    }
    
    let mut current_block = start_block;
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    
    while current_block <= end_block {
        let fetch_end = std::cmp::min(current_block + interval - 1, end_block);
        
        let chunk_base = (current_block / 1000) * 1000;
        let mut all_blocks_in_chunk: HashMap<u64, serde_json::Value> = HashMap::new();
        
        let block_url = format!("{}/canonical_execution_block/1000/{}.parquet", XATU_BASE_URL, chunk_base);
        tracing::info!("Downloading Blocks Parquet from {}", block_url);
        let block_bytes = client.get(&block_url).send().await?.bytes().await?;
        let block_reader = SerializedFileReader::new(block_bytes)?;
        
        for row in block_reader.get_row_iter(None)? {
            let row = row?;
            let b_num = row.get_ulong(2).unwrap_or(0) as u64;
            if b_num < current_block || b_num > fetch_end { continue; }
            
            let ts = row.get_ulong(1).unwrap_or(0) as u64;
            
            let mut block_hash = String::from("0x0000000000000000000000000000000000000000000000000000000000000000");
            if let Ok(hash_bytes) = row.get_bytes(3) {
                if let Ok(hash_str) = std::str::from_utf8(hash_bytes.data()) {
                    block_hash = hash_str.to_string();
                }
            }
            
            let mut miner = String::from("0x0000000000000000000000000000000000000000");
            if let Ok(author_bytes) = row.get_bytes(4) {
                if let Ok(author_str) = std::str::from_utf8(author_bytes.data()) {
                    miner = author_str.to_string();
                }
            }
            
            let gas_used = row.get_ulong(5).unwrap_or(0) as u64;
            let base_fee = row.get_ulong(8).unwrap_or(0) as u64;
            
            let block_json = json!({
                "hash": block_hash,
                "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                "stateRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "transactionsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "receiptsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "difficulty": "0x0",
                "number": format!("0x{:x}", b_num),
                "gasLimit": "0x1c9c380",
                "gasUsed": format!("0x{:x}", gas_used),
                "timestamp": format!("0x{:x}", ts),
                "extraData": "0x",
                "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "nonce": "0x0000000000000000",
                "baseFeePerGas": format!("0x{:x}", base_fee),
                "uncles": [],
                "miner": miner,
                "transactions": []
            });
            all_blocks_in_chunk.insert(b_num, block_json);
        }
        
        tracing::info!("Successfully extracted {} block headers from Parquet chunk {}", all_blocks_in_chunk.len(), chunk_base);

        let tx_url = format!("{}/canonical_execution_transaction/1000/{}.parquet", XATU_BASE_URL, chunk_base);
        tracing::info!("Downloading Transactions Parquet from {}", tx_url);
        let tx_bytes = client.get(&tx_url).send().await?.bytes().await?;
        let tx_reader = SerializedFileReader::new(tx_bytes)?;
        
        let mut tx_map: HashMap<u64, Vec<serde_json::Value>> = HashMap::new();
        for row in tx_reader.get_row_iter(None)? {
            let row = row?;
            let b_num = row.get_ulong(1).unwrap_or(0) as u64;
            if b_num < current_block || b_num > fetch_end { continue; }
            
            let tx_idx = row.get_ulong(2).unwrap_or(0) as u64;
            let nonce = row.get_ulong(4).unwrap_or(0) as u64;
            
            let mut tx_hash = String::from("0x0000000000000000000000000000000000000000000000000000000000000000");
            if let Ok(hash_bytes) = row.get_bytes(3) {
                if let Ok(hash_str) = std::str::from_utf8(hash_bytes.data()) {
                    tx_hash = hash_str.to_string();
                }
            }
            
            let mut from_address = String::from("0x0000000000000000000000000000000000000000");
            if let Ok(from_bytes) = row.get_bytes(5) {
                if let Ok(from_str) = std::str::from_utf8(from_bytes.data()) {
                    from_address = from_str.to_string();
                }
            }
            
            let mut to_address = serde_json::Value::Null;
            if let Ok(to_bytes) = row.get_bytes(6) {
                if let Ok(to_str) = std::str::from_utf8(to_bytes.data()) {
                    if to_str.len() > 10 {
                        to_address = json!(to_str.to_string());
                    }
                }
            }
            
            let mut value_hex = String::from("0x0");
            if let Ok(value_bytes) = row.get_bytes(7) {
                let bytes = value_bytes.data();
                if bytes.len() == 32 {
                    // It's little endian bytes representation in Clickhouse for UInt256
                    // Convert to hex (big endian for EVM U256)
                    let mut le_bytes = [0u8; 32];
                    le_bytes.copy_from_slice(bytes);
                    let mut u256_be = [0u8; 32];
                    for i in 0..32 {
                        u256_be[i] = le_bytes[31 - i];
                    }
                    let u256 = alloy::primitives::U256::from_be_bytes(u256_be);
                    value_hex = format!("0x{:x}", u256);
                }
            }
            
            let mut input_hex = String::from("0x");
            if let Ok(input_bytes) = row.get_bytes(8) {
                if let Ok(input_str) = std::str::from_utf8(input_bytes.data()) {
                    input_hex = input_str.to_string();
                }
            }
            
            let gas_limit = row.get_ulong(9).unwrap_or(0) as u64;
            let gas_price = row.get_ulong(11).unwrap_or(0) as u128;
            let tx_type = row.get_uint(12).unwrap_or(0) as u8;
            
            let max_priority_fee_per_gas = row.get_ulong(13).unwrap_or(0) as u128;
            let max_fee_per_gas = row.get_ulong(14).unwrap_or(0) as u128;
            
            let mut tx_json = json!({
                "hash": tx_hash,
                "nonce": format!("0x{:x}", nonce),
                "blockHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "blockNumber": format!("0x{:x}", b_num),
                "transactionIndex": format!("0x{:x}", tx_idx),
                "from": from_address,
                "to": to_address,
                "value": value_hex,
                "gas": format!("0x{:x}", gas_limit),
                "gasPrice": format!("0x{:x}", gas_price),
                "input": input_hex,
                "type": format!("0x{:x}", tx_type),
                "v": "0x0",
                "r": "0x0",
                "s": "0x0",
                "chainId": "0x1"
            });
            
            if tx_type == 1 || tx_type == 2 || tx_type == 3 {
                tx_json.as_object_mut().unwrap().insert("accessList".to_string(), json!([]));
            }
            
            // Note: Alloy expects "maxFeePerGas" in type 2 and type 3 txs
            if tx_type == 2 || tx_type == 3 {
                tx_json.as_object_mut().unwrap().insert("maxFeePerGas".to_string(), json!(format!("0x{:x}", max_fee_per_gas)));
                tx_json.as_object_mut().unwrap().insert("maxPriorityFeePerGas".to_string(), json!(format!("0x{:x}", max_priority_fee_per_gas)));
            }

            // Alloy expects blob fields for type 3 txs
            if tx_type == 3 {
                tx_json.as_object_mut().unwrap().insert("maxFeePerBlobGas".to_string(), json!("0x0"));
                tx_json.as_object_mut().unwrap().insert("blobVersionedHashes".to_string(), json!([]));
            }

            tx_map.entry(b_num).or_default().push(tx_json);
        }
        
        let mut blocks_arr = Vec::new();
        for target_block in current_block..=fetch_end {
            if let Some(mut b) = all_blocks_in_chunk.remove(&target_block) {
                if let Some(mut txs) = tx_map.remove(&target_block) {
                    // Sorting happens natively anyway, but index is preserved in the array
                    b.as_object_mut().unwrap().insert("transactions".to_string(), json!(txs));
                }
                blocks_arr.push(b);
            }
        }
        
        let file_path = format!("{}/n{}-{}.json", blocks_dir, current_block, fetch_end);
        let json_str = serde_json::to_string_pretty(&blocks_arr)?;
        std::fs::write(&file_path, json_str)?;
        tracing::info!("Successfully saved {} blocks from Xatu to {}", blocks_arr.len(), file_path);
        
        current_block = fetch_end + 1;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        use alloy::rpc::types::{Block, Transaction};
        let json_str = std::fs::read_to_string("blocks/n21500000-21500049.json").unwrap();
        let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        
        let blocks = val.as_array().unwrap();
        for (i, block_val) in blocks.iter().enumerate() {
            let res: Result<Block<Transaction>, _> = serde_json::from_value(block_val.clone());
            if let Err(e) = res {
                println!("Error in block index {}: {}", i, e);
                println!("BLOCK JSON: {}", serde_json::to_string_pretty(&block_val).unwrap());
                panic!("JSON parse failed!");
            }
        }
        println!("All blocks parsed!");
    }
}
