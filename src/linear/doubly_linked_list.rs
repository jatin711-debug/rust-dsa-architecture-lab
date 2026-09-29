//! A doubly-linked deque implemented with unsafe raw pointers.
//!
//! This is the classic layout used by `std::collections::LinkedList`: each node
//! stores `value`, `next`, and `prev` as `Option<NonNull<Node<T>>>`. The list
//! owns the head and tail pointers and frees every node on drop.
//!
//! # Complexity
//!
//! | Operation | Time |
//! |-----------|------|
//! | `push_front` / `push_back` | O(1) |
//! | `pop_front` / `pop_back`   | O(1) |
//! | `front` / `back`           | O(1) |
//! | `len`                      | O(1) |
//! | `get`                      | O(n) |
//!
//! # Safety notes
//!
//! All `unsafe` blocks are localized to node allocation, pointer dereferencing,
//! and `Drop`. The public API is fully safe. The type is `Send` and `Sync` when
//! `T` is, matching the standard library's policy.

use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// Internal node for [`DoublyLinkedList`].
struct Node<T> {
    value: T,
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            next: None,
            prev: None,
        }
    }

    /// Leaks a boxed node into a raw pointer, returning a non-null handle.
    fn into_raw(value: T) -> NonNull<Node<T>> {
        let boxed = Box::new(Node::new(value));
        // SAFETY: Box::into_raw always returns a non-null, correctly aligned pointer.
        unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) }
    }
}

/// A doubly-linked list / deque.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::linear::DoublyLinkedList;
///
/// let mut list = DoublyLinkedList::new();
/// list.push_back(1);
/// list.push_front(0);
/// list.push_back(2);
/// assert_eq!(list.pop_front(), Some(0));
/// assert_eq!(list.pop_back(), Some(2));
/// ```
pub struct DoublyLinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    _marker: PhantomData<Box<Node<T>>>, // owns heap nodes via raw pointers
}

impl<T> DoublyLinkedList<T> {
    /// Creates an empty list. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            _marker: PhantomData,
        }
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
        let new_head = Node::into_raw(value);
        if let Some(old_head) = self.head {
            // SAFETY: old_head is a live node pointer we own.
            unsafe {
                (*old_head.as_ptr()).prev = Some(new_head);
                (*new_head.as_ptr()).next = Some(old_head);
            }
            self.head = Some(new_head);
        } else {
            self.head = Some(new_head);
            self.tail = Some(new_head);
        }
        self.len += 1;
    }

    /// Inserts `value` at the back. O(1).
    pub fn push_back(&mut self, value: T) {
        let new_tail = Node::into_raw(value);
        if let Some(old_tail) = self.tail {
            // SAFETY: old_tail is a live node pointer we own.
            unsafe {
                (*old_tail.as_ptr()).next = Some(new_tail);
                (*new_tail.as_ptr()).prev = Some(old_tail);
            }
            self.tail = Some(new_tail);
        } else {
            self.head = Some(new_tail);
            self.tail = Some(new_tail);
        }
        self.len += 1;
    }

    /// Removes and returns the front element, or `None` if empty. O(1).
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.map(|old_head| {
            // SAFETY: old_head is a live node we own; reading its fields is safe.
            let old_head_ref = unsafe { old_head.as_ref() };
            self.head = old_head_ref.next;
            if let Some(new_head) = self.head {
                // SAFETY: new_head is a live node.
                unsafe { (*new_head.as_ptr()).prev = None };
            } else {
                self.tail = None;
            }
            self.len -= 1;
            // SAFETY: old_head came from Box::into_raw and is no longer referenced.
            unsafe { Box::from_raw(old_head.as_ptr()).value }
        })
    }

    /// Removes and returns the back element, or `None` if empty. O(1).
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.map(|old_tail| {
            // SAFETY: old_tail is a live node we own.
            let old_tail_ref = unsafe { old_tail.as_ref() };
            self.tail = old_tail_ref.prev;
            if let Some(new_tail) = self.tail {
                // SAFETY: new_tail is a live node.
                unsafe { (*new_tail.as_ptr()).next = None };
            } else {
                self.head = None;
            }
            self.len -= 1;
            // SAFETY: old_tail came from Box::into_raw and is no longer referenced.
            unsafe { Box::from_raw(old_tail.as_ptr()).value }
        })
    }

    /// Shared reference to the front element, or `None` if empty. O(1).
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        // SAFETY: head is a live node and the returned lifetime is tied to self.
        self.head.map(|node| unsafe { &(*node.as_ptr()).value })
    }

    /// Mutable reference to the front element, or `None` if empty. O(1).
    #[must_use]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        // SAFETY: head is a live node; exclusive borrow via self.
        self.head.map(|node| unsafe { &mut (*node.as_ptr()).value })
    }

    /// Shared reference to the back element, or `None` if empty. O(1).
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        // SAFETY: tail is a live node and the returned lifetime is tied to self.
        self.tail.map(|node| unsafe { &(*node.as_ptr()).value })
    }

    /// Mutable reference to the back element, or `None` if empty. O(1).
    #[must_use]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        // SAFETY: tail is a live node; exclusive borrow via self.
        self.tail.map(|node| unsafe { &mut (*node.as_ptr()).value })
    }

    /// Clears the list, dropping all elements. O(n).
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    /// Returns an iterator over shared references. O(1) to create.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over mutable references. O(1) to create.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for DoublyLinkedList<T> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for value in self {
            new.push_back(value.clone());
        }
        new
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

