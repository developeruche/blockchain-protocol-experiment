// =============================================================================
// Red-Black Tree — Self-Balancing BST with Color Invariants
// =============================================================================
//
// # What is a Red-Black Tree?
//
// A Red-Black tree is a self-balancing BST where each node carries a color
// (Red or Black) and the tree maintains five invariants:
//
// 1. Every node is either Red or Black.
// 2. The root is Black.
// 3. Every leaf (NIL/null) is Black.
// 4. If a node is Red, both its children are Black (no two consecutive reds).
// 5. For each node, all paths from that node to descendant leaves contain the
//    same number of Black nodes (the "black-height").
//
// These invariants guarantee that the longest path from root to leaf is at most
// **2× the shortest path**, ensuring O(log n) operations.
//
// # Why Red-Black Trees?
//
// - Used by `std::collections::BTreeMap` in many languages and by the Linux
//   kernel's scheduler.
// - Fewer rotations on insert/delete than AVL (at most 2 rotations per insert,
//   3 per delete), making them preferred when writes are frequent.
// - In blockchain: Red-Black trees could index in-memory state where frequent
//   updates (block processing) make AVL's extra rotations costly.
//
// # Complexity Table
//
// | Operation | Average  | Worst    |
// |-----------|----------|----------|
// | Search    | O(log n) | O(log n) |
// | Insert    | O(log n) | O(log n) |
// | Delete    | O(log n) | O(log n) |
// | Space     | O(n)     | O(n)     |
//
// # Implementation Note
//
// We use a "left-leaning" Red-Black Tree (LLRB) variant, invented by Robert
// Sedgewick. It simplifies implementation by maintaining an additional
// invariant: red links lean left. This reduces the number of cases to handle.
// The LLRB maps directly to a 2-3 tree (see `two_three_tree.rs`).

use std::cmp::Ordering;
use std::fmt;

/// Node color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Red,
    Black,
}

/// A node in the Red-Black tree.
#[derive(Debug)]
pub struct RbNode<T: Ord> {
    pub value: T,
    pub color: Color,
    pub left: Option<Box<RbNode<T>>>,
    pub right: Option<Box<RbNode<T>>>,
}

impl<T: Ord> RbNode<T> {
    fn new(value: T) -> Self {
        // New nodes are always Red. This is because inserting a Red node
        // doesn't violate invariant 5 (black-height). We then fix invariant 4
        // (no consecutive reds) via rotations and color flips.
        RbNode {
            value,
            color: Color::Red,
            left: None,
            right: None,
        }
    }
}

/// Checks if a node is Red. `None` (null leaves) are Black by convention.
fn is_red<T: Ord>(node: &Option<Box<RbNode<T>>>) -> bool {
    node.as_ref().is_some_and(|n| n.color == Color::Red)
}

// =============================================================================
// LLRB Operations — Rotations and Color Flips
// =============================================================================

/// Left rotation: makes the right child the new root of this subtree.
/// Used when a red link leans right (violating LLRB invariant).
fn rotate_left<T: Ord>(mut h: Box<RbNode<T>>) -> Box<RbNode<T>> {
    let mut x = h.right.take().expect("rotate_left requires right child");
    h.right = x.left.take();
    x.color = h.color;
    h.color = Color::Red;
    x.left = Some(h);
    x
}

/// Right rotation: makes the left child the new root of this subtree.
/// Used to fix two consecutive left-leaning red links.
fn rotate_right<T: Ord>(mut h: Box<RbNode<T>>) -> Box<RbNode<T>> {
    let mut x = h.left.take().expect("rotate_right requires left child");
    h.left = x.right.take();
    x.color = h.color;
    h.color = Color::Red;
    x.right = Some(h);
    x
}

/// Color flip: toggles the colors of a node and its two children.
/// In a 2-3 tree analogy, this corresponds to splitting a temporary 4-node
/// by pushing the middle key up.
fn flip_colors<T: Ord>(h: &mut Box<RbNode<T>>) {
    h.color = if h.color == Color::Red {
        Color::Black
    } else {
        Color::Red
    };
    if let Some(ref mut left) = h.left {
        left.color = if left.color == Color::Red {
            Color::Black
        } else {
            Color::Red
        };
    }
    if let Some(ref mut right) = h.right {
        right.color = if right.color == Color::Red {
            Color::Black
        } else {
            Color::Red
        };
    }
}

/// Restores LLRB invariants after an insertion or deletion.
fn fixup<T: Ord>(mut h: Box<RbNode<T>>) -> Box<RbNode<T>> {
    // Case 1: Right-leaning red link → rotate left to lean left.
    if is_red(&h.right) && !is_red(&h.left) {
        h = rotate_left(h);
    }
    // Case 2: Two consecutive left-leaning red links → rotate right.
    if is_red(&h.left) && h.left.as_ref().is_some_and(|l| is_red(&l.left)) {
        h = rotate_right(h);
    }
    // Case 3: Both children red → flip colors (split the 4-node).
    if is_red(&h.left) && is_red(&h.right) {
        flip_colors(&mut h);
    }
    h
}

