// =============================================================================
// LeetCode 341 — Flatten Nested List Iterator (Medium)
// =============================================================================
//
// # Problem Summary
// Given a nested list of integers, implement an iterator that returns all
// integers in order by flattening the structure.
//
// # Approach — Lazy Evaluation via Stack
// Instead of eagerly flattening, we use a stack to lazily traverse the nested
// structure. This is Rust-idiomatic: we implement the `Iterator` trait.
//
// On each `next()` call, we ensure the top of the stack is an integer.
// If it's a nested list, we expand it in reverse order onto the stack.
//
// **Blockchain analogy**: Lazy evaluation via iterators is a core Rust pattern
// used in Substrate's runtime (e.g., lazy storage iteration to avoid loading
// entire maps into memory).
//
// # Complexity
// - Time: O(n) amortized over all next() calls.
// - Space: O(d) where d = maximum nesting depth.
//
// Link: https://leetcode.com/problems/flatten-nested-list-iterator/

/// Represents either a single integer or a nested list.
#[derive(Debug, Clone)]
pub enum NestedInteger {
    Int(i32),
    List(Vec<NestedInteger>),
}

/// An iterator that lazily flattens a nested list structure.
pub struct NestedIterator {
    /// Stack of nested integers to process. We push items in reverse order
    /// so that the first item is on top.
    stack: Vec<NestedInteger>,
}

impl NestedIterator {
    /// Creates a new NestedIterator from a nested list.
    pub fn new(nested_list: Vec<NestedInteger>) -> Self {
        let mut stack = Vec::new();
        // Push in reverse so the first element is on top.
        for item in nested_list.into_iter().rev() {
            stack.push(item);
        }
        NestedIterator { stack }
    }

    /// Ensures the top of the stack is an Int (not a List).
    /// Expands any Lists found on top.
    fn make_top_integer(&mut self) {
        while let Some(top) = self.stack.last() {
            match top {
                NestedInteger::Int(_) => break,
                NestedInteger::List(_) => {
                    if let NestedInteger::List(list) = self.stack.pop().unwrap() {
                        for item in list.into_iter().rev() {
                            self.stack.push(item);
                        }
                    }
                }
            }
        }
    }
}

impl Iterator for NestedIterator {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        self.make_top_integer();
        match self.stack.pop() {
            Some(NestedInteger::Int(val)) => Some(val),
            _ => None,
        }
    }
}

/// Convenience function: flattens a nested list into a Vec.
pub fn flatten(nested_list: Vec<NestedInteger>) -> Vec<i32> {
    NestedIterator::new(nested_list).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use NestedInteger::*;

    #[test]
    fn test_basic() {
        // [[1,1],2,[1,1]]
        let input = vec![
            List(vec![Int(1), Int(1)]),
            Int(2),
            List(vec![Int(1), Int(1)]),
        ];
        assert_eq!(flatten(input), vec![1, 1, 2, 1, 1]);
    }

    #[test]
    fn test_deeply_nested() {
        // [1,[4,[6]]]
        let input = vec![
            Int(1),
            List(vec![Int(4), List(vec![Int(6)])]),
        ];
        assert_eq!(flatten(input), vec![1, 4, 6]);
    }

    #[test]
    fn test_empty_lists() {
        // [[], [1], []]
        let input = vec![List(vec![]), List(vec![Int(1)]), List(vec![])];
        assert_eq!(flatten(input), vec![1]);
    }

    #[test]
    fn test_all_ints() {
        let input = vec![Int(1), Int(2), Int(3)];
        assert_eq!(flatten(input), vec![1, 2, 3]);
    }

    #[test]
    fn test_iterator_trait() {
        let input = vec![Int(10), List(vec![Int(20), Int(30)])];
        let iter = NestedIterator::new(input);
        let sum: i32 = iter.sum();
        assert_eq!(sum, 60);
    }
}
