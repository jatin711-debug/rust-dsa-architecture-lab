//! A self-balancing AVL tree.
//!
//! After every insert/remove, the tree rebalances so that for every node the
//! heights of its left and right subtrees differ by at most one. This keeps
//! the height at O(log n) even for sorted input, guaranteeing O(log n) for all
//! operations.
//!
//! # Complexity (worst case)
//!
//! | Operation | Time |
//! |-----------|------|
//! | `insert`  | O(log n) |
//! | `remove`  | O(log n) |
//! | `contains`| O(log n) |
//! | `min`/`max` | O(log n) |
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::tree::AvlTree;
//!
//! let mut tree = AvlTree::new();
//! for value in 0..100 {
//!     tree.insert(value);
//! }
//! assert_eq!(tree.len(), 100);
//! assert!(tree.contains(&42));
//! // Sorted input still yields a balanced tree: height stays O(log n).
//! assert!(tree.height() <= 2 * (tree.len() as f64).log2() as usize + 1);
//! ```

use std::cmp::Ordering;
use std::fmt;

/// Internal AVL node.
struct Node<T> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
    /// Height of this subtree: longest path to a leaf. Leaf = 1.
    height: usize,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
            height: 1,
        }
    }
}

/// In-order (sorted) iterator over the tree's values.
pub struct AvlInOrder<'a, T> {
    stack: Vec<&'a Node<T>>,
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for AvlInOrder<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // Descend to the leftmost node, pushing visited nodes on the stack.
        while let Some(node) = self.current {
            self.stack.push(node);
            self.current = node.left.as_deref();
        }
        // Pop and yield, then descend into the right subtree.
        let node = self.stack.pop()?;
        self.current = node.right.as_deref();
        Some(&node.value)
    }
}

/// A self-balancing AVL tree storing unique values.
pub struct AvlTree<T> {
    root: Option<Box<Node<T>>>,
    len: usize,
}

impl<T: Ord> AvlTree<T> {
    /// Creates an empty tree. O(1).
    #[must_use]
    pub const fn new() -> Self {
        Self { root: None, len: 0 }
    }

    /// Number of values in the tree. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// `true` if the tree is empty. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Height of the tree (0 if empty). O(1).
    #[must_use]
    pub fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |n| n.height)
    }

    /// Inserts `value`. Returns `true` if newly inserted, `false` if duplicate.
    /// O(log n).
    pub fn insert(&mut self, value: T) -> bool {
        let (root, inserted) = avl_insert(self.root.take(), value);
        self.root = root;
        if inserted {
            self.len += 1;
        }
        inserted
    }

    /// Returns `true` if `value` is present. O(log n).
    #[must_use]
    pub fn contains(&self, value: &T) -> bool {
        let mut cur = self.root.as_deref();
        while let Some(node) = cur {
            match value.cmp(&node.value) {
                Ordering::Less => cur = node.left.as_deref(),
                Ordering::Greater => cur = node.right.as_deref(),
                Ordering::Equal => return true,
            }
        }
        false
    }

    /// Reference to the smallest value, or `None` if empty. O(log n).
    #[must_use]
    pub fn min(&self) -> Option<&T> {
        let mut cur = self.root.as_deref();
        while let Some(node) = cur {
            if let Some(left) = node.left.as_deref() {
                cur = Some(left);
            } else {
                return Some(&node.value);
            }
        }
        None
    }

    /// Reference to the largest value, or `None` if empty. O(log n).
    #[must_use]
    pub fn max(&self) -> Option<&T> {
        let mut cur = self.root.as_deref();
        while let Some(node) = cur {
            if let Some(right) = node.right.as_deref() {
                cur = Some(right);
            } else {
                return Some(&node.value);
            }
        }
        None
    }

    /// Removes `value`. Returns `true` if it was present. O(log n).
    pub fn remove(&mut self, value: &T) -> bool {
        let (root, removed) = avl_remove(self.root.take(), value);
        self.root = root;
        if removed {
            self.len -= 1;
        }
        removed
    }

    /// Clears the tree. O(n).
    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    /// Flattens the tree structure into `(value, left_value, right_value)`
    /// triples (children by value, `None` for missing). Used by the WASM
    /// visualizer to snapshot the tree after each mutation. O(n).
    #[must_use]
    pub fn structure(&self) -> Vec<(T, Option<T>, Option<T>)>
    where
        T: Ord + Copy,
    {
        fn walk<T: Ord + Copy>(node: Option<&Node<T>>, out: &mut Vec<(T, Option<T>, Option<T>)>) {
            if let Some(n) = node {
                out.push((
                    n.value,
                    n.left.as_deref().map(|l| l.value),
                    n.right.as_deref().map(|r| r.value),
                ));
                walk(n.left.as_deref(), out);
                walk(n.right.as_deref(), out);
            }
        }
        let mut out = Vec::new();
        walk(self.root.as_deref(), &mut out);
        out
    }

    /// In-order (sorted) iterator over shared references. O(1) to create.
    #[must_use]
    pub fn in_order(&self) -> AvlInOrder<'_, T> {
        AvlInOrder {
            stack: Vec::new(),
            current: self.root.as_deref(),
        }
    }

    /// Verifies the AVL invariant (balance factor in {-1, 0, 1}) for testing.
    #[cfg(test)]
    fn is_balanced(&self) -> bool {
        fn check<T>(node: Option<&Node<T>>) -> bool {
            match node {
                None => true,
                Some(n) => {
                    let lh = height(n.left.as_deref());
                    let rh = height(n.right.as_deref());
                    lh.abs_diff(rh) <= 1
                        && n.height == 1 + lh.max(rh)
                        && check(n.left.as_deref())
                        && check(n.right.as_deref())
                }
            }
        }
        check(self.root.as_deref())
    }
}

