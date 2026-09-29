//! A binary search tree (BST) with iterators.
//!
//! Invariant: for every node, all values in the left subtree are less than the
//! node's value, and all values in the right subtree are greater. This is a
//! *plain* BST — no balancing — so worst-case height is O(n) for sorted input.
//! Use [`AvlTree`](super::avl::AvlTree) when balanced worst cases matter.
//!
//! # Complexity (average case)
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
//! use rust_learning_and_dsa::tree::BinarySearchTree;
//!
//! let mut tree = BinarySearchTree::new();
//! tree.insert(5);
//! tree.insert(3);
//! tree.insert(7);
//! assert!(tree.contains(&5));
//! assert!(!tree.contains(&4));
//! assert_eq!(tree.in_order().collect::<Vec<_>>(), vec![&3, &5, &7]);
//! ```

use std::cmp::Ordering;
use std::fmt;

/// Internal BST node.
struct Node<T> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }
}

/// A binary search tree storing unique values.
pub struct BinarySearchTree<T> {
    root: Option<Box<Node<T>>>,
    len: usize,
}

impl<T: Ord> BinarySearchTree<T> {
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

    /// Inserts `value`. Returns `true` if it was newly inserted, `false` if it
    /// already existed (duplicates are rejected). O(h), where h is tree height.
    pub fn insert(&mut self, value: T) -> bool {
        let mut link = &mut self.root;
        loop {
            match link {
                None => {
                    *link = Some(Box::new(Node::new(value)));
                    self.len += 1;
                    return true;
                }
                Some(node) => match value.cmp(&node.value) {
                    Ordering::Less => link = &mut node.left,
                    Ordering::Greater => link = &mut node.right,
                    Ordering::Equal => return false,
                },
            }
        }
    }

    /// Returns `true` if `value` is present. O(log n).
    #[must_use]
    pub fn contains(&self, value: &T) -> bool {
        self.find(value).is_some()
    }

    /// Reference to the smallest value, or `None` if empty. O(log n).
    #[must_use]
    pub fn min(&self) -> Option<&T> {
        let mut cur = self.root.as_deref();
        while let Some(node) = cur {
            if node.left.is_some() {
                cur = node.left.as_deref();
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
            if node.right.is_some() {
                cur = node.right.as_deref();
            } else {
                return Some(&node.value);
            }
        }
        None
    }

    /// Removes `value`. Returns `true` if it was present. O(h), where h is
    /// tree height. This recursive implementation can exhaust the call stack
    /// on a sufficiently deep, skewed tree.
    pub fn remove(&mut self, value: &T) -> bool {
        if remove_recursive(&mut self.root, value) {
            self.len -= 1;
            true
        } else {
            false
        }
    }

    /// Clears the tree. O(n).
    pub fn clear(&mut self) {
        // Detach children before each node is dropped, so a skewed tree does
        // not trigger recursive Box destruction and overflow the stack.
        let mut pending = Vec::new();
        if let Some(root) = self.root.take() {
            pending.push(root);
        }
        while let Some(mut node) = pending.pop() {
            if let Some(left) = node.left.take() {
                pending.push(left);
            }
            if let Some(right) = node.right.take() {
                pending.push(right);
            }
        }
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
    pub fn in_order(&self) -> InOrder<'_, T> {
        InOrder {
            stack: Vec::new(),
            current: self.root.as_deref(),
        }
    }

    /// Pre-order iterator over shared references. O(1) to create.
    #[must_use]
    pub fn pre_order(&self) -> PreOrder<'_, T> {
        PreOrder {
            stack: self.root.as_deref().into_iter().collect(),
        }
    }

    /// Post-order iterator over shared references. O(1) to create.
    #[must_use]
    pub fn post_order(&self) -> PostOrder<'_, T> {
        PostOrder {
            stack: vec![(self.root.as_deref(), false)],
        }
    }

    /// Finds the node containing `value`. O(log n).
    fn find(&self, value: &T) -> Option<&Node<T>> {
        let mut cur = self.root.as_deref();
        while let Some(node) = cur {
            match value.cmp(&node.value) {
                Ordering::Less => cur = node.left.as_deref(),
                Ordering::Greater => cur = node.right.as_deref(),
                Ordering::Equal => return Some(node),
            }
        }
        None
    }
}

