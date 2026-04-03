// =============================================================================
// 2-3 Tree — Perfectly Balanced Multi-Way Search Tree
// =============================================================================
//
// # What is a 2-3 Tree?
//
// A 2-3 tree is a balanced search tree where:
// - **2-nodes** have 1 key and 2 children.
// - **3-nodes** have 2 keys and 3 children.
// - All leaves are at the same depth (perfectly balanced).
//
// # Why 2-3 Trees?
//
// - **Conceptual foundation**: 2-3 trees are the conceptual model behind
//   Red-Black trees. A Red-Black tree is a 2-3 tree encoded as a binary tree
//   (red links represent 3-nodes). Understanding 2-3 trees makes Red-Black
//   trees intuitive.
// - **Guaranteed O(log n)**: Like B-Trees (of which 2-3 trees are a special
//   case with order 3), all operations are O(log n).
// - **Perfect balance**: All leaves at the same depth — no degenerate cases.
//
// # Complexity Table
//
// | Operation | Time     |
// |-----------|----------|
// | Search    | O(log n) |
// | Insert    | O(log n) |
// | Delete    | O(log n) |
// | Space     | O(n)     |
//
// # Invariants
//
// 1. Each node is either a 2-node or a 3-node.
// 2. All leaves are at the same depth.
// 3. Keys within each node are sorted.
// 4. For a 2-node [a]: left < a < right
//    For a 3-node [a, b]: left < a < middle < b < right
//
// # Implementation Approach
//
// We use an enum to represent 2-nodes and 3-nodes explicitly. Insertion
// may temporarily create a "4-node" (3 keys) that gets split and pushed up.

use std::fmt;

/// A node in the 2-3 tree. We enumerate the possible node types.
///
/// Using an enum makes the different node types explicit in the type system,
/// which is more Rust-idiomatic than using variable-length vectors.
#[derive(Debug, Clone)]
pub enum Node<T: Ord + Clone> {
    /// A leaf holds one or two values (no children).
    Leaf2(T),
    Leaf3(T, T),
    /// Internal 2-node: one key, two children.
    Internal2 {
        key: T,
        left: Box<Node<T>>,
        right: Box<Node<T>>,
    },
    /// Internal 3-node: two keys, three children.
    Internal3 {
        key1: T,
        key2: T,
        left: Box<Node<T>>,
        mid: Box<Node<T>>,
        right: Box<Node<T>>,
    },
}

/// Result of inserting into a subtree. If the node splits, we propagate
/// the middle key and two halves upward.
enum InsertResult<T: Ord + Clone> {
    /// Insertion absorbed — no split needed.
    Absorbed(Node<T>),
    /// Node split. The parent must incorporate the promoted key and two children.
    Split {
        left: Node<T>,
        key: T,
        right: Node<T>,
    },
}

impl<T: Ord + Clone + fmt::Display> Node<T> {
    /// Searches for a value in this subtree.
    fn contains(&self, value: &T) -> bool {
        match self {
            Node::Leaf2(a) => *value == *a,
            Node::Leaf3(a, b) => *value == *a || *value == *b,
            Node::Internal2 { key, left, right } => {
                if *value == *key {
                    true
                } else if *value < *key {
                    left.contains(value)
                } else {
                    right.contains(value)
                }
            }
            Node::Internal3 {
                key1,
                key2,
                left,
                mid,
                right,
            } => {
                if *value == *key1 || *value == *key2 {
                    true
                } else if *value < *key1 {
                    left.contains(value)
                } else if *value < *key2 {
                    mid.contains(value)
                } else {
                    right.contains(value)
                }
            }
        }
    }

