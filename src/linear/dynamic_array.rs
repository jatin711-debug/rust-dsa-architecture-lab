//! A from-scratch dynamic array (`Vec`-like) with manual memory management.
//!
//! This implementation mirrors the core ideas of `std::vec::Vec` without using
//! `Vec` internally. It owns a contiguous heap allocation, tracks length and
//! capacity, grows geometrically, and shrinks when utilization drops.
//!
//! # Complexity
//!
//! | Operation | Average | Worst |
//! |-----------|---------|-------|
//! | `push`    | O(1)*   | O(n)  |
//! | `pop`     | O(1)    | O(1)  |
//! | `get`     | O(1)    | O(1)  |
//! | `insert`  | O(n)    | O(n)  |
//! | `remove`  | O(n)    | O(n)  |
//!
//! *Amortized.

use std::alloc::{self, Layout};
use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr::{self, NonNull};

const INITIAL_CAPACITY: usize = 4;
const GROWTH_FACTOR: usize = 2;

/// A contiguous, growable array type written from scratch.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::linear::DynamicArray;
///
/// let mut arr = DynamicArray::new();
/// arr.push(10);
/// arr.push(20);
/// assert_eq!(arr[0], 10);
/// assert_eq!(arr.pop(), Some(20));
/// ```
pub struct DynamicArray<T> {
    /// Pointer to the heap-allocated, capacity-sized buffer.
    ///
    /// Invariant: either `cap == 0` and `ptr` is dangling, or `ptr` points to
    /// a valid allocation of at least `cap * size_of::<T>()` bytes aligned for `T`.
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
}

