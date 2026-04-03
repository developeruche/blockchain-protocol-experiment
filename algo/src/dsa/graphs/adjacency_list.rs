// =============================================================================
// Adjacency List — Graph Representation
// =============================================================================
//
// # What is an Adjacency List?
//
// An adjacency list represents a graph as a collection of lists, one for each
// vertex, where each list contains the vertex's neighbors. This is the most
// common graph representation for sparse graphs.
//
// # Comparison with Adjacency Matrix
//
// | Feature           | Adjacency List        | Adjacency Matrix    |
// |-------------------|-----------------------|---------------------|
// | Space             | O(V + E)              | O(V²)               |
// | Add edge          | O(1)                  | O(1)                |
// | Remove edge       | O(E/V) avg            | O(1)                |
// | Check edge exists | O(degree) or O(E/V)   | O(1)                |
// | Iterate neighbors | O(degree)             | O(V)                |
// | Best for          | Sparse graphs         | Dense graphs        |
//
// Most real-world graphs (social networks, blockchain peer networks, call
// graphs) are sparse, so adjacency lists are the default choice.
//
// # Blockchain Analogies
//
// - **Peer-to-peer network**: Nodes are peers, edges are connections.
// - **Transaction graph**: Nodes are addresses, edges are transfers.
// - **State dependency graph**: Variables that depend on each other.
//
// # Implementation
//
// We use `HashMap<usize, Vec<(usize, W)>>` for a weighted, directed graph.
// Each vertex ID maps to a list of (neighbor_id, weight) pairs.
// For an unweighted graph, set W = () or use `add_edge` with weight 1.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::hash::Hash;

/// A weighted, directed graph using adjacency list representation.
///
/// Vertices are identified by type `V` (typically `usize` or `String`).
/// Edge weights are of type `W`.
///
/// # Examples
///
/// ```
/// use algo::dsa::graphs::adjacency_list::Graph;
///
/// let mut g: Graph<usize, i32> = Graph::new();
/// g.add_vertex(0);
/// g.add_vertex(1);
/// g.add_edge(0, 1, 10);
/// assert!(g.has_edge(&0, &1));
/// ```
#[derive(Debug)]
pub struct Graph<V: Eq + Hash + Clone, W: Clone> {
    /// Maps each vertex to its list of (neighbor, weight) pairs.
    adjacency: HashMap<V, Vec<(V, W)>>,
}

impl<V: Eq + Hash + Clone, W: Clone> Graph<V, W> {
    /// Creates a new, empty graph.
    pub fn new() -> Self {
        Graph {
            adjacency: HashMap::new(),
        }
    }

    /// Adds a vertex. If it already exists, this is a no-op.
    pub fn add_vertex(&mut self, v: V) {
        self.adjacency.entry(v).or_default();
    }

    /// Returns the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Returns the number of directed edges.
    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|edges| edges.len()).sum()
    }

    /// Adds a directed edge from `from` to `to` with the given weight.
    /// Creates the vertices if they don't exist.
    pub fn add_edge(&mut self, from: V, to: V, weight: W) {
        self.adjacency.entry(to.clone()).or_default();
        self.adjacency.entry(from.clone()).or_default().push((to, weight));
    }

    /// Adds an undirected edge (two directed edges).
    pub fn add_undirected_edge(&mut self, a: V, b: V, weight: W) {
        self.add_edge(a.clone(), b.clone(), weight.clone());
        self.add_edge(b, a, weight);
    }

    /// Checks if a directed edge from `from` to `to` exists.
    pub fn has_edge(&self, from: &V, to: &V) -> bool {
        self.adjacency
            .get(from)
            .is_some_and(|edges| edges.iter().any(|(n, _)| n == to))
    }

    /// Returns the neighbors and weights for a vertex.
    pub fn neighbors(&self, v: &V) -> Option<&Vec<(V, W)>> {
        self.adjacency.get(v)
    }

    /// Returns all vertices.
    pub fn vertices(&self) -> Vec<&V> {
        self.adjacency.keys().collect()
    }

    /// Removes a directed edge from `from` to `to`.
    pub fn remove_edge(&mut self, from: &V, to: &V) {
        if let Some(edges) = self.adjacency.get_mut(from) {
            edges.retain(|(n, _)| n != to);
        }
    }

    /// Removes a vertex and all its incoming/outgoing edges.
    pub fn remove_vertex(&mut self, v: &V) {
        self.adjacency.remove(v);
        // Remove all edges pointing to v.
        for edges in self.adjacency.values_mut() {
            edges.retain(|(n, _)| n != v);
        }
    }
}

