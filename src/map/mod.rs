//! A hash map with separate chaining.
//!
//! Each bucket is a singly-linked list of `Bucket<K, V>` entries. Keys are
//! hashed with a `BuildHasher`; the bucket index is `hash % num_buckets`. The
//! bucket array grows when load factor exceeds `MAX_LOAD_FACTOR` and shrinks
//! when it falls below `MIN_LOAD_FACTOR`.
//!
//! # Complexity
//!
//! | Operation | Average | Worst |
//! |-----------|---------|-------|
//! | `insert`     | O(1)*   | O(n) |
//! | `get`        | O(1)*   | O(n) |
//! | `remove`     | O(1)*   | O(n) |
//! | `contains_key` | O(1)* | O(n) |
//!
//! *Amortized.

use std::alloc::{self, Layout};
use std::fmt;
use std::hash::{BuildHasher, DefaultHasher, Hash};
use std::ptr::NonNull;

const INITIAL_BUCKETS: usize = 8;
// Load-factor thresholds expressed as exact fractions to avoid float casts:
// grow above 3/4, shrink below 1/4.
const GROW_LOAD_DENOM: usize = 4;
const GROW_LOAD_NUM: usize = 3;

/// One entry inside a bucket chain.
struct Bucket<K, V> {
    hash: u64,
    key: K,
    value: V,
    next: Option<NonNull<Bucket<K, V>>>,
}

/// A bucket-array slot: the head link of one chain.
type BucketHead<K, V> = Option<NonNull<Bucket<K, V>>>;

/// A hash map with separate chaining.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::map::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("one", 1);
/// map.insert("two", 2);
/// assert_eq!(map.get(&"one"), Some(&1));
/// ```
pub struct HashMap<K, V, S = DefaultHasherBuilder> {
    buckets: Option<NonNull<BucketHead<K, V>>>,
    num_buckets: usize,
    len: usize,
    hasher: S,
}

impl<K, V> HashMap<K, V, DefaultHasherBuilder>
where
    K: Hash + Eq,
{
    /// Creates an empty hash map. O(1).
    #[must_use]
    pub fn new() -> Self {
        Self::with_hasher(DefaultHasherBuilder)
    }
}

impl<K, V> Default for HashMap<K, V, DefaultHasherBuilder>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self {
            buckets: None,
            num_buckets: 0,
            len: 0,
            hasher: DefaultHasherBuilder,
        }
    }
}

