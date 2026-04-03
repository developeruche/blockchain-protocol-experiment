// =============================================================================
// LeetCode 269 — Alien Dictionary (Hard)
// =============================================================================
//
// # Problem Summary
// Given a sorted list of words in an alien language, derive the order of
// characters. Return any valid order or "" if invalid.
//
// # Approach
// Build a directed graph of character orderings from adjacent word pairs.
// If word[i] and word[i+1] differ at position j, we get edge (word[i][j] → word[i+1][j]).
// Then topologically sort the graph.
//
// Edge case: if a longer word appears before its prefix (e.g., "abc" before "ab"),
// the ordering is invalid.
//
// # Complexity
// - Time: O(C) where C = total characters across all words.
// - Space: O(1) — at most 26 vertices.
//
// Link: https://leetcode.com/problems/alien-dictionary/

use std::collections::{HashMap, HashSet, VecDeque};

/// Derives the alien alphabet order from sorted words.
pub fn alien_order(words: &[String]) -> String {
    // Build adjacency list and in-degree map.
    let mut adj: HashMap<u8, HashSet<u8>> = HashMap::new();
    let mut in_degree: HashMap<u8, i32> = HashMap::new();

    // Initialize all characters.
    for word in words {
        for &b in word.as_bytes() {
            adj.entry(b).or_default();
            in_degree.entry(b).or_insert(0);
        }
    }

    // Compare adjacent words to find ordering constraints.
    for i in 0..words.len() - 1 {
        let w1 = words[i].as_bytes();
        let w2 = words[i + 1].as_bytes();

        // Edge case: longer word before its prefix is invalid.
        if w1.len() > w2.len() && w1.starts_with(w2) {
            return String::new();
        }

        for j in 0..w1.len().min(w2.len()) {
            if w1[j] != w2[j] {
                // First differing character gives us an edge.
                if adj.get_mut(&w1[j]).unwrap().insert(w2[j]) {
                    *in_degree.get_mut(&w2[j]).unwrap() += 1;
                }
                break; // Only the first difference matters.
            }
        }
    }

    // Kahn's topological sort.
    let mut queue: VecDeque<u8> = in_degree
        .iter()
        .filter(|&(_, &d)| d == 0)
        .map(|(&c, _)| c)
        .collect();

    // Sort for deterministic output.
    let mut sorted_init: Vec<u8> = queue.drain(..).collect();
    sorted_init.sort();
    queue.extend(sorted_init);

    let mut result = Vec::new();
    while let Some(c) = queue.pop_front() {
        result.push(c);
        if let Some(neighbors) = adj.get(&c) {
            let mut sorted_neighbors: Vec<u8> = neighbors.iter().copied().collect();
            sorted_neighbors.sort();
            for n in sorted_neighbors {
                *in_degree.get_mut(&n).unwrap() -= 1;
                if in_degree[&n] == 0 {
                    queue.push_back(n);
                }
            }
        }
    }

    if result.len() == in_degree.len() {
        String::from_utf8(result).unwrap()
    } else {
        String::new() // Cycle = invalid ordering.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let words: Vec<String> = vec!["wrt", "wrf", "er", "ett", "rftt"]
            .into_iter().map(String::from).collect();
        let result = alien_order(&words);
        // Valid order must have: w before e, r before t, t before f, e before r
        assert!(!result.is_empty());
        let pos: HashMap<u8, usize> = result.bytes().enumerate().map(|(i,b)| (b, i)).collect();
        assert!(pos[&b'w'] < pos[&b'e']);
        assert!(pos[&b'r'] < pos[&b't']);
        assert!(pos[&b't'] < pos[&b'f']);
        assert!(pos[&b'e'] < pos[&b'r']);
    }

    #[test]
    fn test_cycle() {
        let words: Vec<String> = vec!["z", "x", "z"].into_iter().map(String::from).collect();
        assert_eq!(alien_order(&words), "");
    }

    #[test]
    fn test_prefix_invalid() {
        let words: Vec<String> = vec!["abc", "ab"].into_iter().map(String::from).collect();
        assert_eq!(alien_order(&words), "");
    }
}
