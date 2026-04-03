// =============================================================================
// LeetCode 424 — Longest Repeating Character Replacement (Medium)
// =============================================================================
//
// # Problem Summary
// Given a string `s` and an integer `k`, find the length of the longest
// substring where you can change at most `k` characters to make all
// characters the same.
//
// # Approach / Intuition
// Sliding window: maintain a window [left, right] and track the count of
// the most frequent character within the window. The window is valid if:
//   window_length - max_char_count ≤ k
// i.e., the number of characters to change is at most k.
//
// Key insight: we never need to decrease `max_count` when shrinking the window.
// If a previous window achieved max_count = M, we only care about finding
// a *longer* valid window, which requires max_count ≥ M. So we can keep
// `max_count` as a "high water mark" — this doesn't affect correctness.
//
// # Complexity
// - Time: O(n) — single pass.
// - Space: O(1) — fixed-size frequency array (26 letters).
//
// Link: https://leetcode.com/problems/longest-repeating-character-replacement/

/// Returns the length of the longest substring achievable with at most k replacements.
pub fn character_replacement(s: &str, k: i32) -> i32 {
    let bytes = s.as_bytes();
    let mut counts = [0i32; 26]; // Frequency of each letter in current window.
    let mut max_count = 0; // Max frequency of any single char in current window.
    let mut left = 0;
    let mut result = 0;

    for right in 0..bytes.len() {
        let idx = (bytes[right] - b'A') as usize;
        counts[idx] += 1;
        max_count = max_count.max(counts[idx]);

        // Window length - count of most frequent char = chars to replace.
        // If this exceeds k, shrink from left.
        let window_len = (right - left + 1) as i32;
        if window_len - max_count > k {
            let left_idx = (bytes[left] - b'A') as usize;
            counts[left_idx] -= 1;
            left += 1;
        }

        result = result.max((right - left + 1) as i32);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(character_replacement("ABAB", 2), 4);
    }

    #[test]
    fn test_mixed() {
        assert_eq!(character_replacement("AABABBA", 1), 4);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(character_replacement("AAAA", 2), 4);
    }

    #[test]
    fn test_single_char() {
        assert_eq!(character_replacement("A", 0), 1);
    }

    #[test]
    fn test_k_zero() {
        assert_eq!(character_replacement("ABCDE", 0), 1);
    }
}