// The `hash as usize` casts below intentionally truncate on 32-bit targets;
// bucket indices wrap modulo `num_buckets`, so the truncation is harmless.
#[allow(clippy::cast_possible_truncation)]
impl<K, V, S> HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Creates an empty hash map with a custom hasher. O(1).
    #[must_use]
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            buckets: None,
            num_buckets: 0,
            len: 0,
            hasher,
        }
    }

    /// Number of key-value pairs. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// `true` if the map has no entries. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the map contains `key`. Average O(1).
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Returns a reference to the value associated with `key`. Average O(1).
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash_key(key);
        let node = self.find_node(key, hash)?;
        // SAFETY: node is a live entry owned by the map.
        Some(unsafe { &(*node.as_ptr()).value })
    }

    /// Returns a mutable reference to the value associated with `key`. Average O(1).
    #[must_use]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = self.hash_key(key);
        let node = self.find_node(key, hash)?;
        // SAFETY: exclusive borrow via &mut self.
        Some(unsafe { &mut (*node.as_ptr()).value })
    }

    /// Inserts a key-value pair, returning the old value if `key` existed. Average O(1).
    ///
    /// # Panics
    ///
    /// Only on internal invariant violation: the bucket array is guaranteed to
    /// have been allocated at the top of this method.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_none() {
            self.allocate_buckets(INITIAL_BUCKETS);
        }
        let hash = self.hash_key(&key);
        let index = (hash as usize) % self.num_buckets;

        // SAFETY: buckets is Some and index < num_buckets.
        let first_slot = unsafe { self.buckets.unwrap().as_ptr().add(index) };
        let mut cur = unsafe { *first_slot };
        while let Some(node) = cur {
            // SAFETY: node is live in the chain.
            let node_ref = unsafe { &mut *node.as_ptr() };
            if node_ref.hash == hash && node_ref.key == key {
                return Some(std::mem::replace(&mut node_ref.value, value));
            }
            cur = node_ref.next;
        }

        // Insert new entry at the head of the bucket chain, linking to the
        // old head so colliding entries are not lost.
        let mut new_boxed = Box::new(Bucket {
            hash,
            key,
            value,
            next: None,
        });
        // SAFETY: first_slot is a valid read slot.
        new_boxed.next = unsafe { *first_slot };
        // SAFETY: Box::into_raw always returns a non-null pointer.
        let new_node = unsafe { NonNull::new_unchecked(Box::into_raw(new_boxed)) };
        // SAFETY: first_slot is a valid write slot for an Option<NonNull<...>>.
        unsafe { *first_slot = Some(new_node) };
        self.len += 1;
        self.maybe_grow();
        None
    }

    /// Removes a key, returning its value if present. Average O(1).
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let buckets = self.buckets?;
        let hash = self.hash_key(key);
        let index = (hash as usize) % self.num_buckets;
        // SAFETY: index < num_buckets.
        let head_slot = unsafe { buckets.as_ptr().add(index) };
        let head = unsafe { &mut *head_slot };

        // Track the slot holding the *current* node's next-link so we can
        // unlink by overwriting that slot.
        let mut prev_slot: *mut BucketHead<K, V> = head_slot;
        let mut cur = *head;
        while let Some(node) = cur {
            // SAFETY: node is live.
            let node_ref = unsafe { &*node.as_ptr() };
            if node_ref.hash == hash && node_ref.key == *key {
                // SAFETY: same node, mutable view for next field.
                let next = unsafe { (*node.as_ptr()).next };
                // SAFETY: prev_slot is a valid write slot.
                unsafe { *prev_slot = next };
                self.len -= 1;
                self.maybe_shrink();
                // SAFETY: node is unlinked and was allocated via Box::into_raw.
                return Some(unsafe { Box::from_raw(node.as_ptr()).value });
            }
            // SAFETY: node is live.
            prev_slot = unsafe { &raw mut (*node.as_ptr()).next };
            cur = node_ref.next;
        }
        None
    }

    /// Drops all entries but keeps the bucket array. O(n).
    pub fn clear(&mut self) {
        if let Some(buckets) = self.buckets {
            for i in 0..self.num_buckets {
                // SAFETY: i < num_buckets.
                let first = unsafe { &mut *buckets.as_ptr().add(i) };
                let mut cur = *first;
                while let Some(node) = cur {
                    // SAFETY: node is live.
                    let next = unsafe { (*node.as_ptr()).next };
                    // SAFETY: node came from Box::into_raw.
                    let _ = unsafe { Box::from_raw(node.as_ptr()) };
                    cur = next;
                }
                *first = None;
            }
        }
        self.len = 0;
    }

    /// Returns an iterator over (`&K`, `&V`) entries. O(1) to create.
    pub fn iter(&self) -> Iter<'_, K, V, S> {
        let current = self.first_non_empty_bucket(0);
        Iter {
            map: self,
            bucket_index: 0,
            current,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns a mutable iterator. O(1) to create.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, S> {
        let current = self.first_non_empty_bucket(0);
        IterMut {
            map: self,
            bucket_index: 0,
            current,
            _marker: std::marker::PhantomData,
        }
    }

    fn hash_key(&self, key: &K) -> u64 {
        self.hasher.hash_one(key)
    }

    /// Finds the bucket node matching `key` with the given hash.
    fn find_node(&self, key: &K, hash: u64) -> Option<NonNull<Bucket<K, V>>> {
        let buckets = self.buckets?;
        let index = (hash as usize) % self.num_buckets;
        // SAFETY: index < num_buckets.
        let mut cur = unsafe { *buckets.as_ptr().add(index) };
        while let Some(node) = cur {
            // SAFETY: node is live.
            let node_ref = unsafe { &*node.as_ptr() };
            if node_ref.hash == hash && node_ref.key == *key {
                return Some(node);
            }
            cur = node_ref.next;
        }
        None
    }

    /// Finds the first non-empty bucket starting at `start`.
    fn first_non_empty_bucket(&self, start: usize) -> Option<NonNull<Bucket<K, V>>> {
        let buckets = self.buckets?;
        for i in start..self.num_buckets {
            // SAFETY: i < num_buckets.
            let first = unsafe { *buckets.as_ptr().add(i) };
            if let Some(node) = first {
                return Some(node);
            }
        }
        None
    }

    /// Grows the bucket array if the load factor exceeds the threshold.
    fn maybe_grow(&mut self) {
        if self.num_buckets == 0 {
            return;
        }
        // Equivalent to `len / num_buckets > 3/4` without float casts.
        if self.len.saturating_mul(GROW_LOAD_DENOM) > self.num_buckets * GROW_LOAD_NUM {
            self.rehash(self.num_buckets * 2);
        }
    }

    /// Shrinks the bucket array if the load factor falls below the threshold.
    fn maybe_shrink(&mut self) {
        if self.num_buckets <= INITIAL_BUCKETS {
            return;
        }
        // Equivalent to `len / num_buckets < 1/4` without float casts.
        if self.len.saturating_mul(4) < self.num_buckets {
            let new_size = (self.num_buckets / 2).max(INITIAL_BUCKETS);
            self.rehash(new_size);
        }
    }

    /// Allocates a fresh bucket array of `size` linked-list heads.
    // The cast from `*mut u8` to the typed pointer is sound: `alloc_zeroed`
    // guarantees alignment for `layout`.
    #[allow(clippy::cast_ptr_alignment)]
    fn allocate_buckets(&mut self, size: usize) {
        let layout = Layout::array::<BucketHead<K, V>>(size).expect("bucket layout error");
        // SAFETY: layout has size > 0 because size > 0; alloc_zeroed guarantees
        // alignment for the layout, so the cast to the typed pointer is sound.
        let raw = unsafe { alloc::alloc_zeroed(layout) };
        let Some(ptr) = NonNull::new(raw.cast::<BucketHead<K, V>>()) else {
            alloc::handle_alloc_error(layout);
        };
        self.buckets = Some(ptr);
        self.num_buckets = size;
    }

    /// Rebuilds the bucket array with `new_size` buckets, re-hashing every entry.
    fn rehash(&mut self, new_size: usize) {
        let old_buckets = self.buckets;
        let old_num = self.num_buckets;
        self.allocate_buckets(new_size);
        let mut moved = 0;

        if let Some(old_buckets) = old_buckets {
            for i in 0..old_num {
                // SAFETY: i < old_num.
                let mut cur = unsafe { *old_buckets.as_ptr().add(i) };
                while let Some(node) = cur {
                    // SAFETY: node is live.
                    let raw = node.as_ptr();
                    let hash = unsafe { (*raw).hash };
                    let key = unsafe { std::ptr::read(std::ptr::addr_of!((*raw).key)) };
                    let value = unsafe { std::ptr::read(std::ptr::addr_of!((*raw).value)) };
                    let next = unsafe { (*raw).next };
                    // SAFETY: box dealloc without dropping K/V because we moved them out.
                    unsafe {
                        let layout = Layout::new::<Bucket<K, V>>();
                        alloc::dealloc(raw.cast::<u8>(), layout);
                    }

                    // Insert into the new bucket array.
                    let new_index = (hash as usize) % self.num_buckets;
                    let new_first = unsafe { self.buckets.unwrap().as_ptr().add(new_index) };
                    // SAFETY: Box::into_raw always returns a non-null pointer.
                    let moved_raw = unsafe {
                        NonNull::new_unchecked(Box::into_raw(Box::new(Bucket {
                            hash,
                            key,
                            value,
                            next: *new_first,
                        })))
                    };
                    // SAFETY: new_first is a valid write slot.
                    unsafe { *new_first = Some(moved_raw) };
                    moved += 1;
                    cur = next;
                }
            }
            let layout = Layout::array::<BucketHead<K, V>>(old_num).expect("bucket layout error");
            // SAFETY: layout matches the allocation being freed.
            unsafe { alloc::dealloc(old_buckets.as_ptr().cast::<u8>(), layout) };
        }
        self.len = moved;
    }
}