impl<T: Ord> Default for AvlTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> Clone for AvlTree<T> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for value in self.in_order() {
            new.insert(value.clone());
        }
        new
    }
}

impl<T: Ord + fmt::Debug> fmt::Debug for AvlTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.in_order()).finish()
    }
}

impl<T: Ord> FromIterator<T> for AvlTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

impl<T: Ord> Extend<T> for AvlTree<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

// ---------------------------------------------------------------------------
// Core AVL operations
// ---------------------------------------------------------------------------

/// Height helper for `Option<Box<Node<T>>>`.
fn height<T>(node: Option<&Node<T>>) -> usize {
    node.map_or(0, |n| n.height)
}

/// True if `left` is strictly taller than `right` — no signed casts needed.
fn is_left_heavier<T>(node: &Node<T>) -> bool {
    height(node.left.as_deref()) > height(node.right.as_deref())
}

/// Recomputes `node.height` from its children's heights.
fn update_height<T>(node: &mut Node<T>) {
    node.height = 1 + height(node.left.as_deref()).max(height(node.right.as_deref()));
}

/// Rotates the subtree rooted at `root` to the right (left child becomes root).
// The Box return is required: the tree stores nodes as `Box<Node<T>>`.
#[allow(clippy::unnecessary_box_returns)]
fn rotate_right<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    let mut new_root = root.left.take().expect("rotate_right needs a left child");
    root.left = new_root.right.take();
    update_height(&mut root);
    new_root.right = Some(root);
    update_height(&mut new_root);
    new_root
}

/// Rotates the subtree rooted at `root` to the left (right child becomes root).
// The Box return is required: the tree stores nodes as `Box<Node<T>>`.
#[allow(clippy::unnecessary_box_returns)]
fn rotate_left<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    let mut new_root = root.right.take().expect("rotate_left needs a right child");
    root.right = new_root.left.take();
    update_height(&mut root);
    new_root.left = Some(root);
    update_height(&mut new_root);
    new_root
}

/// Rebalances the subtree rooted at `root` after a mutation. Returns the new
/// subtree root (may be a different node after rotation).
// The Box return is required: the tree stores nodes as `Box<Node<T>>`.
#[allow(clippy::unnecessary_box_returns)]
fn rebalance<T: Ord>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    update_height(&mut root);
    let left_tall = height(root.left.as_deref());
    let right_tall = height(root.right.as_deref());
    if left_tall > right_tall + 1 {
        // Left-heavy. If the left child is right-heavy (its right subtree is
        // taller), first fix with a left rotation on the child (LR case);
        // then rotate right.
        if !is_left_heavier(root.left.as_deref().expect("left-heavy")) {
            let left = root.left.take().expect("left exists");
            root.left = Some(rotate_left(left));
        }
        return rotate_right(root);
    }
    if right_tall > left_tall + 1 {
        // Right-heavy. If the right child is left-heavy (its left subtree is
        // taller), first fix with a right rotation on the child (RL case);
        // then rotate left.
        if is_left_heavier(root.right.as_deref().expect("right-heavy")) {
            let right = root.right.take().expect("right exists");
            root.right = Some(rotate_right(right));
        }
        return rotate_left(root);
    }
    root
}

/// Recursive AVL insert. Returns the (possibly rebalanced) subtree and whether
/// a new node was added.
fn avl_insert<T: Ord>(node: Option<Box<Node<T>>>, value: T) -> (Option<Box<Node<T>>>, bool) {
    let Some(mut this) = node else {
        return (Some(Box::new(Node::new(value))), true);
    };

    match value.cmp(&this.value) {
        Ordering::Less => {
            let (left, inserted) = avl_insert(this.left.take(), value);
            this.left = left;
            (Some(rebalance(this)), inserted)
        }
        Ordering::Greater => {
            let (right, inserted) = avl_insert(this.right.take(), value);
            this.right = right;
            (Some(rebalance(this)), inserted)
        }
        Ordering::Equal => (Some(this), false),
    }
}

