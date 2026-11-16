use alloy::primitives::{Address, B256, Bytes, U256};
use jsonrpsee::{
    core::RpcResult,
    types::{ErrorObjectOwned, Params, error::ErrorCode},
};
use primitives::{
    constants::CHAIN_ID,
    node::BlockState,
    params::{BlockTag, CallRequest},
    response::{
        BlockResponse, TransactionReceiptResponse, TransactionResponse, data_hash, logs_bloom,
        quantity,
    },
};
use serde_json::Value;

use crate::RPCContext;

/// JSON-RPC error code Ethereum clients use for "your request was understood but
/// is not valid", as opposed to a malformed one.
const INVALID_INPUT: i32 = -32000;

/// Turns an internal error into a JSON-RPC error object the client can read.
fn invalid_input(error: impl std::fmt::Display) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(INVALID_INPUT, error.to_string(), None::<()>)
}

fn invalid_params(error: impl std::fmt::Display) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        ErrorCode::InvalidParams.code(),
        error.to_string(),
        None::<()>,
    )
}

/// Parses a 32-byte hash parameter.
fn parse_hash(raw: &str) -> Result<B256, ErrorObjectOwned> {
    raw.parse::<B256>()
        .map_err(|e| invalid_params(format!("invalid 32-byte hash {raw:?}: {e}")))
}

/// Resolves a block tag against the chain head, erroring on a block that does not
/// exist rather than silently substituting the latest one.
fn resolve_tag(state: &BlockState, tag: &BlockTag) -> Result<u64, ErrorObjectOwned> {
    tag.resolve(state.get_block_number()).map_err(invalid_input)
}

fn simulation_snapshot(state: &BlockState, tag: Option<&BlockTag>, miner: Address) -> Result<(u64, std::collections::HashMap<Address, primitives::node::Account>), ErrorObjectOwned> {
    let number = tag.map(|t| resolve_tag(state, t)).transpose()?.unwrap_or(state.get_block_number());
    let mut accounts = state.accounts_at(number).ok_or_else(|| invalid_input("unknown block state"))?;
    if matches!(tag, Some(BlockTag::Pending)) {
        let mut used = 0u64;
        for tx in state.peek_mempool_batch() {
            if tx.gas > primitives::constants::BLOCK_GAS_LIMIT.saturating_sub(used) { continue; }
            if let Ok(result) = node::executor::execute_transaction(&mut accounts, &tx, number + 1, miner, B256::ZERO) {
                used += result.gas_used;
            }
        }
        return Ok((number + 1, accounts));
    }
    Ok((number, accounts))
}


/// The chain's id, used by wallets for EIP-155 replay protection.
pub fn chain_id_handler() -> String {
    quantity(CHAIN_ID)
}


/// The number of the chain head. A freshly started node sits at `0x0`, the genesis
/// block, and climbs from there as the miner seals blocks.
pub fn block_number_handler(context: &RPCContext) -> String {
    quantity(context.chain.read().get_block_number())
}


/// Renders a block, inlining full transaction objects if the caller asked for them.
fn render_block(state: &BlockState, number: u64, full_transactions: bool) -> Value {
    let Some(block) = state.get_block(number) else {
        return Value::Null;
    };

    let transactions: Vec<Value> = if full_transactions {
        block
            .transactions
            .iter()
            .filter_map(|hash| state.get_transaction(hash))
            .map(|tx| {
                serde_json::to_value(TransactionResponse::from(tx))
                    .expect("a transaction response always serializes")
            })
            .collect()
    } else {
        block
            .transactions
            .iter()
            .map(|hash| Value::String(data_hash(hash)))
            .collect()
    };

    // A block's bloom is the union of its receipts' blooms.
    let mut bloom = alloy::primitives::Bloom::ZERO;
    for receipt in block.transactions.iter().filter_map(|hash| state.get_receipt(hash)) {
        bloom.accrue_bloom(&logs_bloom(&receipt.logs));
    }

    serde_json::to_value(BlockResponse::from_block(block, transactions, bloom))
        .expect("a block response always serializes")
}

pub fn get_block_by_number_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<Value> {
    // Ethereum's optional trailing parameters have to be read one at a time: a
    // client may send just the block, or the block and the `fullTransactions`
    // flag, and both are valid.
    let mut sequence = params.sequence();
    let tag: BlockTag = sequence.next()?;
    let full_transactions = sequence.optional_next::<bool>()?.unwrap_or(false);

    let state = context.chain.read();
    let number = resolve_tag(&state, &tag)?;
    Ok(render_block(&state, number, full_transactions))
}

pub fn get_block_by_hash_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<Value> {
    let mut sequence = params.sequence();
    let hash = parse_hash(&sequence.next::<String>()?)?;
    let full_transactions = sequence.optional_next::<bool>()?.unwrap_or(false);

    let state = context.chain.read();
    // An unknown hash is `null`, not an error: the caller asked a legitimate
    // question and the answer is "no such block".
    let Some(number) = state.get_block_by_hash(&hash).map(|block| block.number) else {
        return Ok(Value::Null);
    };
    Ok(render_block(&state, number, full_transactions))
}



