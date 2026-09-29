//! A disjoint-set union (Union-Find) structure.
//!
//! Tracks a partition of `0..n` elements into disjoint sets. Two optimizations
//! make the amortized complexity nearly constant:
//!
//! - **Path compression**: every `find` flattens the path to the root.
//! - **Union by rank**: the shorter tree is attached under the taller one.
//!
//! With both, `m` operations on `n` elements run in O(m α(n)) — α is the
//! inverse Ackermann function, effectively O(1) for any practical input.
//!
//! # Complexity
//!
//! | Operation | Amortized |
//! |-----------|-----------|
//! | `find`    | O(α(n)) |
//! | `union`   | O(α(n)) |
//! | `connected` | O(α(n)) |
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::union_find::UnionFind;
//!
//! let mut uf = UnionFind::new(5);
//! uf.union(0, 1);
//! uf.union(1, 2);
//! assert!(uf.connected(0, 2));
//! assert!(!uf.connected(0, 3));
//! assert_eq!(uf.components(), 3);
//! ```

use std::fmt;

/// A disjoint-set data structure over `0..n` elements.
pub struct UnionFind {
    /// `parent[x]` points to the parent of `x` (its own root if representative).
    parent: Vec<usize>,
    /// `rank[x]` is an upper bound on the height of the tree rooted at `x`.
    rank: Vec<u8>,
    /// Component size is meaningful only at representative indices.
    size: Vec<usize>,
    /// Number of disjoint components.
    components: usize,
}

impl UnionFind {
    /// Creates a structure with `n` singleton sets. O(n).
    ///
    /// # Panics
    ///
    /// Panics if `n` exceeds `isize::MAX` (addressable memory limit).
    #[must_use]
    pub fn new(n: usize) -> Self {
        assert!(isize::try_from(n).is_ok(), "n too large");
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            size: vec![1; n],
            components: n,
        }
    }

    /// Returns the representative (root) of the set containing `x`, applying
    /// path compression. O(α(n)).
    ///
    /// # Panics
    ///
    /// Panics if `x` is out of bounds.
    #[must_use]
    pub fn find(&mut self, x: usize) -> usize {
        self.find_with_path_compression(x)
    }

    /// Merges the sets containing `a` and `b`. Returns `true` if they were
    /// previously disjoint (i.e. a real merge happened). O(α(n)).
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` is out of bounds.
    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let root_a = self.find_with_path_compression(a);
        let root_b = self.find_with_path_compression(b);
        if root_a == root_b {
            return false; // already in the same set
        }
        // Union by rank: attach the shorter tree under the taller one.
        match self.rank[root_a].cmp(&self.rank[root_b]) {
            std::cmp::Ordering::Less => {
                self.parent[root_a] = root_b;
                self.size[root_b] += self.size[root_a];
            }
            std::cmp::Ordering::Greater => {
                self.parent[root_b] = root_a;
                self.size[root_a] += self.size[root_b];
            }
            std::cmp::Ordering::Equal => {
                self.parent[root_b] = root_a;
                self.rank[root_a] += 1;
                self.size[root_a] += self.size[root_b];
            }
        }
        self.components -= 1;
        true
    }

    /// Returns `true` if `a` and `b` are in the same set. O(α(n)).
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` is out of bounds.
    #[must_use]
    pub fn connected(&mut self, a: usize, b: usize) -> bool {
        self.find_with_path_compression(a) == self.find_with_path_compression(b)
    }

    /// Number of elements tracked. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.parent.len()
    }

    /// `true` if no elements are tracked. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    /// Number of disjoint components. O(1).
    #[must_use]
    pub const fn components(&self) -> usize {
        self.components
    }

    /// Size of the set containing `x`. O(α(n)).
    ///
    /// # Panics
    ///
    /// Panics if `x` is out of bounds.
    #[must_use]
    pub fn set_size(&mut self, x: usize) -> usize {
        let root = self.find_with_path_compression(x);
        self.size[root]
    }

    /// Finds the root of `x` with path compression (iterative, two-pass).
    fn find_with_path_compression(&mut self, x: usize) -> usize {
        assert!(x < self.parent.len(), "find index out of bounds");
        // Pass 1: climb to the root.
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        // Pass 2: flatten the path so all nodes point directly at the root.
        let mut cur = x;
        while self.parent[cur] != cur {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }
}

