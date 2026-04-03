// =============================================================================
// LeetCode 560 — Subarray Sum Equals K (Medium)
// =============================================================================
//
// # Problem Summary
// Given an integer array and an integer k, return the total number of
// contiguous subarrays whose sum equals k.
//
// # Approach / Intuition — Prefix Sum + Hash Map
// The sum of subarray [i..j] = prefix_sum[j] - prefix_sum[i-1].
// So we need: prefix_sum[j] - prefix_sum[i-1] = k
//     → prefix_sum[i-1] = prefix_sum[j] - k
//
// As we compute running prefix sums, we use a HashMap to count how many
// times each prefix sum has occurred. For the current prefix sum S, the
// number of valid subarrays ending here = count of (S - k) in the map.
//
// # Complexity
// - Time: O(n) — single pass over the array.
// - Space: O(n) — the hash map stores at most n prefix sums.
//
// Link: https://leetcode.com/problems/subarray-sum-equals-k/

use std::collections::HashMap;

/// Returns the count of contiguous subarrays whose sum equals k.
pub fn subarray_sum(nums: &[i32], k: i32) -> i32 {
    // Map: prefix_sum → count of occurrences.
    let mut prefix_count: HashMap<i32, i32> = HashMap::new();
    // Empty prefix (sum = 0) occurs once before we start.
    prefix_count.insert(0, 1);

    let mut sum = 0;
    let mut count = 0;

    for &num in nums {
        sum += num;
        // How many previous prefix sums equal (sum - k)?
        if let Some(&c) = prefix_count.get(&(sum - k)) {
            count += c;
        }
        *prefix_count.entry(sum).or_insert(0) += 1;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(subarray_sum(&[1, 1, 1], 2), 2);
    }

    #[test]
    fn test_mixed() {
        assert_eq!(subarray_sum(&[1, 2, 3], 3), 2); // [1,2] and [3]
    }

    #[test]
    fn test_negative() {
        assert_eq!(subarray_sum(&[1, -1, 0], 0), 3); // [1,-1], [-1,0], [0]
    }

    #[test]
    fn test_single() {
        assert_eq!(subarray_sum(&[5], 5), 1);
        assert_eq!(subarray_sum(&[5], 0), 0);
    }

    #[test]
    fn test_all_zeros() {
        assert_eq!(subarray_sum(&[0, 0, 0], 0), 6);
    }
}
