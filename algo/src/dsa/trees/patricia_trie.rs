// =============================================================================
// Patricia Trie (Radix Tree / Compact Prefix Tree)
// =============================================================================
//
// # What is a Patricia Trie?
//
// A Patricia Trie (Practical Algorithm to Retrieve Information Coded in
// Alphanumeric) is a space-optimized trie where each node with a single
// child is merged with its child. This "path compression" drastically
// reduces memory usage while maintaining O(k) lookup where k is the key length.
//
// ```text
// Standard Trie:        Patricia Trie:
//       (root)              (root)
//       /                   /
//      t                  "te"
//     /                  /    \
//    e                 "a"    "st"
//   / \                 |       |
//  a   s              (tea)   (test)
//  |   |
// (tea) t
//       |
//      (test)
// ```
//
// # Why Patricia Tries?
//
// - **Ethereum's core data structure**: The Merkle Patricia Trie (MPT) is
//   the fundamental state storage structure in Ethereum. Every account
//   balance, contract storage, and transaction receipt is stored in an MPT.
// - **IP routing tables**: Patricia tries are used in routers for longest
//   prefix matching.
// - **Autocomplete**: Prefix lookups are O(k) regardless of the number of
//   keys in the trie.
//
// # Complexity Table
//
// | Operation       | Time     | Notes                      |
// |-----------------|----------|----------------------------|
// | Insert          | O(k)     | k = key length             |
// | Search          | O(k)     |                            |
// | Delete          | O(k)     |                            |
// | Prefix search   | O(k + m) | m = number of matches      |
// | Space           | O(n·k)   | Worst case; usually better |
//
// # Implementation
//
// We implement a string-keyed Patricia trie using `HashMap` for child
// pointers. Each edge label is a string (the compressed path segment).

use std::collections::HashMap;
use std::fmt;

/// A node in the Patricia Trie.
///
/// Each edge from parent to child carries a string label (the compressed
/// path segment). A node may optionally store a value, indicating that a
/// key terminates here.
#[derive(Debug)]
pub struct PatriciaNode {
    /// Children keyed by the first character of their edge label.
    /// The value is (edge_label, child_node).
    children: HashMap<char, (String, PatriciaNode)>,
    /// True if a key terminates at this node.
    is_terminal: bool,
}

impl PatriciaNode {
    fn new(is_terminal: bool) -> Self {
        PatriciaNode {
            children: HashMap::new(),
            is_terminal,
        }
    }
}

/// A Patricia Trie (Radix Tree) for string keys.
///
/// # Examples
///
/// ```
/// use algo::dsa::trees::patricia_trie::PatriciaTrie;
///
/// let mut trie = PatriciaTrie::new();
/// trie.insert("test");
/// trie.insert("team");
/// assert!(trie.contains("test"));
/// assert!(!trie.contains("tea")); // "tea" was not inserted
/// ```
#[derive(Debug)]
pub struct PatriciaTrie {
    root: PatriciaNode,
    size: usize,
}

