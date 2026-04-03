// =============================================================================
// Splay Tree — Self-Adjusting BST
// =============================================================================
//
// # What is a Splay Tree?
//
// A splay tree is a self-adjusting BST where every access (search, insert,
// delete) moves the accessed node to the root via a sequence of rotations
// called **splaying**. This provides **amortized O(log n)** performance for
// all operations, even though individual operations can be O(n).
//
// # Why Splay Trees?
//
// - **Cache-friendly**: Recently accessed elements are near the root, so
//   repeated accesses are fast (temporal locality).
// - **No extra storage**: Unlike AVL (height) or Red-Black (color), splay
//   trees don't store any balance metadata — the structure is implicit.
// - **Optimal for skewed access patterns**: If some keys are accessed much
//   more frequently, splay trees adapt to give those keys shorter paths.
// - **Blockchain**: Useful for caching frequently accessed state (e.g., hot
//   accounts in a transaction pool).
//
// # Complexity Table
//
// | Operation | Amortized | Worst (single op) |
// |-----------|-----------|-------------------|
// | Search    | O(log n)  | O(n)              |
// | Insert    | O(log n)  | O(n)              |
// | Delete    | O(log n)  | O(n)              |
// | Space     | O(n)      | O(n)              |
//
// # Splay Operation — Three Cases
//
// The splay operation moves a node to the root using these rotation patterns:
//
// 1. **Zig**: Node is a child of the root → single rotation.
// 2. **Zig-Zig**: Node and parent are both left (or both right) children →
//    rotate parent first, then node (same direction).
// 3. **Zig-Zag**: Node is a left child and parent is a right child (or vice
//    versa) → rotate node twice (alternating directions, like AVL double rotation).
//
// The zig-zig case is what distinguishes splay trees from simple
// "move-to-root" heuristics. By rotating the grandparent first, the tree
// roughly halves the depth of all nodes on the access path.

use std::cmp::Ordering;
use std::fmt;

/// A node in the splay tree.
#[derive(Debug)]
pub struct SplayNode<T: Ord> {
    pub value: T,
    pub left: Option<Box<SplayNode<T>>>,
    pub right: Option<Box<SplayNode<T>>>,
}

impl<T: Ord> SplayNode<T> {
    fn new(value: T) -> Self {
        SplayNode {
            value,
            left: None,
            right: None,
        }
    }
}

// =============================================================================
// Splay operations using top-down splaying
// =============================================================================
//
// Top-down splaying is preferred over bottom-up because it doesn't require
// parent pointers or a stack to track the path. We split the tree into three
// parts as we walk down: left tree, right tree, and middle (current subtree).

/// Splays the tree so that the given value (or the closest value) becomes
/// the root. Returns the new root.
fn splay<T: Ord>(mut root: Box<SplayNode<T>>, value: &T) -> Box<SplayNode<T>> {
    // We'll build left_tree and right_tree by moving subtrees off the path.
    // At the end, we reassemble: left_tree — root — right_tree.
    let mut left_tree: Option<Box<SplayNode<T>>> = None;
    let mut right_tree: Option<Box<SplayNode<T>>> = None;

    // Mutable references to the rightmost node in left_tree and leftmost in right_tree.
    // We use raw pointers to avoid borrow checker issues in this classic
    // pointer-manipulation algorithm.
    let mut left_tail: *mut Option<Box<SplayNode<T>>> = &mut left_tree;
    let mut right_tail: *mut Option<Box<SplayNode<T>>> = &mut right_tree;

    loop {
        match value.cmp(&root.value) {
            Ordering::Less => {
                match root.left.take() {
                    None => break,
                    Some(mut left) => {
                        if *value < left.value {
                            // Zig-zig: rotate right first.
                            root.left = left.right.take();
                            left.right = Some(root);
                            root = left;
                            match root.left.take() {
                                None => break,
                                Some(new_left) => {
                                    // Link root into right_tree.
                                    unsafe {
                                        *right_tail = Some(root);
                                        right_tail = &mut (*right_tail).as_mut().unwrap().left;
                                    }
                                    root = new_left;
                                }
                            }
                        } else {
                            // Zig: link root into right_tree, continue with left child.
                            unsafe {
                                *right_tail = Some(root);
                                right_tail = &mut (*right_tail).as_mut().unwrap().left;
                            }
                            root = left;
                        }
                    }
                }
            }
            Ordering::Greater => {
                match root.right.take() {
                    None => break,
                    Some(mut right) => {
                        if *value > right.value {
                            // Zig-zig: rotate left first.
                            root.right = right.left.take();
                            right.left = Some(root);
                            root = right;
                            match root.right.take() {
                                None => break,
                                Some(new_right) => {
                                    unsafe {
                                        *left_tail = Some(root);
                                        left_tail = &mut (*left_tail).as_mut().unwrap().right;
                                    }
                                    root = new_right;
                                }
                            }
                        } else {
                            unsafe {
                                *left_tail = Some(root);
                                left_tail = &mut (*left_tail).as_mut().unwrap().right;
                            }
                            root = right;
                        }
                    }
                }
            }
            Ordering::Equal => break,
        }
    }

    // Reassemble the tree.
    unsafe {
        *left_tail = root.left.take();
        *right_tail = root.right.take();
    }
    root.left = left_tree;
    root.right = right_tree;
    root
}