    /// Inserts a value into this subtree.
    fn insert(self, value: T) -> InsertResult<T> {
        match self {
            // Leaf with one key — absorb into a 2-key leaf (becomes 3-node).
            Node::Leaf2(a) => {
                if value == a {
                    InsertResult::Absorbed(Node::Leaf2(a))
                } else if value < a {
                    InsertResult::Absorbed(Node::Leaf3(value, a))
                } else {
                    InsertResult::Absorbed(Node::Leaf3(a, value))
                }
            }
            // Leaf with two keys — "overflow": must split.
            Node::Leaf3(a, b) => {
                if value == a || value == b {
                    InsertResult::Absorbed(Node::Leaf3(a, b))
                } else {
                    let mut sorted = [a, b, value];
                    sorted.sort();
                    let [lo, mid, hi] = sorted;
                    InsertResult::Split {
                        left: Node::Leaf2(lo),
                        key: mid,
                        right: Node::Leaf2(hi),
                    }
                }
            }
            // Internal 2-node.
            Node::Internal2 { key, left, right } => {
                if value == key {
                    return InsertResult::Absorbed(Node::Internal2 { key, left, right });
                }
                if value < key {
                    match left.insert(value) {
                        InsertResult::Absorbed(new_left) => {
                            InsertResult::Absorbed(Node::Internal2 {
                                key,
                                left: Box::new(new_left),
                                right,
                            })
                        }
                        InsertResult::Split {
                            left: sl,
                            key: sk,
                            right: sr,
                        } => {
                            // Promote split key — 2-node becomes 3-node (no further split).
                            InsertResult::Absorbed(Node::Internal3 {
                                key1: sk,
                                key2: key,
                                left: Box::new(sl),
                                mid: Box::new(sr),
                                right,
                            })
                        }
                    }
                } else {
                    match right.insert(value) {
                        InsertResult::Absorbed(new_right) => {
                            InsertResult::Absorbed(Node::Internal2 {
                                key,
                                left,
                                right: Box::new(new_right),
                            })
                        }
                        InsertResult::Split {
                            left: sl,
                            key: sk,
                            right: sr,
                        } => {
                            InsertResult::Absorbed(Node::Internal3 {
                                key1: key,
                                key2: sk,
                                left,
                                mid: Box::new(sl),
                                right: Box::new(sr),
                            })
                        }
                    }
                }
            }
            // Internal 3-node.
            Node::Internal3 {
                key1,
                key2,
                left,
                mid,
                right,
            } => {
                if value == key1 || value == key2 {
                    return InsertResult::Absorbed(Node::Internal3 {
                        key1,
                        key2,
                        left,
                        mid,
                        right,
                    });
                }
                if value < key1 {
                    match left.insert(value) {
                        InsertResult::Absorbed(new_left) => {
                            InsertResult::Absorbed(Node::Internal3 {
                                key1,
                                key2,
                                left: Box::new(new_left),
                                mid,
                                right,
                            })
                        }
                        InsertResult::Split {
                            left: sl,
                            key: sk,
                            right: sr,
                        } => {
                            // 3-node must split further.
                            InsertResult::Split {
                                left: Node::Internal2 {
                                    key: sk,
                                    left: Box::new(sl),
                                    right: Box::new(sr),
                                },
                                key: key1,
                                right: Node::Internal2 {
                                    key: key2,
                                    left: mid,
                                    right,
                                },
                            }
                        }
                    }
                } else if value < key2 {
                    match mid.insert(value) {
                        InsertResult::Absorbed(new_mid) => {
                            InsertResult::Absorbed(Node::Internal3 {
                                key1,
                                key2,
                                left,
                                mid: Box::new(new_mid),
                                right,
                            })
                        }
                        InsertResult::Split {
                            left: sl,
                            key: sk,
                            right: sr,
                        } => {
                            InsertResult::Split {
                                left: Node::Internal2 {
                                    key: key1,
                                    left,
                                    right: Box::new(sl),
                                },
                                key: sk,
                                right: Node::Internal2 {
                                    key: key2,
                                    left: Box::new(sr),
                                    right,
                                },
                            }
                        }
                    }
                } else {
                    match right.insert(value) {
                        InsertResult::Absorbed(new_right) => {
                            InsertResult::Absorbed(Node::Internal3 {
                                key1,
                                key2,
                                left,
                                mid,
                                right: Box::new(new_right),
                            })
                        }
                        InsertResult::Split {
                            left: sl,
                            key: sk,
                            right: sr,
                        } => {
                            InsertResult::Split {
                                left: Node::Internal2 {
                                    key: key1,
                                    left,
                                    right: mid,
                                },
                                key: key2,
                                right: Node::Internal2 {
                                    key: sk,
                                    left: Box::new(sl),
                                    right: Box::new(sr),
                                },
                            }
                        }
                    }
                }
            }
        }
    }

