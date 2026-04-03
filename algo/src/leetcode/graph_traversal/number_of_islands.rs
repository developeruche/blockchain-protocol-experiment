// =============================================================================
// LeetCode 200 — Number of Islands (Medium)
// =============================================================================
//
// # Problem Summary
// Given a 2D grid of '1's (land) and '0's (water), count the number of islands.
// An island is surrounded by water and formed by connecting adjacent lands
// horizontally or vertically.
//
// # Approach
// For each unvisited '1', start a BFS/DFS to mark all connected land cells.
// Each BFS/DFS invocation discovers one island.
//
// # Complexity
// - Time: O(m × n).
// - Space: O(m × n) for the visited set (or in-place modification).
//
// Link: https://leetcode.com/problems/number-of-islands/

/// Counts the number of islands using DFS.
pub fn num_islands(grid: &[Vec<char>]) -> i32 {
    if grid.is_empty() { return 0; }
    let rows = grid.len();
    let cols = grid[0].len();
    let mut visited = vec![vec![false; cols]; rows];
    let mut count = 0;

    for r in 0..rows {
        for c in 0..cols {
            if grid[r][c] == '1' && !visited[r][c] {
                dfs_mark(grid, &mut visited, r, c);
                count += 1;
            }
        }
    }
    count
}

fn dfs_mark(grid: &[Vec<char>], visited: &mut [Vec<bool>], r: usize, c: usize) {
    if r >= grid.len() || c >= grid[0].len() || visited[r][c] || grid[r][c] == '0' {
        return;
    }
    visited[r][c] = true;
    if r > 0 { dfs_mark(grid, visited, r - 1, c); }
    dfs_mark(grid, visited, r + 1, c);
    if c > 0 { dfs_mark(grid, visited, r, c - 1); }
    dfs_mark(grid, visited, r, c + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let grid = vec![
            vec!['1','1','1','1','0'],
            vec!['1','1','0','1','0'],
            vec!['1','1','0','0','0'],
            vec!['0','0','0','0','0'],
        ];
        assert_eq!(num_islands(&grid), 1);
    }

    #[test]
    fn test_multiple_islands() {
        let grid = vec![
            vec!['1','1','0','0','0'],
            vec!['1','1','0','0','0'],
            vec!['0','0','1','0','0'],
            vec!['0','0','0','1','1'],
        ];
        assert_eq!(num_islands(&grid), 3);
    }

    #[test]
    fn test_empty() {
        assert_eq!(num_islands(&[]), 0);
    }
}
