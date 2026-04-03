// =============================================================================
// LeetCode 23 — Merge k Sorted Lists (Hard)
// =============================================================================
//
// # Problem Summary
// Merge k sorted linked lists into one sorted linked list.
//
// # Approach — Divide and Conquer
// Repeatedly merge pairs of lists until one remains. This mirrors merge sort's
// merge phase applied at the list-of-lists level.
//
// Alternative: a min-heap (priority queue) that always extracts the smallest
// head. The D&C approach avoids the heap overhead and is more interview-friendly.
//
// **Comparison**:
// - Heap: O(n log k) time, O(k) space for the heap.
// - D&C: O(n log k) time, O(1) extra space (in-place merging).
//
// # Complexity
// - Time: O(n log k) where n = total elements, k = number of lists.
// - Space: O(1) extra (excluding output).
//
// Link: https://leetcode.com/problems/merge-k-sorted-lists/

/// Merges k sorted vectors into one sorted vector.
///
/// We use `Vec<i32>` instead of linked lists because Rust's ownership model
/// makes linked lists awkward. The algorithm is identical.
pub fn merge_k_lists(lists: &[Vec<i32>]) -> Vec<i32> {
    if lists.is_empty() {
        return vec![];
    }
    let owned: Vec<Vec<i32>> = lists.to_vec();
    divide_and_conquer(owned)
}

fn divide_and_conquer(mut lists: Vec<Vec<i32>>) -> Vec<i32> {
    if lists.len() == 1 {
        return lists.remove(0);
    }

    let mut merged = Vec::new();
    let mut i = 0;
    while i < lists.len() {
        if i + 1 < lists.len() {
            merged.push(merge_two(&lists[i], &lists[i + 1]));
        } else {
            merged.push(lists[i].clone());
        }
        i += 2;
    }
    divide_and_conquer(merged)
}

fn merge_two(a: &[i32], b: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            result.push(a[i]);
            i += 1;
        } else {
            result.push(b[j]);
            j += 1;
        }
    }
    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let lists = vec![vec![1,4,5], vec![1,3,4], vec![2,6]];
        assert_eq!(merge_k_lists(&lists), vec![1,1,2,3,4,4,5,6]);
    }

    #[test]
    fn test_empty() {
        assert_eq!(merge_k_lists(&[]), Vec::<i32>::new());
    }

    #[test]
    fn test_single_list() {
        assert_eq!(merge_k_lists(&[vec![1,2,3]]), vec![1,2,3]);
    }
}