/// A Splay Tree providing amortized O(log n) operations.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::splay_tree::SplayTree;
///
/// let mut tree = SplayTree::new();
/// tree.insert(5);
/// tree.insert(3);
/// assert!(tree.contains(&3));
/// // After searching for 3, it is now at the root.
/// ```
#[derive(Debug)]
pub struct SplayTree<T: Ord> {
    root: Option<Box<SplayNode<T>>>,
    size: usize,
}

impl<T: Ord> SplayTree<T> {
    pub fn new() -> Self {
        SplayTree {
            root: None,
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Inserts a value. After insertion, the new value is at the root.
    pub fn insert(&mut self, value: T) {
        match self.root.take() {
            None => {
                self.root = Some(Box::new(SplayNode::new(value)));
                self.size += 1;
            }
            Some(root) => {
                let root = splay(root, &value);
                match value.cmp(&root.value) {
                    Ordering::Equal => {
                        // Value already exists.
                        self.root = Some(root);
                    }
                    Ordering::Less => {
                        let mut new_node = Box::new(SplayNode::new(value));
                        new_node.right = Some(root);
                        // The left subtree of the old root becomes our left subtree.
                        new_node.left = new_node.right.as_mut().unwrap().left.take();
                        self.root = Some(new_node);
                        self.size += 1;
                    }
                    Ordering::Greater => {
                        let mut new_node = Box::new(SplayNode::new(value));
                        new_node.left = Some(root);
                        new_node.right = new_node.left.as_mut().unwrap().right.take();
                        self.root = Some(new_node);
                        self.size += 1;
                    }
                }
            }
        }
    }

    /// Searches for a value. If found, it becomes the root.
    pub fn contains(&mut self, value: &T) -> bool {
        match self.root.take() {
            None => false,
            Some(root) => {
                let root = splay(root, value);
                let found = root.value == *value;
                self.root = Some(root);
                found
            }
        }
    }

    /// Deletes a value from the tree.
    pub fn delete(&mut self, value: &T) {
        match self.root.take() {
            None => {}
            Some(root) => {
                let root = splay(root, value);
                if root.value != *value {
                    self.root = Some(root);
                    return;
                }
                // root.value == value. Now join left and right subtrees.
                match root.left {
                    None => {
                        self.root = root.right;
                    }
                    Some(left) => {
                        // Splay the maximum of the left subtree to become root.
                        let new_root = splay(left, value);
                        // new_root has no right child because value was the
                        // successor of everything in left.
                        let mut new_root = new_root;
                        new_root.right = root.right;
                        self.root = Some(new_root);
                    }
                }
                self.size -= 1;
            }
        }
    }

    /// In-order traversal yielding sorted values.
    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    fn inorder_recursive<'a>(
        node: &'a Option<Box<SplayNode<T>>>,
        result: &mut Vec<&'a T>,
    ) {
        if let Some(n) = node {
            Self::inorder_recursive(&n.left, result);
            result.push(&n.value);
            Self::inorder_recursive(&n.right, result);
        }
    }
}

impl<T: Ord> Default for SplayTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + fmt::Display> fmt::Display for SplayTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.inorder();
        let strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        write!(f, "Splay[{}]", strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_search() {
        let mut tree = SplayTree::new();
        tree.insert(5);
        tree.insert(3);
        tree.insert(7);
        tree.insert(1);
        tree.insert(4);
        assert!(tree.contains(&5));
        assert!(tree.contains(&3));
        assert!(!tree.contains(&99));
    }

    #[test]
    fn test_recently_accessed_at_root() {
        let mut tree = SplayTree::new();
        tree.insert(5);
        tree.insert(3);
        tree.insert(7);
        tree.contains(&3);
        // After accessing 3, it should be at the root.
        assert_eq!(tree.root.as_ref().unwrap().value, 3);
    }

    #[test]
    fn test_delete() {
        let mut tree = SplayTree::new();
        for v in [10, 5, 15, 3, 7, 12, 20] {
            tree.insert(v);
        }
        tree.delete(&10);
        assert!(!tree.contains(&10));
        assert!(tree.contains(&5));
        assert!(tree.contains(&15));
        assert_eq!(tree.len(), 6);
    }

    #[test]
    fn test_inorder_sorted() {
        let mut tree = SplayTree::new();
        for v in [50, 30, 70, 20, 40, 60, 80] {
            tree.insert(v);
        }
        assert_eq!(tree.inorder(), vec![&20, &30, &40, &50, &60, &70, &80]);
    }

    #[test]
    fn test_sequential_access_pattern() {
        // Splay trees excel when access is sequential/repeated.
        let mut tree = SplayTree::new();
        for v in 1..=100 {
            tree.insert(v);
        }
        // Access elements repeatedly — they should become cheap.
        for _ in 0..10 {
            assert!(tree.contains(&50));
        }
    }

    #[test]
    fn test_display() {
        let mut tree = SplayTree::new();
        tree.insert(2);
        tree.insert(1);
        tree.insert(3);
        let s = format!("{}", tree);
        assert!(s.starts_with("Splay["));
    }
}