/// A Left-Leaning Red-Black Tree.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::red_black_tree::RedBlackTree;
///
/// let mut tree = RedBlackTree::new();
/// tree.insert(5);
/// tree.insert(3);
/// tree.insert(7);
/// assert!(tree.contains(&5));
/// ```
#[derive(Debug)]
pub struct RedBlackTree<T: Ord> {
    root: Option<Box<RbNode<T>>>,
    size: usize,
}

impl<T: Ord> RedBlackTree<T> {
    pub fn new() -> Self {
        RedBlackTree {
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

    /// Inserts a value into the tree.
    pub fn insert(&mut self, value: T) {
        let (new_root, inserted) = Self::insert_recursive(self.root.take(), value);
        self.root = Some(new_root);
        // Invariant 2: root is always Black.
        if let Some(ref mut root) = self.root {
            root.color = Color::Black;
        }
        if inserted {
            self.size += 1;
        }
    }

    fn insert_recursive(
        node: Option<Box<RbNode<T>>>,
        value: T,
    ) -> (Box<RbNode<T>>, bool) {
        match node {
            None => (Box::new(RbNode::new(value)), true),
            Some(mut h) => {
                let inserted;
                match value.cmp(&h.value) {
                    Ordering::Less => {
                        let (new_left, ins) = Self::insert_recursive(h.left.take(), value);
                        h.left = Some(new_left);
                        inserted = ins;
                    }
                    Ordering::Greater => {
                        let (new_right, ins) = Self::insert_recursive(h.right.take(), value);
                        h.right = Some(new_right);
                        inserted = ins;
                    }
                    Ordering::Equal => return (h, false),
                }
                (fixup(h), inserted)
            }
        }
    }

    /// Searches for a value. O(log n).
    pub fn contains(&self, value: &T) -> bool {
        Self::search_recursive(&self.root, value)
    }

    fn search_recursive(node: &Option<Box<RbNode<T>>>, value: &T) -> bool {
        match node {
            None => false,
            Some(n) => match value.cmp(&n.value) {
                Ordering::Less => Self::search_recursive(&n.left, value),
                Ordering::Greater => Self::search_recursive(&n.right, value),
                Ordering::Equal => true,
            },
        }
    }

    /// Deletes the minimum value from the tree (used as a building block).
    pub fn delete_min(&mut self)
    where
        T: Clone,
    {
        if self.root.is_none() {
            return;
        }
        // If both children of root are black, set root to red.
        if let Some(ref root) = self.root
            && !is_red(&root.left) && !is_red(&root.right)
                && let Some(ref mut root) = self.root {
                    root.color = Color::Red;
                }
        let root = self.root.take().unwrap();
        let (new_root, deleted) = Self::delete_min_recursive(root);
        self.root = new_root;
        if let Some(ref mut root_node) = self.root {
            root_node.color = Color::Black;
        }
        if deleted {
            self.size -= 1;
        }
    }

    fn delete_min_recursive(
        mut h: Box<RbNode<T>>,
    ) -> (Option<Box<RbNode<T>>>, bool)
    where
        T: Clone,
    {
        if h.left.is_none() {
            return (None, true);
        }
        if !is_red(&h.left) && !h.left.as_ref().is_some_and(|l| is_red(&l.left)) {
            h = Self::move_red_left(h);
        }
        let (new_left, deleted) = Self::delete_min_recursive(h.left.take().unwrap());
        h.left = new_left;
        (Some(fixup(h)), deleted)
    }

    fn move_red_left<T2: Ord>(mut h: Box<RbNode<T2>>) -> Box<RbNode<T2>> {
        flip_colors(&mut h);
        if h.right.as_ref().is_some_and(|r| is_red(&r.left)) {
            h.right = Some(rotate_right(h.right.take().unwrap()));
            h = rotate_left(h);
            flip_colors(&mut h);
        }
        h
    }

    /// Deletes a value from the tree.
    pub fn delete(&mut self, value: &T)
    where
        T: Clone,
    {
        if !self.contains(value) {
            return;
        }
        if let Some(ref root) = self.root
            && !is_red(&root.left) && !is_red(&root.right)
                && let Some(ref mut root) = self.root {
                    root.color = Color::Red;
                }
        let root = self.root.take().unwrap();
        let new_root = Self::delete_recursive(root, value);
        self.root = new_root;
        if let Some(ref mut root_node) = self.root {
            root_node.color = Color::Black;
        }
        self.size -= 1;
    }

    fn delete_recursive(mut h: Box<RbNode<T>>, value: &T) -> Option<Box<RbNode<T>>>
    where
        T: Clone,
    {
        if *value < h.value {
            if !is_red(&h.left) && !h.left.as_ref().is_some_and(|l| is_red(&l.left)) {
                h = Self::move_red_left(h);
            }
            if let Some(left) = h.left.take() {
                h.left = Self::delete_recursive(left, value);
            }
        } else {
            if is_red(&h.left) {
                h = rotate_right(h);
            }
            if *value == h.value && h.right.is_none() {
                return None;
            }
            if !is_red(&h.right) && !h.right.as_ref().is_some_and(|r| is_red(&r.left)) {
                h = Self::move_red_right(h);
            }
            if *value == h.value {
                let min = Self::find_min(h.right.as_ref().unwrap());
                h.value = min;
                let (new_right, _) = Self::delete_min_recursive(h.right.take().unwrap());
                h.right = new_right;
            } else if let Some(right) = h.right.take() {
                h.right = Self::delete_recursive(right, value);
            }
        }
        Some(fixup(h))
    }

    fn move_red_right(mut h: Box<RbNode<T>>) -> Box<RbNode<T>>
    where
        T: Clone,
    {
        flip_colors(&mut h);
        if h.left.as_ref().is_some_and(|l| is_red(&l.left)) {
            h = rotate_right(h);
            flip_colors(&mut h);
        }
        h
    }

    fn find_min(node: &Box<RbNode<T>>) -> T
    where
        T: Clone,
    {
        match &node.left {
            None => node.value.clone(),
            Some(left) => Self::find_min(left),
        }
    }

    /// In-order traversal yielding sorted values.
    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    fn inorder_recursive<'a>(node: &'a Option<Box<RbNode<T>>>, result: &mut Vec<&'a T>) {
        if let Some(n) = node {
            Self::inorder_recursive(&n.left, result);
            result.push(&n.value);
            Self::inorder_recursive(&n.right, result);
        }
    }

    /// Verifies the Red-Black tree invariants hold.
    pub fn is_valid(&self) -> bool {
        // Check root is black.
        if let Some(ref root) = self.root
            && root.color != Color::Black {
                return false;
            }
        // Check no consecutive reds and consistent black-height.
        Self::check_invariants(&self.root).is_some()
    }

    /// Returns Some(black_height) if invariants hold, None if violated.
    fn check_invariants(node: &Option<Box<RbNode<T>>>) -> Option<usize> {
        match node {
            None => Some(1), // NIL nodes are black, contributing 1 to black-height.
            Some(n) => {
                // Check invariant 4: no consecutive reds.
                if n.color == Color::Red
                    && (is_red(&n.left) || is_red(&n.right)) {
                        return None;
                    }
                let left_bh = Self::check_invariants(&n.left)?;
                let right_bh = Self::check_invariants(&n.right)?;
                // Check invariant 5: equal black-height on both sides.
                if left_bh != right_bh {
                    return None;
                }
                // Add 1 if this node is black.
                Some(left_bh + if n.color == Color::Black { 1 } else { 0 })
            }
        }
    }
}

impl<T: Ord> Default for RedBlackTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + fmt::Display> fmt::Display for RedBlackTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.inorder();
        let strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        write!(f, "RB[{}]", strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_search() {
        let mut tree = RedBlackTree::new();
        for v in [10, 20, 30, 40, 50, 25] {
            tree.insert(v);
        }
        for v in [10, 20, 30, 40, 50, 25] {
            assert!(tree.contains(&v), "should contain {}", v);
        }
        assert!(!tree.contains(&99));
    }

    #[test]
    fn test_invariants_after_sorted_insert() {
        let mut tree = RedBlackTree::new();
        for v in 1..=20 {
            tree.insert(v);
        }
        assert!(tree.is_valid(), "RB invariants should hold after sorted insert");
        assert_eq!(tree.len(), 20);
    }

    #[test]
    fn test_delete() {
        let mut tree = RedBlackTree::new();
        for v in [10, 5, 15, 3, 7, 12, 20] {
            tree.insert(v);
        }
        tree.delete(&15);
        assert!(!tree.contains(&15));
        assert!(tree.is_valid());
        tree.delete(&10);
        assert!(!tree.contains(&10));
        assert!(tree.is_valid());
    }

    #[test]
    fn test_inorder_sorted() {
        let mut tree = RedBlackTree::new();
        for v in [50, 30, 70, 20, 40, 60, 80] {
            tree.insert(v);
        }
        assert_eq!(tree.inorder(), vec![&20, &30, &40, &50, &60, &70, &80]);
    }

    #[test]
    fn test_delete_min() {
        let mut tree = RedBlackTree::new();
        for v in [5, 3, 7, 1, 4] {
            tree.insert(v);
        }
        tree.delete_min();
        assert!(!tree.contains(&1));
        assert!(tree.is_valid());
    }

    #[test]
    fn test_display() {
        let mut tree = RedBlackTree::new();
        tree.insert(2);
        tree.insert(1);
        tree.insert(3);
        assert_eq!(format!("{}", tree), "RB[1, 2, 3]");
    }
}