    /// Collects all values in sorted order.
    fn inorder<'a>(&'a self, result: &mut Vec<&'a T>) {
        match self {
            Node::Leaf2(a) => result.push(a),
            Node::Leaf3(a, b) => {
                result.push(a);
                result.push(b);
            }
            Node::Internal2 { key, left, right } => {
                left.inorder(result);
                result.push(key);
                right.inorder(result);
            }
            Node::Internal3 {
                key1,
                key2,
                left,
                mid,
                right,
            } => {
                left.inorder(result);
                result.push(key1);
                mid.inorder(result);
                result.push(key2);
                right.inorder(result);
            }
        }
    }
}

/// A 2-3 Tree.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::two_three_tree::TwoThreeTree;
///
/// let mut tree = TwoThreeTree::new();
/// tree.insert(10);
/// tree.insert(20);
/// tree.insert(5);
/// assert!(tree.contains(&10));
/// ```
#[derive(Debug)]
pub struct TwoThreeTree<T: Ord + Clone + fmt::Display> {
    root: Option<Node<T>>,
    size: usize,
}

impl<T: Ord + Clone + fmt::Display> TwoThreeTree<T> {
    pub fn new() -> Self {
        TwoThreeTree {
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

    pub fn contains(&self, value: &T) -> bool {
        match &self.root {
            None => false,
            Some(root) => root.contains(value),
        }
    }

    pub fn insert(&mut self, value: T) {
        if self.contains(&value) {
            return;
        }
        match self.root.take() {
            None => {
                self.root = Some(Node::Leaf2(value));
                self.size += 1;
            }
            Some(root) => {
                match root.insert(value) {
                    InsertResult::Absorbed(new_root) => {
                        self.root = Some(new_root);
                    }
                    InsertResult::Split { left, key, right } => {
                        // The root split — create a new root (tree grows in height).
                        self.root = Some(Node::Internal2 {
                            key,
                            left: Box::new(left),
                            right: Box::new(right),
                        });
                    }
                }
                self.size += 1;
            }
        }
    }

    /// Deletes a value.
    ///
    /// 2-3 tree deletion is complex due to the need to merge nodes and
    /// redistribute keys to maintain the invariant that all leaves are at the
    /// same depth. We implement a simplified version that rebuilds from the
    /// remaining elements for correctness.
    pub fn delete(&mut self, value: &T) {
        if !self.contains(value) {
            return;
        }
        // Collect all values except the one to delete, then rebuild.
        // This is O(n) but correct. A production implementation would use
        // proper bottom-up merging and redistribution.
        let values: Vec<T> = self
            .inorder()
            .into_iter()
            .filter(|v| *v != value)
            .cloned()
            .collect();
        self.root = None;
        self.size = 0;
        for v in values {
            self.insert(v);
        }
    }

    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        if let Some(ref root) = self.root {
            root.inorder(&mut result);
        }
        result
    }
}

impl<T: Ord + Clone + fmt::Display> Default for TwoThreeTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone + fmt::Display> fmt::Display for TwoThreeTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.inorder();
        let strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        write!(f, "2-3Tree[{}]", strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_search() {
        let mut tree = TwoThreeTree::new();
        for v in [10, 20, 30, 40, 50, 25, 5, 15] {
            tree.insert(v);
        }
        for v in [10, 20, 30, 40, 50, 25, 5, 15] {
            assert!(tree.contains(&v));
        }
        assert!(!tree.contains(&99));
    }

    #[test]
    fn test_inorder_sorted() {
        let mut tree = TwoThreeTree::new();
        for v in [50, 30, 70, 20, 40, 60, 80] {
            tree.insert(v);
        }
        assert_eq!(tree.inorder(), vec![&20, &30, &40, &50, &60, &70, &80]);
    }

    #[test]
    fn test_delete() {
        let mut tree = TwoThreeTree::new();
        for v in [10, 5, 15, 3, 7, 12, 20] {
            tree.insert(v);
        }
        tree.delete(&10);
        assert!(!tree.contains(&10));
        assert!(tree.contains(&5));
        assert!(tree.contains(&15));
    }

    #[test]
    fn test_large_insert() {
        let mut tree = TwoThreeTree::new();
        for v in 1..=50 {
            tree.insert(v);
        }
        assert_eq!(tree.len(), 50);
        for v in 1..=50 {
            assert!(tree.contains(&v));
        }
    }

    #[test]
    fn test_display() {
        let mut tree = TwoThreeTree::new();
        tree.insert(2);
        tree.insert(1);
        tree.insert(3);
        assert_eq!(format!("{}", tree), "2-3Tree[1, 2, 3]");
    }
}