impl<T> DynamicArray<T> {
    /// Returns an empty array with no heap allocation. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            // A dangling pointer is safe when capacity is zero; we never deref it.
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    /// Returns an empty array with at least the specified capacity reserved. O(n).
    ///
    /// # Panics
    ///
    /// Panics if the requested capacity exceeds `isize::MAX / size_of::<T>()`.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }
        let layout = Self::layout_for(capacity);
        // SAFETY: layout has non-zero size because capacity > 0.
        let ptr = unsafe { alloc::alloc(layout) };
        let Some(ptr) = NonNull::new(ptr.cast::<T>()) else {
            alloc::handle_alloc_error(layout);
        };
        Self {
            ptr,
            len: 0,
            cap: capacity,
        }
    }

    /// Number of stored elements. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// `true` if no elements are stored. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Total number of elements that can be stored without reallocating. O(1).
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.cap
    }

    /// Shared reference to the element at `index`, or `None` if out of bounds. O(1).
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: index is within initialized [0, len).
            Some(unsafe { self.ptr.as_ptr().add(index).as_ref().unwrap_unchecked() })
        } else {
            None
        }
    }

    /// Mutable reference to the element at `index`, or `None` if out of bounds. O(1).
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            // SAFETY: index is within initialized [0, len); exclusive mutable borrow.
            Some(unsafe { self.ptr.as_ptr().add(index).as_mut().unwrap_unchecked() })
        } else {
            None
        }
    }

    /// Appends an element to the back. Amortized O(1).
    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        // SAFETY: after grow, len < cap; pointer is valid for writes at offset len.
        unsafe {
            self.ptr.as_ptr().add(self.len).write(value);
        }
        self.len += 1;
    }

    /// Removes and returns the last element, or `None` if empty. O(1).
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: len was just decremented, so it points to an initialized value.
        let value = unsafe { self.ptr.as_ptr().add(self.len).read() };
        self.shrink_if_needed();
        Some(value)
    }

    /// Inserts `value` at `index`, shifting later elements right. O(n).
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "insert index out of bounds");
        if self.len == self.cap {
            self.grow();
        }
        // SAFETY: index is in [0, len]; source/dest regions are valid and non-overlapping.
        unsafe {
            let ptr = self.ptr.as_ptr().add(index);
            ptr::copy(ptr, ptr.add(1), self.len - index);
            ptr.write(value);
        }
        self.len += 1;
    }

    /// Removes and returns the element at `index`, shifting later elements left. O(n).
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "remove index out of bounds");
        // SAFETY: index is valid; read the value before overwriting the slot.
        let value = unsafe { self.ptr.as_ptr().add(index).read() };
        unsafe {
            let ptr = self.ptr.as_ptr().add(index);
            ptr::copy(ptr.add(1), ptr, self.len - index - 1);
        }
        self.len -= 1;
        self.shrink_if_needed();
        value
    }

    /// Clears all elements but keeps the current capacity. O(n).
    pub fn clear(&mut self) {
        // Drop elements in place, then reset length. Capacity stays unchanged.
        let _ = self.drain(..);
    }

    /// Ensures the total capacity is at least `additional` more than current len. O(n).
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows addressable memory.
    pub fn reserve(&mut self, additional: usize) {
        let required = self.len.checked_add(additional).expect("capacity overflow");
        if required > self.cap {
            self.reallocate(required);
        }
    }

    /// Shrinks capacity to fit the current length, freeing excess memory. O(n).
    pub fn shrink_to_fit(&mut self) {
        if self.cap > self.len {
            self.reallocate(self.len.max(INITIAL_CAPACITY));
        }
    }

    /// Returns a slice view of the array. O(1).
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: [0, len) is initialized and the borrow matches self.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns a mutable slice view of the array. O(1).
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: [0, len) is initialized and the borrow matches self exclusively.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Returns an iterator over shared references. O(1) to create.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            ptr: self.ptr.as_ptr(),
            len: self.len,
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns an iterator over mutable references. O(1) to create.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            ptr: self.ptr.as_ptr(),
            len: self.len,
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// Grows the buffer by the configured growth factor. O(n).
    fn grow(&mut self) {
        let new_cap = if self.cap == 0 {
            INITIAL_CAPACITY
        } else {
            self.cap.saturating_mul(GROWTH_FACTOR)
        };
        self.reallocate(new_cap.max(1));
    }

    /// Reallocates the buffer to exactly `new_cap` and copies existing elements. O(n).
    ///
    /// # Panics
    ///
    /// Panics if `new_cap < self.len` or on allocation failure.
    fn reallocate(&mut self, new_cap: usize) {
        assert!(new_cap >= self.len, "new capacity smaller than length");
        if new_cap == self.cap {
            return;
        }

        let new_layout = Self::layout_for(new_cap);
        let new_ptr = if self.cap == 0 {
            // SAFETY: new_layout has non-zero size.
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Self::layout_for(self.cap);
            // SAFETY: old layout matches the live allocation; new layout fits.
            unsafe {
                alloc::realloc(
                    self.ptr.as_ptr().cast::<u8>(),
                    old_layout,
                    new_layout.size(),
                )
            }
        };

        let Some(new_ptr) = NonNull::new(new_ptr.cast::<T>()) else {
            alloc::handle_alloc_error(new_layout);
        };

        self.ptr = new_ptr;
        self.cap = new_cap;
    }

    /// If utilization is low enough, halves capacity. O(n) when triggered.
    fn shrink_if_needed(&mut self) {
        // Shrink when 1/4 full or less, but never below INITIAL_CAPACITY.
        if self.cap > INITIAL_CAPACITY && self.len * 4 <= self.cap {
            let new_cap = (self.cap / 2).max(INITIAL_CAPACITY);
            self.reallocate(new_cap);
        }
    }

    /// Builds a layout for `capacity` elements of `T`.
    fn layout_for(capacity: usize) -> Layout {
        Layout::array::<T>(capacity).expect("capacity overflow")
    }
}

impl<T> Default for DynamicArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for DynamicArray<T> {
    fn clone(&self) -> Self {
        let mut new = Self::with_capacity(self.len);
        for value in self {
            // We could use ptr::write, but pushing a clone is simpler and still O(n).
            new.push(value.clone());
        }
        new
    }
}

impl<T> Drop for DynamicArray<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }
        // Drop initialized elements first.
        for i in 0..self.len {
            // SAFETY: each index in [0, len) is initialized.
            unsafe { self.ptr.as_ptr().add(i).drop_in_place() };
        }
        // Then free the raw allocation.
        let layout = Self::layout_for(self.cap);
        // SAFETY: layout matches the original allocation.
        unsafe { alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), layout) };
    }
}

impl<T> Deref for DynamicArray<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for DynamicArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> Index<I> for DynamicArray<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> IndexMut<I> for DynamicArray<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}

impl<T: fmt::Debug> fmt::Debug for DynamicArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Extend<T> for DynamicArray<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        let (lower, _upper) = iterator.size_hint();
        self.reserve(lower);
        for value in iterator {
            self.push(value);
        }
    }
}

impl<T> FromIterator<T> for DynamicArray<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut arr = Self::new();
        arr.extend(iter);
        arr
    }
}

impl<T: PartialEq> PartialEq for DynamicArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq> Eq for DynamicArray<T> {}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

