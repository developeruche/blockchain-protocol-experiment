// =============================================================================
// Binary Tree — Foundation Data Structure
// =============================================================================
//
// # What is a Binary Tree?
//
// A binary tree is a hierarchical data structure where each node has at most
// two children, referred to as the *left* and *right* child. Binary trees are
// the foundation for nearly all tree-based data structures (BSTs, AVL, Red-Black,
// B-Trees, Tries) and appear everywhere in blockchain engineering:
//
// - **Merkle Trees** are binary hash trees used to verify data integrity in
//   blocks (Bitcoin, Ethereum).
// - **State Tries** in Ethereum extend tree concepts to map keys → values.
// - **Execution traces** can be represented as binary trees for fraud proofs.
//
// # Variant: Binary Search Tree (BST)
//
// This implementation is specifically a **Binary Search Tree (BST)**, which
// maintains the invariant:
//
//   For every node N:
//     - All values in N's left subtree are **< N.value**
//     - All values in N's right subtree are **> N.value**
//
// This invariant enables O(log n) average-case search, insert, and delete.
//
// # Complexity Table
//
// | Operation   | Average  | Worst (degenerate) |
// |-------------|----------|--------------------|
// | Search      | O(log n) | O(n)               |
// | Insert      | O(log n) | O(n)               |
// | Delete      | O(log n) | O(n)               |
// | Traversal   | O(n)     | O(n)               |
// | Space       | O(n)     | O(n)               |
//
// Worst case occurs when the tree degenerates into a linked list (e.g., inserting
// sorted data). Self-balancing trees (AVL, Red-Black) fix this.
//
// # Why `Box<T>` for child pointers?
//
// In Rust, recursive types must have a known size at compile time. `Box<T>`
// provides heap allocation with a fixed-size pointer on the stack. We use
// `Option<Box<Node<T>>>` because a child may or may not exist.
//
// We do NOT need `Rc<RefCell<>>` here because ownership flows strictly downward
// from parent to child — there is no shared ownership or need for interior
// mutability.

use std::fmt;

/// A single node in the binary search tree.
///
/// Each node owns its children via `Box`, giving us clear, single-owner
/// semantics that Rust's borrow checker can verify at compile time.
#[derive(Debug)]
pub struct Node<T: Ord> {
    /// The value stored in this node.
    pub value: T,
    /// Left child — contains values strictly less than `self.value`.
    pub left: Option<Box<Node<T>>>,
    /// Right child — contains values strictly greater than `self.value`.
    pub right: Option<Box<Node<T>>>,
}

impl<T: Ord> Node<T> {
    /// Creates a new leaf node (no children).
    pub fn new(value: T) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }
}

/// A Binary Search Tree (BST) with single-owner heap-allocated nodes.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::binary_tree::BinarySearchTree;
///
/// let mut bst = BinarySearchTree::new();
/// bst.insert(5);
/// bst.insert(3);
/// bst.insert(7);
/// assert!(bst.contains(&5));
/// assert!(!bst.contains(&4));
/// ```
#[derive(Debug)]
pub struct BinarySearchTree<T: Ord> {
    /// The root node of the tree. `None` means the tree is empty.
    pub root: Option<Box<Node<T>>>,
    /// Number of elements in the tree. Maintained to provide O(1) `len()`.
    size: usize,
}

impl<T: Ord + Clone> BinarySearchTree<T> {
    /// Creates a new, empty BST.
    pub fn new() -> Self {
        BinarySearchTree {
            root: None,
            size: 0,
        }
    }

    /// Returns the number of elements in the tree.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns `true` if the tree contains no elements.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Inserts a value into the BST.
    ///
    /// If the value already exists, this is a no-op (BST stores unique values).
    ///
    /// # Algorithm
    /// Walk down the tree comparing values:
    /// - If `value < current`, go left.
    /// - If `value > current`, go right.
    /// - If `value == current`, the value already exists — return.
    /// When we find a `None` slot, insert the new node there.
    pub fn insert(&mut self, value: T) {
        if Self::insert_recursive(&mut self.root, value) {
            self.size += 1;
        }
    }

    /// Recursive helper for insert. Returns `true` if a new node was created.
    fn insert_recursive(node: &mut Option<Box<Node<T>>>, value: T) -> bool {
        match node {
            // We found an empty slot — create a new leaf node here.
            None => {
                *node = Some(Box::new(Node::new(value)));
                true
            }
            Some(n) => {
                use std::cmp::Ordering;
                match value.cmp(&n.value) {
                    // Value is smaller — recurse into the left subtree.
                    Ordering::Less => Self::insert_recursive(&mut n.left, value),
                    // Value is larger — recurse into the right subtree.
                    Ordering::Greater => Self::insert_recursive(&mut n.right, value),
                    // Value already exists — BST stores unique values only.
                    Ordering::Equal => false,
                }
            }
        }
    }

