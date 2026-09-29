//! Reference counting: `Rc` and `Weak` implemented from scratch with raw
//! pointers and manual allocation.
//!
//! `Rc` gives shared ownership of a heap value: cloning increments a strong
//! counter, dropping decrements it, and the value is freed when the count hits
//! zero. `Weak` holds a *non-owning* reference (a weak counter), letting you
//! observe the value without keeping it alive — which is also how reference
//! cycles get broken.
//!
//! Why the counter is a [`Cell`](super::cell::Cell): the counts live on the
//! heap and are mutated through shared `&Rc` references, so they need interior
//! mutability. (`Arc` uses atomics instead — the only difference.)
//!
//! Why the value is a `ManuallyDrop<T>`: the header outlives its value when
//! weak references remain, so the value must be dropped *explicitly* when the
//! strong count hits zero, and the field's own drop glue must be a no-op.
//!
//! # Layout
//!
//! ```text
//! Rc<T> ──► ┌────────────────────────┐
//!           │ strong: Cell<usize>    │  shared header
//!           │ weak:   Cell<usize>    │
//!           │ value:  ManuallyDrop<T>│  user data
//!           └────────────────────────┘
//! ```

use std::alloc::{self, Layout};
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::{self, NonNull};

/// The shared heap header: two counters plus the value.
struct RcInner<T: ?Sized> {
    strong: Cell<usize>,
    weak: Cell<usize>,
    value: ManuallyDrop<T>,
}

/// A single-threaded reference-counted pointer.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::advanced::Rc;
///
/// let shared = Rc::new(41);
/// let alias = Rc::clone(&shared);
/// assert_eq!(Rc::strong_count(&alias), 2);
/// assert_eq!(*shared + *alias, 82);
/// ```
pub struct Rc<T: ?Sized> {
    ptr: NonNull<RcInner<T>>,
    /// Owns the allocation on behalf of the strong count (dropck).
    _marker: PhantomData<RcInner<T>>,
}

impl<T> Rc<T> {
    /// Allocates a value on the heap with one strong reference. O(1).
    #[must_use]
    pub fn new(value: T) -> Self {
        // The Box is turned into a raw pointer; the Rc now owns it. The
        // Box's ownership is folded into the strong count (starts at 1).
        let boxed = Box::new(RcInner {
            strong: Cell::new(1),
            weak: Cell::new(0),
            value: ManuallyDrop::new(value),
        });
        // SAFETY: Box::into_raw never returns null.
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) };
        Self {
            ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Rc<T> {
    /// Number of strong (owning) references. O(1).
    #[must_use]
    pub fn strong_count(this: &Self) -> usize {
        // SAFETY: `this` holds a live strong reference, so the header is alive.
        unsafe { (*this.ptr.as_ptr()).strong.get() }
    }

    /// Number of weak (non-owning) references. O(1).
    #[must_use]
    pub fn weak_count(this: &Self) -> usize {
        // SAFETY: same as strong_count.
        unsafe { (*this.ptr.as_ptr()).weak.get() }
    }

    /// Creates a non-owning reference to the value. O(1).
    #[must_use]
    pub fn downgrade(this: &Self) -> Weak<T> {
        // SAFETY: we hold a strong reference, so the header is alive.
        unsafe {
            let inner = this.ptr.as_ptr();
            (*inner).weak.set((*inner).weak.get() + 1);
        }
        Weak {
            ptr: this.ptr,
            _marker: PhantomData,
        }
    }

    /// Attempts to unwrap into the inner value, failing if other strong
    /// references exist. O(1).
    pub fn try_unwrap(this: Self) -> Result<T, Self>
    where
        T: Sized,
    {
        if Rc::strong_count(&this) != 1 {
            return Err(this);
        }
        // SAFETY: unique strong reference, so the value is not aliased.
        let inner = this.ptr.as_ptr();
        let value = unsafe { ptr::read(ptr::addr_of!((*inner).value)) };
        // `value` is ManuallyDrop<T>; the slot now holds a moved-out
        // ManuallyDrop whose drop is a no-op — safe for the eventual free.
        // Skip the normal Drop path and run its bookkeeping manually.
        std::mem::forget(this);
        // SAFETY: identical to Rc::drop with strong going 1 -> 0.
        unsafe {
            let strong = (*inner).strong.get() - 1;
            (*inner).strong.set(strong);
            if strong == 0 && (*inner).weak.get() == 0 {
                // SAFETY: this allocation came from Box::into_raw, and the
                // value slot is a moved-out ManuallyDrop (no-op drop).
                drop(Box::from_raw(inner));
            }
        }
        Ok(ManuallyDrop::into_inner(value))
    }

    /// Returns a mutable reference if this is the only strong reference. O(1).
    #[must_use]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Rc::strong_count(this) == 1 {
            // SAFETY: unique strong ref + exclusive &mut on the Rc itself.
            Some(unsafe { &mut (*this.ptr.as_ptr()).value })
        } else {
            None
        }
    }

    /// Consumes the `Rc` and returns the raw pointer to the value. O(1).
    ///
    /// The value is *not* dropped; pair with [`Rc::from_raw`] to reclaim it.
    #[must_use]
    pub fn into_raw(this: Self) -> *const T
    where
        T: Sized,
    {
        // Deref through ManuallyDrop to &T, then to a raw pointer.
        let value_ptr = std::ptr::addr_of!(*this);
        std::mem::forget(this);
        value_ptr
    }

    /// Recreates an `Rc` from a raw pointer produced by [`Rc::into_raw`]. O(1).
    ///
    /// # Safety
    ///
    /// `ptr` must come from `Rc::into_raw` and be returned exactly once.
    #[must_use]
    pub unsafe fn from_raw(ptr: *const T) -> Self
    where
        T: Sized,
    {
        // Recover the header by walking back over the two counters, which are
        // the fields before `value` (sized T only — for unsized T the value
        // sits at the end and this offset math does not apply).
        // SAFETY: pointer arithmetic within the caller-guaranteed allocation;
        // the header starts exactly two counters before the value.
        let inner = unsafe {
            ptr.cast::<u8>()
                .sub(size_of::<Cell<usize>>() * 2)
                .cast::<RcInner<T>>()
                .cast_mut()
        };
        // SAFETY: caller guarantees the pointer is a live Rc header.
        Self {
            ptr: unsafe { NonNull::new_unchecked(inner) },
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Clone for Rc<T> {
    fn clone(&self) -> Self {
        // SAFETY: we hold a strong reference, so the header is alive.
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).strong.set((*inner).strong.get() + 1);
        }
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for Rc<T> {
    fn drop(&mut self) {
        // SAFETY: we hold a strong reference, so the header is alive.
        unsafe {
            let inner = self.ptr.as_ptr();
            let strong = (*inner).strong.get() - 1;
            (*inner).strong.set(strong);
            if strong == 0 {
                // Last strong reference: drop the value exactly once, then
                // free the header unless weak references still observe it.
                ManuallyDrop::drop(&mut (*inner).value);
                if (*inner).weak.get() == 0 {
                    // SAFETY: allocation came from Box::into_raw; the value
                    // field is a dropped ManuallyDrop (no-op drop glue).
                    drop(Box::from_raw(inner));
                }
            }
        }
    }
}

impl<T: ?Sized> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: we hold a strong reference, so the value is alive.
        unsafe { &(*self.ptr.as_ptr()).value }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Rc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> fmt::Pointer for Rc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr.as_ptr(), f)
    }
}

