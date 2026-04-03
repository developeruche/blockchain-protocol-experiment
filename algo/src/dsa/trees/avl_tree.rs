// =============================================================================
// AVL Tree — Self-Balancing Binary Search Tree
// =============================================================================
//
// # What is an AVL Tree?
//
// Named after Adelson-Velsky and Landis (1962), an AVL tree is a self-balancing
// BST where the **balance factor** of every node is in {-1, 0, 1}.
//
//   balance_factor(node) = height(left_subtree) - height(right_subtree)
//
// When an insertion or deletion violates this invariant, we restore balance
// through **rotations**: small, local restructurings that preserve the BST
// ordering invariant while fixing the height imbalance.
//
// # Why AVL Trees?
//
// - **Guaranteed O(log n)** for search, insert, and delete (unlike plain BSTs).
// - **Stricter balance** than Red-Black trees, so lookups are faster (shorter
//   average path), but insertions/deletions require more rotations.
// - Used in blockchain when you need a verifiable sorted structure with
//   worst-case logarithmic access.
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
// # Rotation Types
//
// There are four imbalance cases, each fixed by one or two rotations:
//
// 1. **Left-Left (LL)** → Right rotation
// 2. **Right-Right (RR)** → Left rotation
// 3. **Left-Right (LR)** → Left rotation on left child, then right rotation
// 4. **Right-Left (RL)** → Right rotation on right child, then left rotation

use std::cmp::Ordering;
use std::fmt;

/// A node in the AVL tree.
///
/// We store the height in each node to avoid recomputing it during rebalancing.
/// This is a classic **space-time trade-off**: O(n) extra space for O(1) balance
/// factor computation.
#[derive(Debug)]
pub struct AvlNode<T: Ord> {
    pub value: T,
    pub left: Option<Box<AvlNode<T>>>,
    pub right: Option<Box<AvlNode<T>>>,
    /// Cached height of this subtree. A leaf has height 1.
    height: i32,
}

impl<T: Ord> AvlNode<T> {
    fn new(value: T) -> Self {
        AvlNode {
            value,
            left: None,
            right: None,
            height: 1, // A new node is a leaf, height = 1.
        }
    }
}

/// Returns the height of a node, handling the `None` case (empty tree has height 0).
fn height<T: Ord>(node: &Option<Box<AvlNode<T>>>) -> i32 {
    node.as_ref().map_or(0, |n| n.height)
}

/// Recomputes and caches the height of a node from its children's heights.
fn update_height<T: Ord>(node: &mut Box<AvlNode<T>>) {
    node.height = 1 + height(&node.left).max(height(&node.right));
}

/// Computes the balance factor: height(left) - height(right).
///
/// - Positive → left-heavy
/// - Negative → right-heavy
/// - |balance_factor| > 1 → imbalanced, needs rotation
fn balance_factor<T: Ord>(node: &Box<AvlNode<T>>) -> i32 {
    height(&node.left) - height(&node.right)
}

// =============================================================================
// Rotations — The Core of AVL Rebalancing
// =============================================================================
//
// Rotations are the mechanism by which we restore AVL balance. Each rotation
// is a local operation that:
// 1. Preserves the BST ordering invariant.
// 2. Reduces the height difference between subtrees.
//
// A rotation takes O(1) time — it's just pointer reassignment.

/// Right rotation (for Left-Left imbalance).
///
/// ```text
///       y                x
///      / \             /   \
///     x   C    →      A     y
///    / \                   / \
///   A   B                 B   C
/// ```
///
/// After rotation: `x` becomes the new root, `y` becomes `x`'s right child,
/// and `B` (which was between `x` and `y` in value) becomes `y`'s left child.
fn rotate_right<T: Ord>(mut y: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
    // `x` is y's left child — it will become the new root.
    let mut x = y.left.take().expect("rotate_right requires a left child");
    // `B` moves from x's right to y's left.
    y.left = x.right.take();
    // Update heights bottom-up: y first (it's now lower), then x.
    update_height(&mut y);
    x.right = Some(y);
    update_height(&mut x);
    x
}

/// Left rotation (for Right-Right imbalance).
///
/// ```text
///     x                  y
///    / \               /   \
///   A   y      →      x     C
///      / \           / \
///     B   C         A   B
/// ```
fn rotate_left<T: Ord>(mut x: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
    let mut y = x.right.take().expect("rotate_left requires a right child");
    x.right = y.left.take();
    update_height(&mut x);
    y.left = Some(x);
    update_height(&mut y);
    y
}

