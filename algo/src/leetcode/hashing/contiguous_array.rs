// =============================================================================
// LeetCode 525 — Contiguous Array (Medium)
// =============================================================================
//
// # Problem Summary
// Given a binary array (containing only 0s and 1s), find the maximum length
// of a contiguous subarray with an equal number of 0s and 1s.
//
// # Approach / Intuition — Prefix Sum with Hash Map
// Replace every 0 with -1. Now the problem becomes: find the longest subarray
// with sum = 0.
//
// This is a prefix-sum problem: if prefix_sum[i] == prefix_sum[j], then the
// subarray (i, j] has sum 0. We store the first occurrence of each prefix
// sum in a HashMap. For each new prefix sum, if we've seen it before, the
// subarray between the two occurrences has equal 0s and 1s.
//
// # Complexity
// - Time: O(n) — single pass.
// - Space: O(n) — hash map stores at most n prefix sums.
//
// Link: https://leetcode.com/problems/contiguous-array/

use std::collections::HashMap;

/// Returns the max length of a contiguous subarray with equal 0s and 1s.
pub fn find_max_length(nums: &[i32]) -> i32 {
    let mut prefix_to_index: HashMap<i32, i32> = HashMap::new();
    // Empty prefix at index -1 (before the array starts).
    prefix_to_index.insert(0, -1);

    let mut sum = 0;
    let mut max_len = 0;

    for (i, &num) in nums.iter().enumerate() {
        // Treat 0 as -1 so equal counts of 0 and 1 sum to 0.
        sum += if num == 1 { 1 } else { -1 };

        if let Some(&first_idx) = prefix_to_index.get(&sum) {
            max_len = max_len.max(i as i32 - first_idx);
        } else {
            prefix_to_index.insert(sum, i as i32);
        }
    }

    max_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(find_max_length(&[0, 1]), 2);
    }

    #[test]
    fn test_mixed() {
        assert_eq!(find_max_length(&[0, 1, 0]), 2);
    }

    #[test]
    fn test_longer() {
        assert_eq!(find_max_length(&[0, 0, 1, 0, 0, 0, 1, 1]), 6);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(find_max_length(&[0, 0, 0]), 0);
    }

    #[test]
    fn test_alternating() {
        assert_eq!(find_max_length(&[0, 1, 0, 1, 0, 1]), 6);
    }
}
