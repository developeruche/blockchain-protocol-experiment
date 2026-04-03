// =============================================================================
// LeetCode 235 — Lowest Common Ancestor of a BST (Medium)
// =============================================================================
//
// # Problem Summary
// Given a BST and two nodes p and q, find their lowest common ancestor (LCA).
// The LCA is the deepest node that is an ancestor of both p and q.
//
// # Approach / Intuition
// Leverage the BST property: if both p and q are less than the current node,
// LCA is in the left subtree. If both are greater, LCA is in the right
// subtree. Otherwise, the current node is the LCA.
//
// **Blockchain analogy**: LCA in proof systems corresponds to finding the
// common prefix in two Merkle proofs — the point where the paths diverge.
//
// # Complexity
// - Time: O(h) where h = height of the BST.
// - Space: O(1) iterative, O(h) recursive.
//
// Link: https://leetcode.com/problems/lowest-common-ancestor-of-a-binary-search-tree/

use std::cell::RefCell;
use std::rc::Rc;

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

/// Finds the lowest common ancestor of p and q in a BST (iterative).
pub fn lowest_common_ancestor(
    root: Option<Rc<RefCell<TreeNode>>>,
    p: i32,
    q: i32,
) -> Option<Rc<RefCell<TreeNode>>> {
    let mut current = root;

    while let Some(node) = current {
        let val = node.borrow().val;

        if p < val && q < val {
            // Both values are smaller — LCA is in the left subtree.
            current = node.borrow().left.clone();
        } else if p > val && q > val {
            // Both values are larger — LCA is in the right subtree.
            current = node.borrow().right.clone();
        } else {
            // p and q are on different sides (or one equals current).
            // Current node IS the LCA.
            return Some(node);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn build_bst(vals: &[Option<i32>]) -> Option<Rc<RefCell<TreeNode>>> {
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
    fn test_basic() {
        //       6
        //      / \
        //     2   8
        //    / \ / \
        //   0  4 7  9
        let tree = build_bst(&[
            Some(6), Some(2), Some(8), Some(0), Some(4), Some(7), Some(9),
        ]);
        let lca = lowest_common_ancestor(tree, 2, 8).unwrap();
        assert_eq!(lca.borrow().val, 6);
    }

    #[test]
    fn test_same_subtree() {
        let tree = build_bst(&[
            Some(6), Some(2), Some(8), Some(0), Some(4), Some(7), Some(9),
        ]);
        let lca = lowest_common_ancestor(tree, 2, 4).unwrap();
        assert_eq!(lca.borrow().val, 2);
    }

    #[test]
    fn test_root_is_one() {
        let tree = build_bst(&[Some(2), Some(1)]);
        let lca = lowest_common_ancestor(tree, 2, 1).unwrap();
        assert_eq!(lca.borrow().val, 2);
    }
}
