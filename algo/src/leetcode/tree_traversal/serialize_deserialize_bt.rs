// =============================================================================
// LeetCode 297 — Serialize and Deserialize Binary Tree (Hard)
// =============================================================================
//
// # Problem Summary
// Design a method to serialize a binary tree to a string and deserialize
// it back. The format is up to you.
//
// # Approach / Intuition
// Use pre-order traversal with "null" markers for missing children. This
// uniquely defines the tree structure.
//
// Serialize: pre-order DFS, writing values and "#" for null nodes.
// Deserialize: read tokens in pre-order, recursively building the tree.
//
// **Blockchain analogy**: This mirrors **block serialization** (RLP in
// Ethereum). A block's state trie is serialized for network transmission
// and deserialized by receiving nodes. The key requirement is lossless
// round-tripping.
//
// # Complexity
// - Time: O(n) for both serialize and deserialize.
// - Space: O(n) for the string and recursion stack.
//
// Link: https://leetcode.com/problems/serialize-and-deserialize-binary-tree/

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// A binary tree node using Rc<RefCell<>> for shared ownership.
///
/// We use `Rc<RefCell<>>` here (unlike the BST in our DSA module) because
/// the LeetCode API requires shared references — multiple pointers may
/// reference the same node during construction. `Rc` provides reference
/// counting, `RefCell` provides interior mutability.
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

/// Serializes a binary tree to a comma-separated string.
///
/// Format: pre-order traversal with "#" for null nodes.
/// Example: tree [1, 2, 3, null, null, 4, 5] → "1,2,#,#,3,4,#,#,5,#,#"
pub fn serialize(root: &Option<Rc<RefCell<TreeNode>>>) -> String {
    let mut result = Vec::new();
    serialize_helper(root, &mut result);
    result.join(",")
}

fn serialize_helper(node: &Option<Rc<RefCell<TreeNode>>>, result: &mut Vec<String>) {
    match node {
        None => result.push("#".to_string()),
        Some(n) => {
            let n = n.borrow();
            result.push(n.val.to_string());
            serialize_helper(&n.left, result);
            serialize_helper(&n.right, result);
        }
    }
}

/// Deserializes a string back to a binary tree.
pub fn deserialize(data: &str) -> Option<Rc<RefCell<TreeNode>>> {
    if data.is_empty() {
        return None;
    }
    let mut tokens: VecDeque<&str> = data.split(',').collect();
    deserialize_helper(&mut tokens)
}

fn deserialize_helper(tokens: &mut VecDeque<&str>) -> Option<Rc<RefCell<TreeNode>>> {
    let token = tokens.pop_front()?;
    if token == "#" {
        return None;
    }

    let val: i32 = token.parse().ok()?;
    let node = Rc::new(RefCell::new(TreeNode::new(val)));
    node.borrow_mut().left = deserialize_helper(tokens);
    node.borrow_mut().right = deserialize_helper(tokens);
    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_round_trip() {
        let tree = build_tree(&[Some(1), Some(2), Some(3), None, None, Some(4), Some(5)]);
        let serialized = serialize(&tree);
        let deserialized = deserialize(&serialized);
        let reserialized = serialize(&deserialized);
        assert_eq!(serialized, reserialized);
    }

    #[test]
    fn test_empty() {
        let serialized = serialize(&None);
        assert_eq!(serialized, "#");
        let deserialized = deserialize("#");
        assert!(deserialized.is_none());
    }

    #[test]
    fn test_single_node() {
        let tree = build_tree(&[Some(42)]);
        let serialized = serialize(&tree);
        let deserialized = deserialize(&serialized);
        assert_eq!(deserialized.unwrap().borrow().val, 42);
    }
}
