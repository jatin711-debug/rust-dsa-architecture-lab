//! Interior mutability: `Cell` and `RefCell` implemented from scratch.
//!
//! Rust's aliasing rules say "either many readers or one writer". Interior
//! mutability bends that rule by moving the check from compile time to runtime.
//! The trick is [`UnsafeCell`], which tells the compiler "this value may be
//! mutated through a shared reference" — the one sound escape hatch.
//!
//! - **`Cell`** — always safe (it never hands out references to its contents,
//!   only copies in/out), so it has zero runtime cost. Works only with `Copy`
//!   types.
//! - **`RefCell`** — moves the borrow check to runtime via a counter. Costs a
//!   branch per borrow; panics if the rules are violated.

use std::cell::UnsafeCell;
use std::fmt;
use std::ops::{Deref, DerefMut};

// ---------------------------------------------------------------------------
// Cell
// ---------------------------------------------------------------------------

/// A mutable memory location with `Copy` semantics and no runtime cost.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::advanced::Cell;
///
/// let cell = Cell::new(41);
/// cell.set(42);
/// assert_eq!(cell.get(), 42);
/// ```
pub struct Cell<T> {
    value: UnsafeCell<T>,
}

impl<T> Cell<T> {
    /// Creates a new cell containing `value`. O(1).
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    /// Replaces the contained value with `value`, returning the old value. O(1).
    pub fn replace(&self, value: T) -> T {
        // SAFETY: we never expose references into the cell, so the read+write
        // pair cannot alias anything the caller can observe.
        unsafe { std::mem::replace(&mut *self.value.get(), value) }
    }

    /// Overwrites the contained value. O(1).
    pub fn set(&self, value: T) {
        let _ = self.replace(value);
    }
}

impl<T: Copy> Cell<T> {
    /// Returns a copy of the contained value. O(1).
    #[must_use]
    pub fn get(&self) -> T {
        // SAFETY: reading a Copy value from a cell is sound because no
        // reference to it can escape — callers only ever get copies.
        unsafe { *self.value.get() }
    }
}

impl<T: Default> Default for Cell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Copy + fmt::Debug> fmt::Debug for Cell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cell").field("value", &self.get()).finish()
    }
}

// ---------------------------------------------------------------------------
// RefCell
// ---------------------------------------------------------------------------

/// Runtime borrow state: `0` = no borrows, `n > 0` = n shared borrows,
/// `-1` = one exclusive borrow.
const UNUSED: isize = 0;
const WRITING: isize = -1;

/// A mutable memory location with runtime-checked borrows.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::advanced::RefCell;
///
/// let cell = RefCell::new(10);
/// {
///     let mut value = cell.borrow_mut();
///     *value += 5;
/// } // the mutable borrow ends here
/// assert_eq!(*cell.borrow(), 15);
/// ```
pub struct RefCell<T: ?Sized> {
    borrow: Cell<isize>,
    value: UnsafeCell<T>,
}

impl<T> RefCell<T> {
    /// Creates a new cell containing `value`. O(1).
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            borrow: Cell::new(UNUSED),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> RefCell<T> {
    /// Returns `true` if a shared borrow is currently active. O(1).
    #[must_use]
    pub fn is_borrowed(&self) -> bool {
        self.borrow.get() != UNUSED
    }

    /// Returns `true` if an exclusive borrow is currently active. O(1).
    #[must_use]
    pub fn is_borrowed_mut(&self) -> bool {
        self.borrow.get() == WRITING
    }

    /// Attempts a shared borrow; returns an error instead of panicking. O(1).
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        if self.borrow.get() == WRITING {
            return Err(BorrowError);
        }
        self.borrow.set(self.borrow.get() + 1);
        // SAFETY: the borrow counter guarantees at most one writer, and we are
        // not the writer (checked above), so handing out a shared reference is
        // sound.
        Ok(Ref {
            value: unsafe { &*self.value.get() },
            cell: self,
        })
    }

