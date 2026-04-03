// =============================================================================
// LeetCode 992 — Subarrays with K Different Integers (Hard)
// =============================================================================
//
// # Problem Summary
// Given an integer array `nums` and an integer `k`, return the number of
// contiguous subarrays that contain exactly `k` distinct integers.
//
// # Approach / Intuition — atMost(K) − atMost(K−1) Trick
//
// Counting subarrays with *exactly* K distinct values is hard with a single
// sliding window because the window condition isn't monotonic. Instead:
//
//   exactly(K) = atMost(K) − atMost(K−1)
//
// `atMost(K)` counts subarrays with ≤ K distinct values, which IS monotonic:
// if a window has ≤ K distinct values, all sub-windows also satisfy this.
//
// For `atMost(K)`: use a sliding window [left, right], maintaining a frequency
// map. When distinct count exceeds K, shrink from left. For each right
// position, all subarrays [left..=right], [left+1..=right], ..., [right..=right]
// are valid, contributing (right - left + 1) subarrays.
//
// # Complexity
// - Time: O(n) — two passes of atMost.
// - Space: O(k) — the frequency map has at most k+1 entries.
//
// Link: https://leetcode.com/problems/subarrays-with-k-different-integers/

use std::collections::HashMap;

/// Returns the number of subarrays with exactly `k` distinct integers.
pub fn subarrays_with_k_distinct(nums: &[i32], k: i32) -> i32 {
    at_most_k_distinct(nums, k) - at_most_k_distinct(nums, k - 1)
}

/// Counts subarrays with at most `k` distinct integers.
fn at_most_k_distinct(nums: &[i32], k: i32) -> i32 {
    if k <= 0 {
        return 0;
    }

    let mut freq: HashMap<i32, i32> = HashMap::new();
    let mut left = 0;
    let mut count = 0;
    let mut distinct = 0;

    for right in 0..nums.len() {
        let entry = freq.entry(nums[right]).or_insert(0);
        if *entry == 0 {
            distinct += 1;
        }
        *entry += 1;

        // Shrink window until we have at most k distinct values.
        while distinct > k {
            let left_entry = freq.get_mut(&nums[left]).unwrap();
            *left_entry -= 1;
            if *left_entry == 0 {
                distinct -= 1;
            }
            left += 1;
        }

        // All subarrays ending at `right` with start in [left, right] are valid.
        count += (right - left + 1) as i32;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(subarrays_with_k_distinct(&[1, 2, 1, 2, 3], 2), 7);
    }

    #[test]
    fn test_three_distinct() {
        assert_eq!(subarrays_with_k_distinct(&[1, 2, 1, 3, 4], 3), 3);
    }

    #[test]
    fn test_single_element() {
        assert_eq!(subarrays_with_k_distinct(&[1], 1), 1);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(subarrays_with_k_distinct(&[1, 1, 1, 1], 1), 10);
    }

    #[test]
    fn test_k_equals_n() {
        assert_eq!(subarrays_with_k_distinct(&[1, 2, 3], 3), 1);
    }
}
