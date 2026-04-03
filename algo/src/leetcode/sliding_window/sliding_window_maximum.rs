// =============================================================================
// LeetCode 239 — Sliding Window Maximum (Hard)
// =============================================================================
//
// # Problem Summary
// Given an array of integers and a window size k, return the maximum value
// in each sliding window of size k as it moves from left to right.
//
// # Approach / Intuition
// Use a **monotonic deque** (double-ended queue) that maintains indices of
// elements in decreasing order of their values. The front of the deque always
// holds the index of the maximum element in the current window.
//
// For each new element:
// 1. Remove indices from the back that point to values ≤ the new element
//    (they can never be the maximum while the new element is in the window).
// 2. Add the new index to the back.
// 3. Remove the front if it's outside the window.
// 4. The front is the maximum for the current window.
//
// **Blockchain analogy**: This deque-based pruning is analogous to priority
// queue management in block production — stale candidates (lower priority)
// are pruned as better ones arrive, maintaining only viable candidates.
//
// # Complexity
// - Time: O(n) — each element is pushed and popped at most once.
// - Space: O(k) — the deque holds at most k indices.
//
// Link: https://leetcode.com/problems/sliding-window-maximum/

use std::collections::VecDeque;

/// Returns the maximum value in each sliding window of size k.
pub fn max_sliding_window(nums: &[i32], k: usize) -> Vec<i32> {
    if nums.is_empty() || k == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(nums.len() - k + 1);
    // Deque stores indices, not values. Indices are in decreasing order
    // of their corresponding values.
    let mut deque: VecDeque<usize> = VecDeque::new();

    for i in 0..nums.len() {
        // Remove elements from the back that are ≤ the current element.
        // Invariant: deque maintains decreasing order of values.
        while let Some(&back) = deque.back() {
            if nums[back] <= nums[i] {
                deque.pop_back();
            } else {
                break;
            }
        }

        deque.push_back(i);

        // Remove the front if it's outside the current window [i-k+1, i].
        if let Some(&front) = deque.front()
            && front + k <= i {
                deque.pop_front();
            }

        // Once we've processed at least k elements, record the window max.
        if i >= k - 1 {
            result.push(nums[*deque.front().unwrap()]);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let nums = vec![1, 3, -1, -3, 5, 3, 6, 7];
        assert_eq!(max_sliding_window(&nums, 3), vec![3, 3, 5, 5, 6, 7]);
    }

    #[test]
    fn test_single_element_window() {
        let nums = vec![1, -1];
        assert_eq!(max_sliding_window(&nums, 1), vec![1, -1]);
    }

    #[test]
    fn test_window_equals_array() {
        let nums = vec![1, 3, 2];
        assert_eq!(max_sliding_window(&nums, 3), vec![3]);
    }

    #[test]
    fn test_decreasing() {
        let nums = vec![5, 4, 3, 2, 1];
        assert_eq!(max_sliding_window(&nums, 2), vec![5, 4, 3, 2]);
    }

    #[test]
    fn test_empty() {
        assert_eq!(max_sliding_window(&[], 3), Vec::<i32>::new());
    }
}
