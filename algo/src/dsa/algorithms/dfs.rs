// =============================================================================
// Depth-First Search (DFS) on Binary Trees
// =============================================================================
//
// # What is DFS?
//
// Depth-First Search explores a tree (or graph) by going as deep as possible
// along each branch before backtracking. It uses a **stack** (explicitly or
// via the call stack for recursion).
//
// # DFS Variants on Binary Trees
//
// DFS on binary trees manifests as three classic traversal orders:
//
// - **Pre-order**  (Node, Left, Right): Visit root first. Useful for copying/
//   serializing the tree structure.
// - **In-order**   (Left, Node, Right): Visit root between subtrees. For BSTs,
//   yields sorted order.
// - **Post-order** (Left, Right, Node): Visit root last. Useful for computing
//   aggregate properties bottom-up (sizes, hashes, freeing memory).
//
// # BFS vs DFS Revisited
//
// | Property          | BFS              | DFS              |
// |-------------------|------------------|------------------|
// | Uses              | Queue (FIFO)     | Stack/recursion  |
// | Memory            | O(width)         | O(height)        |
// | Traversal order   | Level by level   | Branch by branch |
// | Best for          | Shortest paths   | Exhaustive search|
//
// # Blockchain Analogy
//
// DFS post-order is exactly how **Merkle tree hashing** works: you must
// compute the hashes of child nodes before computing the parent's hash.
// DFS pre-order corresponds to state trie traversal for fast proof generation.
//
// # Time Complexity: O(n) — each node visited once.
// # Space Complexity: O(h) where h = height of the tree.
//   - Balanced tree: O(log n)
//   - Degenerate (linked list): O(n)

use crate::dsa::trees::binary_tree::{BinarySearchTree, Node};

// =============================================================================
// Iterative DFS implementations (using explicit stack)
// =============================================================================

/// Iterative pre-order DFS traversal.
///
/// Pre-order visits the current node *before* its children.
/// This produces the sequence: root → left subtree → right subtree.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::binary_tree::BinarySearchTree;
/// use algo::dsa::algorithms::dfs::preorder_iterative;
///
/// let mut bst = BinarySearchTree::new();
/// for v in [4, 2, 6, 1, 3] {
///     bst.insert(v);
/// }
/// let result = preorder_iterative(&bst);
/// assert_eq!(result, vec![&4, &2, &1, &3, &6]);
/// ```
pub fn preorder_iterative<T: Ord>(tree: &BinarySearchTree<T>) -> Vec<&T> {
    let mut result = Vec::new();
    let root = match &tree.root {
        None => return result,
        Some(r) => r,
    };

    // Explicit stack replaces the call stack.
    let mut stack: Vec<&Node<T>> = vec![root];

    while let Some(node) = stack.pop() {
        result.push(&node.value);

        // Push right first so left is processed first (LIFO order).
        if let Some(ref right) = node.right {
            stack.push(right);
        }
        if let Some(ref left) = node.left {
            stack.push(left);
        }
    }
    result
}

/// Iterative in-order DFS traversal.
///
/// In-order visits: left subtree → current node → right subtree.
/// For BSTs, this produces values in sorted (ascending) order.
///
/// The iterative version uses a stack to simulate the recursive call stack.
/// We push nodes as we go left, then pop and process, then go right.
pub fn inorder_iterative<T: Ord>(tree: &BinarySearchTree<T>) -> Vec<&T> {
    let mut result = Vec::new();
    let mut stack: Vec<&Node<T>> = Vec::new();
    let mut current = tree.root.as_deref();

    loop {
        // Go as far left as possible, pushing nodes onto the stack.
        while let Some(node) = current {
            stack.push(node);
            current = node.left.as_deref();
        }

        // If stack is empty, we've visited everything.
        match stack.pop() {
            None => break,
            Some(node) => {
                result.push(&node.value);
                // Now process the right subtree.
                current = node.right.as_deref();
            }
        }
    }
    result
}