impl<T: Ord> Default for BinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for BinarySearchTree<T> {
    fn drop(&mut self) {
        // This cannot call clear because clear lives in the Ord impl.
        let mut pending = Vec::new();
        if let Some(root) = self.root.take() {
            pending.push(root);
        }
        while let Some(mut node) = pending.pop() {
            if let Some(left) = node.left.take() {
                pending.push(left);
            }
            if let Some(right) = node.right.take() {
                pending.push(right);
            }
        }
    }
}

impl<T: Ord + Clone> Clone for BinarySearchTree<T> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for value in self.in_order() {
            new.insert(value.clone());
        }
        new
    }
}

impl<T: Ord + fmt::Debug> fmt::Debug for BinarySearchTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.in_order()).finish()
    }
}

impl<T: Ord> FromIterator<T> for BinarySearchTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

impl<T: Ord> Extend<T> for BinarySearchTree<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

/// Recursive removal. Returns `true` if a node was removed.
///
/// Cases:
/// - leaf: detach the node;
/// - one child: splice the child into the node's place;
/// - two children: replace the value with the in-order successor (leftmost
///   node of the right subtree), then remove the successor.
fn remove_recursive<T: Ord>(node: &mut Option<Box<Node<T>>>, value: &T) -> bool {
    let Some(mut this) = node.take() else {
        return false;
    };

    match value.cmp(&this.value) {
        Ordering::Less => {
            let removed = remove_recursive(&mut this.left, value);
            *node = Some(this);
            removed
        }
        Ordering::Greater => {
            let removed = remove_recursive(&mut this.right, value);
            *node = Some(this);
            removed
        }
        Ordering::Equal => {
            match (this.left.take(), this.right.take()) {
                // Leaf: nothing to re-link.
                (None, None) => {}
                // One child: promote it.
                (Some(left), None) => {
                    *node = Some(left);
                    return true;
                }
                (None, Some(right)) => {
                    *node = Some(right);
                    return true;
                }
                // Two children: take the successor's value out of the right
                // subtree and write it into this node.
                (Some(left), Some(right)) => {
                    let (successor_value, remaining_right) = take_leftmost_value(right);
                    this.value = successor_value;
                    this.left = Some(left);
                    this.right = remaining_right;
                    *node = Some(this);
                }
            }
            true
        }
    }
}

/// Removes the leftmost node from the subtree rooted at `root`, returning its
/// value along with the remaining (re-linked) subtree. O(log n).
fn take_leftmost_value<T>(mut root: Box<Node<T>>) -> (T, Option<Box<Node<T>>>) {
    // If the root has no left child, it *is* the leftmost node.
    if root.left.is_none() {
        return (root.value, root.right);
    }
    // Walk down the left spine; `link` points at the leftmost node's slot.
    let mut link = &mut root.left;
    while link.as_ref().is_some_and(|n| n.left.is_some()) {
        link = &mut link.as_mut().expect("checked above").left;
    }
    // `link` now points at the leftmost node; take it and promote its right child.
    let leftmost = link.take().expect("checked above");
    *link = leftmost.right;
    (leftmost.value, Some(root))
}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

/// In-order (sorted) iterator for [`BinarySearchTree`].
pub struct InOrder<'a, T> {
    stack: Vec<&'a Node<T>>,
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for InOrder<'a, T> {
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

/// Pre-order iterator for [`BinarySearchTree`].
pub struct PreOrder<'a, T> {
    stack: Vec<&'a Node<T>>,
}

impl<'a, T> Iterator for PreOrder<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        if let Some(right) = node.right.as_deref() {
            self.stack.push(right);
        }
        if let Some(left) = node.left.as_deref() {
            self.stack.push(left);
        }
        Some(&node.value)
    }
}

/// Post-order iterator for [`BinarySearchTree`].
pub struct PostOrder<'a, T> {
    /// `(node, visited)` — `visited` means children were already pushed.
    stack: Vec<(Option<&'a Node<T>>, bool)>,
}