/// Shared-reference iterator for [`DynamicArray`].
pub struct Iter<'a, T> {
    ptr: *const T,
    len: usize,
    pos: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.len {
            // SAFETY: pos < len, so the element is initialized and the lifetime is correct.
            let value = unsafe { &*self.ptr.add(self.pos) };
            self.pos += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.pos;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

/// Mutable-reference iterator for [`DynamicArray`].
pub struct IterMut<'a, T> {
    ptr: *mut T,
    len: usize,
    pos: usize,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.len {
            // SAFETY: pos < len; exclusive borrow enforced by IterMut's lifetime.
            let value = unsafe { &mut *self.ptr.add(self.pos) };
            self.pos += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.pos;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}

/// Consuming iterator for [`DynamicArray`].
pub struct IntoIter<T> {
    arr: DynamicArray<T>,
    pos: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.arr.len {
            // SAFETY: pos < len; read the value and advance. Drop guard handles the rest.
            let value = unsafe { self.arr.ptr.as_ptr().add(self.pos).read() };
            self.pos += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.arr.len - self.pos;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> IntoIterator for DynamicArray<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { arr: self, pos: 0 }
    }
}

impl<'a, T> IntoIterator for &'a DynamicArray<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut DynamicArray<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// ---------------------------------------------------------------------------
// Drain helper (used by clear)
// ---------------------------------------------------------------------------

struct Drain<'a, T> {
    src: &'a mut DynamicArray<T>,
    pos: usize,
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.src.len {
            // SAFETY: pos < len; read value then mark slot as moved.
            let value = unsafe { self.src.ptr.as_ptr().add(self.pos).read() };
            self.pos += 1;
            Some(value)
        } else {
            self.src.len = 0;
            None
        }
    }
}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        // Consume any remaining items so they are dropped.
        while self.next().is_some() {}
        self.src.len = 0;
    }
}

impl<T> DynamicArray<T> {
    fn drain(&mut self, _range: std::ops::RangeFull) -> Drain<'_, T> {
        Drain { src: self, pos: 0 }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let arr: DynamicArray<i32> = DynamicArray::new();
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.capacity(), 0);
    }

    #[test]
    fn push_pop_and_growth() {
        let mut arr = DynamicArray::new();
        for i in 0..100 {
            arr.push(i);
        }
        assert_eq!(arr.len(), 100);
        assert!(arr.capacity() >= 100);
        for i in (0..100).rev() {
            assert_eq!(arr.pop(), Some(i));
        }
        assert!(arr.is_empty());
    }

    #[test]
    fn insert_and_remove() {
        let mut arr = DynamicArray::new();
        arr.push(1);
        arr.push(3);
        arr.insert(1, 2);
        assert_eq!(arr.as_slice(), &[1, 2, 3]);
        assert_eq!(arr.remove(1), 2);
        assert_eq!(arr.as_slice(), &[1, 3]);
    }

    #[test]
    fn indexing() {
        let mut arr = DynamicArray::new();
        arr.extend([10, 20, 30]);
        assert_eq!(arr[0], 10);
        arr[1] = 25;
        assert_eq!(arr[1], 25);
    }

    #[test]
    fn iterators() {
        let mut arr = DynamicArray::new();
        arr.extend([1, 2, 3]);
        let sum: i32 = arr.iter().sum();
        assert_eq!(sum, 6);

        for v in &mut arr {
            *v *= 2;
        }
        let collected: Vec<i32> = arr.into_iter().collect();
        assert_eq!(collected, vec![2, 4, 6]);
    }

    #[test]
    fn clone_and_eq() {
        let mut arr = DynamicArray::new();
        arr.extend([1, 2, 3]);
        let cloned = arr.clone();
        assert_eq!(arr, cloned);
        arr.push(4);
        assert_ne!(arr, cloned);
    }

    #[test]
    fn reserve_and_shrink() {
        let mut arr = DynamicArray::new();
        arr.reserve(100);
        assert!(arr.capacity() >= 100);
        arr.extend([1, 2, 3]);
        arr.shrink_to_fit();
        assert_eq!(arr.capacity(), INITIAL_CAPACITY);
    }

    #[test]
    fn zst_handling() {
        // Zero-sized types must not panic and should behave correctly.
        let mut arr: DynamicArray<()> = DynamicArray::new();
        for _ in 0..10 {
            arr.push(());
        }
        assert_eq!(arr.len(), 10);
        for _ in 0..10 {
            arr.pop();
        }
        assert!(arr.is_empty());
    }
}
