// =============================================================================
// LeetCode 332 — Reconstruct Itinerary (Hard)
// =============================================================================
//
// # Problem Summary
// Given a list of airline tickets [from, to], reconstruct the itinerary in
// lexicographic order starting from "JFK". All tickets must be used exactly once.
//
// # Approach — Hierholzer's Algorithm for Eulerian Path
// An Eulerian path visits every EDGE exactly once. Hierholzer's algorithm:
// 1. Start at "JFK".
// 2. Follow edges (removing them) until stuck.
// 3. Backtrack and prepend nodes to the result.
//
// We sort destinations to ensure lexicographic order.
//
// # Complexity
// - Time: O(E log E) for sorting edges.
// - Space: O(E).
//
// Link: https://leetcode.com/problems/reconstruct-itinerary/

use std::collections::HashMap;

/// Reconstructs the itinerary using Hierholzer's algorithm.
pub fn find_itinerary(tickets: &[[String; 2]]) -> Vec<String> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for ticket in tickets {
        graph.entry(ticket[0].as_str()).or_default().push(ticket[1].as_str());
    }
    // Sort in reverse so we can pop the smallest efficiently.
    for dests in graph.values_mut() {
        dests.sort_unstable_by(|a, b| b.cmp(a));
    }

    let mut result = Vec::new();
    let mut stack = vec!["JFK"];

    while let Some(&airport) = stack.last() {
        if graph.get(airport).is_some_and(|d| !d.is_empty()) {
            let next = graph.get_mut(airport).unwrap().pop().unwrap();
            stack.push(next);
        } else {
            result.push(stack.pop().unwrap().to_string());
        }
    }

    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let tickets = vec![
            ["MUC".to_string(), "LHR".to_string()],
            ["JFK".to_string(), "MUC".to_string()],
            ["SFO".to_string(), "SJC".to_string()],
            ["LHR".to_string(), "SFO".to_string()],
        ];
        assert_eq!(
            find_itinerary(&tickets),
            vec!["JFK", "MUC", "LHR", "SFO", "SJC"]
        );
    }

    #[test]
    fn test_lexicographic() {
        let tickets = vec![
            ["JFK".to_string(), "SFO".to_string()],
            ["JFK".to_string(), "ATL".to_string()],
            ["SFO".to_string(), "ATL".to_string()],
            ["ATL".to_string(), "JFK".to_string()],
            ["ATL".to_string(), "SFO".to_string()],
        ];
        assert_eq!(
            find_itinerary(&tickets),
            vec!["JFK", "ATL", "JFK", "SFO", "ATL", "SFO"]
        );
    }

    #[test]
    fn test_single_ticket() {
        let tickets = vec![["JFK".to_string(), "LAX".to_string()]];
        assert_eq!(find_itinerary(&tickets), vec!["JFK", "LAX"]);
    }
}