/// BFS and DFS on the generic graph. These require `V: Eq + Hash + Clone`.
impl<V: Eq + Hash + Clone, W: Clone> Graph<V, W> {
    /// Breadth-first search from a source vertex.
    /// Returns vertices in BFS order.
    pub fn bfs(&self, start: &V) -> Vec<V> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        visited.insert(start.clone());
        queue.push_back(start.clone());

        while let Some(v) = queue.pop_front() {
            result.push(v.clone());
            if let Some(neighbors) = self.adjacency.get(&v) {
                for (neighbor, _) in neighbors {
                    if visited.insert(neighbor.clone()) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        result
    }

    /// Depth-first search from a source vertex.
    /// Returns vertices in DFS order (iterative, using a stack).
    pub fn dfs(&self, start: &V) -> Vec<V> {
        let mut visited = HashSet::new();
        let mut stack = vec![start.clone()];
        let mut result = Vec::new();

        while let Some(v) = stack.pop() {
            if !visited.insert(v.clone()) {
                continue;
            }
            result.push(v.clone());
            if let Some(neighbors) = self.adjacency.get(&v) {
                // Push neighbors in reverse order so we visit them in the
                // "natural" order (leftmost neighbor first).
                for (neighbor, _) in neighbors.iter().rev() {
                    if !visited.contains(neighbor) {
                        stack.push(neighbor.clone());
                    }
                }
            }
        }
        result
    }
}

impl<V: Eq + Hash + Clone, W: Clone> Default for Graph<V, W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Eq + Hash + Clone + fmt::Display + Ord, W: Clone + fmt::Display> fmt::Display
    for Graph<V, W>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut vertices: Vec<&V> = self.adjacency.keys().collect();
        vertices.sort();
        writeln!(f, "Graph {{")?;
        for v in vertices {
            let edges = &self.adjacency[v];
            let edge_strs: Vec<String> = edges
                .iter()
                .map(|(n, w)| format!("{}({})", n, w))
                .collect();
            writeln!(f, "  {} -> [{}]", v, edge_strs.join(", "))?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_query() {
        let mut g: Graph<usize, i32> = Graph::new();
        g.add_edge(0, 1, 5);
        g.add_edge(0, 2, 3);
        g.add_edge(1, 2, 1);

        assert!(g.has_edge(&0, &1));
        assert!(g.has_edge(&0, &2));
        assert!(!g.has_edge(&1, &0)); // Directed graph.
        assert_eq!(g.vertex_count(), 3);
        assert_eq!(g.edge_count(), 3);
    }

    #[test]
    fn test_undirected_edge() {
        let mut g: Graph<usize, i32> = Graph::new();
        g.add_undirected_edge(0, 1, 10);
        assert!(g.has_edge(&0, &1));
        assert!(g.has_edge(&1, &0));
    }

    #[test]
    fn test_remove() {
        let mut g: Graph<usize, i32> = Graph::new();
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 1);
        g.remove_edge(&0, &1);
        assert!(!g.has_edge(&0, &1));
        assert!(g.has_edge(&1, &2));

        g.remove_vertex(&2);
        assert!(!g.has_edge(&1, &2));
    }

    #[test]
    fn test_bfs() {
        let mut g: Graph<usize, ()> = Graph::new();
        g.add_edge(0, 1, ());
        g.add_edge(0, 2, ());
        g.add_edge(1, 3, ());
        g.add_edge(2, 3, ());
        let order = g.bfs(&0);
        assert_eq!(order[0], 0);
        assert!(order.contains(&1));
        assert!(order.contains(&2));
        assert!(order.contains(&3));
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn test_dfs() {
        let mut g: Graph<usize, ()> = Graph::new();
        g.add_edge(0, 1, ());
        g.add_edge(0, 2, ());
        g.add_edge(1, 3, ());
        g.add_edge(2, 3, ());
        let order = g.dfs(&0);
        assert_eq!(order[0], 0);
        assert!(order.contains(&1));
        assert!(order.contains(&2));
        assert!(order.contains(&3));
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn test_display() {
        let mut g: Graph<usize, i32> = Graph::new();
        g.add_edge(0, 1, 5);
        let s = format!("{}", g);
        assert!(s.contains("Graph"));
    }
}
