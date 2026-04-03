// =============================================================================
// Directed Acyclic Graph (DAG)
// =============================================================================
//
// # What is a DAG?
//
// A Directed Acyclic Graph is a directed graph with no cycles. This means
// you can never follow a sequence of directed edges and return to the
// starting vertex.
//
// # Why DAGs?
//
// DAGs are ubiquitous in computer science and blockchain:
//
// - **Blockchain consensus**: IOTA's Tangle and other DAG-based protocols
//   use DAGs instead of linear chains for parallel transaction processing.
// - **Build systems**: Make, Bazel, and Cargo use DAGs for dependency resolution.
// - **Topological sorting**: Any DAG can be topologically sorted, producing
//   a linear ordering where every edge (u, v) has u before v.
// - **Transaction ordering**: In blockchain, transaction dependencies form a
//   DAG that must be topologically sorted for execution.
// - **Data pipelines**: ETL workflows, compiler optimizations (SSA form).
//
// # Complexity Table
//
// | Operation         | Time     |
// |-------------------|----------|
// | Add vertex        | O(1)     |
// | Add edge          | O(V + E) | (cycle detection)
// | Topological sort  | O(V + E) |
// | Cycle detection   | O(V + E) |
// | Space             | O(V + E) |
//
// # Key Property: Topological Order
//
// A topological ordering of a DAG is a linear sequence of all vertices such
// that for every directed edge (u, v), u comes before v. Every DAG has at
// least one topological ordering (and exactly one iff it's a path).
//
// We implement two topological sort algorithms:
// 1. **Kahn's algorithm** (BFS-based): Uses in-degree tracking.
// 2. **DFS-based**: Uses post-order reversal.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// A Directed Acyclic Graph with edge validation.
///
/// Edges are validated on insertion to ensure the graph remains acyclic.
///
/// # Examples
///
/// ```
/// use algo::dsa::graphs::dag::Dag;
///
/// let mut dag = Dag::new();
/// dag.add_edge(0, 1);
/// dag.add_edge(1, 2);
/// assert!(dag.add_edge(2, 0).is_err()); // Would create a cycle!
///
/// let order = dag.topological_sort_kahn().unwrap();
/// assert_eq!(order, vec![0, 1, 2]);
/// ```
#[derive(Debug)]
pub struct Dag {
    /// Adjacency list: vertex → set of successors.
    adjacency: HashMap<usize, HashSet<usize>>,
}

impl Dag {
    pub fn new() -> Self {
        Dag {
            adjacency: HashMap::new(),
        }
    }

    pub fn add_vertex(&mut self, v: usize) {
        self.adjacency.entry(v).or_default();
    }

