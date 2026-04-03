// =============================================================================
// Breadth-First Search (BFS) on Binary Trees
// =============================================================================
//
// # What is BFS?
//
// Breadth-First Search explores a tree (or graph) level by level, visiting
// all nodes at depth d before visiting nodes at depth d+1. It uses a **queue**
// (FIFO) to maintain the frontier of nodes to visit next.
//
// # BFS vs DFS — Trade-offs
//
// | Property          | BFS              | DFS              |
// |-------------------|------------------|------------------|
// | Data structure    | Queue (FIFO)     | Stack (LIFO)     |
// | Visit order       | Level by level   | Branch by branch |
// | Space complexity  | O(w) (max width) | O(h) (height)    |
// | Finds shortest?   | Yes (unweighted) | No               |
// | Complete?         | Yes              | Yes (finite)     |
//
// Where w = max width of the tree and h = height.
//
// For a **balanced** binary tree: w ≈ n/2, h ≈ log n.
// So BFS uses more memory on wide trees, DFS uses more on deep trees.
//
// # Blockchain Analogy
//
// BFS is analogous to **level-order verification** in Merkle trees:
// when verifying a proof, you process hashes level by level from leaves
// to root. BFS also models **block propagation** in P2P networks —
// information spreads to all neighbors before going further.
//
// # Time Complexity: O(n) — we visit every node exactly once.
// # Space Complexity: O(w) where w is the maximum width of the tree.

use crate::dsa::trees::binary_tree::{BinarySearchTree, Node};
use std::collections::VecDeque;

/// Performs BFS (level-order traversal) on a binary tree.
///
/// Returns a vector of values in level-order.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::binary_tree::BinarySearchTree;
/// use algo::dsa::algorithms::bfs::level_order_traversal;
///
/// let mut bst = BinarySearchTree::new();
/// for v in [4, 2, 6, 1, 3, 5, 7] {
///     bst.insert(v);
/// }
/// let levels = level_order_traversal(&bst);
/// assert_eq!(levels, vec![&4, &2, &6, &1, &3, &5, &7]);
/// ```
pub fn level_order_traversal<T: Ord>(tree: &BinarySearchTree<T>) -> Vec<&T> {
    let mut result = Vec::new();
    let root = match &tree.root {
        None => return result,
        Some(r) => r,
    };

    // The queue holds references to nodes. We start with the root.
    let mut queue: VecDeque<&Node<T>> = VecDeque::new();
    queue.push_back(root);

    while let Some(node) = queue.pop_front() {
        result.push(&node.value);

        // Enqueue left child first (so left is visited before right at each level).
        if let Some(ref left) = node.left {
            queue.push_back(left);
        }
        if let Some(ref right) = node.right {
            queue.push_back(right);
        }
    }
    result
}

/// BFS that returns values grouped by level.
///
/// This is useful when you need to know which level each node is on,
/// e.g., for printing a tree or computing per-level aggregates.
pub fn level_order_by_level<T: Ord>(tree: &BinarySearchTree<T>) -> Vec<Vec<&T>> {
    let mut levels: Vec<Vec<&T>> = Vec::new();
    let root = match &tree.root {
        None => return levels,
        Some(r) => r,
    };

    let mut queue: VecDeque<&Node<T>> = VecDeque::new();
    queue.push_back(root);

    while !queue.is_empty() {
        let level_size = queue.len();
        let mut current_level = Vec::new();

        // Process all nodes at the current level.
        for _ in 0..level_size {
            let node = queue.pop_front().unwrap();
            current_level.push(&node.value);

            if let Some(ref left) = node.left {
                queue.push_back(left);
            }
            if let Some(ref right) = node.right {
                queue.push_back(right);
            }
        }
        levels.push(current_level);
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_tree() -> BinarySearchTree<i32> {
        //       4
        //      / \
        //     2   6
        //    / \ / \
        //   1  3 5  7
        let mut bst = BinarySearchTree::new();
        for v in [4, 2, 6, 1, 3, 5, 7] {
            bst.insert(v);
        }
        bst
    }

    #[test]
    fn test_level_order() {
        let bst = build_test_tree();
        let result = level_order_traversal(&bst);
        assert_eq!(result, vec![&4, &2, &6, &1, &3, &5, &7]);
    }

    #[test]
    fn test_level_order_by_level() {
        let bst = build_test_tree();
        let levels = level_order_by_level(&bst);
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0], vec![&4]);
        assert_eq!(levels[1], vec![&2, &6]);
        assert_eq!(levels[2], vec![&1, &3, &5, &7]);
    }

    #[test]
    fn test_empty_tree() {
        let bst: BinarySearchTree<i32> = BinarySearchTree::new();
        assert!(level_order_traversal(&bst).is_empty());
        assert!(level_order_by_level(&bst).is_empty());
    }

    #[test]
    fn test_single_node() {
        let mut bst = BinarySearchTree::new();
        bst.insert(42);
        assert_eq!(level_order_traversal(&bst), vec![&42]);
        assert_eq!(level_order_by_level(&bst), vec![vec![&42]]);
    }

    #[test]
    fn test_left_skewed() {
        // Degenerate tree: 5 → 4 → 3 → 2 → 1
        let mut bst = BinarySearchTree::new();
        for v in (1..=5).rev() {
            bst.insert(v);
        }
        let result = level_order_traversal(&bst);
        assert_eq!(result, vec![&5, &4, &3, &2, &1]);
    }
}
