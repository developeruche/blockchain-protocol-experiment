// =============================================================================
// LeetCode 1 — Two Sum (Easy)
// =============================================================================
//
// # Problem Summary
// Given an array of integers and a target, return indices of the two numbers
// whose sum equals the target. Assume exactly one solution exists.
//
// # Approach / Intuition
// Use a HashMap to store each number's index as we iterate. For each number x,
// check if (target - x) is already in the map. If yes, we found our pair.
//
// The HashMap gives us O(1) lookup, turning the naive O(n²) two-loop approach
// into a single-pass O(n) solution.
//
// **Hashing mental model**: The hash map acts as a "memory" of what we've seen.
// In blockchain, this is analogous to UTXO lookups — checking if a required
// input exists in constant time.
//
// # Complexity
// - Time: O(n) — single pass.
// - Space: O(n) — the map stores at most n entries.
//
// Link: https://leetcode.com/problems/two-sum/

use std::collections::HashMap;

/// Returns indices of two numbers that add up to the target.
pub fn two_sum(nums: &[i32], target: i32) -> Vec<i32> {
    // Map: value → index.
    let mut seen: HashMap<i32, usize> = HashMap::new();

    for (i, &num) in nums.iter().enumerate() {
        let complement = target - num;
        if let Some(&j) = seen.get(&complement) {
            return vec![j as i32, i as i32];
        }
        seen.insert(num, i);
    }

    vec![] // Unreachable if exactly one solution exists.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(two_sum(&[2, 7, 11, 15], 9), vec![0, 1]);
    }

    #[test]
    fn test_middle() {
        assert_eq!(two_sum(&[3, 2, 4], 6), vec![1, 2]);
    }

    #[test]
    fn test_same_values() {
        assert_eq!(two_sum(&[3, 3], 6), vec![0, 1]);
    }

    #[test]
    fn test_negative() {
        assert_eq!(two_sum(&[-1, -2, -3, -4, -5], -8), vec![2, 4]);
    }

    #[test]
    fn test_large() {
        let nums: Vec<i32> = (0..1000).collect();
        let result = two_sum(&nums, 999 + 998);
        assert_eq!(result, vec![998, 999]);
    }
}
