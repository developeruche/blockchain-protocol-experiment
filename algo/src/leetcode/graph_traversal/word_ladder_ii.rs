// =============================================================================
// LeetCode 126 — Word Ladder II (Hard)
// =============================================================================
//
// # Problem Summary
// Given begin_word, end_word, and a word list, find all shortest
// transformation sequences from begin to end, changing one letter at a time,
// where each intermediate word must be in the word list.
//
// # Approach — BFS + Backtrack
// 1. BFS from begin_word to find shortest distances to each word.
// 2. Backtrack from end_word following decreasing distances to reconstruct paths.
//
// # Complexity
// - Time: O(N · L² + paths) where N=words, L=word length.
// - Space: O(N · L).
//
// Link: https://leetcode.com/problems/word-ladder-ii/

use std::collections::{HashMap, HashSet, VecDeque};

/// Finds all shortest transformation sequences.
pub fn find_ladders(
    begin_word: &str,
    end_word: &str,
    word_list: &[String],
) -> Vec<Vec<String>> {
    let word_set: HashSet<&str> = word_list.iter().map(|s| s.as_str()).collect();
    if !word_set.contains(end_word) {
        return vec![];
    }

    // BFS to find distances from begin_word.
    let mut dist: HashMap<String, usize> = HashMap::new();
    dist.insert(begin_word.to_string(), 0);
    let mut queue = VecDeque::new();
    queue.push_back(begin_word.to_string());
    let mut found = false;

    while let Some(word) = queue.pop_front() {
        let d = dist[&word];
        if word == end_word {
            found = true;
            // Don't break — explore all same-distance words.
        }
        if found && d + 1 > dist.get(end_word).copied().unwrap_or(usize::MAX) {
            break;
        }
        let bytes = word.as_bytes().to_vec();
        for i in 0..bytes.len() {
            let mut next_bytes = bytes.clone();
            for c in b'a'..=b'z' {
                if c == bytes[i] { continue; }
                next_bytes[i] = c;
                let next = String::from_utf8(next_bytes.clone()).unwrap();
                if word_set.contains(next.as_str()) && !dist.contains_key(&next) {
                    dist.insert(next.clone(), d + 1);
                    queue.push_back(next);
                }
            }
        }
    }

    if !found {
        return vec![];
    }

    // Backtrack from end_word to build all paths.
    let mut result = Vec::new();
    let mut path = vec![end_word.to_string()];
    backtrack(end_word, begin_word, &dist, &mut path, &mut result);
    result
}

fn backtrack(
    word: &str,
    begin: &str,
    dist: &HashMap<String, usize>,
    path: &mut Vec<String>,
    result: &mut Vec<Vec<String>>,
) {
    if word == begin {
        let mut p = path.clone();
        p.reverse();
        result.push(p);
        return;
    }

    let d = dist[word];
    let bytes = word.as_bytes().to_vec();
    for i in 0..bytes.len() {
        let mut prev_bytes = bytes.clone();
        for c in b'a'..=b'z' {
            if c == bytes[i] { continue; }
            prev_bytes[i] = c;
            let prev = String::from_utf8(prev_bytes.clone()).unwrap();
            if dist.get(&prev).copied() == Some(d - 1) {
                path.push(prev.clone());
                backtrack(&prev, begin, dist, path, result);
                path.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let words: Vec<String> = vec!["hot","dot","dog","lot","log","cog"]
            .into_iter().map(String::from).collect();
        let result = find_ladders("hit", "cog", &words);
        assert!(!result.is_empty());
        // All paths should have length 5.
        for path in &result {
            assert_eq!(path.len(), 5);
            assert_eq!(path[0], "hit");
            assert_eq!(path[4], "cog");
        }
    }

    #[test]
    fn test_no_path() {
        let words: Vec<String> = vec!["hot","dot","dog","lot","log"]
            .into_iter().map(String::from).collect();
        let result = find_ladders("hit", "cog", &words);
        assert!(result.is_empty());
    }

    #[test]
    fn test_one_step() {
        let words = vec!["hot".to_string()];
        let result = find_ladders("hot", "hot", &words);
        // begin == end
        assert!(result.is_empty() || result[0].len() == 1);
    }
}