impl fmt::Debug for UnionFind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionFind")
            .field("elements", &self.parent.len())
            .field("components", &self.components)
            // The internal parent/rank arrays are implementation details.
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let uf = UnionFind::new(5);
        assert_eq!(uf.len(), 5);
        assert_eq!(uf.components(), 5);
        assert!(!uf.is_empty());
    }

    #[test]
    fn union_and_connected() {
        let mut uf = UnionFind::new(5);
        assert!(uf.union(0, 1));
        assert!(uf.union(1, 2));
        assert!(uf.connected(0, 2));
        assert!(uf.connected(1, 2));
        assert!(!uf.connected(0, 3));
        assert_eq!(uf.components(), 3);
    }

    #[test]
    fn union_returns_false_when_already_merged() {
        let mut uf = UnionFind::new(3);
        assert!(uf.union(0, 1));
        assert!(!uf.union(0, 1));
        assert!(!uf.union(1, 0));
        assert_eq!(uf.components(), 2);
    }

    #[test]
    fn cycle_detection() {
        let mut uf = UnionFind::new(6);
        let edges = [(0, 1), (1, 2), (3, 4), (4, 5)];
        for (a, b) in edges {
            assert!(uf.union(a, b));
        }
        // Edge (2, 3) connects two components — still a tree.
        assert!(uf.union(2, 3));
        // Edge (0, 5) now closes a cycle.
        assert!(!uf.union(0, 5));
        assert_eq!(uf.components(), 1);
    }

    #[test]
    fn path_compression_flattens() {
        // Build a chain 0 -> 1 -> 2 -> 3 (root 3), then find(0).
        let mut uf = UnionFind::new(4);
        // Manually create the chain to verify compression behavior.
        uf.union(0, 1);
        uf.union(1, 2);
        uf.union(2, 3);
        // find(0) must resolve to the same root as find(3).
        assert_eq!(uf.find(0), uf.find(3));
        // After path compression, parent[0] points directly at the root.
        let root = uf.find(0);
        // Compression is idempotent: a second find returns the same root.
        assert_eq!(uf.find(0), root);
        assert_eq!(uf.parent[0], root);
    }

    #[test]
    fn set_size() {
        let mut uf = UnionFind::new(7);
        uf.union(0, 1);
        uf.union(1, 2);
        uf.union(5, 6);
        assert_eq!(uf.set_size(0), 3);
        assert_eq!(uf.set_size(5), 2);
        assert_eq!(uf.set_size(3), 1);
    }

    #[test]
    fn kruskal_minimum_spanning_tree() {
        // Classic usage: Kruskal's MST on a small weighted graph.
        let mut edges = vec![
            (1, 0, 1),
            (2, 0, 2),
            (3, 1, 3),
            (4, 2, 4),
            (5, 3, 5),
            (6, 4, 5),
            (7, 2, 5),
        ];
        edges.sort_unstable_by_key(|&(w, _, _)| w);
        let mut uf = UnionFind::new(6);
        let mut mst_weight = 0;
        let mut mst_edges = 0;
        for (w, a, b) in edges {
            if uf.union(a, b) {
                mst_weight += w;
                mst_edges += 1;
            }
        }
        assert_eq!(mst_edges, 5); // V - 1 edges in the MST
        assert_eq!(mst_weight, 1 + 2 + 3 + 4 + 5);
    }

    #[test]
    fn many_operations_fast() {
        // 100k unions + finds should complete quickly thanks to the two
        // optimizations (this is effectively the amortized-constant claim).
        let mut uf = UnionFind::new(100_000);
        for i in 0..99_999 {
            uf.union(i, i + 1);
        }
        assert_eq!(uf.components(), 1);
        assert!(uf.connected(0, 99_999));
        for i in 0..50_000 {
            assert!(uf.connected(i, i + 50_000));
        }
    }
}