impl PatriciaTrie {
    /// Creates a new, empty Patricia trie.
    pub fn new() -> Self {
        PatriciaTrie {
            root: PatriciaNode::new(false),
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Inserts a string key into the trie.
    ///
    /// # Algorithm
    ///
    /// Walk down the trie matching the key against edge labels:
    /// 1. If no edge matches the first character — create a new edge.
    /// 2. If an edge partially matches — split the edge at the mismatch point.
    /// 3. If an edge fully matches — consume the matched prefix and continue
    ///    with the remaining key into the child node.
    pub fn insert(&mut self, key: &str) {
        if key.is_empty() {
            if !self.root.is_terminal {
                self.root.is_terminal = true;
                self.size += 1;
            }
            return;
        }
        if Self::insert_at(&mut self.root, key) {
            self.size += 1;
        }
    }

    fn insert_at(node: &mut PatriciaNode, remaining: &str) -> bool {
        let first_char = remaining.chars().next().unwrap();

        match node.children.remove(&first_char) {
            None => {
                // No matching edge — create a new one.
                node.children.insert(
                    first_char,
                    (remaining.to_string(), PatriciaNode::new(true)),
                );
                true
            }
            Some((label, mut child)) => {
                // Find the common prefix length between the edge label and remaining key.
                let common_len = label
                    .chars()
                    .zip(remaining.chars())
                    .take_while(|(a, b)| a == b)
                    .count();

                if common_len == label.len() && common_len == remaining.len() {
                    // Exact match — mark this node as terminal.
                    let was_terminal = child.is_terminal;
                    child.is_terminal = true;
                    node.children.insert(first_char, (label, child));
                    !was_terminal
                } else if common_len == label.len() {
                    // Edge label is a prefix of remaining — continue into child.
                    let rest = &remaining[common_len..];
                    let inserted = Self::insert_at(&mut child, rest);
                    node.children.insert(first_char, (label, child));
                    inserted
                } else if common_len == remaining.len() {
                    // Remaining key is a prefix of the edge label — split edge.
                    let suffix = label[common_len..].to_string();
                    let suffix_first = suffix.chars().next().unwrap();
                    let mut new_node = PatriciaNode::new(true);
                    new_node.children.insert(suffix_first, (suffix, child));
                    let prefix = label[..common_len].to_string();
                    let prefix_first = prefix.chars().next().unwrap();
                    node.children.insert(prefix_first, (prefix, new_node));
                    true
                } else {
                    // Partial match — split the edge at the mismatch.
                    let common_prefix = label[..common_len].to_string();
                    let label_suffix = label[common_len..].to_string();
                    let key_suffix = remaining[common_len..].to_string();

                    let label_suffix_first = label_suffix.chars().next().unwrap();
                    let key_suffix_first = key_suffix.chars().next().unwrap();

                    let mut split_node = PatriciaNode::new(false);
                    split_node
                        .children
                        .insert(label_suffix_first, (label_suffix, child));
                    split_node.children.insert(
                        key_suffix_first,
                        (key_suffix, PatriciaNode::new(true)),
                    );

                    let prefix_first = common_prefix.chars().next().unwrap();
                    node.children
                        .insert(prefix_first, (common_prefix, split_node));
                    true
                }
            }
        }
    }

    /// Searches for an exact key match.
    pub fn contains(&self, key: &str) -> bool {
        if key.is_empty() {
            return self.root.is_terminal;
        }
        Self::search_at(&self.root, key)
    }

    fn search_at(node: &PatriciaNode, remaining: &str) -> bool {
        let first_char = match remaining.chars().next() {
            None => return node.is_terminal,
            Some(c) => c,
        };

        match node.children.get(&first_char) {
            None => false,
            Some((label, child)) => {
                if remaining.starts_with(label.as_str()) {
                    if remaining.len() == label.len() {
                        child.is_terminal
                    } else {
                        Self::search_at(child, &remaining[label.len()..])
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Returns all keys that start with the given prefix.
    ///
    /// This is the killer feature of tries: prefix search in O(prefix_len + matches).
    pub fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut result = Vec::new();
        self.collect_with_prefix(&self.root, prefix, String::new(), &mut result);
        result
    }

    fn collect_with_prefix(
        &self,
        node: &PatriciaNode,
        remaining_prefix: &str,
        current_path: String,
        result: &mut Vec<String>,
    ) {
        if remaining_prefix.is_empty() {
            // We've consumed the entire prefix — collect all keys below.
            self.collect_all(node, &current_path, result);
            return;
        }

        let first_char = remaining_prefix.chars().next().unwrap();
        if let Some((label, child)) = node.children.get(&first_char) {
            if remaining_prefix.starts_with(label.as_str()) {
                // Edge label is a prefix of remaining prefix.
                self.collect_with_prefix(
                    child,
                    &remaining_prefix[label.len()..],
                    format!("{}{}", current_path, label),
                    result,
                );
            } else if label.starts_with(remaining_prefix) {
                // Remaining prefix is a prefix of the edge label.
                self.collect_all(child, &format!("{}{}", current_path, label), result);
            }
        }
    }

    fn collect_all(&self, node: &PatriciaNode, path: &str, result: &mut Vec<String>) {
        if node.is_terminal {
            result.push(path.to_string());
        }
        for (label, child) in node.children.values() {
            self.collect_all(child, &format!("{}{}", path, label), result);
        }
    }

    /// Deletes a key from the trie.
    pub fn delete(&mut self, key: &str) {
        if key.is_empty() {
            if self.root.is_terminal {
                self.root.is_terminal = false;
                self.size -= 1;
            }
            return;
        }
        if Self::delete_at(&mut self.root, key) {
            self.size -= 1;
        }
    }

    fn delete_at(node: &mut PatriciaNode, remaining: &str) -> bool {
        let first_char = match remaining.chars().next() {
            None => {
                if node.is_terminal {
                    node.is_terminal = false;
                    return true;
                }
                return false;
            }
            Some(c) => c,
        };

        let should_compact;
        {
            match node.children.get_mut(&first_char) {
                None => return false,
                Some((label, child)) => {
                    let label_clone = label.clone();
                    if remaining == label_clone {
                        if !child.is_terminal {
                            return false;
                        }
                        child.is_terminal = false;
                        should_compact = child.children.is_empty()
                            || (child.children.len() == 1 && !child.is_terminal);
                    } else if remaining.starts_with(label_clone.as_str()) {
                        let deleted = Self::delete_at(child, &remaining[label_clone.len()..]);
                        if !deleted {
                            return false;
                        }
                        should_compact = child.children.len() == 1 && !child.is_terminal;
                    } else {
                        return false;
                    }
                }
            }
        }

        if should_compact {
            let (label, child) = node.children.remove(&first_char).unwrap();
            if child.children.is_empty() {
                // Node has no children and is not terminal — remove entirely.
            } else if child.children.len() == 1 && !child.is_terminal {
                // Merge with single child.
                let (_, (child_label, grandchild)) = child.children.into_iter().next().unwrap();
                let merged_label = format!("{}{}", label, child_label);
                let merged_first = merged_label.chars().next().unwrap();
                node.children
                    .insert(merged_first, (merged_label, grandchild));
            }
        }
        true
    }
}

impl Default for PatriciaTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PatriciaTrie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut keys: Vec<String> = Vec::new();
        self.collect_all(&self.root, "", &mut keys);
        keys.sort();
        write!(f, "PatriciaTrie[{}]", keys.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_search() {
        let mut trie = PatriciaTrie::new();
        trie.insert("test");
        trie.insert("team");
        trie.insert("toast");
        assert!(trie.contains("test"));
        assert!(trie.contains("team"));
        assert!(trie.contains("toast"));
        assert!(!trie.contains("tea"));
        assert!(!trie.contains("tes"));
    }

    #[test]
    fn test_prefix_search() {
        let mut trie = PatriciaTrie::new();
        trie.insert("apple");
        trie.insert("application");
        trie.insert("apply");
        trie.insert("banana");

        let mut result = trie.keys_with_prefix("app");
        result.sort();
        assert_eq!(result, vec!["apple", "application", "apply"]);

        let result = trie.keys_with_prefix("ban");
        assert_eq!(result, vec!["banana"]);

        let result = trie.keys_with_prefix("xyz");
        assert!(result.is_empty());
    }

    #[test]
    fn test_delete() {
        let mut trie = PatriciaTrie::new();
        trie.insert("test");
        trie.insert("team");
        trie.insert("tea");
        assert_eq!(trie.len(), 3);

        trie.delete("tea");
        assert!(!trie.contains("tea"));
        assert!(trie.contains("team"));
        assert!(trie.contains("test"));
        assert_eq!(trie.len(), 2);
    }

    #[test]
    fn test_edge_splitting() {
        // Inserting keys that share a common prefix forces edge splitting.
        let mut trie = PatriciaTrie::new();
        trie.insert("romane");
        trie.insert("romanus");
        trie.insert("romulus");
        trie.insert("rubens");
        trie.insert("ruber");
        assert!(trie.contains("romane"));
        assert!(trie.contains("romanus"));
        assert!(trie.contains("romulus"));
        assert!(trie.contains("rubens"));
        assert!(trie.contains("ruber"));
        assert!(!trie.contains("rom"));
    }

    #[test]
    fn test_display() {
        let mut trie = PatriciaTrie::new();
        trie.insert("a");
        trie.insert("b");
        let s = format!("{}", trie);
        assert!(s.starts_with("PatriciaTrie["));
    }

    #[test]
    fn test_empty_and_single() {
        let mut trie = PatriciaTrie::new();
        assert!(trie.is_empty());
        trie.insert("hello");
        assert_eq!(trie.len(), 1);
        assert!(trie.contains("hello"));
    }
}
