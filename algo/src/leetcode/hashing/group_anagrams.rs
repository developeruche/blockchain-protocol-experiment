// =============================================================================
// LeetCode 49 — Group Anagrams (Medium)
// =============================================================================
//
// # Problem Summary
// Given an array of strings, group the anagrams together. An anagram is a word
// formed by rearranging the letters of another.
//
// # Approach / Intuition
// **Key normalization**: For each string, compute a canonical key that is the
// same for all anagrams. Two approaches:
// 1. Sort the characters → sorted string is the key.
// 2. Count character frequencies → frequency array is the key.
//
// We use approach 1 (sorting) for simplicity. Approach 2 is O(n) per string
// but the constant is larger for short strings.
//
// **Blockchain analogy**: This is like canonical key encoding — different
// representations of the same data (e.g., RLP vs SSZ) must map to the same
// canonical form for comparison.
//
// # Complexity
// - Time: O(n · k log k) where n = number of strings, k = max string length.
// - Space: O(n · k) for the output.
//
// Link: https://leetcode.com/problems/group-anagrams/

use std::collections::HashMap;

/// Groups anagrams from the input strings.
pub fn group_anagrams(strs: &[String]) -> Vec<Vec<String>> {
    let mut map: HashMap<Vec<u8>, Vec<String>> = HashMap::new();

    for s in strs {
        // Sort the characters to create a canonical key.
        let mut key: Vec<u8> = s.bytes().collect();
        key.sort();
        map.entry(key).or_default().push(s.clone());
    }

    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_groups(groups: Vec<Vec<String>>) -> Vec<Vec<String>> {
        let mut g = groups;
        for group in &mut g {
            group.sort();
        }
        g.sort();
        g
    }

    #[test]
    fn test_basic() {
        let input: Vec<String> = vec!["eat", "tea", "tan", "ate", "nat", "bat"]
            .into_iter()
            .map(String::from)
            .collect();
        let result = sorted_groups(group_anagrams(&input));
        let expected = sorted_groups(vec![
            vec!["eat".into(), "tea".into(), "ate".into()],
            vec!["tan".into(), "nat".into()],
            vec!["bat".into()],
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_string() {
        let input = vec!["".to_string()];
        let result = group_anagrams(&input);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_single() {
        let input = vec!["a".to_string()];
        let result = group_anagrams(&input);
        assert_eq!(result, vec![vec!["a".to_string()]]);
    }
}
