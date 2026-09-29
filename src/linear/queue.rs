//! A FIFO queue implemented as a circular buffer with raw memory management.
//!
//! This is the same strategy used by `std::collections::VecDeque`: elements are
//! stored in a contiguous buffer and the logical front wraps around the physical
//! end of the allocation. Both enqueue and dequeue are O(1) amortized.
//!
//! # Complexity
//!
//! | Operation | Time |
//! |-----------|------|
//! | `enqueue` | O(1) amortized |
//! | `dequeue` | O(1) amortized |
//! | `peek`    | O(1) |
//! | `len`     | O(1) |

use std::alloc::{self, Layout};
use std::fmt;
use std::ptr::{self, NonNull};

const INITIAL_CAPACITY: usize = 4;
const GROWTH_FACTOR: usize = 2;

/// A first-in-first-out (FIFO) queue backed by a ring buffer.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::linear::Queue;
///
/// let mut queue = Queue::new();
/// queue.enqueue(1);
/// queue.enqueue(2);
/// assert_eq!(queue.peek(), Some(&1));
/// assert_eq!(queue.dequeue(), Some(1));
/// ```
pub struct Queue<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
    head: usize, // index of the front element
}

impl<T> Queue<T> {
    /// Creates an empty queue. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
            head: 0,
        }
    }

    /// Number of elements in the queue. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// `true` if the queue has no elements. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Total capacity of the backing buffer. O(1).
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.cap
    }

    /// Adds `value` to the back of the queue. Amortized O(1).
    pub fn enqueue(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        // SAFETY: len < cap after grow; tail index wraps modulo cap.
        let tail = (self.head + self.len) % self.cap;
        unsafe {
            self.ptr.as_ptr().add(tail).write(value);
        }
        self.len += 1;
    }

    /// Removes and returns the front element, or `None` if empty. Amortized O(1).
    pub fn dequeue(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: head points to an initialized element.
        let value = unsafe { self.ptr.as_ptr().add(self.head).read() };
        self.head = (self.head + 1) % self.cap;
        self.len -= 1;
        self.shrink_if_needed();
        Some(value)
    }

    /// Shared reference to the front element, or `None` if empty. O(1).
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: head points to an initialized element.
        Some(unsafe { self.ptr.as_ptr().add(self.head).as_ref().unwrap_unchecked() })
    }

    /// Mutable reference to the front element, or `None` if empty. O(1).
    #[must_use]
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: head points to an initialized element; exclusive borrow.
        Some(unsafe { self.ptr.as_ptr().add(self.head).as_mut().unwrap_unchecked() })
    }

    /// Clears the queue, keeping the current capacity. O(n).
    pub fn clear(&mut self) {
        while self.dequeue().is_some() {}
    }

    /// Grows the ring buffer and re-centers elements at the front. O(n).
    fn grow(&mut self) {
        let new_cap = if self.cap == 0 {
            INITIAL_CAPACITY
        } else {
            self.cap.saturating_mul(GROWTH_FACTOR)
        };
        self.reallocate(new_cap);
    }

    /// Reallocates to `new_cap` and compacts elements so head == 0. O(n).
    fn reallocate(&mut self, new_cap: usize) {
        assert!(new_cap >= self.len, "new capacity smaller than length");
        if new_cap == self.cap {
            return;
        }

        let new_layout = Self::layout_for(new_cap);
        // Always allocate a fresh buffer. This avoids the subtle hazards of
        // `alloc::realloc` (possible move invalidating the old pointer before
        // we copy wrapped data).
        let new_ptr = unsafe { alloc::alloc(new_layout) };
        let Some(new_ptr) = NonNull::new(new_ptr.cast::<T>()) else {
            alloc::handle_alloc_error(new_layout);
        };

        // Compact: move elements so they start at index 0 if they currently wrap.
        if self.len > 0 && self.head + self.len > self.cap {
            let split = self.cap - self.head;
            // SAFETY: both regions are valid and non-overlapping in the new buffer.
            unsafe {
                ptr::copy(self.ptr.as_ptr().add(self.head), new_ptr.as_ptr(), split);
                ptr::copy(
                    self.ptr.as_ptr(),
                    new_ptr.as_ptr().add(split),
                    self.len - split,
                );
            }
        } else if self.len > 0 {
            // SAFETY: contiguous block starting at head.
            unsafe {
                ptr::copy(self.ptr.as_ptr().add(self.head), new_ptr.as_ptr(), self.len);
            }
        }

        if self.cap > 0 {
            let old_layout = Self::layout_for(self.cap);
            // SAFETY: layout matches the original allocation being freed.
            unsafe { alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), old_layout) };
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
        self.head = 0;
    }

    fn shrink_if_needed(&mut self) {
        if self.cap > INITIAL_CAPACITY && self.len * 4 <= self.cap {
            let new_cap = (self.cap / 2).max(INITIAL_CAPACITY);
            self.reallocate(new_cap);
        }
    }

    fn layout_for(capacity: usize) -> Layout {
        Layout::array::<T>(capacity).expect("capacity overflow")
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for Queue<T> {
    fn clone(&self) -> Self {
        let mut new = Self::with_capacity(self.len);
        for i in 0..self.len {
            let idx = (self.head + i) % self.cap;
            // SAFETY: idx is within initialized region.
            let value = unsafe { self.ptr.as_ptr().add(idx).as_ref().unwrap_unchecked() };
            new.enqueue(value.clone());
        }
        new
    }
}

impl<T> Queue<T> {
    /// Creates an empty queue with reserved capacity. O(n).
    #[must_use]
    fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }
        let layout = Self::layout_for(capacity);
        let ptr = unsafe { alloc::alloc(layout) };
        let Some(ptr) = NonNull::new(ptr.cast::<T>()) else {
            alloc::handle_alloc_error(layout);
        };
        Self {
            ptr,
            cap: capacity,
            len: 0,
            head: 0,
        }
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }
        self.clear();
        let layout = Self::layout_for(self.cap);
        // SAFETY: layout matches the live allocation.
        unsafe { alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), layout) };
    }
}

