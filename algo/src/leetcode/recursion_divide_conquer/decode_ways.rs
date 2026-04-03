// =============================================================================
// LeetCode 91 — Decode Ways (Medium)
// =============================================================================
//
// # Problem Summary
// Given a string of digits, count the number of ways to decode it where
// 'A'=1, 'B'=2, ..., 'Z'=26.
//
// # Approach — DP with Recursive Structure
// dp[i] = number of ways to decode s[0..i].
// - If s[i] is '1'-'9', we can decode it as a single digit: dp[i] += dp[i-1].
// - If s[i-1..=i] forms 10-26, we can decode as two digits: dp[i] += dp[i-2].
//
// **Blockchain analogy**: This is similar to SCALE-like variable-length
// encoding in Substrate, where a byte sequence can be decoded in multiple
// valid ways depending on length prefixes.
//
// # Complexity
// - Time: O(n).
// - Space: O(1) — only need previous two values.
//
// Link: https://leetcode.com/problems/decode-ways/

/// Returns the number of ways to decode the digit string.
pub fn num_decodings(s: &str) -> i32 {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] == b'0' {
        return 0;
    }

    let n = bytes.len();
    // Two variables instead of a full array: dp_prev2 = dp[i-2], dp_prev1 = dp[i-1].
    let mut dp_prev2 = 1i32; // Empty string has one decoding (base case).
    let mut dp_prev1 = 1i32; // First char is non-zero (checked above).

    for i in 1..n {
        let mut current = 0;

        // Single digit: s[i] can stand alone if it's 1-9.
        if bytes[i] != b'0' {
            current += dp_prev1;
        }

        // Two digits: s[i-1..=i] can form 10-26.
        let two_digit = (bytes[i - 1] - b'0') * 10 + (bytes[i] - b'0');
        if (10..=26).contains(&two_digit) {
            current += dp_prev2;
        }

        dp_prev2 = dp_prev1;
        dp_prev1 = current;
    }

    dp_prev1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(num_decodings("12"), 2); // "AB" or "L"
    }

    #[test]
    fn test_longer() {
        assert_eq!(num_decodings("226"), 3); // "BZ", "VF", "BBF"
    }

    #[test]
    fn test_leading_zero() {
        assert_eq!(num_decodings("06"), 0);
    }

    #[test]
    fn test_contains_zero() {
        assert_eq!(num_decodings("10"), 1); // Only "J"
        assert_eq!(num_decodings("27"), 1); // Only "BG" (27 > 26)
    }

    #[test]
    fn test_all_ones() {
        assert_eq!(num_decodings("111"), 3);
    }
}
