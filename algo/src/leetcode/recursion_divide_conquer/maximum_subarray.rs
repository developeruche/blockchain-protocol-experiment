// =============================================================================
// LeetCode 53 — Maximum Subarray (Medium)
// =============================================================================
//
// # Problem Summary
// Find the contiguous subarray with the largest sum.
//
// # Approach 1 — Kadane's Algorithm (DP)
// Maintain a running max ending at each position:
//   max_ending_here = max(num, max_ending_here + num)
//   max_so_far = max(max_so_far, max_ending_here)
//
// # Approach 2 — Divide and Conquer
// Split array in half. The max subarray is either:
// 1. Entirely in the left half.
// 2. Entirely in the right half.
// 3. Crossing the midpoint.
//
// # Complexity
// - Kadane: O(n) time, O(1) space.
// - D&C: O(n log n) time, O(log n) space (recursion).
//
// Link: https://leetcode.com/problems/maximum-subarray/

/// Kadane's algorithm — O(n).
pub fn max_subarray_kadane(nums: &[i32]) -> i32 {
    let mut max_ending_here = nums[0];
    let mut max_so_far = nums[0];

    for &num in &nums[1..] {
        // Either extend the current subarray or start a new one.
        max_ending_here = num.max(max_ending_here + num);
        max_so_far = max_so_far.max(max_ending_here);
    }

    max_so_far
}

/// Divide and conquer — O(n log n).
pub fn max_subarray_dc(nums: &[i32]) -> i32 {
    dc_helper(nums, 0, nums.len() - 1)
}

fn dc_helper(nums: &[i32], left: usize, right: usize) -> i32 {
    if left == right {
        return nums[left];
    }

    let mid = left + (right - left) / 2;
    let left_max = dc_helper(nums, left, mid);
    let right_max = dc_helper(nums, mid + 1, right);
    let cross_max = max_crossing_subarray(nums, left, mid, right);

    left_max.max(right_max).max(cross_max)
}

fn max_crossing_subarray(nums: &[i32], left: usize, mid: usize, right: usize) -> i32 {
    let mut left_sum = i32::MIN;
    let mut sum = 0;
    for i in (left..=mid).rev() {
        sum += nums[i];
        left_sum = left_sum.max(sum);
    }

    let mut right_sum = i32::MIN;
    sum = 0;
    for i in (mid + 1)..=right {
        sum += nums[i];
        right_sum = right_sum.max(sum);
    }

    left_sum + right_sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kadane_basic() {
        assert_eq!(max_subarray_kadane(&[-2,1,-3,4,-1,2,1,-5,4]), 6);
    }

    #[test]
    fn test_dc_basic() {
        assert_eq!(max_subarray_dc(&[-2,1,-3,4,-1,2,1,-5,4]), 6);
    }

    #[test]
    fn test_all_negative() {
        assert_eq!(max_subarray_kadane(&[-3,-2,-1]), -1);
        assert_eq!(max_subarray_dc(&[-3,-2,-1]), -1);
    }

    #[test]
    fn test_single() {
        assert_eq!(max_subarray_kadane(&[5]), 5);
        assert_eq!(max_subarray_dc(&[5]), 5);
    }

    #[test]
    fn test_match() {
        let nums = [-2,1,-3,4,-1,2,1,-5,4];
        assert_eq!(max_subarray_kadane(&nums), max_subarray_dc(&nums));
    }
}
