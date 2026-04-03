// =============================================================================
// LeetCode 3 — Longest Substring Without Repeating Characters (Medium)
// =============================================================================
//
// # Problem Summary
// Given a string `s`, find the length of the longest substring without any
// repeating characters.
//
// # Approach / Intuition
// Classic sliding window: maintain a window [left, right] where all characters
// are unique. Use a HashSet (or array for ASCII) to track which characters
// are in the current window. When a duplicate is found, shrink from left
// until the duplicate is removed.
//
// # Complexity
// - Time: O(n) — each character is added and removed from the set at most once.
// - Space: O(min(n, charset_size)) — the set size is bounded by the character set.
//
// Link: https://leetcode.com/problems/longest-substring-without-repeating-characters/

use std::collections::HashMap;

/// Returns the length of the longest substring without repeating characters.
pub fn length_of_longest_substring(s: &str) -> i32 {
    // Map each character to its most recent index in the string.
    // This allows us to jump `left` directly past the duplicate.
    let mut last_seen: HashMap<u8, usize> = HashMap::new();
    let mut max_len = 0;
    let mut left = 0;
    let bytes = s.as_bytes();

    for (right, &c) in bytes.iter().enumerate() {
        if let Some(&prev_idx) = last_seen.get(&c) {
            // If the duplicate is within our current window, move left past it.
            // The max() handles cases where the duplicate is before our window.
            left = left.max(prev_idx + 1);
        }
        last_seen.insert(c, right);
        max_len = max_len.max(right - left + 1);
    }

    max_len as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(length_of_longest_substring("abcabcbb"), 3); // "abc"
    }

    #[test]
    fn test_all_same() {
        assert_eq!(length_of_longest_substring("bbbbb"), 1); // "b"
    }

    #[test]
    fn test_mixed() {
        assert_eq!(length_of_longest_substring("pwwkew"), 3); // "wke"
    }

    #[test]
    fn test_empty() {
        assert_eq!(length_of_longest_substring(""), 0);
    }

    #[test]
    fn test_all_unique() {
        assert_eq!(length_of_longest_substring("abcdef"), 6);
    }
}
