// =============================================================================
// LeetCode 76 — Minimum Window Substring (Hard)
// =============================================================================
//
// # Problem Summary
// Given two strings `s` and `t`, find the smallest substring of `s` that
// contains all characters of `t` (including duplicates). If no such window
// exists, return an empty string.
//
// # Approach / Intuition
// Use a **sliding window** with two pointers (left, right) and a frequency map:
// 1. Expand the window by moving `right` to include more characters.
// 2. Once all characters of `t` are covered, try to shrink from `left`.
// 3. Track the minimum valid window seen so far.
//
// The key data structure is a frequency map (HashMap) tracking how many of
// each character we still "need". When all counts reach ≤ 0, the window is valid.
//
// **Blockchain analogy**: This frequency-map + sliding-window pattern mirrors
// resource accounting in transaction pools — tracking which resources are
// "satisfied" as you include more transactions.
//
// # Complexity
// - Time: O(|s| + |t|) — each character is visited at most twice (once by
//   right, once by left).
// - Space: O(|t|) — the frequency map size is bounded by the characters in t.
//
// Link: https://leetcode.com/problems/minimum-window-substring/

use std::collections::HashMap;

/// Finds the minimum window substring of `s` that contains all characters of `t`.
pub fn min_window(s: &str, t: &str) -> String {
    if t.is_empty() || s.len() < t.len() {
        return String::new();
    }

    // Build frequency map of characters we need from t.
    let mut need: HashMap<u8, i32> = HashMap::new();
    for &b in t.as_bytes() {
        *need.entry(b).or_insert(0) += 1;
    }

    // `required` = number of distinct characters in t that we still need
    // to satisfy. When this reaches 0, the current window is valid.
    let mut required = need.len();
    let s_bytes = s.as_bytes();
    let mut left = 0;
    let mut min_len = usize::MAX;
    let mut min_start = 0;

    // `window_counts` tracks character frequencies in our current window.
    let mut window_counts: HashMap<u8, i32> = HashMap::new();

    for right in 0..s_bytes.len() {
        let c = s_bytes[right];
        *window_counts.entry(c).or_insert(0) += 1;

        // If this character is in t and we now have enough of it,
        // decrement `required`.
        if let Some(&needed) = need.get(&c)
            && window_counts[&c] == needed {
                required -= 1;
            }

        // Shrink window from left while it's still valid.
        while required == 0 {
            let window_len = right - left + 1;
            if window_len < min_len {
                min_len = window_len;
                min_start = left;
            }

            // Remove leftmost character from window.
            let left_char = s_bytes[left];
            *window_counts.get_mut(&left_char).unwrap() -= 1;
            if let Some(&needed) = need.get(&left_char)
                && window_counts[&left_char] < needed {
                    required += 1; // We lost a needed character.
                }
            left += 1;
        }
    }

    if min_len == usize::MAX {
        String::new()
    } else {
        s[min_start..min_start + min_len].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(min_window("ADOBECODEBANC", "ABC"), "BANC");
    }

    #[test]
    fn test_exact_match() {
        assert_eq!(min_window("a", "a"), "a");
    }

    #[test]
    fn test_no_match() {
        assert_eq!(min_window("a", "aa"), "");
        assert_eq!(min_window("abc", "xyz"), "");
    }

    #[test]
    fn test_t_empty() {
        assert_eq!(min_window("abc", ""), "");
    }

    #[test]
    fn test_whole_string() {
        assert_eq!(min_window("ab", "ab"), "ab");
    }
}