impl<'a, T> Iterator for PostOrder<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (node, visited) = self.stack.pop()?;
            let node = node?;
            if visited {
                return Some(&node.value);
            }
            self.stack.push((Some(node), true));
            if let Some(right) = node.right.as_deref() {
                self.stack.push((Some(right), false));
            }
            if let Some(left) = node.left.as_deref() {
                self.stack.push((Some(left), false));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skewed_tree_can_be_cleared_and_dropped() {
        let mut tree = BinarySearchTree::new();
        for value in 0..10_000 {
            assert!(tree.insert(value));
        }
        assert_eq!(tree.max(), Some(&9_999));
        tree.clear();
        assert!(tree.is_empty());
        for value in 0..10_000 {
            tree.insert(value);
        }
        // Drop happens here and must not recurse through the right spine.
    }

    fn sample_tree() -> BinarySearchTree<i32> {
        let mut tree = BinarySearchTree::new();
        for value in [5, 3, 7, 2, 4, 6, 8] {
            tree.insert(value);
        }
        tree
    }

    #[test]
    fn insert_and_contains() {
        let mut tree = BinarySearchTree::new();
        assert!(tree.insert(5));
        assert!(tree.insert(3));
        assert!(!tree.insert(5)); // duplicate rejected
        assert_eq!(tree.len(), 2);
        assert!(tree.contains(&3));
        assert!(!tree.contains(&4));
    }

    #[test]
    fn in_order_is_sorted() {
        let tree = sample_tree();
        assert_eq!(
            tree.in_order().copied().collect::<Vec<_>>(),
            vec![2, 3, 4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn pre_and_post_order() {
        let tree = sample_tree();
        assert_eq!(
            tree.pre_order().copied().collect::<Vec<_>>(),
            vec![5, 3, 2, 4, 7, 6, 8]
        );
        assert_eq!(
            tree.post_order().copied().collect::<Vec<_>>(),
            vec![2, 4, 3, 6, 8, 7, 5]
        );
    }

    #[test]
    fn min_and_max() {
        let tree = sample_tree();
        assert_eq!(tree.min(), Some(&2));
        assert_eq!(tree.max(), Some(&8));
        let empty: BinarySearchTree<i32> = BinarySearchTree::new();
        assert_eq!(empty.min(), None);
    }

    #[test]
    fn remove_leaf() {
        let mut tree = sample_tree();
        assert!(tree.remove(&2));
        assert!(!tree.contains(&2));
        assert_eq!(tree.len(), 6);
    }

    #[test]
    fn remove_one_child() {
        let mut tree = BinarySearchTree::new();
        for value in [5, 3, 2] {
            tree.insert(value);
        }
        assert!(tree.remove(&3));
        assert!(!tree.contains(&3));
        assert_eq!(tree.in_order().copied().collect::<Vec<_>>(), vec![2, 5]);
    }

    #[test]
    fn remove_two_children() {
        let mut tree = sample_tree();
        assert!(tree.remove(&3));
        assert!(!tree.contains(&3));
        // Successor (4) should have taken 3's place.
        assert_eq!(
            tree.in_order().copied().collect::<Vec<_>>(),
            vec![2, 4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn remove_root_and_missing() {
        let mut tree = sample_tree();
        assert!(tree.remove(&5));
        assert!(!tree.remove(&99));
        assert_eq!(
            tree.in_order().copied().collect::<Vec<_>>(),
            vec![2, 3, 4, 6, 7, 8]
        );
    }

    #[test]
    fn remove_to_empty() {
        let mut tree = BinarySearchTree::new();
        for value in [1, 2, 3] {
            tree.insert(value);
        }
        tree.remove(&1);
        tree.remove(&2);
        tree.remove(&3);
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn stress_randomized() {
        use std::collections::BTreeSet;
        let mut tree = BinarySearchTree::new();
        let mut set = BTreeSet::new();
        // Insert 1000 distinct values, then remove every other one (by sort index).
        for i in 0..1000u32 {
            let value = i.wrapping_mul(2_654_435_761).wrapping_add(12345);
            assert_eq!(tree.insert(value), set.insert(value));
        }
        // Snapshot the set so we can mutate both structures during the loop.
        let snapshot: Vec<u32> = set.iter().copied().collect();
        for (i, value) in snapshot.into_iter().enumerate() {
            if i % 2 == 0 {
                assert!(tree.remove(&value));
                set.remove(&value);
            }
        }
        // `set` now contains exactly the values that were kept.
        let remaining: Vec<u32> = tree.in_order().copied().collect();
        let expected: Vec<u32> = set.iter().copied().collect();
        assert_eq!(remaining, expected);
        assert_eq!(tree.len(), expected.len());
    }
}