/// Recursive AVL removal. Returns the (possibly rebalanced) subtree and whether
/// a node was removed.
fn avl_remove<T: Ord>(node: Option<Box<Node<T>>>, value: &T) -> (Option<Box<Node<T>>>, bool) {
    let Some(mut this) = node else {
        return (None, false);
    };

    match value.cmp(&this.value) {
        Ordering::Less => {
            let (left, removed) = avl_remove(this.left.take(), value);
            this.left = left;
            (Some(rebalance(this)), removed)
        }
        Ordering::Greater => {
            let (right, removed) = avl_remove(this.right.take(), value);
            this.right = right;
            (Some(rebalance(this)), removed)
        }
        Ordering::Equal => match (this.left.take(), this.right.take()) {
            // Leaf: nothing to re-link.
            (None, None) => (None, true),
            // One child: promote it.
            (Some(left), None) => (Some(left), true),
            (None, Some(right)) => (Some(right), true),
            // Two children: replace with the in-order successor.
            (Some(left), Some(right)) => {
                let (successor_value, remaining_right) = take_min(right);
                this.value = successor_value;
                this.left = Some(left);
                this.right = remaining_right;
                (Some(rebalance(this)), true)
            }
        },
    }
}

/// Removes the leftmost node from the subtree rooted at `root`, returning its
/// value and the re-linked remainder.
fn take_min<T>(mut root: Box<Node<T>>) -> (T, Option<Box<Node<T>>>) {
    if root.left.is_none() {
        return (root.value, root.right);
    }
    // Walk to the leftmost node using a &mut link, then take it and promote
    // its right child. The caller rebalances the parent afterwards.
    let mut link = &mut root.left;
    while link.as_ref().is_some_and(|n| n.left.is_some()) {
        link = &mut link.as_mut().expect("checked above").left;
    }
    let leftmost = link.take().expect("checked above");
    *link = leftmost.right;
    (leftmost.value, Some(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_duplicates_rejected() {
        let mut tree = AvlTree::new();
        assert!(tree.insert(5));
        assert!(!tree.insert(5));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn sorted_insert_stays_balanced() {
        let mut tree = AvlTree::new();
        for value in 0..10_000 {
            tree.insert(value);
        }
        assert_eq!(tree.len(), 10_000);
        assert!(tree.is_balanced());
        // AVL guarantee: height <= ~1.44 * log2(n); assert a generous bound.
        let bound = 2.0 * (10_000f64).log2() + 1.0;
        assert!(tree.height() as f64 <= bound, "height {}", tree.height());
    }

    #[test]
    fn random_insert_remove_stays_balanced() {
        let mut tree = AvlTree::new();
        let values: Vec<u32> = (0..5_000u32)
            .map(|i| i.wrapping_mul(2_654_435_761).wrapping_add(12_345))
            .collect();
        for &v in &values {
            tree.insert(v);
        }
        for (i, v) in values.into_iter().enumerate() {
            if i % 2 == 0 {
                assert!(tree.remove(&v));
            }
        }
        assert!(tree.is_balanced());
        assert_eq!(tree.len(), 2_500);
        // In-order must be sorted (sanity).
        let sorted: Vec<u32> = tree.in_order().copied().collect();
        assert!(sorted.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn min_max_height() {
        let mut tree = AvlTree::new();
        for value in [10, 5, 15, 3, 7] {
            tree.insert(value);
        }
        assert_eq!(tree.min(), Some(&3));
        assert_eq!(tree.max(), Some(&15));
        assert_eq!(tree.height(), 3);
    }

    #[test]
    fn remove_all_stays_balanced() {
        let mut tree = AvlTree::new();
        for value in 0..100 {
            tree.insert(value);
        }
        for value in 0..100 {
            assert!(tree.remove(&value));
            assert!(tree.is_balanced());
        }
        assert!(tree.is_empty());
        assert_eq!(tree.height(), 0);
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut tree = AvlTree::new();
        tree.insert(1);
        assert!(!tree.remove(&2));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn matches_btreeset() {
        use std::collections::BTreeSet;
        let mut tree = AvlTree::new();
        let mut set = BTreeSet::new();
        for i in 0..3_000u32 {
            let v = i.wrapping_mul(2_654_435_761).wrapping_add(7);
            assert_eq!(tree.insert(v), set.insert(v));
        }
        let snapshot: Vec<u32> = set.iter().copied().collect();
        for (i, v) in snapshot.into_iter().enumerate() {
            if i % 3 == 0 {
                assert_eq!(tree.remove(&v), set.remove(&v));
            }
        }
        assert_eq!(
            tree.in_order().copied().collect::<Vec<_>>(),
            set.iter().copied().collect::<Vec<_>>()
        );
    }
}
