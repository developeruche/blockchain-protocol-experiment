//! Utility functions for pod-core

use alloy::signers::{Signature, Signer, SignerSync, k256::ecdsa::SigningKey, local::LocalSigner};

use crate::primitives::{
    errors::PodError,
    pod::{POD_CHAIN_ID, PodTransaction},
};

/// Calculates the median value of a mutable slice of `u64` integers.
///
/// The function sorts the input slice in-place and returns the median value.
/// - If the slice is empty, returns `0`.
/// - For odd-length slices, returns the middle element.
/// - For even-length slices, returns the lower middle element (as per the referenced paper's implementation).
///
/// # Arguments
///
/// * `data` - A mutable slice of `u64` integers.
///
/// # Examples
///
/// ```
/// use pod_core::utils::median;
/// let mut values = [3, 1, 4, 1, 5];
/// let m = median(&mut values);
/// assert_eq!(m, 3);
/// ```
pub fn median(data: &mut [u64]) -> u64 {
    if data.is_empty() {
        return 0;
    }
    data.sort();
    let mid = data.len() / 2;
    if data.len() % 2 == 1 {
        data[mid]
    } else {
        // As per the paper's implementation of median, which returns Y[floor(|Y|/2)]
        data[mid - 1]
    }
}

pub fn sign_tx(
    tx: &PodTransaction,
    signer: &LocalSigner<SigningKey>,
) -> Result<Signature, PodError> {
    let hash = tx.hash();
    let sig = signer
        .sign_hash_sync(&hash)
        .map_err(|e| PodError::SignatureFailed(e.to_string()))?;

    Ok(sig)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_slice() {
        let mut data: [u64; 0] = [];
        assert_eq!(median(&mut data), 0);
    }

    #[test]
    fn test_single_element() {
        let mut data = [42];
        assert_eq!(median(&mut data), 42);
    }

    #[test]
    fn test_odd_length() {
        let mut data = [3, 1, 4, 1, 5];
        assert_eq!(median(&mut data), 3);
    }

    #[test]
    fn test_even_length() {
        let mut data = [10, 2, 8, 4];
        assert_eq!(median(&mut data), 4);
    }

    #[test]
    fn test_sorted_input() {
        let mut data = [1, 2, 3, 4, 5, 6];
        assert_eq!(median(&mut data), 3);
    }
}
