// =============================================================================
// B-Tree — Balanced Multi-Way Search Tree
// =============================================================================
//
// # What is a B-Tree?
//
// A B-Tree of order `m` is a self-balancing tree where:
// - Each internal node has at most `m` children and `m-1` keys.
// - Each internal node (except root) has at least ⌈m/2⌉ children.
// - The root has at least 2 children if it is not a leaf.
// - All leaves appear at the same depth (perfect balancing).
//
// # Why B-Trees?
//
// B-Trees are optimized for systems that read/write large blocks of data:
// - **Databases**: File-system indexes, SQLite, PostgreSQL all use B-Trees.
// - **Blockchain**: LevelDB/RocksDB (used by Ethereum clients for state storage)
//   use LSM-trees that share B-Tree concepts, and B+ Trees are used for
//   on-disk sorted indexes.
// - They minimize disk I/O by storing many keys per node (high fanout), so
//   tree height stays very small even for millions of keys.
//
// # B-Tree vs B+ Tree
//
// - **B-Tree**: Keys and values in all nodes.
// - **B+ Tree**: Values only in leaves; internal nodes are pure index.
//   B+ Trees are better for range scans (leaves form a linked list).
//
// This implementation is a standard B-Tree of order `m` (configurable).
// We default to order 4 (a 2-3-4 tree), which maps directly to a Red-Black tree.
//
// # Complexity Table
//
// | Operation | Time         |
// |-----------|-------------|
// | Search    | O(log n)    |
// | Insert    | O(log n)    |
// | Delete    | O(log n)    |
// | Space     | O(n)        |
//
// The base of the logarithm is m/2, so B-Trees are very shallow: a B-Tree
// of order 1000 with 1 billion keys has height ≤ 3.
//
// # Invariants
//
// 1. Keys within each node are sorted.
// 2. For a node with keys [k₀, k₁, ..., kₙ], child[i] contains keys in
//    range (k_{i-1}, k_i). Child[0] < k₀, child[n+1] > kₙ.
// 3. All leaves are at the same depth.
// 4. Each node has between ⌈m/2⌉-1 and m-1 keys (except root).

use std::fmt;

/// The order (maximum number of children per node).
/// Order 4 gives us a 2-3-4 tree.
const DEFAULT_ORDER: usize = 4;

/// A B-Tree node.
///
/// Uses `Vec` for keys and children for simplicity. In a production system,
/// you might use fixed-size arrays for cache locality.
#[derive(Debug, Clone)]
pub struct BTreeNode<T: Ord + Clone> {
    /// Sorted keys stored in this node.
    pub keys: Vec<T>,
    /// Child pointers. `children.len()` is always `keys.len() + 1` for
    /// internal nodes, or 0 for leaf nodes.
    pub children: Vec<Box<BTreeNode<T>>>,
    /// Whether this node is a leaf (has no children).
    pub leaf: bool,
}

impl<T: Ord + Clone> BTreeNode<T> {
    fn new(leaf: bool) -> Self {
        BTreeNode {
            keys: Vec::new(),
            children: Vec::new(),
            leaf,
        }
    }

    /// Returns true if this node is "full" (has m-1 keys).
    fn is_full(&self, order: usize) -> bool {
        self.keys.len() == order - 1
    }

    /// Searches for a key within this subtree.
    fn search(&self, key: &T) -> bool {
        // Binary search within the sorted keys of this node.
        match self.keys.binary_search(key) {
            Ok(_) => true, // Found the key in this node.
            Err(i) => {
                if self.leaf {
                    false // Key not found and we're at a leaf.
                } else {
                    // Recurse into the appropriate child.
                    // `i` is the index where the key would be inserted,
                    // so child[i] contains keys in the correct range.
                    self.children[i].search(key)
                }
            }
        }
    }

    /// Collects all keys in sorted order via in-order traversal.
    fn inorder<'a>(&'a self, result: &mut Vec<&'a T>) {
        for i in 0..self.keys.len() {
            if !self.leaf {
                self.children[i].inorder(result);
            }
            result.push(&self.keys[i]);
        }
        // Don't forget the rightmost child.
        if !self.leaf
            && let Some(last_child) = self.children.last() {
                last_child.inorder(result);
            }
    }
}