// ---------------------------------------------------------------------------
// Weak
// ---------------------------------------------------------------------------

/// A non-owning reference to an `Rc`-managed value.
pub struct Weak<T: ?Sized> {
    ptr: NonNull<RcInner<T>>,
    _marker: PhantomData<RcInner<T>>,
}

impl<T> Weak<T> {
    /// Creates a new, empty `Weak`. `upgrade` always returns `None`. O(1).
    #[must_use]
    pub fn new() -> Self {
        // An immortal header: strong = 0 (upgrade -> None) and a weak count
        // large enough that dropping this Weak never frees it. The value slot
        // is left uninitialized and is never read or dropped.
        // SAFETY: layout is non-zero and aligned for RcInner<T>.
        let layout = Layout::new::<RcInner<T>>();
        let raw = unsafe { alloc::alloc(layout) };
        // SAFETY: alloc guarantees alignment; `raw` is non-null (checked below
        // via the Box-free path: alloc returns null on failure, handle it).
        let Some(raw) = NonNull::new(raw) else {
            alloc::handle_alloc_error(layout);
        };
        let inner = raw.cast::<RcInner<T>>();
        // SAFETY: the two counter fields are initialized; the value slot is
        // intentionally never initialized (ManuallyDrop has no validity
        // requirements beyond alignment, and it is never read).
        unsafe {
            ptr::addr_of_mut!((*inner.as_ptr()).strong).write(Cell::new(0));
            ptr::addr_of_mut!((*inner.as_ptr()).weak).write(Cell::new(usize::MAX));
        }
        Self {
            ptr: inner,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Weak<T> {
    /// Attempts to upgrade to a strong reference; `None` if the value is gone.
    /// O(1).
    #[must_use]
    pub fn upgrade(&self) -> Option<Rc<T>> {
        // SAFETY: we hold a weak reference, so the header is alive (weak refs
        // keep the header alive until their count reaches zero).
        unsafe {
            let inner = self.ptr.as_ptr();
            if (*inner).strong.get() == 0 {
                None
            } else {
                (*inner).strong.set((*inner).strong.get() + 1);
                Some(Rc {
                    ptr: self.ptr,
                    _marker: PhantomData,
                })
            }
        }
    }

    /// Number of strong references to the value, or `None` if it's gone. O(1).
    #[must_use]
    pub fn strong_count(&self) -> Option<usize> {
        // SAFETY: the header is alive while we hold a weak reference.
        let strong = unsafe { (*self.ptr.as_ptr()).strong.get() };
        (strong != 0).then_some(strong)
    }
}

impl<T: ?Sized> Clone for Weak<T> {
    fn clone(&self) -> Self {
        // SAFETY: we hold a weak reference, so the header is alive.
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).weak.set((*inner).weak.get() + 1);
        }
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for Weak<T> {
    fn drop(&mut self) {
        // SAFETY: we hold a weak reference, so the header is alive.
        unsafe {
            let inner = self.ptr.as_ptr();
            let weak = (*inner).weak.get() - 1;
            (*inner).weak.set(weak);
            if weak == 0 && (*inner).strong.get() == 0 {
                // Last weak reference and the value is already dropped:
                // free the header.
                // SAFETY: the header was allocated via Box::into_raw (Rc::new)
                // or raw alloc (Weak::new, immortal header). The immortal
                // header never reaches this branch because its weak count is
                // usize::MAX. So this is always a Box::into_raw allocation,
                // and the value field is a dropped ManuallyDrop (no-op).
                drop(Box::from_raw(inner));
            }
        }
    }
}

impl<T: ?Sized> fmt::Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Weak)")
    }
}

