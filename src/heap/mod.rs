//! A binary heap and a priority queue built on top of it.
//!
//! The heap is stored in a contiguous array with the usual indexing scheme:
//! for node at index `i`, children are at `2i + 1` and `2i + 2`, and the parent
//! is at `(i - 1) / 2`. `push` sifts up; `pop` swaps the root with the last
//! element and sifts down.
//!
//! # Complexity
//!
//! | Operation | Time |
//! |-----------|------|
//! | `push`    | O(log n) |
//! | `pop`     | O(log n) |
//! | `peek`    | O(1) |
//! | `len`     | O(1) |
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::heap::BinaryHeap;
//!
//! let mut heap = BinaryHeap::new();
//! heap.push(5);
//! heap.push(1);
//! heap.push(9);
//! assert_eq!(heap.peek(), Some(&9)); // max-heap by default
//! assert_eq!(heap.pop(), Some(9));
//! ```

/// A max-heap: the largest value is always at the front.
#[derive(Clone, Default)]
pub struct BinaryHeap<T> {
    items: Vec<T>,
}

impl<T: Ord> BinaryHeap<T> {
    /// Creates an empty heap. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Creates a heap from a slice in O(n) via Floyd's build-heap. O(n).
    #[must_use]
    pub fn from_slice(slice: &[T]) -> Self
    where
        T: Clone,
    {
        let mut heap = Self {
            items: slice.to_vec(),
        };
        heap.heapify();
        heap
    }

    /// Number of elements. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    /// `true` if the heap is empty. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Inserts `value`. O(log n).
    pub fn push(&mut self, value: T) {
        self.items.push(value);
        self.sift_up(self.items.len() - 1);
    }

    /// Removes and returns the largest value, or `None` if empty. O(log n).
    ///
    /// # Panics
    ///
    /// Never in practice: the `expect` below is guarded by the `len() >= 2`
    /// match arm and is only defensive.
    pub fn pop(&mut self) -> Option<T> {
        match self.items.len() {
            0 => None,
            1 => self.items.pop(),
            _ => {
                let last = self.items.pop().expect("len > 1 checked above");
                let root = std::mem::replace(&mut self.items[0], last);
                self.sift_down(0);
                Some(root)
            }
        }
    }

    /// Shared reference to the largest value, or `None` if empty. O(1).
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.items.first()
    }

    /// Builds the heap in place using Floyd's algorithm: sift down from the
    /// deepest parents to the root. O(n).
    fn heapify(&mut self) {
        let n = self.items.len();
        if n <= 1 {
            return;
        }
        let first_parent = (n - 1) / 2;
        for i in (0..=first_parent).rev() {
            self.sift_down(i);
        }
    }

    /// Moves `items[idx]` up until the heap property holds. O(log n).
    fn sift_up(&mut self, mut idx: usize) {
        while idx > 0 {
            let parent = (idx - 1) / 2;
            if self.items[idx] <= self.items[parent] {
                break;
            }
            self.items.swap(idx, parent);
            idx = parent;
        }
    }

    /// Moves `items[idx]` down, swapping with the larger child. O(log n).
    fn sift_down(&mut self, mut idx: usize) {
        let len = self.items.len();
        loop {
            let left = idx * 2 + 1;
            let right = left + 1;
            let mut largest = idx;
            if left < len && self.items[left] > self.items[largest] {
                largest = left;
            }
            if right < len && self.items[right] > self.items[largest] {
                largest = right;
            }
            if largest == idx {
                break;
            }
            self.items.swap(idx, largest);
            idx = largest;
        }
    }
}

impl<T: Ord> FromIterator<T> for BinaryHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let items: Vec<T> = iter.into_iter().collect();
        let mut heap = Self { items };
        heap.heapify();
        heap
    }
}

impl<T: Ord> Extend<T> for BinaryHeap<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for BinaryHeap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.items.iter()).finish()
    }
}

impl<T: Ord> IntoIterator for BinaryHeap<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

// ---------------------------------------------------------------------------
// Priority queue
// ---------------------------------------------------------------------------

/// A priority queue: entries are removed in priority order.
///
/// Built on a [`BinaryHeap`] of `(priority, item)` pairs, so ties are broken
/// by the item itself. This matches the "bundle" style used in most
/// algorithm implementations.
#[derive(Clone, Default)]
pub struct PriorityQueue<P, T> {
    heap: BinaryHeap<(P, T)>,
}

impl<P: Ord, T: Ord> PriorityQueue<P, T> {
    /// Creates an empty priority queue. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    /// Number of entries. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.heap.len()
    }

    /// `true` if the queue is empty. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Inserts `item` with priority `priority`. O(log n).
    pub fn push(&mut self, priority: P, item: T) {
        self.heap.push((priority, item));
    }

    /// Removes and returns the entry with the highest priority. O(log n).
    pub fn pop(&mut self) -> Option<(P, T)> {
        self.heap.pop()
    }

    /// Shared reference to the highest-priority entry. O(1).
    #[must_use]
    pub fn peek(&self) -> Option<&(P, T)> {
        self.heap.peek()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_heap_order() {
        let mut heap = BinaryHeap::new();
        for value in [3, 1, 4, 1, 5, 9, 2, 6] {
            heap.push(value);
        }
        assert_eq!(heap.len(), 8);
        assert_eq!(heap.peek(), Some(&9));
        let mut popped = Vec::new();
        while let Some(v) = heap.pop() {
            popped.push(v);
        }
        assert_eq!(popped, vec![9, 6, 5, 4, 3, 2, 1, 1]);
        assert!(heap.is_empty());
    }

    #[test]
    fn heapify_from_slice() {
        let heap = BinaryHeap::from_slice(&[3, 1, 4, 1, 5, 9, 2, 6]);
        assert_eq!(heap.peek(), Some(&9));
        assert_eq!(heap.len(), 8);
    }

    #[test]
    fn from_iterator() {
        let heap: BinaryHeap<i32> = [10, 20, 5, 30].into_iter().collect();
        assert_eq!(heap.peek(), Some(&30));
    }

    #[test]
    fn heap_property_holds() {
        // After arbitrary insert/remove churn, verify heap property directly.
        let mut heap: BinaryHeap<i32> = BinaryHeap::new();
        for i in 0..1000 {
            heap.push(i % 37 * 3 - 50);
        }
        for _ in 0..500 {
            heap.pop();
            heap.push(42);
        }
        let n = heap.len();
        for i in 0..n {
            let left = i * 2 + 1;
            let right = left + 1;
            if left < n {
                assert!(heap.items[i] >= heap.items[left]);
            }
            if right < n {
                assert!(heap.items[i] >= heap.items[right]);
            }
        }
    }

    #[test]
    fn priority_queue_order() {
        let mut pq = PriorityQueue::new();
        pq.push(3, "three");
        pq.push(1, "one");
        pq.push(2, "two");
        assert_eq!(pq.pop(), Some((3, "three")));
        assert_eq!(pq.pop(), Some((2, "two")));
        assert_eq!(pq.pop(), Some((1, "one")));
        assert_eq!(pq.pop(), None);
    }

    #[test]
    fn single_element() {
        let mut heap = BinaryHeap::new();
        heap.push(7);
        assert_eq!(heap.pop(), Some(7));
        assert_eq!(heap.pop(), None);
    }
}
