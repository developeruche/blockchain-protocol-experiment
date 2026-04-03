// =============================================================================
// LeetCode 1192 — Critical Connections in a Network (Hard)
// =============================================================================
//
// # Problem Summary
// Given n servers and connections between them, find all critical connections
// (bridges) — edges whose removal disconnects the graph.
//
// # Approach — Tarjan's Bridge-Finding Algorithm
// DFS with discovery times (disc) and low-link values (low):
// - disc[u] = when node u was first discovered.
// - low[u] = earliest disc reachable from u's subtree.
//
// An edge (u, v) is a bridge if low[v] > disc[u], meaning v cannot reach
// anything at or above u without using edge (u, v).
//
// # Complexity
// - Time: O(V + E) — single DFS.
// - Space: O(V + E).
//
// Link: https://leetcode.com/problems/critical-connections-in-a-network/

/// Finds all bridges (critical connections) using Tarjan's algorithm.
pub fn critical_connections(n: i32, connections: &[[i32; 2]]) -> Vec<Vec<i32>> {
    let n = n as usize;
    let mut adj = vec![vec![]; n];
    for conn in connections {
        adj[conn[0] as usize].push(conn[1] as usize);
        adj[conn[1] as usize].push(conn[0] as usize);
    }

    let mut disc = vec![-1i32; n];
    let mut low = vec![0i32; n];
    let mut timer = 0;
    let mut bridges = Vec::new();

    tarjan_dfs(0, usize::MAX, &adj, &mut disc, &mut low, &mut timer, &mut bridges);
    bridges
}

fn tarjan_dfs(
    u: usize,
    parent: usize,
    adj: &[Vec<usize>],
    disc: &mut [i32],
    low: &mut [i32],
    timer: &mut i32,
    bridges: &mut Vec<Vec<i32>>,
) {
    disc[u] = *timer;
    low[u] = *timer;
    *timer += 1;

    for &v in &adj[u] {
        if disc[v] == -1 {
            // Tree edge — recurse.
            tarjan_dfs(v, u, adj, disc, low, timer, bridges);
            low[u] = low[u].min(low[v]);

            // Bridge condition: v cannot reach above u.
            if low[v] > disc[u] {
                bridges.push(vec![u as i32, v as i32]);
            }
        } else if v != parent {
            // Back edge — update low.
            low[u] = low[u].min(disc[v]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let connections = vec![[0,1],[1,2],[2,0],[1,3]];
        let mut result = critical_connections(4, &connections);
        for bridge in &mut result {
            bridge.sort();
        }
        result.sort();
        assert_eq!(result, vec![vec![1, 3]]);
    }

    #[test]
    fn test_all_bridges() {
        // Linear graph: 0-1-2-3, every edge is a bridge.
        let connections = vec![[0,1],[1,2],[2,3]];
        let result = critical_connections(4, &connections);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_no_bridges() {
        // Complete graph K3 has no bridges.
        let connections = vec![[0,1],[1,2],[2,0]];
        let result = critical_connections(3, &connections);
        assert!(result.is_empty());
    }
}
