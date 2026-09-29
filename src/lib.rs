//! # Rust Learning and DSA
//!
//! A from-scratch, educational collection of data structures and
//! algorithms implemented in Rust. Every structure is documented, unit-tested,
//! and built with idiomatic Rust patterns: ownership, lifetimes, iterators,
//! `Drop` guards, and careful error handling.
//!
//! ## Module overview
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`linear`] | Dynamic array, stack, queue, singly/doubly linked lists |
//! | [`tree`] | Binary search tree, AVL tree, trie |
//! | [`heap`] | Binary heap and priority queue |
//! | [`map`] | Hash map with separate chaining |
//! | [`graph`] | Adjacency-list graph with BFS/DFS/Dijkstra |
//! | [`union_find`] | Disjoint-set with path compression and union by rank |
//! | [`algorithms`] | Sorting, binary search, edit distance |
//! | [`advanced`] | Rust's advanced topics: Cell/RefCell, Rc/Weak, dyn dispatch, errors, threads, async, FFI, macros |
//!
//! ## Design principles
//!
//! 1. **Correct first.** Every public operation has documented time/space
//!    complexity and unit tests covering edge cases.
//! 2. **Idiomatic Rust.** We implement `Iterator`, `IntoIterator`, `Drop`,
//!    `Debug`, `Default`, `FromIterator`, `Extend`, and `Clone` where it makes
//!    sense.
//! 3. **No hidden panics.** Bounds-checked APIs return `Option` or `Result`;
//!    indexing is provided only through explicit, documented traits.
//! 4. **Production aware.** We avoid recursive `Drop`, document unsafe blocks,
//!    and keep memory layout explicit.

#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)] // We document errors per-method instead.

pub mod advanced;
pub mod algorithms;
pub mod graph;
pub mod heap;
pub mod linear;
pub mod map;
pub mod tree;
pub mod union_find;

/// Re-export the most commonly used types at crate root for convenience.
pub mod prelude {
    pub use crate::advanced::{Rc, RefCell, ThreadPool, block_on};
    pub use crate::algorithms::{heapsort, insertion_sort, mergesort, quicksort};
    pub use crate::graph::Graph;
    pub use crate::heap::{BinaryHeap, PriorityQueue};
    pub use crate::linear::{DoublyLinkedList, DynamicArray, Queue, SinglyLinkedList, Stack};
    pub use crate::map::HashMap;
    pub use crate::tree::{AvlTree, BinarySearchTree, Trie};
    pub use crate::union_find::UnionFind;
}
