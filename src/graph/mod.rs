//! A weighted, directed graph with adjacency lists and classic algorithms:
//! BFS, DFS, and Dijkstra's shortest paths.
//!
//! Vertices are identified by `usize` ids (0-based); the graph owns its
//! adjacency lists in a `Vec`. Edges are directed and weighted with a generic
//! cost type `W`.
//!
//! # Complexity
//!
//! | Operation | Time |
//! |-----------|------|
//! | `add_vertex` | O(1) amortized |
//! | `add_edge`   | O(1) |
//! | BFS / DFS    | O(V + E) |
//! | Dijkstra     | O((V + E) log V) |
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::graph::Graph;
//!
//! let mut graph = Graph::new();
//! for v in 0..4 {
//!     graph.add_vertex();
//! }
//! graph.add_edge(0, 1, 1);
//! graph.add_edge(1, 2, 1);
//! graph.add_edge(0, 2, 5);
//! assert_eq!(graph.bfs_distances(0)[2], 1);
//! ```

use std::collections::VecDeque;
use std::fmt;

/// A single outgoing edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge<W> {
    /// Destination vertex id.
    pub to: usize,
    /// Edge weight/cost.
    pub weight: W,
}

/// A directed, weighted graph stored as adjacency lists.
#[derive(Default)]
pub struct Graph<W> {
    /// `adj[v]` holds all outgoing edges from vertex `v`.
    adj: Vec<Vec<Edge<W>>>,
}

