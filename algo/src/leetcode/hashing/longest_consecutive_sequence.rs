// =============================================================================
// LeetCode 128 — Longest Consecutive Sequence (Medium)
// =============================================================================
//
// # Problem Summary
// Given an unsorted array of integers, find the length of the longest
// consecutive elements sequence. Must run in O(n) time.
//
// # Approach / Intuition
// Insert all numbers into a HashSet. For each number that is the *start*
// of a sequence (i.e., num-1 is NOT in the set), count consecutive elements.
//
// Key: by only starting from sequence beginnings, each number is visited at
// most twice (once for insertion, once for counting), giving O(n) total.
//
// # Complexity
// - Time: O(n) — the while loop runs at most n times total across all starts.
// - Space: O(n) — the set stores all numbers.
//
// Link: https://leetcode.com/problems/longest-consecutive-sequence/

use std::collections::HashSet;

/// Returns the length of the longest consecutive sequence.
pub fn longest_consecutive(nums: &[i32]) -> i32 {
    let set: HashSet<i32> = nums.iter().copied().collect();
    let mut max_len = 0;

    for &num in &set {
        // Only start counting from sequence beginnings.
        if !set.contains(&(num - 1)) {
            let mut current = num;
            let mut length = 1;
            while set.contains(&(current + 1)) {
                current += 1;
                length += 1;
            }
            max_len = max_len.max(length);
        }
    }

    max_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(longest_consecutive(&[100, 4, 200, 1, 3, 2]), 4); // 1,2,3,4
    }

    #[test]
    fn test_with_duplicates() {
        assert_eq!(longest_consecutive(&[0, 3, 7, 2, 5, 8, 4, 6, 0, 1]), 9);
    }

    #[test]
    fn test_empty() {
        assert_eq!(longest_consecutive(&[]), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(longest_consecutive(&[42]), 1);
    }

    #[test]
    fn test_no_consecutive() {
        assert_eq!(longest_consecutive(&[10, 20, 30]), 1);
    }
}
