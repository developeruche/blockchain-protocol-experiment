// =============================================================================
// Binary Search — Efficient Search on Sorted Data
// =============================================================================
//
// # What is Binary Search?
//
// Binary search finds a target value in a **sorted** collection by repeatedly
// halving the search space. At each step, we compare the target to the middle
// element:
// - If equal → found.
// - If target < middle → search the left half.
// - If target > middle → search the right half.
//
// # Why Binary Search?
//
// - **O(log n) time**: Dramatically faster than linear search for large data.
// - **Foundation for many algorithms**: Binary search underpins BSTs, sorted
//   array operations, and interpolation search.
// - **Blockchain**: Used in binary search over block numbers (e.g., finding
//   the block where a state change occurred), and as the core operation in BST
//   lookup which underlies sorted state indexes.
//
// # Comparison with Linear Search
//
// | Algorithm     | Time     | Requires Sorted? | Space |
// |---------------|----------|------------------|-------|
// | Linear Search | O(n)     | No               | O(1)  |
// | Binary Search | O(log n) | Yes              | O(1)  |
//
// For n = 1,000,000: linear search needs up to 1M comparisons;
// binary search needs at most 20. That's the power of logarithmic time.
//
// # Implementations
//
// We provide:
// 1. **Iterative binary search on a slice** — the standard implementation.
// 2. **Recursive binary search on a slice** — for educational comparison.
// 3. **Binary search on a BST** — leveraging the BST ordering invariant.

use crate::dsa::trees::binary_tree::BinarySearchTree;

/// Iterative binary search on a sorted slice.
///
/// Returns `Some(index)` if found, `None` otherwise.
///
/// # Why iterative over recursive?
///
/// In practice, iterative binary search is preferred because:
/// - No function call overhead per step.
/// - No risk of stack overflow (though log n is tiny).
/// - Compilers can optimize the tight loop better.
///
/// # Examples
///
/// ```
/// use algo::dsa::algorithms::binary_search::binary_search_iterative;
///
/// let arr = vec![1, 3, 5, 7, 9, 11, 13];
/// assert_eq!(binary_search_iterative(&arr, &7), Some(3));
/// assert_eq!(binary_search_iterative(&arr, &6), None);
/// ```
pub fn binary_search_iterative<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    if arr.is_empty() {
        return None;
    }

    let mut low = 0usize;
    let mut high = arr.len() - 1;

    while low <= high {
        // Avoid overflow: `(low + high) / 2` can overflow for large indices.
        // Using `low + (high - low) / 2` is safe.
        let mid = low + (high - low) / 2;

        match target.cmp(&arr[mid]) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => {
                // Target is in the left half.
                if mid == 0 {
                    break; // Prevent underflow on usize.
                }
                high = mid - 1;
            }
            std::cmp::Ordering::Greater => {
                // Target is in the right half.
                low = mid + 1;
            }
        }
    }
    None
}

/// Recursive binary search on a sorted slice.
///
/// Returns `Some(index)` if found, `None` otherwise.
///
/// Included for educational purposes — the recursive formulation makes the
/// "divide" step explicit and mirrors the mathematical recurrence:
///   T(n) = T(n/2) + O(1) → T(n) = O(log n)
pub fn binary_search_recursive<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    if arr.is_empty() {
        return None;
    }
    binary_search_recursive_helper(arr, target, 0, arr.len() - 1)
}

fn binary_search_recursive_helper<T: Ord>(
    arr: &[T],
    target: &T,
    low: usize,
    high: usize,
) -> Option<usize> {
    if low > high {
        return None;
    }

    let mid = low + (high - low) / 2;
    match target.cmp(&arr[mid]) {
        std::cmp::Ordering::Equal => Some(mid),
        std::cmp::Ordering::Less => {
            if mid == 0 {
                None
            } else {
                binary_search_recursive_helper(arr, target, low, mid - 1)
            }
        }
        std::cmp::Ordering::Greater => {
            binary_search_recursive_helper(arr, target, mid + 1, high)
        }
    }
}

/// Binary search on a BST.
///
/// This simply delegates to `BinarySearchTree::contains`, but we include it
/// here to show that binary search on a BST is conceptually the same
/// algorithm as binary search on a sorted array:
///
/// - **Array**: The middle element divides the array into left/right halves.
/// - **BST**: Each node divides the tree into left/right subtrees.
///
/// The key difference is that a BST can become unbalanced (O(n) worst case),
/// while a sorted array always gives O(log n). Self-balancing BSTs (AVL,
/// Red-Black) fix this.
pub fn binary_search_bst<T: Ord + Clone>(tree: &BinarySearchTree<T>, target: &T) -> bool {
    tree.contains(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterative_found() {
        let arr = vec![2, 4, 6, 8, 10, 12, 14, 16];
        assert_eq!(binary_search_iterative(&arr, &8), Some(3));
        assert_eq!(binary_search_iterative(&arr, &2), Some(0)); // First element.
        assert_eq!(binary_search_iterative(&arr, &16), Some(7)); // Last element.
    }

    #[test]
    fn test_iterative_not_found() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(binary_search_iterative(&arr, &4), None);
        assert_eq!(binary_search_iterative(&arr, &0), None); // Below range.
        assert_eq!(binary_search_iterative(&arr, &10), None); // Above range.
    }

    #[test]
    fn test_iterative_empty() {
        let arr: Vec<i32> = vec![];
        assert_eq!(binary_search_iterative(&arr, &5), None);
    }

    #[test]
    fn test_iterative_single() {
        assert_eq!(binary_search_iterative(&[42], &42), Some(0));
        assert_eq!(binary_search_iterative(&[42], &7), None);
    }

    #[test]
    fn test_recursive_matches_iterative() {
        let arr: Vec<i32> = (0..100).collect();
        for target in -5..105 {
            assert_eq!(
                binary_search_iterative(&arr, &target),
                binary_search_recursive(&arr, &target),
                "Mismatch for target {}",
                target
            );
        }
    }

    #[test]
    fn test_bst_search() {
        let mut bst = BinarySearchTree::new();
        for v in [10, 5, 15, 3, 7, 12, 20] {
            bst.insert(v);
        }
        assert!(binary_search_bst(&bst, &10));
        assert!(binary_search_bst(&bst, &3));
        assert!(!binary_search_bst(&bst, &99));
    }

    #[test]
    fn test_strings() {
        let arr = vec!["apple", "banana", "cherry", "date", "elderberry"];
        assert_eq!(binary_search_iterative(&arr, &"cherry"), Some(2));
        assert_eq!(binary_search_iterative(&arr, &"fig"), None);
    }
}
