// =============================================================================
// Bubble Sort — The Simplest Comparison Sort
// =============================================================================
//
// # What is Bubble Sort?
//
// Bubble sort repeatedly steps through the list, compares adjacent elements,
// and swaps them if they are in the wrong order. On each pass, the largest
// unsorted element "bubbles up" to its final position.
//
// # Why Bubble Sort?
//
// Bubble sort is almost never used in production (it's O(n²)), but it's
// valuable for education:
// - It's the simplest sorting algorithm to understand and implement.
// - It introduces the concept of **inversions** (out-of-order pairs).
// - It's the baseline against which we measure better algorithms.
//
// # Comparison with Other Sorts
//
// | Algorithm   | Best    | Average  | Worst    | Space | Stable? |
// |-------------|---------|----------|----------|-------|---------|
// | Bubble Sort | O(n)    | O(n²)   | O(n²)   | O(1)  | Yes     |
// | Insertion   | O(n)    | O(n²)   | O(n²)   | O(1)  | Yes     |
// | Merge Sort  | O(n lg n)| O(n lg n)| O(n lg n)| O(n) | Yes    |
// | Quick Sort  | O(n lg n)| O(n lg n)| O(n²)  | O(lg n)| No    |
// | Heap Sort   | O(n lg n)| O(n lg n)| O(n lg n)| O(1) | No    |
//
// Bubble sort's only advantage: it can detect an already-sorted array in O(n)
// (with the `swapped` optimization below). Insertion sort does this too and
// is faster in practice.
//
// # Invariant
//
// After pass `i`, the last `i` elements are in their final sorted positions.
// This means each pass only needs to consider the first `n - i` elements.

/// Sorts a mutable slice in ascending order using bubble sort.
///
/// # Optimizations
///
/// 1. **Early termination**: If no swaps occur during a pass, the array is
///    already sorted — return immediately. This gives O(n) best case.
/// 2. **Shrinking range**: Each pass places the next-largest element, so we
///    reduce the range by 1 each pass.
///
/// # Examples
///
/// ```
/// use algo::dsa::algorithms::bubble_sort::bubble_sort;
///
/// let mut arr = vec![64, 34, 25, 12, 22, 11, 90];
/// bubble_sort(&mut arr);
/// assert_eq!(arr, vec![11, 12, 22, 25, 34, 64, 90]);
/// ```
pub fn bubble_sort<T: Ord>(arr: &mut [T]) {
    let n = arr.len();
    if n <= 1 {
        return;
    }

    for i in 0..n {
        let mut swapped = false;
        // Only iterate up to `n - 1 - i` because the last `i` elements
        // are already in their final positions.
        for j in 0..n - 1 - i {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
                swapped = true;
            }
        }
        // If no swaps occurred in this pass, the array is already sorted.
        // This optimization changes the best case from O(n²) to O(n).
        if !swapped {
            break;
        }
    }
}

/// Bubble sort in descending order.
pub fn bubble_sort_desc<T: Ord>(arr: &mut [T]) {
    let n = arr.len();
    if n <= 1 {
        return;
    }

    for i in 0..n {
        let mut swapped = false;
        for j in 0..n - 1 - i {
            if arr[j] < arr[j + 1] {
                arr.swap(j, j + 1);
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sort() {
        let mut arr = vec![5, 3, 8, 1, 2];
        bubble_sort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3, 5, 8]);
    }

    #[test]
    fn test_already_sorted() {
        let mut arr = vec![1, 2, 3, 4, 5];
        bubble_sort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_reverse_sorted() {
        let mut arr = vec![5, 4, 3, 2, 1];
        bubble_sort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_empty_and_single() {
        let mut empty: Vec<i32> = vec![];
        bubble_sort(&mut empty);
        assert!(empty.is_empty());

        let mut single = vec![42];
        bubble_sort(&mut single);
        assert_eq!(single, vec![42]);
    }

    #[test]
    fn test_duplicates() {
        let mut arr = vec![3, 1, 2, 1, 3, 2];
        bubble_sort(&mut arr);
        assert_eq!(arr, vec![1, 1, 2, 2, 3, 3]);
    }

    #[test]
    fn test_descending() {
        let mut arr = vec![1, 3, 2, 5, 4];
        bubble_sort_desc(&mut arr);
        assert_eq!(arr, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_strings() {
        let mut arr = vec!["banana", "apple", "cherry"];
        bubble_sort(&mut arr);
        assert_eq!(arr, vec!["apple", "banana", "cherry"]);
    }
}