/// Iterative post-order DFS traversal.
///
/// Post-order visits: left subtree → right subtree → current node.
/// This is the trickiest to implement iteratively because we need to
/// process children *before* their parent.
///
/// # Algorithm (two-stack approach)
///
/// 1. Push root to stack1.
/// 2. Pop from stack1, push to stack2, push children to stack1.
/// 3. stack2 now has post-order in reverse — pop all from stack2.
pub fn postorder_iterative<T: Ord>(tree: &BinarySearchTree<T>) -> Vec<&T> {
    let root = match &tree.root {
        None => return Vec::new(),
        Some(r) => r,
    };

    let mut stack1: Vec<&Node<T>> = vec![root];
    let mut stack2: Vec<&Node<T>> = Vec::new();

    while let Some(node) = stack1.pop() {
        stack2.push(node);
        if let Some(ref left) = node.left {
            stack1.push(left);
        }
        if let Some(ref right) = node.right {
            stack1.push(right);
        }
    }

    // stack2 has post-order in reverse; reverse it.
    stack2.iter().rev().map(|n| &n.value).collect()
}

// =============================================================================
// Recursive DFS implementations (for comparison)
// =============================================================================

/// Recursive pre-order DFS. Elegant but uses the call stack (O(h) space).
pub fn preorder_recursive<T: Ord>(tree: &BinarySearchTree<T>) -> Vec<&T> {
    let mut result = Vec::new();
    fn recurse<'a, T: Ord>(node: &'a Option<Box<Node<T>>>, result: &mut Vec<&'a T>) {
        if let Some(n) = node {
            result.push(&n.value);
            recurse(&n.left, result);
            recurse(&n.right, result);
        }
    }
    recurse(&tree.root, &mut result);
    result
}

/// Recursive in-order DFS.
pub fn inorder_recursive<T: Ord + Clone>(tree: &BinarySearchTree<T>) -> Vec<&T> {
    // This is the same as BinarySearchTree::inorder(), included here for
    // completeness and to contrast with the iterative version.
    tree.inorder()
}

/// Recursive post-order DFS.
pub fn postorder_recursive<T: Ord + Clone>(tree: &BinarySearchTree<T>) -> Vec<&T> {
    tree.postorder()
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
    fn test_preorder_iterative() {
        let bst = build_test_tree();
        assert_eq!(preorder_iterative(&bst), vec![&4, &2, &1, &3, &6, &5, &7]);
    }

    #[test]
    fn test_inorder_iterative() {
        let bst = build_test_tree();
        assert_eq!(
            inorder_iterative(&bst),
            vec![&1, &2, &3, &4, &5, &6, &7]
        );
    }

    #[test]
    fn test_postorder_iterative() {
        let bst = build_test_tree();
        assert_eq!(
            postorder_iterative(&bst),
            vec![&1, &3, &2, &5, &7, &6, &4]
        );
    }

    #[test]
    fn test_recursive_matches_iterative() {
        let bst = build_test_tree();
        assert_eq!(preorder_iterative(&bst), preorder_recursive(&bst));
        assert_eq!(inorder_iterative(&bst), inorder_recursive(&bst));
        assert_eq!(postorder_iterative(&bst), postorder_recursive(&bst));
    }

    #[test]
    fn test_empty_tree() {
        let bst: BinarySearchTree<i32> = BinarySearchTree::new();
        let empty: Vec<&i32> = vec![];
        assert_eq!(preorder_iterative(&bst), empty);
        assert_eq!(inorder_iterative(&bst), empty);
        assert_eq!(postorder_iterative(&bst), empty);
    }

    #[test]
    fn test_single_node() {
        let mut bst = BinarySearchTree::new();
        bst.insert(42);
        assert_eq!(preorder_iterative(&bst), vec![&42]);
        assert_eq!(inorder_iterative(&bst), vec![&42]);
        assert_eq!(postorder_iterative(&bst), vec![&42]);
    }
}