// SAFETY: DoublyLinkedList owns its nodes and can be sent across threads if T can.
unsafe impl<T: Send> Send for DoublyLinkedList<T> {}

// SAFETY: Shared access is safe when T is Sync because nodes are disjoint.
unsafe impl<T: Sync> Sync for DoublyLinkedList<T> {}

impl<T: fmt::Debug> fmt::Debug for DoublyLinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Extend<T> for DoublyLinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push_back(value);
        }
    }
}

impl<T> FromIterator<T> for DoublyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

/// Shared-reference iterator for [`DoublyLinkedList`].
pub struct Iter<'a, T> {
    next: Option<NonNull<Node<T>>>,
    _marker: PhantomData<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // SAFETY: node is a live node; lifetime tied to Iter's phantom data.
            let node_ref = unsafe { &*node.as_ptr() };
            self.next = node_ref.next;
            &node_ref.value
        })
    }
}

/// Mutable-reference iterator for [`DoublyLinkedList`].
pub struct IterMut<'a, T> {
    next: Option<NonNull<Node<T>>>,
    _marker: PhantomData<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // SAFETY: node is a live node; exclusive borrow via IterMut.
            let node_ref = unsafe { &mut *node.as_ptr() };
            self.next = node_ref.next;
            &mut node_ref.value
        })
    }
}

/// Consuming iterator for [`DoublyLinkedList`].
pub struct IntoIter<T> {
    list: DoublyLinkedList<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.list.pop_front()
    }
}

impl<T> IntoIterator for DoublyLinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

impl<'a, T> IntoIterator for &'a DoublyLinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut DoublyLinkedList<T> {
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
    fn push_and_pop_both_ends() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_front(0);
        list.push_back(2);
        assert_eq!(list.len(), 3);
        assert_eq!(list.front(), Some(&0));
        assert_eq!(list.back(), Some(&2));
        assert_eq!(list.pop_front(), Some(0));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn iter_mut() {
        let mut list = DoublyLinkedList::new();
        list.extend([1, 2, 3]);
        for v in &mut list {
            *v *= 10;
        }
        let collected: Vec<i32> = list.iter().copied().collect();
        assert_eq!(collected, vec![10, 20, 30]);
    }

    #[test]
    fn clone_preserves_order() {
        let mut list = DoublyLinkedList::new();
        list.extend([1, 2, 3]);
        let cloned = list.clone();
        let original: Vec<i32> = list.into_iter().collect();
        let cloned_vec: Vec<i32> = cloned.into_iter().collect();
        assert_eq!(original, cloned_vec);
    }

    #[test]
    fn front_and_back_mut() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        *list.front_mut().unwrap() = 10;
        *list.back_mut().unwrap() = 20;
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.pop_back(), Some(20));
    }

    #[test]
    fn long_list_no_stack_overflow() {
        let mut list = DoublyLinkedList::new();
        for i in 0..100_000 {
            list.push_back(i);
        }
        assert_eq!(list.len(), 100_000);
        list.clear();
        assert!(list.is_empty());
    }
}