    /// Shared borrow; panics if a mutable borrow is active. O(1).
    ///
    /// # Panics
    ///
    /// Panics if the cell is currently mutably borrowed.
    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow()
            .unwrap_or_else(|_| panic!("RefCell already mutably borrowed"))
    }

    /// Attempts an exclusive borrow; returns an error instead of panicking. O(1).
    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        if self.borrow.get() != UNUSED {
            return Err(BorrowMutError);
        }
        self.borrow.set(WRITING);
        // SAFETY: the counter is now WRITING, which blocks all future borrows
        // until the guard drops, so the returned &mut is exclusive.
        Ok(RefMut {
            value: unsafe { &mut *self.value.get() },
            cell: self,
        })
    }

    /// Exclusive borrow; panics if any borrow is active. O(1).
    ///
    /// # Panics
    ///
    /// Panics if the cell is currently borrowed (shared or exclusive).
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.try_borrow_mut()
            .unwrap_or_else(|_| panic!("RefCell already borrowed"))
    }

    /// Returns a mutable reference without any runtime check. O(1).
    ///
    /// Sound because the `&mut self` borrow guarantees exclusivity already.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: &mut self is exclusive; nothing else can be borrowed.
        unsafe { &mut *self.value.get() }
    }
}

impl<T: Default> Default for RefCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RefCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_borrow() {
            Ok(value) => f.debug_struct("RefCell").field("value", &value).finish(),
            Err(_) => f
                .debug_struct("RefCell")
                .field("value", &"<borrowed>")
                .finish(),
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Ref<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

/// The error type for failed shared borrows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorrowError;

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "already mutably borrowed")
    }
}

impl std::error::Error for BorrowError {}

/// The error type for failed exclusive borrows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorrowMutError;

impl fmt::Display for BorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "already borrowed")
    }
}

impl std::error::Error for BorrowMutError {}

// ---------------------------------------------------------------------------
// Borrow guards
// ---------------------------------------------------------------------------

/// A shared borrow guard: restores the counter on drop.
pub struct Ref<'a, T: ?Sized> {
    value: &'a T,
    cell: &'a RefCell<T>,
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: ?Sized> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        self.cell.borrow.set(self.cell.borrow.get() - 1);
    }
}

/// An exclusive borrow guard: restores the counter on drop.
pub struct RefMut<'a, T: ?Sized> {
    value: &'a mut T,
    cell: &'a RefCell<T>,
}

impl<T: ?Sized> Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: ?Sized> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T: ?Sized> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        self.cell.borrow.set(UNUSED);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_get_set_replace() {
        let cell = Cell::new(5);
        assert_eq!(cell.get(), 5);
        assert_eq!(cell.replace(10), 5);
        cell.set(15);
        assert_eq!(cell.get(), 15);
    }

    #[test]
    fn cell_mutated_through_shared_reference() {
        let cell = Cell::new(0);
        let shared: &Cell<i32> = &cell; // interior mutability in action
        shared.set(7);
        assert_eq!(cell.get(), 7);
    }

    #[test]
    fn refcell_shared_and_mut_borrows() {
        let cell = RefCell::new(vec![1, 2, 3]);
        assert_eq!(cell.borrow().len(), 3);
        {
            let mut guard = cell.borrow_mut();
            guard.push(4);
        }
        assert_eq!(cell.borrow().len(), 4);
        assert!(!cell.is_borrowed());
    }

    #[test]
    fn try_borrow_conflict_errors() {
        let cell = RefCell::new(1);
        let _guard = cell.borrow_mut();
        assert!(matches!(cell.try_borrow(), Err(BorrowError)));
        assert!(matches!(cell.try_borrow_mut(), Err(BorrowMutError)));
    }

    #[test]
    fn multiple_shared_borrows_are_fine() {
        let cell = RefCell::new(42);
        let a = cell.borrow();
        let b = cell.borrow();
        assert_eq!(*a + *b, 84);
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn borrow_mut_panics_while_borrowed() {
        let cell = RefCell::new(1);
        let _guard = cell.borrow();
        let _ = cell.borrow_mut();
    }

    #[test]
    #[should_panic(expected = "already mutably borrowed")]
    fn borrow_panics_while_mut_borrowed() {
        let cell = RefCell::new(1);
        let _guard = cell.borrow_mut();
        let _ = cell.borrow();
    }

    #[test]
    fn get_mut_skips_checks() {
        let mut cell = RefCell::new(1);
        *cell.get_mut() = 99;
        assert_eq!(*cell.borrow(), 99);
    }

    #[test]
    fn guards_release_on_drop() {
        let cell = RefCell::new(0);
        {
            let _g1 = cell.borrow();
            assert!(cell.is_borrowed());
            let _g2 = cell.borrow();
        }
        assert!(!cell.is_borrowed());
        let mut g = cell.borrow_mut();
        *g = 1;
        drop(g);
        assert!(!cell.is_borrowed_mut());
    }
}
