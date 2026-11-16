use alloy::consensus::transaction::SignerRecoverable;
use alloy::consensus::{Transaction as TxTrait, TxEnvelope, TxType};
use alloy::eips::Decodable2718;
use alloy::primitives::{Address, B256, Bytes, TxKind, U256};
use primitives::{
    constants::{CHAIN_ID, GAS_PRICE, TRANSFER_GAS},
    node::{BlockState, PendingTransaction},
};

/// Calculates the intrinsic gas cost of a transaction:
/// - 21,000 base cost
/// - 4 gas per zero byte of calldata/initcode
/// - 16 gas per non-zero byte of calldata/initcode
/// - 32,000 additional gas for contract creation (TxKind::Create)
pub fn calculate_intrinsic_gas(input: &[u8], is_create: bool) -> u64 {
    let mut gas = TRANSFER_GAS;
    if is_create {
        gas += 32_000;
    }
    for &byte in input {
        if byte == 0 {
            gas += 4;
        } else {
            gas += 16;
        }
    }
    gas
}

/// A signed transaction that has been decoded and checked against the protocol
/// rules, but not yet against chain state.
struct DecodedTransaction {
    hash: B256,
    from: Address,
    /// `None` for contract creation (`TxKind::Create`).
    to: Option<Address>,
    value: U256,
    nonce: u64,
    gas: u64,
    gas_price: u128,
    chain_id: u64,
    #[allow(dead_code)]
    input: Bytes,
    v: u64,
    r: U256,
    s: U256,
}

/// Decodes a raw signed transaction and applies every rule that can be checked
/// without looking at chain state.
fn decode_and_check(raw: &Bytes) -> Result<DecodedTransaction, anyhow::Error> {
    if raw.is_empty() {
        anyhow::bail!("raw transaction is empty");
    }

    let envelope = TxEnvelope::decode_2718(&mut raw.as_ref())
        .map_err(|e| anyhow::anyhow!("could not RLP-decode the raw transaction: {e}"))?;

    // Breeja predates typed transactions, so it only understands legacy ones.
    // There is no base fee here for an EIP-1559 transaction to bid against.
    if !matches!(envelope.tx_type(), TxType::Legacy) {
        anyhow::bail!(
            "unsupported transaction type {:?}: this node only accepts legacy \
             (pre-EIP-1559) transactions, so sign with --legacy",
            envelope.tx_type()
        );
    }

    // The sender is derived from the signature, never declared.
    let from = envelope
        .recover_signer()
        .map_err(|e| anyhow::anyhow!("invalid signature, could not recover sender: {e}"))?;

    // EIP-155 replay protection: a transaction signed for another chain must not
    // be mineable here. A legacy transaction with no chain id at all predates
    // EIP-155 and is accepted, exactly as on Ethereum.
    let chain_id = match envelope.chain_id() {
        Some(id) if id == CHAIN_ID => id,
        Some(id) => {
            anyhow::bail!("transaction is signed for chain {id}, but this chain is {CHAIN_ID}")
        }
        None => CHAIN_ID,
    };

    let to = match envelope.kind() {
        TxKind::Call(to) => Some(to),
        TxKind::Create => None,
    };

    let input = envelope.input().clone();
    let intrinsic_gas = calculate_intrinsic_gas(&input, to.is_none());

    let gas = envelope.gas_limit();
    if gas < intrinsic_gas {
        anyhow::bail!(
            "gas limit {gas} is below the intrinsic cost ({intrinsic_gas})"
        );
    }

    let gas_price = envelope.max_fee_per_gas();
    if gas_price < GAS_PRICE {
        anyhow::bail!("gas price {gas_price} is below this chain's fixed price of {GAS_PRICE}");
    }

    let signature = envelope.signature();
    Ok(DecodedTransaction {
        hash: *envelope.tx_hash(),
        from,
        to,
        value: envelope.value(),
        nonce: envelope.nonce(),
        gas,
        gas_price,
        chain_id,
        input,
        v: legacy_v(signature.v(), envelope.chain_id()),
        r: signature.r(),
        s: signature.s(),
    })
}

/// Reconstructs the `v` value a legacy transaction reports over JSON-RPC.
///
/// `v` encodes the signature's y-parity, and since EIP-155 it also encodes the
/// chain id, which is how replay protection is folded into the signature itself.
fn legacy_v(y_parity: bool, chain_id: Option<u64>) -> u64 {
    match chain_id {
        Some(id) => y_parity as u64 + 35 + id * 2,
        None => y_parity as u64 + 27,
    }
}

/// Validates a raw transaction against the current chain state and admits it to
/// the mempool, returning its hash.
///
/// The state checks use *pending* values — the nonce and balance including
/// everything already queued — so that a client can fire several transactions
/// back to back without waiting for a block in between.
pub fn admit(state: &mut BlockState, raw: &Bytes) -> Result<B256, anyhow::Error> {
    let tx = decode_and_check(raw)?;

    if state.get_transaction(&tx.hash).is_some() {
        anyhow::bail!("transaction {} has already been mined", tx.hash);
    }
    if state.mempool_contains(&tx.hash) {
        anyhow::bail!("transaction {} is already pending", tx.hash);
    }

    let expected_nonce = state.get_pending_nonce(&tx.from);
    if tx.nonce != expected_nonce {
        anyhow::bail!(
            "nonce too {}: {} expects nonce {expected_nonce}, transaction has {}",
            if tx.nonce < expected_nonce {
                "low"
            } else {
                "high"
            },
            tx.from,
            tx.nonce
        );
    }

    // Ethereum reserves the full gas limit up front and refunds whatever the
    // transaction does not burn. Breeja always burns exactly the intrinsic cost,
    // but it reserves the limit anyway so the accounting matches.
    let max_cost = tx.value + U256::from(tx.gas) * U256::from(tx.gas_price);
    let available = state.get_pending_balance(&tx.from);
    if available < max_cost {
        anyhow::bail!(
            "insufficient funds: {} has {available} available but the transaction \
             needs up to {max_cost} (value plus gas)",
            tx.from
        );
    }

    state.add_to_mempool(PendingTransaction {
        hash: tx.hash,
        from: tx.from,
        to: tx.to,
        value: tx.value,
        nonce: tx.nonce,
        gas: tx.gas,
        gas_price: tx.gas_price,
        chain_id: tx.chain_id,
        input: tx.input,
        v: tx.v,
        r: tx.r,
        s: tx.s,
    });

    Ok(tx.hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_legacy_v_encodes_parity_and_chain_id() {
        // Pre-EIP-155: 27 or 28.
        assert_eq!(legacy_v(false, None), 27);
        assert_eq!(legacy_v(true, None), 28);
        // EIP-155 on chain 1: 37 or 38.
        assert_eq!(legacy_v(false, Some(1)), 37);
        assert_eq!(legacy_v(true, Some(1)), 38);
    }

    #[test]
    fn garbage_bytes_are_rejected_not_panicked_on() {
        assert!(decode_and_check(&Bytes::new()).is_err());
        assert!(decode_and_check(&Bytes::from_static(&[0xff, 0x00, 0x13])).is_err());
    }
}
