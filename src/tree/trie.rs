//! A trie (prefix tree) for fast string prefix operations.
//!
//! Each node holds a map of `char -> child` plus a word count. A node is a
//! "word end" when `count > 0`. This design supports:
//!
//! - `insert` / `contains` in O(word length)
//! - `starts_with` (prefix existence) in O(prefix length)
//! - `autocomplete` in O(prefix + results)
//! - `remove` in O(word length) with cleanup of now-empty branches
//!
//! # Complexity
//!
//! | Operation | Time |
//! |-----------|------|
//! | `insert`  | O(L) |
//! | `contains`| O(L) |
//! | `starts_with` | O(L) |
//! | `remove`  | O(L) |
//! | `autocomplete` | O(L + K) |
//!
//! where `L` is the string length and `K` the number of results.
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::tree::Trie;
//!
//! let mut trie = Trie::new();
//! trie.insert("apple");
//! trie.insert("app");
//! assert!(trie.contains("apple"));
//! assert!(trie.starts_with("app"));
//! assert_eq!(trie.autocomplete("app"), vec!["app", "apple"]);
//! ```

use std::collections::HashMap;
use std::fmt;

/// A single trie node: children indexed by `char`, plus a word-end counter.
#[derive(Default)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    /// Number of words ending exactly at this node (supports duplicates).
    count: usize,
}

/// A prefix tree storing strings (words).
#[derive(Default)]
pub struct Trie {
    root: TrieNode,
    /// Total number of words currently stored.
    len: usize,
}

impl Trie {
    /// Creates an empty trie. O(1).
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: TrieNode {
                children: HashMap::new(),
                count: 0,
            },
            len: 0,
        }
    }

    /// Number of stored words. O(1).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// `true` if no words are stored. O(1).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Inserts `word`. Returns `true` if it was newly added. O(L).
    pub fn insert(&mut self, word: &str) -> bool {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }
        let is_new = node.count == 0;
        node.count += 1;
        if is_new {
            self.len += 1;
        }
        is_new
    }

    /// Returns `true` if `word` was inserted at least once. O(L).
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        self.find_node(word).is_some_and(|node| node.count > 0)
    }

    /// Returns `true` if any stored word starts with `prefix`. O(L).
    #[must_use]
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.find_node(prefix).is_some()
    }

    /// Removes one occurrence of `word`. Returns `true` if it was present. O(L).
    pub fn remove(&mut self, word: &str) -> bool {
        // `true` only when the last occurrence was removed (the word no longer
        // exists), so `len` stays accurate for duplicate inserts.
        if remove_recursive(&mut self.root, word.chars().peekable()) {
            self.len -= 1;
            true
        } else {
            false
        }
    }

    /// Returns all stored words in lexicographic order that start with `prefix`.
    /// O(L + K).
    #[must_use]
    pub fn autocomplete(&self, prefix: &str) -> Vec<String> {
        let Some(node) = self.find_node(prefix) else {
            return Vec::new();
        };
        let mut results = Vec::new();
        let mut buffer = prefix.to_string();
        collect_words(node, &mut buffer, &mut results);
        results
    }

    /// Iterator over all stored words in lexicographic order. O(1) to create.
    #[must_use]
    pub fn iter(&self) -> TrieIter<'_> {
        TrieIter::new(&self.root)
    }

    /// Walks to the node for `word` (the node after consuming every char).
    fn find_node(&self, word: &str) -> Option<&TrieNode> {
        let mut node = &self.root;
        for ch in word.chars() {
            node = node.children.get(&ch)?;
        }
        Some(node)
    }
}

/// Recursively removes `chars` from `node`'s subtree, pruning empty branches.
///
/// Returns `true` when the *last* occurrence of the word was removed, i.e. the
/// word no longer exists in the trie.
fn remove_recursive(
    node: &mut TrieNode,
    mut chars: std::iter::Peekable<std::str::Chars<'_>>,
) -> bool {
    let Some(ch) = chars.next() else {
        if node.count == 0 {
            return false; // word not present
        }
        node.count -= 1;
        return node.count == 0; // last occurrence?
    };
    let Some(child) = node.children.get_mut(&ch) else {
        return false;
    };
    let fully_removed = remove_recursive(child, chars);
    // Prune the child if it no longer holds any words.
    if fully_removed && child.count == 0 && child.children.is_empty() {
        node.children.remove(&ch);
    }
    fully_removed
}

/// DFS-collects all words rooted at `node` into `out` in lexicographic order.
fn collect_words(node: &TrieNode, buffer: &mut String, out: &mut Vec<String>) {
    if node.count > 0 {
        out.push(buffer.clone());
    }
    let mut keys: Vec<char> = node.children.keys().copied().collect();
    keys.sort_unstable();
    for ch in keys {
        buffer.push(ch);
        collect_words(&node.children[&ch], buffer, out);
        buffer.pop();
    }
}

impl Clone for Trie {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for word in self {
            new.insert(&word);
        }
        new
    }
}

impl fmt::Debug for Trie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Trie")
            .field("words", &self.len)
            .finish_non_exhaustive()
    }
}

