// =============================================================================
// LeetCode 208 — Implement Trie (Prefix Tree) (Medium)
// =============================================================================
//
// # Problem Summary
// Implement a trie with insert, search, and startsWith methods.
//
// # Approach / Intuition
// A trie stores strings by sharing common prefixes. Each node has up to 26
// children (for lowercase English letters). A boolean flag marks word endings.
//
// **Blockchain analogy**: This is a direct analogue to the **Merkle Patricia
// Trie** used in Ethereum. The MPT extends this concept with hashing at each
// node and path compression (see our `patricia_trie.rs` in the DSA module).
//
// # Complexity
// - Insert: O(m) where m = word length.
// - Search: O(m).
// - StartsWith: O(m).
// - Space: O(n · m) worst case for n words of length m.
//
// Link: https://leetcode.com/problems/implement-trie-prefix-tree/

/// A node in the trie. Using a fixed-size array for O(1) child lookup.
#[derive(Debug)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    is_end: bool,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: Default::default(),
            is_end: false,
        }
    }
}

/// A Trie (prefix tree) supporting insert, search, and prefix matching.
#[derive(Debug)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    /// Creates a new, empty Trie.
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    /// Inserts a word into the trie.
    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for b in word.bytes() {
            let idx = (b - b'a') as usize;
            node = node.children[idx].get_or_insert_with(|| Box::new(TrieNode::new()));
        }
        node.is_end = true;
    }

    /// Returns true if the word exists in the trie.
    pub fn search(&self, word: &str) -> bool {
        self.find_node(word).is_some_and(|n| n.is_end)
    }

    /// Returns true if any word in the trie starts with the given prefix.
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.find_node(prefix).is_some()
    }

    /// Traverses the trie following the given string, returning the final node.
    fn find_node(&self, s: &str) -> Option<&TrieNode> {
        let mut node = &self.root;
        for b in s.bytes() {
            let idx = (b - b'a') as usize;
            match &node.children[idx] {
                Some(child) => node = child,
                None => return None,
            }
        }
        Some(node)
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut trie = Trie::new();
        trie.insert("apple");
        assert!(trie.search("apple"));
        assert!(!trie.search("app"));
        assert!(trie.starts_with("app"));
    }

    #[test]
    fn test_insert_then_search() {
        let mut trie = Trie::new();
        trie.insert("app");
        assert!(trie.search("app"));
        assert!(trie.starts_with("app"));
        assert!(!trie.search("apple"));
    }

    #[test]
    fn test_multiple_words() {
        let mut trie = Trie::new();
        trie.insert("apple");
        trie.insert("app");
        trie.insert("banana");
        assert!(trie.search("apple"));
        assert!(trie.search("app"));
        assert!(trie.search("banana"));
        assert!(!trie.search("ban"));
    }
}