impl<K, V, S> Drop for HashMap<K, V, S> {
    fn drop(&mut self) {
        if let Some(buckets) = self.buckets.take() {
            // Drop all entries first.
            for i in 0..self.num_buckets {
                // SAFETY: i < num_buckets.
                let first = unsafe { &mut *buckets.as_ptr().add(i) };
                let mut cur = *first;
                while let Some(node) = cur {
                    // SAFETY: node is live.
                    let next = unsafe { (*node.as_ptr()).next };
                    // SAFETY: Box::from_raw recovers the box, which runs K/V drop.
                    let _ = unsafe { Box::from_raw(node.as_ptr()) };
                    cur = next;
                }
            }
            let layout =
                Layout::array::<BucketHead<K, V>>(self.num_buckets).expect("bucket layout error");
            // SAFETY: layout matches the allocation.
            unsafe { alloc::dealloc(buckets.as_ptr().cast::<u8>(), layout) };
            self.num_buckets = 0;
        }
    }
}

impl<K, V, S> fmt::Debug for HashMap<K, V, S>
where
    K: Hash + Eq + fmt::Debug,
    V: fmt::Debug,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

// SAFETY: HashMap owns its nodes via raw pointers; safe to send/sync with T.
unsafe impl<K: Send, V: Send, S: Send> Send for HashMap<K, V, S> {}
unsafe impl<K: Sync, V: Sync, S: Sync> Sync for HashMap<K, V, S> {}

impl<K, V, S> FromIterator<(K, V)> for HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = Self::with_hasher(S::default());
        map.extend(iter);
        map
    }
}

impl<K, V, S> Extend<(K, V)> for HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

