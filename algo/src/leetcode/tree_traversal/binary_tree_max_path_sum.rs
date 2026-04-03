// =============================================================================
// LeetCode 124 — Binary Tree Maximum Path Sum (Hard)
// =============================================================================
//
// # Problem Summary
// Given a binary tree, find the maximum path sum. A path is any sequence of
// nodes where each pair of adjacent nodes has an edge. The path doesn't need
// to go through the root.
//
// # Approach / Intuition
// Post-order DFS: for each node, compute the max path sum that can be
// "extended" upward (single branch) and the max path sum passing through
// this node (both branches + node).
//
// Key insight: at each node, the max path through it is:
//   node.val + max(0, left_gain) + max(0, right_gain)
// But when returning to the parent, we can only extend one branch:
//   node.val + max(0, max(left_gain, right_gain))
//
// We use a global max to track the best path seen anywhere in the tree.
//
// # Complexity
// - Time: O(n) — visit each node once.
// - Space: O(h) — recursion depth equals tree height.
//
// Link: https://leetcode.com/problems/binary-tree-maximum-path-sum/

use std::cell::RefCell;
use std::rc::Rc;

/// Reusing the TreeNode definition for consistency with other tree problems.
#[derive(Debug)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }
}

/// Returns the maximum path sum in the binary tree.
pub fn max_path_sum(root: &Option<Rc<RefCell<TreeNode>>>) -> i32 {
    let mut max_sum = i32::MIN;
    dfs(root, &mut max_sum);
    max_sum
}

/// Returns the max gain the current node can contribute to its parent.
/// Side effect: updates `max_sum` if the path through this node is better.
fn dfs(node: &Option<Rc<RefCell<TreeNode>>>, max_sum: &mut i32) -> i32 {
    match node {
        None => 0,
        Some(n) => {
            let n = n.borrow();
            // Compute gain from left and right subtrees.
            // Use max(0, ...) because we can choose not to include a negative subtree.
            let left_gain = dfs(&n.left, max_sum).max(0);
            let right_gain = dfs(&n.right, max_sum).max(0);

            // Path through this node: both branches + node value.
            let path_sum = n.val + left_gain + right_gain;
            *max_sum = (*max_sum).max(path_sum);

            // Return to parent: can only extend one branch.
            n.val + left_gain.max(right_gain)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn build_tree(vals: &[Option<i32>]) -> Option<Rc<RefCell<TreeNode>>> {
        if vals.is_empty() || vals[0].is_none() {
            return None;
        }
        let root = Rc::new(RefCell::new(TreeNode::new(vals[0].unwrap())));
        let mut queue = VecDeque::new();
        queue.push_back(root.clone());
        let mut i = 1;
        while i < vals.len() {
            if let Some(node) = queue.pop_front() {
                if i < vals.len() {
                    if let Some(v) = vals[i] {
                        let left = Rc::new(RefCell::new(TreeNode::new(v)));
                        node.borrow_mut().left = Some(left.clone());
                        queue.push_back(left);
                    }
                    i += 1;
                }
                if i < vals.len() {
                    if let Some(v) = vals[i] {
                        let right = Rc::new(RefCell::new(TreeNode::new(v)));
                        node.borrow_mut().right = Some(right.clone());
                        queue.push_back(right);
                    }
                    i += 1;
                }
            }
        }
        Some(root)
    }

    #[test]
    fn test_simple() {
        // [1, 2, 3] → max path = 2 + 1 + 3 = 6
        let tree = build_tree(&[Some(1), Some(2), Some(3)]);
        assert_eq!(max_path_sum(&tree), 6);
    }

    #[test]
    fn test_negative() {
        // [-10, 9, 20, null, null, 15, 7] → max path = 15 + 20 + 7 = 42
        let tree = build_tree(&[Some(-10), Some(9), Some(20), None, None, Some(15), Some(7)]);
        assert_eq!(max_path_sum(&tree), 42);
    }

    #[test]
    fn test_single_node() {
        let tree = build_tree(&[Some(-3)]);
        assert_eq!(max_path_sum(&tree), -3);
    }
}