/// Rebalances a node if its balance factor is outside {-1, 0, 1}.
///
/// This function detects which of the four imbalance cases applies and
/// performs the appropriate rotation(s).
fn rebalance<T: Ord>(mut node: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
    update_height(&mut node);
    let bf = balance_factor(&node);

    if bf > 1 {
        // Left-heavy.
        if balance_factor(node.left.as_ref().unwrap()) < 0 {
            // Left-Right case: left child is right-heavy.
            // First rotate the left child left, converting to Left-Left case.
            node.left = Some(rotate_left(node.left.take().unwrap()));
        }
        // Left-Left case: single right rotation fixes it.
        rotate_right(node)
    } else if bf < -1 {
        // Right-heavy.
        if balance_factor(node.right.as_ref().unwrap()) > 0 {
            // Right-Left case: right child is left-heavy.
            node.right = Some(rotate_right(node.right.take().unwrap()));
        }
        // Right-Right case: single left rotation fixes it.
        rotate_left(node)
    } else {
        // Already balanced — no rotation needed.
        node
    }
}

/// An AVL Tree providing guaranteed O(log n) operations.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::avl_tree::AvlTree;
///
/// let mut tree = AvlTree::new();
/// tree.insert(3);
/// tree.insert(1);
/// tree.insert(2); // Triggers Left-Right rotation!
/// assert!(tree.contains(&2));
/// ```
#[derive(Debug)]
pub struct AvlTree<T: Ord> {
    root: Option<Box<AvlNode<T>>>,
    size: usize,
}

impl<T: Ord> AvlTree<T> {
    /// Creates a new, empty AVL tree.
    pub fn new() -> Self {
        AvlTree {
            root: None,
            size: 0,
        }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns `true` if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Inserts a value, rebalancing as needed.
    pub fn insert(&mut self, value: T) {
        let (new_root, inserted) = Self::insert_recursive(self.root.take(), value);
        self.root = Some(new_root);
        if inserted {
            self.size += 1;
        }
    }

    /// Recursive insert that returns the (possibly rotated) new subtree root
    /// and whether a new node was actually inserted.
    fn insert_recursive(
        node: Option<Box<AvlNode<T>>>,
        value: T,
    ) -> (Box<AvlNode<T>>, bool) {
        match node {
            None => (Box::new(AvlNode::new(value)), true),
            Some(mut n) => {
                let inserted;
                match value.cmp(&n.value) {
                    Ordering::Less => {
                        let (new_left, ins) = Self::insert_recursive(n.left.take(), value);
                        n.left = Some(new_left);
                        inserted = ins;
                    }
                    Ordering::Greater => {
                        let (new_right, ins) = Self::insert_recursive(n.right.take(), value);
                        n.right = Some(new_right);
                        inserted = ins;
                    }
                    Ordering::Equal => return (n, false), // Duplicate — skip rebalance.
                }
                // Rebalance on the way back up the recursion stack.
                // This is what makes AVL trees self-balancing: every ancestor
                // of the inserted node is checked and potentially rotated.
                (rebalance(n), inserted)
            }
        }
    }

    /// Searches for a value. O(log n) guaranteed.
    pub fn contains(&self, value: &T) -> bool {
        Self::search_recursive(&self.root, value)
    }

    fn search_recursive(node: &Option<Box<AvlNode<T>>>, value: &T) -> bool {
        match node {
            None => false,
            Some(n) => match value.cmp(&n.value) {
                Ordering::Less => Self::search_recursive(&n.left, value),
                Ordering::Greater => Self::search_recursive(&n.right, value),
                Ordering::Equal => true,
            },
        }
    }

    /// Deletes a value, rebalancing as needed.
    pub fn delete(&mut self, value: &T)
    where
        T: Clone,
    {
        let (new_root, deleted) = Self::delete_recursive(self.root.take(), value);
        self.root = new_root;
        if deleted {
            self.size -= 1;
        }
    }

    fn delete_recursive(
        node: Option<Box<AvlNode<T>>>,
        value: &T,
    ) -> (Option<Box<AvlNode<T>>>, bool)
    where
        T: Clone,
    {
        match node {
            None => (None, false),
            Some(mut n) => {
                let deleted;
                match value.cmp(&n.value) {
                    Ordering::Less => {
                        let (new_left, del) = Self::delete_recursive(n.left.take(), value);
                        n.left = new_left;
                        deleted = del;
                    }
                    Ordering::Greater => {
                        let (new_right, del) = Self::delete_recursive(n.right.take(), value);
                        n.right = new_right;
                        deleted = del;
                    }
                    Ordering::Equal => {
                        deleted = true;
                        match (n.left.take(), n.right.take()) {
                            (None, None) => return (None, true),
                            (Some(left), None) => return (Some(rebalance(left)), true),
                            (None, Some(right)) => return (Some(rebalance(right)), true),
                            (left, right) => {
                                // Two children: replace with in-order successor.
                                n.left = left;
                                n.right = right;
                                let min_val = Self::find_min(n.right.as_ref().unwrap());
                                n.value = min_val;
                                let (new_right, _) =
                                    Self::delete_recursive(n.right.take(), &n.value);
                                n.right = new_right;
                            }
                        }
                    }
                }
                (Some(rebalance(n)), deleted)
            }
        }
    }

    fn find_min(node: &Box<AvlNode<T>>) -> T
    where
        T: Clone,
    {
        match &node.left {
            None => node.value.clone(),
            Some(left) => Self::find_min(left),
        }
    }

    /// Returns the height of the tree. O(1) because we cache heights.
    pub fn height(&self) -> i32 {
        height(&self.root)
    }

    /// In-order traversal yielding sorted values.
    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    fn inorder_recursive<'a>(node: &'a Option<Box<AvlNode<T>>>, result: &mut Vec<&'a T>) {
        if let Some(n) = node {
            Self::inorder_recursive(&n.left, result);
            result.push(&n.value);
            Self::inorder_recursive(&n.right, result);
        }
    }