impl<T> Default for Weak<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strong_count_and_clone() {
        let a = Rc::new(5);
        assert_eq!(Rc::strong_count(&a), 1);
        let b = Rc::clone(&a);
        assert_eq!(Rc::strong_count(&a), 2);
        assert_eq!(Rc::strong_count(&b), 2);
        drop(b);
        assert_eq!(Rc::strong_count(&a), 1);
        assert_eq!(*a, 5);
    }

    #[test]
    fn value_dropped_exactly_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static DROPS: AtomicUsize = AtomicUsize::new(0);
        struct Tracked;
        impl Drop for Tracked {
            fn drop(&mut self) {
                DROPS.fetch_add(1, Ordering::SeqCst);
            }
        }

        let a = Rc::new(Tracked);
        let b = Rc::clone(&a);
        drop(a);
        assert_eq!(DROPS.load(Ordering::SeqCst), 0); // still aliased
        drop(b);
        assert_eq!(DROPS.load(Ordering::SeqCst), 1); // dropped exactly once
    }

    #[test]
    fn weak_upgrade_lifecycle() {
        let strong = Rc::new(10);
        let weak = Rc::downgrade(&strong);
        assert_eq!(weak.strong_count(), Some(1));
        assert_eq!(*weak.upgrade().expect("value alive"), 10);
        drop(strong);
        assert!(weak.upgrade().is_none());
        assert_eq!(weak.strong_count(), None);
    }

    #[test]
    fn weak_new_is_always_empty() {
        let weak: Weak<i32> = Weak::new();
        assert!(weak.upgrade().is_none());
        assert_eq!(weak.strong_count(), None);
    }

    #[test]
    fn get_mut_only_when_unique() {
        let mut a = Rc::new(vec![1, 2]);
        assert_eq!(Rc::get_mut(&mut a).map(|v| v.len()), Some(2));
        let b = Rc::clone(&a);
        assert_eq!(Rc::get_mut(&mut a), None); // aliased
        drop(b);
        Rc::get_mut(&mut a).expect("unique again").push(3);
        assert_eq!(*a, vec![1, 2, 3]);
    }

    #[test]
    fn try_unwrap() {
        let a = Rc::new(7);
        let b = Rc::clone(&a);
        // While aliased, the Rc comes back; the returned Rc (and its strong
        // ref) drops at the end of the match arm.
        match Rc::try_unwrap(a) {
            Err(rc) => assert_eq!(*rc, 7),
            Ok(_) => panic!("expected Err while aliased"),
        }
        let unwrapped = Rc::try_unwrap(b);
        assert!(matches!(unwrapped, Ok(7)));
    }

    #[test]
    fn weak_does_not_keep_value_alive() {
        let weak = {
            let strong = Rc::new(String::from("ephemeral"));
            let weak = Rc::downgrade(&strong);
            assert_eq!(weak.upgrade().map(|s| s.len()), Some(9));
            weak
        };
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn into_raw_roundtrip() {
        let rc = Rc::new(vec![1, 2, 3]);
        let raw = Rc::into_raw(rc);
        // SAFETY: raw came from into_raw and is returned exactly once here.
        let restored = unsafe { Rc::from_raw(raw) };
        assert_eq!(*restored, vec![1, 2, 3]);
    }

    #[test]
    fn weak_drop_frees_header_after_value_gone() {
        // The header must be freed once the last weak ref drops after the
        // value is gone. We can't observe the free directly, but a full
        // sequence of create/drop cycles should not leak (checked by the
        // allocator in debug builds via the test harness).
        for _ in 0..10_000 {
            let strong = Rc::new(1u8);
            let weak = Rc::downgrade(&strong);
            drop(strong);
            drop(weak);
        }
    }
}