    /// Returns `true` if the tree contains the given value.
    ///
    /// Leverages the BST invariant to achieve O(log n) average-case search
    /// by eliminating half the remaining tree at each step.
    pub fn contains(&self, value: &T) -> bool {
        Self::search_recursive(&self.root, value)
    }

    /// Recursive helper for search.
    fn search_recursive(node: &Option<Box<Node<T>>>, value: &T) -> bool {
        match node {
            None => false,
            Some(n) => {
                use std::cmp::Ordering;
                match value.cmp(&n.value) {
                    Ordering::Less => Self::search_recursive(&n.left, value),
                    Ordering::Greater => Self::search_recursive(&n.right, value),
                    Ordering::Equal => true,
                }
            }
        }
    }

    /// Deletes a value from the BST.
    ///
    /// # Algorithm — Three Cases
    ///
    /// 1. **Leaf node**: Simply remove it.
    /// 2. **One child**: Replace the node with its single child.
    /// 3. **Two children**: Replace the node's value with its **in-order
    ///    successor** (smallest value in the right subtree), then delete the
    ///    successor from the right subtree.
    ///
    /// The in-order successor approach maintains the BST invariant because:
    /// - The successor is greater than everything in the left subtree (it was
    ///   in the right subtree).
    /// - The successor is the *smallest* in the right subtree, so everything
    ///   remaining in the right subtree is still greater.
    pub fn delete(&mut self, value: &T) {
        if Self::delete_recursive(&mut self.root, value) {
            self.size -= 1;
        }
    }

    /// Recursive helper for delete. Returns `true` if a node was removed.
    fn delete_recursive(node: &mut Option<Box<Node<T>>>, value: &T) -> bool {
        // Take ownership of the current node option, allowing us to restructure
        // the tree without fighting the borrow checker.
        let current = match node {
            None => return false,
            Some(n) => n,
        };

        use std::cmp::Ordering;
        match value.cmp(&current.value) {
            Ordering::Less => Self::delete_recursive(&mut current.left, value),
            Ordering::Greater => Self::delete_recursive(&mut current.right, value),
            Ordering::Equal => {
                // Found the node to delete.
                match (current.left.take(), current.right.take()) {
                    // Case 1: Leaf node — remove it entirely.
                    (None, None) => {
                        *node = None;
                    }
                    // Case 2a: Only left child — promote it.
                    (left, None) => {
                        *node = left;
                    }
                    // Case 2b: Only right child — promote it.
                    (None, right) => {
                        *node = right;
                    }
                    // Case 3: Two children — find in-order successor.
                    (left, right) => {
                        // Put children back before we restructure.
                        current.left = left;
                        current.right = right;
                        // Find the smallest value in the right subtree.
                        let successor_value = Self::find_min(&current.right).unwrap();
                        // Replace our value with the successor's value.
                        // Safety: we know the right subtree is non-empty because we're
                        // in the two-children case.
                        current.value = successor_value;
                        // Delete the successor from the right subtree (it's now a
                        // duplicate since we copied its value up).
                        Self::delete_recursive(&mut current.right, &current.value);
                    }
                }
                true
            }
        }
    }

    /// Finds the minimum value in a subtree (leftmost node).
    fn find_min(node: &Option<Box<Node<T>>>) -> Option<T>
    where
        T: Clone,
    {
        match node {
            None => None,
            Some(n) => {
                if n.left.is_none() {
                    // This is the leftmost node — it holds the minimum.
                    Some(n.value.clone())
                } else {
                    Self::find_min(&n.left)
                }
            }
        }
    }

    // =========================================================================
    // Traversal Methods
    // =========================================================================
    //
    // Three classic depth-first traversals:
    //
    // - **In-order** (Left, Node, Right): Yields values in sorted order for BSTs.
    //   This is the most common traversal for BSTs.
    //
    // - **Pre-order** (Node, Left, Right): Useful for serialization — you can
    //   reconstruct the tree from a pre-order sequence.
    //
    // - **Post-order** (Left, Right, Node): Useful for deletion and computing
    //   aggregate properties bottom-up (e.g., subtree sizes, Merkle hashes).