impl<T: fmt::Debug> fmt::Debug for Queue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

impl<T> Extend<T> for Queue<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.enqueue(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_order() {
        let mut q = Queue::new();
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        assert_eq!(q.len(), 3);
        assert_eq!(q.peek(), Some(&1));
        assert_eq!(q.dequeue(), Some(1));
        assert_eq!(q.dequeue(), Some(2));
        assert_eq!(q.dequeue(), Some(3));
        assert_eq!(q.dequeue(), None);
        assert!(q.is_empty());
    }

    #[test]
    fn wraparound_behavior() {
        let mut q = Queue::new();
        for i in 0..10 {
            q.enqueue(i);
        }
        for _ in 0..7 {
            q.dequeue();
        }
        for i in 10..20 {
            q.enqueue(i);
        }
        let collected: Vec<i32> = std::iter::from_fn(|| q.dequeue()).collect();
        assert_eq!(collected, (7..20).collect::<Vec<_>>());
    }

    #[test]
    fn clone_preserves_order() {
        let mut q = Queue::new();
        q.extend([1, 2, 3]);
        let mut cloned = q.clone();
        let original: Vec<i32> = std::iter::from_fn(|| q.dequeue()).collect();
        let cloned_vec: Vec<i32> = std::iter::from_fn(|| cloned.dequeue()).collect();
        assert_eq!(original, cloned_vec);
    }

    #[test]
    fn large_queue_no_overflow() {
        let mut q = Queue::new();
        for i in 0..100_000 {
            q.enqueue(i);
        }
        assert_eq!(q.len(), 100_000);
        for i in 0..100_000 {
            assert_eq!(q.dequeue(), Some(i));
        }
    }
}
