//! Algorithm implementations on top of the data structures.

pub mod dynamic_programming;
pub mod searching;
pub mod sorting;

pub use dynamic_programming::edit_distance;
pub use searching::{binary_search_first, lower_bound};
pub use sorting::{bubble_sort, heapsort, insertion_sort, mergesort, quicksort, selection_sort};