    /// Verifies the AVL invariant: |balance_factor| ≤ 1 for every node.
    /// Returns `true` if the tree is valid. Used in tests.
    pub fn is_balanced(&self) -> bool {
        Self::check_balanced(&self.root)
    }

    fn check_balanced(node: &Option<Box<AvlNode<T>>>) -> bool {
        match node {
            None => true,
            Some(n) => {
                let bf = height(&n.left) - height(&n.right);
                bf.abs() <= 1
                    && Self::check_balanced(&n.left)
                    && Self::check_balanced(&n.right)
            }
        }
    }
}

impl<T: Ord> Default for AvlTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + fmt::Display> fmt::Display for AvlTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.inorder();
        let strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        write!(f, "AVL[{}]", strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_search() {
        let mut tree = AvlTree::new();
        for v in [10, 20, 30, 40, 50, 25] {
            tree.insert(v);
        }
        for v in [10, 20, 30, 40, 50, 25] {
            assert!(tree.contains(&v));
        }
        assert!(!tree.contains(&99));
        assert_eq!(tree.len(), 6);
    }

    #[test]
    fn test_balance_after_right_right() {
        // Inserting sorted values would degenerate a plain BST.
        // AVL should remain balanced via left rotations.
        let mut tree = AvlTree::new();
        for v in 1..=10 {
            tree.insert(v);
        }
        assert!(tree.is_balanced());
        assert!(tree.height() <= 4); // log2(10) ≈ 3.3, AVL allows up to ~1.44*log2(n)
    }

    #[test]
    fn test_balance_after_left_right() {
        // Triggers LR rotation: insert 30, then 10, then 20.
        let mut tree = AvlTree::new();
        tree.insert(30);
        tree.insert(10);
        tree.insert(20); // LR case: left child (10) is right-heavy.
        assert!(tree.is_balanced());
        assert_eq!(tree.inorder(), vec![&10, &20, &30]);
    }

    #[test]
    fn test_delete_and_rebalance() {
        let mut tree = AvlTree::new();
        for v in [10, 5, 15, 3, 7, 12, 20, 1] {
            tree.insert(v);
        }
        tree.delete(&20);
        tree.delete(&15);
        assert!(tree.is_balanced());
        assert!(!tree.contains(&20));
        assert!(!tree.contains(&15));
    }

    #[test]
    fn test_inorder_sorted() {
        let mut tree = AvlTree::new();
        for v in [50, 30, 70, 20, 40, 60, 80] {
            tree.insert(v);
        }
        assert_eq!(tree.inorder(), vec![&20, &30, &40, &50, &60, &70, &80]);
    }

    #[test]
    fn test_duplicate_insert() {
        let mut tree = AvlTree::new();
        tree.insert(5);
        tree.insert(5);
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_display() {
        let mut tree = AvlTree::new();
        tree.insert(2);
        tree.insert(1);
        tree.insert(3);
        assert_eq!(format!("{}", tree), "AVL[1, 2, 3]");
    }
}
