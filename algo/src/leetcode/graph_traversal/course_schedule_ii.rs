// =============================================================================
// LeetCode 210 — Course Schedule II (Medium)
// =============================================================================
//
// # Problem Summary
// Given n courses labeled 0..n-1 and prerequisite pairs, return a valid order
// to take all courses (topological sort). Return empty if impossible (cycle).
//
// # Approach
// Classic DAG topological sort. We implement BOTH Kahn's (BFS) and DFS:
// - **Kahn's**: Track in-degrees, process zero-in-degree nodes first.
// - **DFS**: Detect cycles with coloring (white/gray/black).
//
// # Complexity
// - Time: O(V + E).
// - Space: O(V + E).
//
// Link: https://leetcode.com/problems/course-schedule-ii/

use std::collections::VecDeque;

/// Returns a valid course ordering using Kahn's algorithm (BFS).
pub fn find_order_kahn(num_courses: i32, prerequisites: &[[i32; 2]]) -> Vec<i32> {
    let n = num_courses as usize;
    let mut adj = vec![vec![]; n];
    let mut in_degree = vec![0; n];

    for pre in prerequisites {
        let (course, prereq) = (pre[0] as usize, pre[1] as usize);
        adj[prereq].push(course);
        in_degree[course] += 1;
    }

    let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut result = Vec::with_capacity(n);

    while let Some(node) = queue.pop_front() {
        result.push(node as i32);
        for &next in &adj[node] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                queue.push_back(next);
            }
        }
    }

    if result.len() == n { result } else { vec![] }
}

/// Returns a valid course ordering using DFS.
pub fn find_order_dfs(num_courses: i32, prerequisites: &[[i32; 2]]) -> Vec<i32> {
    let n = num_courses as usize;
    let mut adj = vec![vec![]; n];
    for pre in prerequisites {
        adj[pre[1] as usize].push(pre[0] as usize);
    }

    // 0 = white (unvisited), 1 = gray (in progress), 2 = black (done).
    let mut color = vec![0u8; n];
    let mut stack = Vec::with_capacity(n);

    for i in 0..n {
        if color[i] == 0 && !dfs_visit(i, &adj, &mut color, &mut stack) {
            return vec![]; // Cycle detected.
        }
    }

    stack.reverse();
    stack.iter().map(|&x| x as i32).collect()
}

fn dfs_visit(
    node: usize,
    adj: &[Vec<usize>],
    color: &mut [u8],
    stack: &mut Vec<usize>,
) -> bool {
    color[node] = 1; // Gray — currently being processed.
    for &next in &adj[node] {
        if color[next] == 1 {
            return false; // Back edge = cycle.
        }
        if color[next] == 0 && !dfs_visit(next, adj, color, stack) {
            return false;
        }
    }
    color[node] = 2; // Black — fully processed.
    stack.push(node);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_order(n: i32, prereqs: &[[i32; 2]], order: &[i32]) -> bool {
        if order.len() != n as usize { return false; }
        let mut pos = vec![0usize; n as usize];
        for (i, &c) in order.iter().enumerate() {
            pos[c as usize] = i;
        }
        prereqs.iter().all(|p| pos[p[1] as usize] < pos[p[0] as usize])
    }

    #[test]
    fn test_kahn_basic() {
        let order = find_order_kahn(4, &[[1,0],[2,0],[3,1],[3,2]]);
        assert!(valid_order(4, &[[1,0],[2,0],[3,1],[3,2]], &order));
    }

    #[test]
    fn test_dfs_basic() {
        let order = find_order_dfs(4, &[[1,0],[2,0],[3,1],[3,2]]);
        assert!(valid_order(4, &[[1,0],[2,0],[3,1],[3,2]], &order));
    }

    #[test]
    fn test_cycle() {
        assert!(find_order_kahn(2, &[[0,1],[1,0]]).is_empty());
        assert!(find_order_dfs(2, &[[0,1],[1,0]]).is_empty());
    }
}
