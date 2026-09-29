//! A generic LIFO stack backed by a [`DynamicArray`](super::dynamic_array::DynamicArray).
//!
//! # Complexity
//!
//! | Operation | Time |
//! |-----------|------|
//! | `push`    | O(1) amortized |
//! | `pop`     | O(1) |
//! | `peek`    | O(1) |
//! | `len`     | O(1) |

use std::fmt;

use super::dynamic_array::DynamicArray;

/// A last-in-first-out (LIFO) stack.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::linear::Stack;
///
/// let mut stack = Stack::new();
/// stack.push(1);
/// stack.push(2);
/// assert_eq!(stack.peek(), Some(&2));
/// assert_eq!(stack.pop(), Some(2));
/// ```
#[derive(Clone, Default)]
pub struct Stack<T> {
    inner: DynamicArray<T>,
}

impl<T> Stack<T> {
    /// Creates an empty stack. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: DynamicArray::new(),
        }
    }

    /// Pushes `value` onto the top of the stack. Amortized O(1).
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// Removes and returns the top element, or `None` if empty. O(1).
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Shared reference to the top element, or `None` if empty. O(1).
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.inner.get(self.inner.len().saturating_sub(1))
    }

    /// Mutable reference to the top element, or `None` if empty. O(1).
    #[must_use]
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        let idx = self.inner.len().saturating_sub(1);
        self.inner.get_mut(idx)
    }

    /// Number of elements in the stack. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// `true` if the stack has no elements. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Empties the stack. O(n).
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns an iterator over shared references, bottom to top. O(1) to create.
    #[must_use]
    pub fn iter(&self) -> super::dynamic_array::Iter<'_, T> {
        self.inner.iter()
    }
}

impl<T> Extend<T> for Stack<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.inner.extend(iter);
    }
}

impl<T: fmt::Debug> fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stack").field("items", &self.inner).finish()
    }
}

impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = super::dynamic_array::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = super::dynamic_array::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_operations() {
        let mut stack = Stack::new();
        assert!(stack.is_empty());
        stack.push(10);
        stack.push(20);
        stack.push(30);
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.peek(), Some(&30));
        assert_eq!(stack.pop(), Some(30));
        assert_eq!(stack.pop(), Some(20));
        assert_eq!(stack.peek(), Some(&10));
        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn peek_mut_updates_top() {
        let mut stack = Stack::new();
        stack.push(5);
        if let Some(top) = stack.peek_mut() {
            *top = 99;
        }
        assert_eq!(stack.pop(), Some(99));
    }

    #[test]
    fn into_iter_consumes_bottom_to_top() {
        let mut stack = Stack::new();
        stack.extend([1, 2, 3]);
        let collected: Vec<i32> = stack.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }
}