    pub fn vertex_count(&self) -> usize {
        self.adjacency.len()
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|s| s.len()).sum()
    }

    /// Adds a directed edge, returning `Err` if it would create a cycle.
    ///
    /// We detect cycles by checking if `to` can reach `from` (i.e., there
    /// exists a path from `to` to `from`). If so, adding `from → to` would
    /// complete a cycle.
    pub fn add_edge(&mut self, from: usize, to: usize) -> Result<(), &'static str> {
        self.add_vertex(from);
        self.add_vertex(to);

        // Self-loop is always a cycle.
        if from == to {
            return Err("Self-loop would create a cycle");
        }

        // Check if `to` can reach `from` via existing edges.
        if self.can_reach(to, from) {
            return Err("Edge would create a cycle");
        }

        self.adjacency.get_mut(&from).unwrap().insert(to);
        Ok(())
    }

    /// Returns true if there exists a path from `start` to `target`.
    fn can_reach(&self, start: usize, target: usize) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![start];
        while let Some(v) = stack.pop() {
            if v == target {
                return true;
            }
            if visited.insert(v)
                && let Some(neighbors) = self.adjacency.get(&v) {
                    for &n in neighbors {
                        stack.push(n);
                    }
                }
        }
        false
    }

    /// Returns the successors of a vertex.
    pub fn successors(&self, v: usize) -> Option<&HashSet<usize>> {
        self.adjacency.get(&v)
    }

    // =========================================================================
    // Topological Sort — Kahn's Algorithm (BFS)
    // =========================================================================
    //
    // Kahn's algorithm:
    // 1. Compute in-degree of every vertex.
    // 2. Enqueue all vertices with in-degree 0 (they have no prerequisites).
    // 3. While the queue is not empty:
    //    a. Dequeue vertex u, add to result.
    //    b. For each neighbor v of u, decrement v's in-degree.
    //    c. If v's in-degree reaches 0, enqueue v.
    // 4. If result contains all vertices → valid topological order.
    //    Otherwise → the graph has a cycle (shouldn't happen for a validated DAG).

    /// Topological sort using Kahn's algorithm (BFS-based).
    pub fn topological_sort_kahn(&self) -> Result<Vec<usize>, &'static str> {
        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        for &v in self.adjacency.keys() {
            in_degree.entry(v).or_insert(0);
        }
        for neighbors in self.adjacency.values() {
            for &n in neighbors {
                *in_degree.entry(n).or_insert(0) += 1;
            }
        }

        // Start with all vertices that have no incoming edges.
        let mut queue: VecDeque<usize> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&v, _)| v)
            .collect();
        // Sort for deterministic order in tests.
        let mut sorted_queue: Vec<usize> = queue.drain(..).collect();
        sorted_queue.sort();
        queue.extend(sorted_queue);

        let mut result = Vec::new();
        while let Some(v) = queue.pop_front() {
            result.push(v);
            if let Some(neighbors) = self.adjacency.get(&v) {
                let mut sorted_neighbors: Vec<usize> = neighbors.iter().copied().collect();
                sorted_neighbors.sort();
                for n in sorted_neighbors {
                    let deg = in_degree.get_mut(&n).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(n);
                    }
                }
            }
        }

        if result.len() == self.adjacency.len() {
            Ok(result)
        } else {
            Err("Graph contains a cycle")
        }
    }

    // =========================================================================
    // Topological Sort — DFS-based
    // =========================================================================
    //
    // DFS-based topological sort:
    // 1. For each unvisited vertex, perform DFS.
    // 2. After all descendants of a vertex are processed, push it to a stack.
    // 3. The stack in reverse is the topological order.
    //
    // This works because DFS naturally finds "leaf" tasks first (tasks with
    // no dependencies) and processes them before their dependents.

    /// Topological sort using DFS post-order reversal.
    pub fn topological_sort_dfs(&self) -> Vec<usize> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        // Process vertices in sorted order for determinism.
        let mut vertices: Vec<usize> = self.adjacency.keys().copied().collect();
        vertices.sort();

        for v in vertices {
            if !visited.contains(&v) {
                self.dfs_topo(&v, &mut visited, &mut stack);
            }
        }

        stack.reverse();
        stack
    }

    fn dfs_topo(&self, v: &usize, visited: &mut HashSet<usize>, stack: &mut Vec<usize>) {
        visited.insert(*v);
        if let Some(neighbors) = self.adjacency.get(v) {
            let mut sorted: Vec<usize> = neighbors.iter().copied().collect();
            sorted.sort();
            for n in sorted {
                if !visited.contains(&n) {
                    self.dfs_topo(&n, visited, stack);
                }
            }
        }
        stack.push(*v);
    }

    /// Returns all vertices.
    pub fn vertices(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self.adjacency.keys().copied().collect();
        v.sort();
        v
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Dag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut vertices: Vec<usize> = self.adjacency.keys().copied().collect();
        vertices.sort();
        writeln!(f, "DAG {{")?;
        for v in vertices {
            let mut succs: Vec<usize> = self.adjacency[&v].iter().copied().collect();
            succs.sort();
            let succ_strs: Vec<String> = succs.iter().map(|s| s.to_string()).collect();
            writeln!(f, "  {} -> [{}]", v, succ_strs.join(", "))?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_edges() {
        let mut dag = Dag::new();
        assert!(dag.add_edge(0, 1).is_ok());
        assert!(dag.add_edge(1, 2).is_ok());
        assert!(dag.add_edge(0, 2).is_ok());
        assert_eq!(dag.vertex_count(), 3);
        assert_eq!(dag.edge_count(), 3);
    }

    #[test]
    fn test_cycle_detection() {
        let mut dag = Dag::new();
        dag.add_edge(0, 1).unwrap();
        dag.add_edge(1, 2).unwrap();
        // 2 → 0 would create cycle 0 → 1 → 2 → 0.
        assert!(dag.add_edge(2, 0).is_err());
    }

    #[test]
    fn test_self_loop() {
        let mut dag = Dag::new();
        assert!(dag.add_edge(0, 0).is_err());
    }

    #[test]
    fn test_topological_sort_kahn() {
        let mut dag = Dag::new();
        dag.add_edge(5, 2).unwrap();
        dag.add_edge(5, 0).unwrap();
        dag.add_edge(4, 0).unwrap();
        dag.add_edge(4, 1).unwrap();
        dag.add_edge(2, 3).unwrap();
        dag.add_edge(3, 1).unwrap();

        let order = dag.topological_sort_kahn().unwrap();
        // Verify: for each edge (u, v), u appears before v.
        for (&v, neighbors) in &dag.adjacency {
            let v_pos = order.iter().position(|x| *x == v).unwrap();
            for &n in neighbors {
                let n_pos = order.iter().position(|x| *x == n).unwrap();
                assert!(
                    v_pos < n_pos,
                    "{} should come before {} in topological order",
                    v,
                    n
                );
            }
        }
    }

    #[test]
    fn test_topological_sort_dfs() {
        let mut dag = Dag::new();
        dag.add_edge(0, 1).unwrap();
        dag.add_edge(0, 2).unwrap();
        dag.add_edge(1, 3).unwrap();
        dag.add_edge(2, 3).unwrap();

        let order = dag.topological_sort_dfs();
        // Same verification as above.
        for (&v, neighbors) in &dag.adjacency {
            let v_pos = order.iter().position(|x| *x == v).unwrap();
            for &n in neighbors {
                let n_pos = order.iter().position(|x| *x == n).unwrap();
                assert!(v_pos < n_pos);
            }
        }
    }

    #[test]
    fn test_display() {
        let mut dag = Dag::new();
        dag.add_edge(0, 1).unwrap();
        let s = format!("{}", dag);
        assert!(s.contains("DAG"));
    }
}