/// A B-Tree with configurable order.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::b_tree::BTree;
///
/// let mut tree = BTree::new(4); // Order 4 (2-3-4 tree)
/// tree.insert(10);
/// tree.insert(20);
/// tree.insert(5);
/// assert!(tree.contains(&10));
/// ```
#[derive(Debug)]
pub struct BTree<T: Ord + Clone> {
    root: Option<Box<BTreeNode<T>>>,
    order: usize,
    size: usize,
}

impl<T: Ord + Clone> BTree<T> {
    /// Creates a new B-Tree with the given order (max children per node).
    ///
    /// # Panics
    /// Panics if `order < 3`. A B-Tree of order 2 is just a BST, which defeats
    /// the purpose.
    pub fn new(order: usize) -> Self {
        assert!(order >= 3, "B-Tree order must be at least 3");
        BTree {
            root: None,
            order,
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Searches for a key. O(log_m n) where m is the order.
    pub fn contains(&self, key: &T) -> bool {
        match &self.root {
            None => false,
            Some(root) => root.search(key),
        }
    }

    /// Inserts a key into the B-Tree.
    ///
    /// # Algorithm — Proactive Splitting
    ///
    /// We use the "proactive" (top-down) approach: as we walk down the tree
    /// looking for the insertion point, we split any full node we encounter.
    /// This ensures that when we reach a leaf, there is always room to insert.
    ///
    /// This is simpler than the "reactive" approach (insert then split upward)
    /// and requires only a single pass down the tree.
    pub fn insert(&mut self, key: T) {
        match self.root.take() {
            None => {
                // Tree is empty — create a leaf root.
                let mut root = BTreeNode::new(true);
                root.keys.push(key);
                self.root = Some(Box::new(root));
                self.size += 1;
            }
            Some(mut root) => {
                if root.is_full(self.order) {
                    // Root is full — we need to split it, creating a new root.
                    // This is the ONLY way the tree grows in height.
                    let (left, median, right) = self.split_node(*root);
                    let mut new_root = BTreeNode::new(false);
                    new_root.keys.push(median);
                    new_root.children.push(Box::new(left));
                    new_root.children.push(Box::new(right));
                    self.insert_non_full(&mut new_root, key);
                    self.root = Some(Box::new(new_root));
                } else {
                    self.insert_non_full(&mut root, key);
                    self.root = Some(root);
                }
                self.size += 1;
            }
        }
    }

    /// Inserts into a node that is guaranteed to not be full.
    fn insert_non_full(&self, node: &mut BTreeNode<T>, key: T) {
        if node.leaf {
            // Find the correct position and insert.
            let pos = node.keys.binary_search(&key).unwrap_or_else(|i| i);
            node.keys.insert(pos, key);
        } else {
            // Find which child to recurse into.
            let mut i = node.keys.binary_search(&key).unwrap_or_else(|i| i);
            // If the child is full, split it first.
            if node.children[i].is_full(self.order) {
                let child = *node.children.remove(i);
                let (left, median, right) = self.split_node(child);
                node.keys.insert(i, median);
                node.children.insert(i, Box::new(left));
                node.children.insert(i + 1, Box::new(right));
                // After splitting, decide which of the two children to descend into.
                if key > node.keys[i] {
                    i += 1;
                }
            }
            self.insert_non_full(&mut node.children[i], key);
        }
    }

    /// Splits a full node into (left_half, median_key, right_half).
    ///
    /// For a node with `order - 1` keys (indices 0..order-2):
    /// - The median is at index `(order - 1) / 2`.
    /// - Left gets keys `[0..median_idx)`.
    /// - Right gets keys `(median_idx..end]`.
    /// - Children are split correspondingly.
    fn split_node(&self, mut node: BTreeNode<T>) -> (BTreeNode<T>, T, BTreeNode<T>) {
        let mid = (self.order - 1) / 2;

        // Split: keys = [0..mid-1] ++ [mid] ++ [mid+1..end]
        let right_keys = node.keys.split_off(mid + 1);
        let median = node.keys.pop().unwrap(); // pop the median
        // node.keys now contains [0..mid-1] = left keys

        let mut left = BTreeNode::new(node.leaf);
        left.keys = node.keys;

        let mut right = BTreeNode::new(node.leaf);
        right.keys = right_keys;

        if !node.leaf {
            // Children: left gets [0..mid], right gets [mid+1..end]
            let right_children = node.children.split_off(mid + 1);
            left.children = node.children;
            right.children = right_children;
        }

        (left, median, right)
    }

    /// In-order traversal returns all keys sorted.
    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        if let Some(ref root) = self.root {
            root.inorder(&mut result);
        }
        result
    }

    /// Deletes a key from the B-Tree.
    ///
    /// B-Tree deletion is significantly more complex than insertion because
    /// we must maintain the minimum key count invariant. We handle three cases:
    /// 1. Key is in a leaf — remove directly (if the leaf has enough keys).
    /// 2. Key is in an internal node — replace with predecessor/successor.
    /// 3. Key is in a child that might be too small — merge/borrow first.
    pub fn delete(&mut self, key: &T) {
        if self.root.is_none() {
            return;
        }
        let contained = self.contains(key);
        if !contained {
            return;
        }
        let mut root = self.root.take().unwrap();
        Self::delete_from_node(&mut root, key, self.order);
        // If root became empty (can happen after merging), make the single
        // child the new root. This is how the tree shrinks in height.
        if root.keys.is_empty() && !root.leaf {
            self.root = Some(root.children.remove(0));
        } else {
            self.root = Some(root);
        }
        self.size -= 1;
    }

    fn delete_from_node(node: &mut BTreeNode<T>, key: &T, order: usize) {
        let min_keys = (order - 1) / 2; // Minimum keys for non-root nodes.

        if let Ok(idx) = node.keys.binary_search(key) {
            if node.leaf {
                // Case 1: Key in leaf — simply remove.
                node.keys.remove(idx);
            } else {
                // Case 2: Key in internal node.
                // Replace with predecessor (max of left child) or successor.
                if node.children[idx].keys.len() > min_keys {
                    // Use predecessor.
                    let pred = Self::get_predecessor(&node.children[idx]);
                    node.keys[idx] = pred.clone();
                    Self::delete_from_node(&mut node.children[idx], &pred, order);
                } else if node.children[idx + 1].keys.len() > min_keys {
                    // Use successor.
                    let succ = Self::get_successor(&node.children[idx + 1]);
                    node.keys[idx] = succ.clone();
                    Self::delete_from_node(&mut node.children[idx + 1], &succ, order);
                } else {
                    // Both children at minimum — merge them.
                    Self::merge_children(node, idx);
                    Self::delete_from_node(&mut node.children[idx], key, order);
                }
            }
        } else if !node.leaf {
            // Key not in this node — find the child to recurse into.
            let idx = node.keys.binary_search(key).unwrap_or_else(|i| i);
            // Ensure the child has enough keys before recursing.
            if node.children[idx].keys.len() <= min_keys {
                Self::fill_child(node, idx, order);
            }
            // After filling, the index might have changed due to merging.
            let idx = node.keys.binary_search(key).unwrap_or_else(|i| i);
            let idx = idx.min(node.children.len() - 1);
            Self::delete_from_node(&mut node.children[idx], key, order);
        }
    }

    fn get_predecessor(node: &BTreeNode<T>) -> T {
        let mut current = node;
        while !current.leaf {
            current = current.children.last().unwrap();
        }
        current.keys.last().unwrap().clone()
    }

    fn get_successor(node: &BTreeNode<T>) -> T {
        let mut current = node;
        while !current.leaf {
            current = &current.children[0];
        }
        current.keys[0].clone()
    }

    fn fill_child(node: &mut BTreeNode<T>, idx: usize, order: usize) {
        let min_keys = (order - 1) / 2;
        if idx > 0 && node.children[idx - 1].keys.len() > min_keys {
            // Borrow from left sibling.
            Self::borrow_from_left(node, idx);
        } else if idx < node.children.len() - 1 && node.children[idx + 1].keys.len() > min_keys {
            // Borrow from right sibling.
            Self::borrow_from_right(node, idx);
        } else {
            // Neither sibling can spare a key — merge.
            if idx < node.children.len() - 1 {
                Self::merge_children(node, idx);
            } else {
                Self::merge_children(node, idx - 1);
            }
        }
    }

    fn borrow_from_left(node: &mut BTreeNode<T>, idx: usize) {
        // Move parent key down to child, move sibling's last key up to parent.
        let parent_key = node.keys[idx - 1].clone();
        let sibling_key = node.children[idx - 1].keys.pop().unwrap();
        node.keys[idx - 1] = sibling_key;
        node.children[idx].keys.insert(0, parent_key);
        if !node.children[idx - 1].leaf {
            let child = node.children[idx - 1].children.pop().unwrap();
            node.children[idx].children.insert(0, child);
        }
    }

    fn borrow_from_right(node: &mut BTreeNode<T>, idx: usize) {
        let parent_key = node.keys[idx].clone();
        let sibling_key = node.children[idx + 1].keys.remove(0);
        node.keys[idx] = sibling_key;
        node.children[idx].keys.push(parent_key);
        if !node.children[idx + 1].leaf {
            let child = node.children[idx + 1].children.remove(0);
            node.children[idx].children.push(child);
        }
    }

    fn merge_children(node: &mut BTreeNode<T>, idx: usize) {
        let separator = node.keys.remove(idx);
        let right = node.children.remove(idx + 1);
        node.children[idx].keys.push(separator);
        node.children[idx].keys.extend(right.keys);
        node.children[idx].children.extend(right.children);
    }
}

impl<T: Ord + Clone> Default for BTree<T> {
    fn default() -> Self {
        Self::new(DEFAULT_ORDER)
    }
}

impl<T: Ord + Clone + fmt::Display> fmt::Display for BTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.inorder();
        let strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        write!(f, "BTree[{}]", strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_search() {
        let mut tree = BTree::new(4);
        for v in [10, 20, 5, 6, 12, 30, 7, 17] {
            tree.insert(v);
        }
        for v in [10, 20, 5, 6, 12, 30, 7, 17] {
            assert!(tree.contains(&v), "should contain {}", v);
        }
        assert!(!tree.contains(&99));
        assert_eq!(tree.len(), 8);
    }

    #[test]
    fn test_inorder_sorted() {
        let mut tree = BTree::new(3);
        let values = [15, 5, 25, 10, 20, 30, 1, 8];
        for v in values {
            tree.insert(v);
        }
        let sorted = tree.inorder();
        let mut expected: Vec<i32> = values.to_vec();
        expected.sort();
        let expected_refs: Vec<&i32> = expected.iter().collect();
        assert_eq!(sorted, expected_refs);
    }

    #[test]
    fn test_large_insert() {
        let mut tree = BTree::new(5);
        for v in (0..100).rev() {
            tree.insert(v);
        }
        assert_eq!(tree.len(), 100);
        for v in 0..100 {
            assert!(tree.contains(&v));
        }
    }

    #[test]
    fn test_delete() {
        let mut tree = BTree::new(4);
        for v in [10, 20, 5, 6, 12, 30, 7, 17] {
            tree.insert(v);
        }
        tree.delete(&6);
        assert!(!tree.contains(&6));
        assert!(tree.contains(&7));
        tree.delete(&20);
        assert!(!tree.contains(&20));
        // Verify remaining elements.
        for v in [10, 5, 12, 30, 7, 17] {
            assert!(tree.contains(&v), "should still contain {}", v);
        }
    }

    #[test]
    fn test_display() {
        let mut tree = BTree::new(3);
        tree.insert(3);
        tree.insert(1);
        tree.insert(2);
        let s = format!("{}", tree);
        assert!(s.starts_with("BTree["));
    }

    #[test]
    fn test_delete_all() {
        let mut tree = BTree::new(4);
        let values = vec![5, 3, 7, 1, 4, 6, 8];
        for &v in &values {
            tree.insert(v);
        }
        for &v in &values {
            tree.delete(&v);
        }
        assert!(tree.is_empty());
    }
}