pub fn get_transaction_by_hash_handler(
    params: Params<'_>,
    context: &RPCContext,
) -> RpcResult<Value> {
    let hash = parse_hash(&params.one::<String>()?)?;

    let state = context.chain.read();
    match state.get_transaction(&hash) {
        Some(tx) => Ok(serde_json::to_value(TransactionResponse::from(tx))
            .expect("a transaction response always serializes")),
        None => Ok(Value::Null),
    }
}

/// A receipt exists only once a transaction has been mined. A transaction still
/// sitting in the mempool correctly returns `null` here, which is exactly how a
/// client polls for confirmation.
pub fn get_transaction_receipt_handler(
    params: Params<'_>,
    context: &RPCContext,
) -> RpcResult<Value> {
    let hash = parse_hash(&params.one::<String>()?)?;

    let state = context.chain.read();
    let Some(receipt) = state.get_receipt(&hash) else {
        return Ok(Value::Null);
    };

    // `logIndex` counts across the whole block, so start after the logs of every
    // earlier transaction in it.
    let first_log_index = state
        .get_block(receipt.block_number)
        .map(|block| {
            block.transactions[..receipt.transaction_index as usize]
                .iter()
                .filter_map(|hash| state.get_receipt(hash))
                .map(|earlier| earlier.logs.len() as u64)
                .sum()
        })
        .unwrap_or(0);

    Ok(
        serde_json::to_value(TransactionReceiptResponse::from_receipt(receipt, first_log_index))
            .expect("a receipt response always serializes"),
    )
}


pub fn get_balance_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<String> {
    let mut sequence = params.sequence();
    let address: Address = sequence.next()?;
    let tag = sequence
        .optional_next::<BlockTag>()?
        .unwrap_or(BlockTag::Latest);

    let state = context.chain.read();
    let balance = match tag {
        // "pending" means "after everything in the mempool lands", which is what a
        // wallet wants before it builds the next transaction.
        BlockTag::Pending => state.get_pending_balance(&address),
        _ => {
            let number = resolve_tag(&state, &tag)?;
            state
                .get_account_at(&address, number)
                .map(|account| account.balance)
                .unwrap_or(U256::ZERO)
        }
    };

    Ok(quantity(balance))
}

/// The account's nonce: how many transactions it has sent.
pub fn get_transaction_count_handler(
    params: Params<'_>,
    context: &RPCContext,
) -> RpcResult<String> {
    let mut sequence = params.sequence();
    let address: Address = sequence.next()?;
    let tag = sequence
        .optional_next::<BlockTag>()?
        .unwrap_or(BlockTag::Latest);

    let state = context.chain.read();
    let nonce = match tag {
        BlockTag::Pending => state.get_pending_nonce(&address),
        _ => {
            let number = resolve_tag(&state, &tag)?;
            state
                .get_account_at(&address, number)
                .map(|account| account.nonce)
                .unwrap_or(0)
        }
    };

    Ok(quantity(nonce))
}


/// Accepts a signed transaction, validates it, and queues it for mining.
///
/// The response is the transaction hash, returned the moment the transaction is
/// *accepted* — not when it is mined. The client then polls
/// `eth_getTransactionReceipt` until a receipt appears, which is the standard
/// Ethereum confirmation flow.
pub fn send_raw_transaction_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<String> {
    let raw: String = params.one()?;
    let raw: Bytes = raw
        .parse()
        .map_err(|e| invalid_params(format!("raw transaction is not valid hex: {e}")))?;

    let hash = context
        .chain
        .submit_raw_transaction(&raw)
        .map_err(invalid_input)?;

    Ok(data_hash(&hash))
}


pub fn call_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<String> {
    let mut sequence = params.sequence();
    let request: CallRequest = sequence.next()?;
    let tag = sequence.optional_next::<BlockTag>()?;

    // Validate the block tag even though the answer does not depend on state, so
    // a caller asking about a nonexistent block gets told.
    if let Some(tag) = tag {
        resolve_tag(&context.chain.read(), &tag)?;
    }

    let state = context.chain.read();
    let (number, snapshot) = simulation_snapshot(&state, tag.as_ref(), context.chain.config().miner)?;
    drop(state);
    let gas = request.gas.unwrap_or(U256::from(primitives::constants::BLOCK_GAS_LIMIT)).min(U256::from(primitives::constants::BLOCK_GAS_LIMIT)).to::<u64>();
    let result = node::executor::simulate_call(&snapshot, &request, number, Address::ZERO, gas).map_err(invalid_input)?;
    if !result.success { return Err(invalid_input("execution reverted or halted")); }
    Ok(format!("0x{}", alloy::hex::encode(result.output)))
}