    /// Returns elements in **in-order** (sorted) sequence.
    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    fn inorder_recursive<'a>(node: &'a Option<Box<Node<T>>>, result: &mut Vec<&'a T>) {
        if let Some(n) = node {
            Self::inorder_recursive(&n.left, result);
            result.push(&n.value);
            Self::inorder_recursive(&n.right, result);
        }
    }

    /// Returns elements in **pre-order** (root first) sequence.
    pub fn preorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::preorder_recursive(&self.root, &mut result);
        result
    }

    fn preorder_recursive<'a>(node: &'a Option<Box<Node<T>>>, result: &mut Vec<&'a T>) {
        if let Some(n) = node {
            result.push(&n.value);
            Self::preorder_recursive(&n.left, result);
            Self::preorder_recursive(&n.right, result);
        }
    }

    /// Returns elements in **post-order** (leaves first) sequence.
    pub fn postorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::postorder_recursive(&self.root, &mut result);
        result
    }

    fn postorder_recursive<'a>(node: &'a Option<Box<Node<T>>>, result: &mut Vec<&'a T>) {
        if let Some(n) = node {
            Self::postorder_recursive(&n.left, result);
            Self::postorder_recursive(&n.right, result);
            result.push(&n.value);
        }
    }

    /// Returns the height of the tree.
    ///
    /// The height of an empty tree is 0. The height of a single node is 1.
    /// This is used by self-balancing trees to check balance factors.
    pub fn height(&self) -> usize {
        Self::height_recursive(&self.root)
    }

    fn height_recursive(node: &Option<Box<Node<T>>>) -> usize {
        match node {
            None => 0,
            Some(n) => {
                let left_height = Self::height_recursive(&n.left);
                let right_height = Self::height_recursive(&n.right);
                1 + left_height.max(right_height)
            }
        }
    }
}

/// Default for `BinarySearchTree` creates an empty tree.
impl<T: Ord + Clone> Default for BinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Display implementation that prints the tree's in-order traversal.
///
/// For visual debugging, we show the sorted sequence of values. A more
/// elaborate pretty-printer could show the tree structure, but the sorted
/// output is most useful for verifying BST correctness.
impl<T: Ord + Clone + fmt::Display> fmt::Display for BinarySearchTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.inorder();
        let strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        write!(f, "BST[{}]", strs.join(", "))
    }
}

// =============================================================================
// Unit Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut bst = BinarySearchTree::new();
        bst.insert(10);
        bst.insert(5);
        bst.insert(15);
        bst.insert(3);
        bst.insert(7);

        assert!(bst.contains(&10));
        assert!(bst.contains(&5));
        assert!(bst.contains(&15));
        assert!(bst.contains(&3));
        assert!(bst.contains(&7));
        assert!(!bst.contains(&1));
        assert!(!bst.contains(&20));
        assert_eq!(bst.len(), 5);
    }

    #[test]
    fn test_duplicate_insert() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(5); // Duplicate — should be ignored.
        assert_eq!(bst.len(), 1);
    }

    #[test]
    fn test_inorder_sorted() {
        let mut bst = BinarySearchTree::new();
        // Insert in arbitrary order; in-order should yield sorted sequence.
        for &val in &[8, 3, 10, 1, 6, 14, 4, 7, 13] {
            bst.insert(val);
        }
        let sorted: Vec<&i32> = bst.inorder();
        let expected: Vec<i32> = vec![1, 3, 4, 6, 7, 8, 10, 13, 14];
        assert_eq!(sorted, expected.iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_delete_leaf() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.delete(&3); // Delete a leaf.
        assert!(!bst.contains(&3));
        assert!(bst.contains(&5));
        assert!(bst.contains(&7));
        assert_eq!(bst.len(), 2);
    }

    #[test]
    fn test_delete_node_with_one_child() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.insert(6);
        bst.delete(&7); // Delete node with one left child (6).
        assert!(!bst.contains(&7));
        assert!(bst.contains(&6));
        assert_eq!(bst.len(), 3);
    }

    #[test]
    fn test_delete_node_with_two_children() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.insert(6);
        bst.insert(8);
        bst.delete(&7); // Delete node with two children (6 and 8).
        assert!(!bst.contains(&7));
        assert!(bst.contains(&6));
        assert!(bst.contains(&8));
        // In-order should still be sorted.
        let sorted: Vec<&i32> = bst.inorder();
        assert_eq!(sorted, vec![&3, &5, &6, &8]);
    }

    #[test]
    fn test_delete_root() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.delete(&5); // Delete root.
        assert!(!bst.contains(&5));
        assert!(bst.contains(&3));
        assert!(bst.contains(&7));
        assert_eq!(bst.len(), 2);
    }

    #[test]
    fn test_height() {
        let mut bst = BinarySearchTree::new();
        assert_eq!(bst.height(), 0);
        bst.insert(5);
        assert_eq!(bst.height(), 1);
        bst.insert(3);
        bst.insert(7);
        assert_eq!(bst.height(), 2);
        bst.insert(1);
        assert_eq!(bst.height(), 3);
    }

    #[test]
    fn test_display() {
        let mut bst = BinarySearchTree::new();
        bst.insert(3);
        bst.insert(1);
        bst.insert(5);
        assert_eq!(format!("{}", bst), "BST[1, 3, 5]");
    }

    #[test]
    fn test_empty_tree() {
        let bst: BinarySearchTree<i32> = BinarySearchTree::new();
        assert!(bst.is_empty());
        assert_eq!(bst.len(), 0);
        assert!(!bst.contains(&1));
        assert_eq!(bst.height(), 0);
        let empty: Vec<&i32> = vec![];
        assert_eq!(bst.inorder(), empty);
    }
}
