// =============================================================================
// LeetCode 438 — Find All Anagrams in a String (Medium)
// =============================================================================
//
// # Problem Summary
// Given strings `s` and `p`, find all start indices of `p`'s anagrams in `s`.
//
// # Approach / Intuition
// Rolling window + frequency comparison: maintain a window of size |p| over s.
// Use a fixed-size frequency array (26 letters) for both p and the window.
// Slide the window one character at a time, updating counts.
//
// # Complexity
// - Time: O(n) where n = |s| — each character is added/removed once.
// - Space: O(1) — fixed 26-element arrays.
//
// Link: https://leetcode.com/problems/find-all-anagrams-in-a-string/

/// Returns start indices of all anagrams of `p` in `s`.
pub fn find_anagrams(s: &str, p: &str) -> Vec<i32> {
    if p.len() > s.len() {
        return vec![];
    }

    let sb = s.as_bytes();
    let pb = p.as_bytes();
    let mut p_counts = [0i32; 26];
    let mut w_counts = [0i32; 26];
    let mut result = Vec::new();

    // Build frequency map for p.
    for &b in pb {
        p_counts[(b - b'a') as usize] += 1;
    }

    for i in 0..sb.len() {
        // Add new character to window.
        w_counts[(sb[i] - b'a') as usize] += 1;

        // Remove character that falls out of the window.
        if i >= pb.len() {
            w_counts[(sb[i - pb.len()] - b'a') as usize] -= 1;
        }

        // Compare frequency arrays.
        if w_counts == p_counts {
            result.push((i + 1 - pb.len()) as i32);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(find_anagrams("cbaebabacd", "abc"), vec![0, 6]);
    }

    #[test]
    fn test_overlapping() {
        assert_eq!(find_anagrams("abab", "ab"), vec![0, 1, 2]);
    }

    #[test]
    fn test_no_match() {
        assert_eq!(find_anagrams("abc", "xyz"), Vec::<i32>::new());
    }

    #[test]
    fn test_p_longer() {
        assert_eq!(find_anagrams("a", "abc"), Vec::<i32>::new());
    }

    #[test]
    fn test_single_char() {
        assert_eq!(find_anagrams("aaaa", "a"), vec![0, 1, 2, 3]);
    }
}