/// Shared iterator over (`&K`, `&V`) entries.
pub struct Iter<'a, K, V, S = DefaultHasherBuilder> {
    map: &'a HashMap<K, V, S>,
    bucket_index: usize,
    current: Option<NonNull<Bucket<K, V>>>,
    _marker: std::marker::PhantomData<S>,
}

impl<'a, K, V, S> Iterator for Iter<'a, K, V, S> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(node) = self.current {
                // SAFETY: node is live in the map; lifetime tied to &map.
                let node_ref = unsafe { &*node.as_ptr() };
                self.current = node_ref.next;
                return Some((&node_ref.key, &node_ref.value));
            }
            self.bucket_index += 1;
            if self.bucket_index >= self.map.num_buckets {
                return None;
            }
            // SAFETY: bucket_index < num_buckets.
            let head = unsafe { *self.map.buckets.unwrap().as_ptr().add(self.bucket_index) };
            self.current = head;
        }
    }
}

/// Mutable iterator over (`&K`, `&mut V`) entries.
pub struct IterMut<'a, K, V, S = DefaultHasherBuilder> {
    map: &'a mut HashMap<K, V, S>,
    bucket_index: usize,
    current: Option<NonNull<Bucket<K, V>>>,
    _marker: std::marker::PhantomData<S>,
}

impl<'a, K, V, S> Iterator for IterMut<'a, K, V, S> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(node) = self.current {
                // SAFETY: exclusive borrow via &mut map.
                let node_ref = unsafe { &mut *node.as_ptr() };
                let next = node_ref.next;
                self.current = next;
                return Some((&node_ref.key, &mut node_ref.value));
            }
            self.bucket_index += 1;
            if self.bucket_index >= self.map.num_buckets {
                return None;
            }
            // SAFETY: bucket_index < num_buckets.
            let head = unsafe { *self.map.buckets.unwrap().as_ptr().add(self.bucket_index) };
            self.current = head;
        }
    }
}

// ---------------------------------------------------------------------------
// Default hasher builder
// ---------------------------------------------------------------------------

/// A `BuildHasher` that produces `DefaultHasher`s from `std::hash`.
#[derive(Clone, Copy, Default, Debug)]
pub struct DefaultHasherBuilder;

impl BuildHasher for DefaultHasherBuilder {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut map = HashMap::new();
        assert_eq!(map.insert("a", 1), None);
        assert_eq!(map.insert("b", 2), None);
        assert_eq!(map.get(&"a"), Some(&1));
        assert_eq!(map.get(&"b"), Some(&2));
        assert_eq!(map.get(&"c"), None);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn insert_overwrites() {
        let mut map = HashMap::new();
        assert_eq!(map.insert("x", 1), None);
        assert_eq!(map.insert("x", 2), Some(1));
        assert_eq!(map.get(&"x"), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove() {
        let mut map = HashMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        assert_eq!(map.remove(&"a"), Some(1));
        assert_eq!(map.get(&"a"), None);
        assert_eq!(map.remove(&"a"), None);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn growth_trigger() {
        let mut map = HashMap::new();
        for i in 0..1000 {
            map.insert(i, i * 10);
        }
        assert_eq!(map.len(), 1000);
        for i in 0..1000 {
            assert_eq!(map.get(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn shrink_trigger() {
        let mut map = HashMap::new();
        for i in 0..100 {
            map.insert(i, i);
        }
        for i in 0..80 {
            map.remove(&i);
        }
        assert_eq!(map.len(), 20);
        for i in 80..100 {
            assert_eq!(map.get(&i), Some(&i));
        }
    }

    #[test]
    fn contains_key() {
        let mut map = HashMap::new();
        map.insert(1u32, "one");
        assert!(map.contains_key(&1));
        assert!(!map.contains_key(&2));
    }

    #[test]
    fn iter_visits_all_entries() {
        let mut map = HashMap::new();
        for i in 0..50 {
            map.insert(i, i);
        }
        let collected: std::collections::HashMap<i32, i32> =
            map.iter().map(|(k, v)| (*k, *v)).collect();
        for i in 0..50 {
            assert_eq!(collected.get(&i), Some(&i));
        }
    }

    #[test]
    fn get_mut_updates() {
        let mut map = HashMap::new();
        map.insert("k", 1);
        if let Some(v) = map.get_mut(&"k") {
            *v = 42;
        }
        assert_eq!(map.get(&"k"), Some(&42));
    }

    #[test]
    fn clear_resets_length() {
        let mut map = HashMap::new();
        for i in 0..100 {
            map.insert(i, i);
        }
        map.clear();
        assert!(map.is_empty());
        for i in 0..100 {
            assert_eq!(map.get(&i), None);
        }
    }
}
