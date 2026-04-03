// =============================================================================
// LeetCode 212 — Word Search II (Hard)
// =============================================================================
//
// # Problem Summary
// Given an m×n board of characters and a list of words, find all words that
// can be formed by sequentially adjacent cells (horizontally or vertically).
// Each cell may only be used once per word.
//
// # Approach / Intuition
// Build a Trie from the word list. Then do DFS/backtracking from each cell,
// following trie edges to prune branches early. Without the trie, we'd need
// to search for each word independently — the trie lets us search for all
// words simultaneously.
//
// **Optimization**: Remove trie leaf nodes after finding a word to avoid
// duplicate results and reduce future search space.
//
// **Blockchain analogy**: This trie + pruning approach mirrors how state
// verification works — you only traverse branches that could lead to valid
// proofs, pruning impossible paths early.
//
// # Complexity
// - Time: O(m · n · 4^L) worst case, but heavily pruned by the trie.
// - Space: O(sum of word lengths) for the trie.
//
// Link: https://leetcode.com/problems/word-search-ii/

#[derive(Default)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    word: Option<String>, // Store the complete word at terminal nodes.
}

/// Finds all words from the dictionary that exist on the board.
pub fn find_words(board: &[Vec<char>], words: &[String]) -> Vec<String> {
    if board.is_empty() || words.is_empty() {
        return vec![];
    }

    // Build trie from word list.
    let mut root = TrieNode::default();
    for word in words {
        let mut node = &mut root;
        for b in word.bytes() {
            let idx = (b - b'a') as usize;
            node = node.children[idx].get_or_insert_with(|| Box::new(TrieNode::default()));
        }
        node.word = Some(word.clone());
    }

    let rows = board.len();
    let cols = board[0].len();
    let mut visited = vec![vec![false; cols]; rows];
    let mut result = Vec::new();

    for r in 0..rows {
        for c in 0..cols {
            dfs(board, &mut root, r, c, &mut visited, &mut result);
        }
    }

    result
}

fn dfs(
    board: &[Vec<char>],
    node: &mut TrieNode,
    r: usize,
    c: usize,
    visited: &mut [Vec<bool>],
    result: &mut Vec<String>,
) {
    let idx = (board[r][c] as u8 - b'a') as usize;
    let child = match &mut node.children[idx] {
        Some(c) => c,
        None => return, // No trie edge — prune.
    };

    // Check if we found a word.
    if let Some(word) = child.word.take() {
        result.push(word);
    }

    visited[r][c] = true;

    // Explore 4 directions.
    let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for (dr, dc) in &directions {
        let nr = r as i32 + dr;
        let nc = c as i32 + dc;
        if nr >= 0
            && nr < board.len() as i32
            && nc >= 0
            && nc < board[0].len() as i32
        {
            let nr = nr as usize;
            let nc = nc as usize;
            if !visited[nr][nc] {
                dfs(board, child, nr, nc, visited, result);
            }
        }
    }

    visited[r][c] = false;

    // Pruning: if the child has no children and no word, remove it.
    if child.children.iter().all(|c| c.is_none()) && child.word.is_none() {
        node.children[idx] = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let board = vec![
            vec!['o', 'a', 'a', 'n'],
            vec!['e', 't', 'a', 'e'],
            vec!['i', 'h', 'k', 'r'],
            vec!['i', 'f', 'l', 'v'],
        ];
        let words: Vec<String> = vec!["oath", "pea", "eat", "rain"]
            .into_iter()
            .map(String::from)
            .collect();
        let mut result = find_words(&board, &words);
        result.sort();
        assert_eq!(result, vec!["eat", "oath"]);
    }

    #[test]
    fn test_single_cell() {
        let board = vec![vec!['a']];
        let words = vec!["a".to_string()];
        assert_eq!(find_words(&board, &words), vec!["a"]);
    }

    #[test]
    fn test_no_match() {
        let board = vec![vec!['a', 'b'], vec!['c', 'd']];
        let words = vec!["xyz".to_string()];
        assert_eq!(find_words(&board, &words), Vec::<String>::new());
    }
}