/// Estimates the gas a transaction would need.
///
/// Without an EVM there is exactly one kind of transaction to estimate, and its
/// cost is a constant: 21,000 gas, the intrinsic cost of a transfer. Anything
/// carrying calldata would need to be executed to be estimated, so it is refused.
pub fn estimate_gas_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<String> {
    let mut sequence = params.sequence();
    let request: CallRequest = sequence.next()?;
    let tag = sequence.optional_next::<BlockTag>()?;

    if let Some(tag) = tag {
        resolve_tag(&context.chain.read(), &tag)?;
    }

    let state = context.chain.read();
    let (number, snapshot) = simulation_snapshot(&state, tag.as_ref(), context.chain.config().miner)?;
    drop(state);
    let max = request.gas.unwrap_or(U256::from(primitives::constants::BLOCK_GAS_LIMIT)).min(U256::from(primitives::constants::BLOCK_GAS_LIMIT)).to::<u64>();
    let run = |gas| node::executor::simulate_call(&snapshot, &request, number, Address::ZERO, gas);
    let top = run(max).map_err(invalid_input)?;
    if !top.success { return Err(invalid_input("execution cannot succeed within block gas limit")); }
    let mut lo = 21_000u64; let mut hi = max;
    for _ in 0..32 { if lo >= hi { break; } let mid = lo + (hi-lo)/2;
        if run(mid).is_ok_and(|r| r.success) { hi=mid; } else { lo=mid+1; }
    }
    Ok(quantity(if hi == 21_000 { hi } else { hi.saturating_add(hi/64).min(max) }))
}

pub fn get_code_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<String> {
    let mut seq = params.sequence();
    let address: Address = seq.next()?;
    let tag = seq.optional_next::<BlockTag>()?.unwrap_or(BlockTag::Latest);
    let state = context.chain.read();
    let (_, snapshot) = simulation_snapshot(&state, Some(&tag), context.chain.config().miner)?;
    let code = snapshot.get(&address).and_then(|a| a.code.as_ref());
    Ok(format!("0x{}", code.map(alloy::hex::encode).unwrap_or_default()))
}

pub fn get_storage_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<String> {
    let mut seq = params.sequence();
    let address: Address = seq.next()?;
    let slot: U256 = seq.next()?;
    let tag = seq.optional_next::<BlockTag>()?.unwrap_or(BlockTag::Latest);
    let state = context.chain.read();
    let (_, snapshot) = simulation_snapshot(&state, Some(&tag), context.chain.config().miner)?;
    let value = snapshot.get(&address).and_then(|a| a.storage.get(&slot)).copied().unwrap_or_default();
    Ok(format!("0x{value:064x}"))
}

pub fn get_logs_handler(params: Params<'_>, context: &RPCContext) -> RpcResult<Vec<Value>> {
    let filter: Value = params.one()?;
    let obj = filter.as_object().ok_or_else(|| invalid_params("filter must be an object"))?;
    if obj.contains_key("blockHash") { return Err(invalid_params("blockHash filters are not supported")); }
    let state = context.chain.read();
    let parse_tag = |key: &str| -> Result<u64, ErrorObjectOwned> {
        let tag = match obj.get(key) { Some(v) => serde_json::from_value::<BlockTag>(v.clone()).map_err(invalid_params)?, None => BlockTag::Latest };
        resolve_tag(&state, &tag)
    };
    let from = parse_tag("fromBlock")?;
    let to = parse_tag("toBlock")?;
    let addresses: Option<Vec<Address>> = match obj.get("address") {
        None | Some(Value::Null) => None,
        Some(Value::Array(a)) => Some(a.iter().map(|v| serde_json::from_value(v.clone()).map_err(invalid_params)).collect::<Result<_,_>>()?),
        Some(v) => Some(vec![serde_json::from_value(v.clone()).map_err(invalid_params)?]),
    };
    let mut topics: Vec<Option<Vec<B256>>> = Vec::new();
    if let Some(v) = obj.get("topics") {
        let entries = v.as_array().ok_or_else(|| invalid_params("topics must be an array"))?;
        if entries.len() > 4 { return Err(invalid_params("at most four topics are allowed")); }
        for entry in entries {
            topics.push(match entry {
                Value::Null => None,
                Value::Array(a) => Some(a.iter().map(|v| serde_json::from_value(v.clone()).map_err(invalid_params)).collect::<Result<_,_>>()?),
                v => Some(vec![serde_json::from_value(v.clone()).map_err(invalid_params)?]),
            });
        }
    }
    let mut output = Vec::new();
    for number in from..=to {
        let Some(block) = state.get_block(number) else { continue };
        let mut log_index = 0u64;
        for hash in &block.transactions {
            let Some(receipt) = state.get_receipt(hash) else { continue };
            for log in &receipt.logs {
                let matches_address = addresses.as_ref().is_none_or(|a| a.contains(&log.address));
                let matches_topics = topics.iter().enumerate().all(|(i, allowed)| log.topics.get(i).is_some_and(|t| allowed.as_ref().is_none_or(|a| a.contains(t))));
                if matches_address && matches_topics {
                    output.push(serde_json::json!({"address":log.address,"topics":log.topics,"data":log.data,
                        "blockNumber":quantity(number),"blockHash":data_hash(&block.hash),"transactionHash":data_hash(hash),
                        "transactionIndex":quantity(receipt.transaction_index),"logIndex":quantity(log_index),"removed":false}));
                }
                log_index += 1;
            }
        }
    }
    Ok(output)
}
