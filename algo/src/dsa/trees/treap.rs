// =============================================================================
// Treap — Randomized BST (Tree + Heap)
// =============================================================================
//
// # What is a Treap?
//
// A treap (tree + heap) combines two invariants:
// 1. **BST invariant on keys**: Left subtree keys < node key < right subtree keys.
// 2. **Heap invariant on priorities**: Each node's priority ≥ its children's priorities
//    (max-heap on priority).
//
// Priorities are assigned randomly, which gives the treap the same expected
// shape as a randomly-built BST — i.e., expected O(log n) height.
//
// # Why Treaps?
//
// - **Simpler than AVL/RB trees**: Rotations are determined by heap priority,
//   not balance factors or colors.
// - **Randomized guarantees**: Expected O(log n) for all operations, which is
//   sufficient for most practical purposes.
// - **Easy to implement split/merge**: Treaps support efficient split and
//   merge operations, making them ideal for order-statistic operations.
// - **Blockchain**: Treaps can be used as a randomized index structure for
//   transaction ordering or mempool management.
//
// # Complexity Table
//
// | Operation | Expected | Worst    |
// |-----------|----------|----------|
// | Search    | O(log n) | O(n)     |
// | Insert    | O(log n) | O(n)     |
// | Delete    | O(log n) | O(n)     |
// | Space     | O(n)     | O(n)     |
//
// Worst case is extremely unlikely with good random priorities.
//
// # Implementation Note
//
// We use a simple LCG (linear congruential generator) to avoid depending on
// external crates. For production code, you'd use `rand::thread_rng()`.

use std::fmt;

/// Simple pseudo-random number generator (LCG).
/// Not cryptographically secure, but sufficient for treap priorities.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        SimpleRng { state: seed.wrapping_add(1) }
    }

    fn next(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes.
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }
}

/// A node in the treap.
#[derive(Debug)]
pub struct TreapNode<T: Ord> {
    pub value: T,
    /// Random priority — higher values bubble up (max-heap).
    pub priority: u64,
    pub left: Option<Box<TreapNode<T>>>,
    pub right: Option<Box<TreapNode<T>>>,
}

impl<T: Ord> TreapNode<T> {
    fn new(value: T, priority: u64) -> Self {
        TreapNode {
            value,
            priority,
            left: None,
            right: None,
        }
    }
}

// =============================================================================
// Rotations
// =============================================================================
// Same rotations as in BST/AVL, but triggered by heap priority violations
// rather than balance factors.

fn rotate_right<T: Ord>(mut y: Box<TreapNode<T>>) -> Box<TreapNode<T>> {
    let mut x = y.left.take().unwrap();
    y.left = x.right.take();
    x.right = Some(y);
    x
}

fn rotate_left<T: Ord>(mut x: Box<TreapNode<T>>) -> Box<TreapNode<T>> {
    let mut y = x.right.take().unwrap();
    x.right = y.left.take();
    y.left = Some(x);
    y
}

/// A Treap (Tree + Heap) providing expected O(log n) operations.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::treap::Treap;
///
/// let mut treap = Treap::new();
/// treap.insert(5);
/// treap.insert(3);
/// treap.insert(7);
/// assert!(treap.contains(&5));
/// ```
pub struct Treap<T: Ord> {
    root: Option<Box<TreapNode<T>>>,
    rng: SimpleRng,
    size: usize,
}

impl<T: Ord + fmt::Debug> fmt::Debug for Treap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Treap")
            .field("root", &self.root)
            .field("size", &self.size)
            .finish()
    }
}

impl<T: Ord> Treap<T> {
    /// Creates a new, empty treap.
    pub fn new() -> Self {
        Treap {
            root: None,
            rng: SimpleRng::new(42), // Deterministic seed for reproducibility in tests.
            size: 0,
        }
    }

