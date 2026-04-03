// =============================================================================
// LeetCode 211 — Design Add and Search Words Data Structure (Medium)
// =============================================================================
//
// # Problem Summary
// Design a data structure that supports adding words and searching with '.'
// wildcards (matches any single character).
//
// # Approach / Intuition
// Use a Trie for storage. Search is a standard trie traversal, except when
// we encounter '.', we must try ALL children (DFS with backtracking).
//
// # Complexity
// - addWord: O(m), m = word length.
// - search: O(26^m) worst case (all dots), but typically much better.
// - Space: O(n · m) for n words.
//
// Link: https://leetcode.com/problems/design-add-and-search-words-data-structure/

#[derive(Debug, Default)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    is_end: bool,
}

/// A word dictionary supporting wildcards in search.
#[derive(Debug, Default)]
pub struct WordDictionary {
    root: TrieNode,
}

impl WordDictionary {
    pub fn new() -> Self {
        WordDictionary {
            root: TrieNode::default(),
        }
    }

    /// Adds a word to the dictionary.
    pub fn add_word(&mut self, word: &str) {
        let mut node = &mut self.root;
        for b in word.bytes() {
            let idx = (b - b'a') as usize;
            node = node.children[idx].get_or_insert_with(|| Box::new(TrieNode::default()));
        }
        node.is_end = true;
    }

    /// Searches for a word, where '.' matches any letter.
    pub fn search(&self, word: &str) -> bool {
        Self::dfs_search(&self.root, word.as_bytes(), 0)
    }

    fn dfs_search(node: &TrieNode, word: &[u8], pos: usize) -> bool {
        if pos == word.len() {
            return node.is_end;
        }

        let ch = word[pos];
        if ch == b'.' {
            // Wildcard: try every child.
            for child in &node.children {
                if let Some(c) = child
                    && Self::dfs_search(c, word, pos + 1) {
                        return true;
                    }
            }
            false
        } else {
            let idx = (ch - b'a') as usize;
            match &node.children[idx] {
                Some(c) => Self::dfs_search(c, word, pos + 1),
                None => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut dict = WordDictionary::new();
        dict.add_word("bad");
        dict.add_word("dad");
        dict.add_word("mad");
        assert!(!dict.search("pad"));
        assert!(dict.search("bad"));
        assert!(dict.search(".ad"));
        assert!(dict.search("b.."));
    }

    #[test]
    fn test_no_match() {
        let mut dict = WordDictionary::new();
        dict.add_word("a");
        assert!(!dict.search("ab"));
    }

    #[test]
    fn test_all_dots() {
        let mut dict = WordDictionary::new();
        dict.add_word("abc");
        assert!(dict.search("..."));
        assert!(!dict.search("...."));
    }
}