impl<W> Graph<W> {
    /// Creates an empty graph with no vertices. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self { adj: Vec::new() }
    }

    /// Number of vertices. O(1).
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.adj.len()
    }

    /// Number of edges. O(V + E) — iterates all adjacency lists.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.adj.iter().map(Vec::len).sum()
    }

    /// Adds a new isolated vertex, returning its id. O(1) amortized.
    pub fn add_vertex(&mut self) -> usize {
        self.adj.push(Vec::new());
        self.adj.len() - 1
    }

    /// Adds a directed edge `from -> to` with the given weight. O(1).
    ///
    /// # Panics
    ///
    /// Panics if `from` or `to` is out of bounds.
    pub fn add_edge(&mut self, from: usize, to: usize, weight: W) {
        assert!(from < self.adj.len(), "from vertex out of bounds");
        assert!(to < self.adj.len(), "to vertex out of bounds");
        self.adj[from].push(Edge { to, weight });
    }

    /// Adds an undirected edge (two directed edges). O(1).
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` is out of bounds.
    pub fn add_undirected_edge(&mut self, a: usize, b: usize, weight: W)
    where
        W: Clone,
    {
        self.add_edge(a, b, weight.clone());
        self.add_edge(b, a, weight);
    }

    /// Immutable access to the outgoing edges of `v`. O(1).
    ///
    /// # Panics
    ///
    /// Panics if `v` is out of bounds.
    #[must_use]
    pub fn edges(&self, v: usize) -> &[Edge<W>] {
        &self.adj[v]
    }

    /// Out-degree of vertex `v`. O(1).
    ///
    /// # Panics
    ///
    /// Panics if `v` is out of bounds.
    #[must_use]
    pub fn out_degree(&self, v: usize) -> usize {
        self.adj[v].len()
    }

    /// Breadth-first distances from `start` to every reachable vertex.
    /// Unreachable vertices get `usize::MAX`. O(V + E).
    ///
    /// # Panics
    ///
    /// Panics if `start` is out of bounds.
    #[must_use]
    pub fn bfs_distances(&self, start: usize) -> Vec<usize> {
        let n = self.adj.len();
        assert!(start < n, "start vertex out of bounds");
        let mut dist = vec![usize::MAX; n];
        let mut queue = VecDeque::with_capacity(n);
        dist[start] = 0;
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            let next_dist = dist[v] + 1;
            for edge in &self.adj[v] {
                if dist[edge.to] == usize::MAX {
                    dist[edge.to] = next_dist;
                    queue.push_back(edge.to);
                }
            }
        }
        dist
    }

    /// Depth-first visit order from `start` (iterative). O(V + E).
    ///
    /// # Panics
    ///
    /// Panics if `start` is out of bounds.
    #[must_use]
    pub fn dfs_order(&self, start: usize) -> Vec<usize> {
        let n = self.adj.len();
        assert!(start < n, "start vertex out of bounds");
        let mut visited = vec![false; n];
        let mut order = Vec::with_capacity(n);
        let mut stack = vec![start];
        while let Some(v) = stack.pop() {
            if visited[v] {
                continue;
            }
            visited[v] = true;
            order.push(v);
            for edge in &self.adj[v] {
                if !visited[edge.to] {
                    stack.push(edge.to);
                }
            }
        }
        order
    }

    /// Shortest distances from `start` using Dijkstra's algorithm.
    /// Unreachable vertices get `usize::MAX`. O((V + E) log V).
    ///
    /// # Panics
    ///
    /// Panics if `start` is out of bounds.
    #[must_use]
    pub fn dijkstra_distances(&self, start: usize) -> Vec<usize>
    where
        W: Into<usize> + Copy,
    {
        let n = self.adj.len();
        assert!(start < n, "start vertex out of bounds");
        let mut dist = vec![usize::MAX; n];
        // Min-heap via std: reverse for max-heap semantics.
        let mut heap = std::collections::BinaryHeap::new();
        dist[start] = 0;
        heap.push(std::cmp::Reverse((0usize, start)));
        while let Some(std::cmp::Reverse((d, v))) = heap.pop() {
            if d > dist[v] {
                continue; // stale entry
            }
            for edge in &self.adj[v] {
                let Some(nd) = d.checked_add(edge.weight.into()) else {
                    continue; // no representable path through this edge
                };
                if nd < dist[edge.to] {
                    dist[edge.to] = nd;
                    heap.push(std::cmp::Reverse((nd, edge.to)));
                }
            }
        }
        dist
    }

    /// Returns a minimum-cost route and its cost, or `None` if unreachable.
    /// A path whose cost reaches or exceeds `usize::MAX` is not representable.
    /// O((V + E) log V) time and O(V + E) space.
    ///
    /// # Panics
    ///
    /// Panics if `start` or `goal` is out of bounds.
    #[must_use]
    pub fn dijkstra_path(&self, start: usize, goal: usize) -> Option<(usize, Vec<usize>)>
    where
        W: Into<usize> + Copy,
    {
        assert!(
            start < self.adj.len() && goal < self.adj.len(),
            "vertex out of bounds"
        );
        let mut dist = vec![usize::MAX; self.adj.len()];
        let mut previous = vec![None; self.adj.len()];
        let mut heap = std::collections::BinaryHeap::new();
        dist[start] = 0;
        heap.push(std::cmp::Reverse((0usize, start)));
        while let Some(std::cmp::Reverse((cost, vertex))) = heap.pop() {
            if cost != dist[vertex] {
                continue;
            }
            if vertex == goal {
                let mut path = vec![goal];
                let mut current = goal;
                while let Some(parent) = previous[current] {
                    path.push(parent);
                    current = parent;
                }
                path.reverse();
                return Some((cost, path));
            }
            for edge in &self.adj[vertex] {
                if let Some(next) = cost.checked_add(edge.weight.into())
                    && next < dist[edge.to]
                {
                    dist[edge.to] = next;
                    previous[edge.to] = Some(vertex);
                    heap.push(std::cmp::Reverse((next, edge.to)));
                }
            }
        }
        None
    }
}

impl<W: fmt::Debug> fmt::Debug for Graph<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Graph")
            .field("vertices", &self.adj.len())
            .field("adj", &self.adj)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_graph() -> Graph<usize> {
        let mut g: Graph<usize> = Graph::new();
        for _ in 0..6 {
            g.add_vertex();
        }
        // 0 - 1 - 2
        // |       |
        // 3 - 4 - 5
        g.add_undirected_edge(0, 1, 1);
        g.add_undirected_edge(1, 2, 1);
        g.add_undirected_edge(0, 3, 1);
        g.add_undirected_edge(3, 4, 1);
        g.add_undirected_edge(4, 5, 1);
        g.add_undirected_edge(2, 5, 1);
        g
    }

    #[test]
    fn counts() {
        let g = small_graph();
        assert_eq!(g.vertex_count(), 6);
        assert_eq!(g.edge_count(), 12); // 6 undirected = 12 directed
    }

    #[test]
    fn bfs_distances() {
        let g = small_graph();
        let dist = g.bfs_distances(0);
        assert_eq!(dist, vec![0, 1, 2, 1, 2, 3]);
    }

    #[test]
    fn bfs_unreachable() {
        let mut g: Graph<usize> = Graph::new();
        g.add_vertex();
        g.add_vertex();
        // No edges: vertex 1 unreachable from 0.
        let dist = g.bfs_distances(0);
        assert_eq!(dist[1], usize::MAX);
    }

    #[test]
    fn dfs_visits_all_reachable() {
        let g = small_graph();
        let order = g.dfs_order(0);
        assert_eq!(order.len(), 6);
        // All vertices present exactly once.
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn dijkstra_weighted() {
        let mut g: Graph<usize> = Graph::new();
        for _ in 0..4 {
            g.add_vertex();
        }
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 1);
        g.add_edge(0, 2, 10); // expensive direct route
        g.add_edge(2, 3, 1);
        let dist = g.dijkstra_distances(0);
        assert_eq!(dist[2], 2); // via 1, not the direct 10
        assert_eq!(dist[3], 3);
        assert_eq!(dist[0], 0);
        assert_eq!(g.dijkstra_path(0, 3), Some((3, vec![0, 1, 2, 3])));
    }

    #[test]
    fn dijkstra_unreachable() {
        let mut g: Graph<usize> = Graph::new();
        g.add_vertex();
        g.add_vertex();
        let dist = g.dijkstra_distances(0);
        assert_eq!(dist[1], usize::MAX);
        assert_eq!(g.dijkstra_path(0, 1), None);
    }

    #[test]
    fn dijkstra_skips_overflowing_paths() {
        let mut g = Graph::<usize>::new();
        for _ in 0..3 {
            g.add_vertex();
        }
        g.add_edge(0, 1, usize::MAX - 1);
        g.add_edge(1, 2, 3);
        g.add_edge(0, 2, 7);
        assert_eq!(g.dijkstra_path(0, 2), Some((7, vec![0, 2])));
        assert_eq!(g.dijkstra_distances(0)[2], 7);
    }

    #[test]
    fn out_degree() {
        let mut g: Graph<usize> = Graph::new();
        for _ in 0..3 {
            g.add_vertex();
        }
        g.add_edge(0, 1, 1);
        g.add_edge(0, 2, 1);
        assert_eq!(g.out_degree(0), 2);
        assert_eq!(g.out_degree(1), 0);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn add_edge_panics_on_bad_vertex() {
        let mut g: Graph<usize> = Graph::new();
        g.add_vertex();
        g.add_edge(0, 5, 1);
    }
}
