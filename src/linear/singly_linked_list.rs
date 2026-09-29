//! A singly-linked list with iterative `Drop`, cached length, and full iterator support.
//!
//! # When to use
//!
//! Singly-linked lists excel at O(1) front insertion/removal and are simple to
//! implement recursively-free in Rust. For random access or frequent tail
//! operations, prefer [`DynamicArray`](super::dynamic_array::DynamicArray).
//!
//! # Complexity
//!
//! | Operation | Time |
//! |-----------|------|
//! | `push_front` | O(1) |
//! | `pop_front`  | O(1) |
//! | `append`     | O(n) |
//! | `get`        | O(n) |
//! | `len`        | O(1) |

use std::fmt;

/// Internal node for [`SinglyLinkedList`].
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self { value, next: None }
    }
}

/// A singly-linked list with cached length and iterative drop.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::linear::SinglyLinkedList;
///
/// let mut list = SinglyLinkedList::new();
/// list.push_front(1);
/// list.push_front(2);
/// assert_eq!(list.pop_front(), Some(2));
/// assert_eq!(list.pop_front(), Some(1));
/// ```
pub struct SinglyLinkedList<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> SinglyLinkedList<T> {
    /// Creates an empty list. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self { head: None, len: 0 }
    }

    /// Number of elements. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// `true` if the list has no elements. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Inserts `value` at the front. O(1).
    pub fn push_front(&mut self, value: T) {
        let mut node = Box::new(Node::new(value));
        node.next = self.head.take();
        self.head = Some(node);
        self.len += 1;
    }

    /// Removes and returns the front element, or `None` if empty. O(1).
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            let node = *node;
            self.head = node.next;
            self.len -= 1;
            node.value
        })
    }

    /// Shared reference to the front element, or `None` if empty. O(1).
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    /// Mutable reference to the front element, or `None` if empty. O(1).
    #[must_use]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.value)
    }

    /// Appends `value` to the tail. O(n).
    pub fn append(&mut self, value: T) {
        let mut cur = &mut self.head;
        while let Some(node) = cur {
            cur = &mut node.next;
        }
        *cur = Some(Box::new(Node::new(value)));
        self.len += 1;
    }

    /// Returns the element at `index`, or `None` if out of bounds. O(n).
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        let mut cur = self.head.as_deref();
        for _ in 0..index {
            cur = cur?.next.as_deref();
        }
        cur.map(|node| &node.value)
    }

    /// Returns a mutable reference to the element at `index`, or `None`. O(n).
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let mut cur = self.head.as_deref_mut();
        for _ in 0..index {
            cur = cur?.next.as_deref_mut();
        }
        cur.map(|node| &mut node.value)
    }

    /// Clears the list, dropping all elements. O(n).
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Returns an iterator over shared references. O(1) to create.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    /// Returns an iterator over mutable references. O(1) to create.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<T> Default for SinglyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for SinglyLinkedList<T> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for value in self {
            new.append(value.clone());
        }
        new
    }
}

impl<T> Drop for SinglyLinkedList<T> {
    /// Iteratively unlinks nodes to avoid stack overflow on long lists.
    fn drop(&mut self) {
        let mut cur = self.head.take();
        while let Some(mut node) = cur {
            cur = node.next.take();
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for SinglyLinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Extend<T> for SinglyLinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.append(value);
        }
    }
}

impl<T> FromIterator<T> for SinglyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

/// Shared-reference iterator for [`SinglyLinkedList`].
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.value
        })
    }
}

/// Mutable-reference iterator for [`SinglyLinkedList`].
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.value
        })
    }
}

/// Consuming iterator for [`SinglyLinkedList`].
pub struct IntoIter<T> {
    list: SinglyLinkedList<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.list.pop_front()
    }
}

impl<T> IntoIterator for SinglyLinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

impl<'a, T> IntoIterator for &'a SinglyLinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SinglyLinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_front_pop_front() {
        let mut list = SinglyLinkedList::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn append_and_indexing() {
        let mut list = SinglyLinkedList::new();
        list.append(10);
        list.append(20);
        list.append(30);
        assert_eq!(list.get(0), Some(&10));
        assert_eq!(list.get(2), Some(&30));
        assert_eq!(list.get(3), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = SinglyLinkedList::new();
        list.extend([1, 2, 3]);
        for v in &mut list {
            *v *= 10;
        }
        let collected: Vec<i32> = list.iter().copied().collect();
        assert_eq!(collected, vec![10, 20, 30]);
    }

    #[test]
    fn long_list_drop_no_stack_overflow() {
        let mut list = SinglyLinkedList::new();
        for i in 0..100_000 {
            list.push_front(i);
        }
        assert_eq!(list.len(), 100_000);
        // Drop happens here; iterative Drop must not overflow.
    }

    #[test]
    fn into_iter() {
        let list: SinglyLinkedList<i32> = [1, 2, 3].into_iter().collect();
        let collected: Vec<i32> = list.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }
}