impl FromIterator<String> for Trie {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut trie = Self::new();
        for word in iter {
            trie.insert(&word);
        }
        trie
    }
}

impl<'a> FromIterator<&'a str> for Trie {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut trie = Self::new();
        for word in iter {
            trie.insert(word);
        }
        trie
    }
}

// ---------------------------------------------------------------------------
// Iterator
// ---------------------------------------------------------------------------

/// Iterator over all words in the trie, lexicographically sorted.
pub struct TrieIter<'a> {
    /// (node, current character prefix leading to it)
    stack: Vec<(&'a TrieNode, String)>,
}

impl<'a> TrieIter<'a> {
    fn new(root: &'a TrieNode) -> Self {
        Self {
            stack: vec![(root, String::new())],
        }
    }

    /// Pushes all children of `node` onto the stack, smallest char last so it
    /// is popped first (lexicographic order via LIFO).
    fn push_children(node: &'a TrieNode, prefix: &str, stack: &mut Vec<(&'a TrieNode, String)>) {
        let mut keys: Vec<char> = node.children.keys().copied().collect();
        keys.sort_unstable();
        for ch in keys.into_iter().rev() {
            let mut child_prefix = prefix.to_string();
            child_prefix.push(ch);
            stack.push((&node.children[&ch], child_prefix));
        }
    }
}

impl Iterator for TrieIter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, prefix)) = self.stack.pop() {
            // Children must be pushed before yielding so a word end at this
            // node still gets visited after (prefixes sort before extensions).
            Self::push_children(node, &prefix, &mut self.stack);
            if node.count > 0 {
                return Some(prefix);
            }
        }
        None
    }
}

impl<'a> IntoIterator for &'a Trie {
    type Item = String;
    type IntoIter = TrieIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_contains() {
        let mut trie = Trie::new();
        assert!(trie.insert("apple"));
        assert!(!trie.insert("apple")); // duplicate
        assert!(trie.insert("app"));
        assert_eq!(trie.len(), 2);
        assert!(trie.contains("apple"));
        assert!(trie.contains("app"));
        assert!(!trie.contains("appl"));
        assert!(!trie.contains("orange"));
    }

    #[test]
    fn empty_string() {
        let mut trie = Trie::new();
        assert!(trie.insert(""));
        assert!(trie.contains(""));
        assert_eq!(trie.len(), 1);
        assert!(trie.remove(""));
        assert!(!trie.contains(""));
        assert!(trie.is_empty());
    }

    #[test]
    fn starts_with() {
        let mut trie = Trie::new();
        trie.insert("hello");
        trie.insert("help");
        assert!(trie.starts_with("he"));
        assert!(trie.starts_with("hell"));
        assert!(trie.starts_with("hello"));
        assert!(!trie.starts_with("held"));
        assert!(!trie.starts_with("x"));
    }

    #[test]
    fn autocomplete() {
        let mut trie = Trie::new();
        for word in ["car", "card", "care", "cargo", "dog"] {
            trie.insert(word);
        }
        assert_eq!(
            trie.autocomplete("car"),
            vec!["car", "card", "care", "cargo"]
        );
        assert_eq!(trie.autocomplete("do"), vec!["dog"]);
        assert_eq!(trie.autocomplete("cat"), Vec::<String>::new());
        assert_eq!(
            trie.autocomplete(""),
            vec!["car", "card", "care", "cargo", "dog"]
        );
    }

    #[test]
    fn remove_prunes_branches() {
        let mut trie = Trie::new();
        trie.insert("hello");
        trie.insert("help");
        assert!(trie.remove("hello"));
        // "help" must survive even though "hello" shared the "hel" prefix.
        assert!(trie.contains("help"));
        assert!(!trie.contains("hello"));
        assert!(trie.starts_with("hel"));
        assert_eq!(trie.len(), 1);
        assert!(trie.remove("help"));
        assert!(trie.is_empty());
        assert!(!trie.starts_with("h"));
    }

    #[test]
    fn remove_missing() {
        let mut trie = Trie::new();
        trie.insert("a");
        assert!(!trie.remove("ab"));
        assert!(!trie.remove("b"));
        assert_eq!(trie.len(), 1);
    }

    #[test]
    fn iterator_sorted() {
        let mut trie = Trie::new();
        for word in ["banana", "apple", "cherry", "apricot"] {
            trie.insert(word);
        }
        let words: Vec<String> = trie.iter().collect();
        assert_eq!(words, vec!["apple", "apricot", "banana", "cherry"]);
    }

    #[test]
    fn large_scale() {
        let mut trie = Trie::new();
        for i in 0..10_000 {
            trie.insert(&format!("word{i}"));
        }
        assert_eq!(trie.len(), 10_000);
        assert!(trie.contains("word9999"));
        assert!(trie.starts_with("word99"));
        // "word9999" is the only word with that exact prefix.
        assert_eq!(trie.autocomplete("word9999"), vec!["word9999"]);
        // All words survive a full iteration.
        assert_eq!(trie.iter().count(), 10_000);
    }
}