    /// Creates a treap with a specific random seed (for testing).
    pub fn with_seed(seed: u64) -> Self {
        Treap {
            root: None,
            rng: SimpleRng::new(seed),
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Inserts a value with a random priority.
    ///
    /// # Algorithm
    /// 1. Insert like a normal BST (by key).
    /// 2. Assign a random priority.
    /// 3. Rotate upward until the heap property is satisfied.
    pub fn insert(&mut self, value: T) {
        let priority = self.rng.next();
        let root = self.root.take();
        let (new_root, inserted) = self.insert_recursive(root, value, priority);
        self.root = Some(new_root);
        if inserted {
            self.size += 1;
        }
    }

    fn insert_recursive(
        &mut self,
        node: Option<Box<TreapNode<T>>>,
        value: T,
        priority: u64,
    ) -> (Box<TreapNode<T>>, bool) {
        match node {
            None => (Box::new(TreapNode::new(value, priority)), true),
            Some(mut n) => {
                use std::cmp::Ordering;
                match value.cmp(&n.value) {
                    Ordering::Equal => (n, false),
                    Ordering::Less => {
                        let (new_left, inserted) = self.insert_recursive(n.left.take(), value, priority);
                        n.left = Some(new_left);
                        // If the left child has higher priority, rotate right
                        // to restore the heap invariant.
                        if n.left.as_ref().unwrap().priority > n.priority {
                            n = rotate_right(n);
                        }
                        (n, inserted)
                    }
                    Ordering::Greater => {
                        let (new_right, inserted) = self.insert_recursive(n.right.take(), value, priority);
                        n.right = Some(new_right);
                        if n.right.as_ref().unwrap().priority > n.priority {
                            n = rotate_left(n);
                        }
                        (n, inserted)
                    }
                }
            }
        }
    }

    /// Searches for a value. O(log n) expected.
    pub fn contains(&self, value: &T) -> bool {
        Self::search_recursive(&self.root, value)
    }

    fn search_recursive(node: &Option<Box<TreapNode<T>>>, value: &T) -> bool {
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

    /// Deletes a value.
    ///
    /// # Algorithm
    /// Rotate the target node down (by choosing the child with higher priority)
    /// until it becomes a leaf, then remove it.
    pub fn delete(&mut self, value: &T) {
        let (new_root, deleted) = Self::delete_recursive(self.root.take(), value);
        self.root = new_root;
        if deleted {
            self.size -= 1;
        }
    }

    fn delete_recursive(
        node: Option<Box<TreapNode<T>>>,
        value: &T,
    ) -> (Option<Box<TreapNode<T>>>, bool) {
        match node {
            None => (None, false),
            Some(mut n) => {
                use std::cmp::Ordering;
                match value.cmp(&n.value) {
                    Ordering::Less => {
                        let (new_left, deleted) = Self::delete_recursive(n.left.take(), value);
                        n.left = new_left;
                        (Some(n), deleted)
                    }
                    Ordering::Greater => {
                        let (new_right, deleted) = Self::delete_recursive(n.right.take(), value);
                        n.right = new_right;
                        (Some(n), deleted)
                    }
                    Ordering::Equal => {
                        // Found the node — rotate it down until it's a leaf.
                        match (n.left.is_some(), n.right.is_some()) {
                            (false, false) => (None, true),
                            (true, false) => (n.left, true),
                            (false, true) => (n.right, true),
                            (true, true) => {
                                let left_pri = n.left.as_ref().unwrap().priority;
                                let right_pri = n.right.as_ref().unwrap().priority;
                                if left_pri > right_pri {
                                    n = rotate_right(n);
                                    let (new_right, deleted) = Self::delete_recursive(n.right.take(), value);
                                    n.right = new_right;
                                    (Some(n), deleted)
                                } else {
                                    n = rotate_left(n);
                                    let (new_left, deleted) = Self::delete_recursive(n.left.take(), value);
                                    n.left = new_left;
                                    (Some(n), deleted)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// In-order traversal yielding sorted values.
    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    fn inorder_recursive<'a>(node: &'a Option<Box<TreapNode<T>>>, result: &mut Vec<&'a T>) {
        if let Some(n) = node {
            Self::inorder_recursive(&n.left, result);
            result.push(&n.value);
            Self::inorder_recursive(&n.right, result);
        }
    }
}

impl<T: Ord> Default for Treap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + fmt::Display> fmt::Display for Treap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.inorder();
        let strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        write!(f, "Treap[{}]", strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_search() {
        let mut treap = Treap::new();
        for v in [10, 5, 20, 3, 7, 15, 25] {
            treap.insert(v);
        }
        for v in [10, 5, 20, 3, 7, 15, 25] {
            assert!(treap.contains(&v));
        }
        assert!(!treap.contains(&99));
    }

    #[test]
    fn test_inorder_sorted() {
        let mut treap = Treap::new();
        for v in [50, 30, 70, 20, 40, 60, 80] {
            treap.insert(v);
        }
        assert_eq!(treap.inorder(), vec![&20, &30, &40, &50, &60, &70, &80]);
    }

    #[test]
    fn test_delete() {
        let mut treap = Treap::new();
        for v in [10, 5, 15, 3, 7] {
            treap.insert(v);
        }
        treap.delete(&10);
        assert!(!treap.contains(&10));
        assert!(treap.contains(&5));
        assert!(treap.contains(&15));
        assert_eq!(treap.len(), 4);
    }

    #[test]
    fn test_large_insert() {
        let mut treap = Treap::new();
        for v in 0..100 {
            treap.insert(v);
        }
        assert_eq!(treap.len(), 100);
        for v in 0..100 {
            assert!(treap.contains(&v));
        }
    }

    #[test]
    fn test_duplicate() {
        let mut treap = Treap::new();
        treap.insert(5);
        treap.insert(5);
        assert_eq!(treap.len(), 1);
    }

    #[test]
    fn test_display() {
        let mut treap = Treap::new();
        treap.insert(2);
        treap.insert(1);
        treap.insert(3);
        assert_eq!(format!("{}", treap), "Treap[1, 2, 3]");
    }
}
