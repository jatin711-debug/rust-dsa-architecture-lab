//! Tree recording: snapshots the *library's* BST and AVL trees after every
//! insert, so the browser can animate node insertion and AVL rotations.
//!
//! Both tracks run the *same* value sequence, which is the whole point of the
//! visualizer: watch the plain BST degenerate while the AVL stays balanced.
//! The structure comes straight from the battle-tested library trees via
//! `structure()`, so the recording is guaranteed correct.
//!
//! # Snapshot encoding (flat u32 array per track)
//!
//! ```text
//! snapshot = header, node[0], node[1], ..., node[count-1]
//! header   = (kind << 28) | (count << 22) | value     (kind: 0 = insert)
//! node     = (value << 20) | ((left + 1) << 10) | (right + 1)   (-1 -> 0)
//! ```
//!
//! Children are stored *by value* (values are unique in both trees); the
//! frontend rebuilds the tree and derives the root (the value nobody points
//! at). Consecutive snapshots differ only where a rotation moved nodes, so
//! lerping node positions between snapshots animates the rotation.

use rust_learning_and_dsa::tree::{AvlTree, BinarySearchTree};

const KIND_INSERT: u32 = 0;

/// A recorded tree track: a flat packed stream of snapshots.
pub struct TreeTrack {
    snapshots: Vec<u32>,
    snapshot_count: usize,
}

impl TreeTrack {
    fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            snapshot_count: 0,
        }
    }

    /// Packs and appends one snapshot from the tree's structure.
    fn push_snapshot(
        &mut self,
        kind: u32,
        value: i32,
        structure: &[(i32, Option<i32>, Option<i32>)],
    ) {
        self.snapshot_count += 1;
        let count = structure.len() as u32;
        self.snapshots
            .push(((kind & 0xF) << 28) | ((count & 0x3FF) << 22) | ((value as u32) & 0x3F_FFFF));
        for &(v, left, right) in structure {
            self.snapshots.push(
                ((v as u32 & 0x3FF) << 20)
                    | (((left.unwrap_or(-1) + 1) as u32 & 0x3FF) << 10)
                    | ((right.unwrap_or(-1) + 1) as u32 & 0x3FF),
            );
        }
    }
}

/// Records both tracks for the same value sequence, snapshotting after every
/// insert. The AVL track emits identical snapshots to the BST track — except
/// where rotations changed the structure.
pub fn record_trees(values: &[i32]) -> (TreeTrack, TreeTrack) {
    let mut bst = BinarySearchTree::new();
    let mut avl = AvlTree::new();
    let mut bst_track = TreeTrack::new();
    let mut avl_track = TreeTrack::new();

    for &v in values {
        bst.insert(v);
        bst_track.push_snapshot(KIND_INSERT, v, &bst.structure());
        avl.insert(v);
        avl_track.push_snapshot(KIND_INSERT, v, &avl.structure());
    }

    (bst_track, avl_track)
}

/// Access used by the exports.
impl TreeTrack {
    /// The flat packed stream (headers interleaved with node records).
    pub fn steps(&self) -> &[u32] {
        &self.snapshots
    }

    /// Number of snapshots (i.e. animatable steps).
    #[cfg(test)]
    pub fn snapshot_count(&self) -> usize {
        self.snapshot_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decodes the flat stream; returns the final snapshot's nodes as
    /// `(value, left_value, right_value)`.
    fn replay(track: &TreeTrack) -> Vec<(i32, Option<i32>, Option<i32>)> {
        let mut final_nodes = Vec::new();
        let mut i = 0;
        while i < track.snapshots.len() {
            let header = track.snapshots[i];
            // kind sits in the top 4 bits; count is 6 bits below it. Mask the
            // kind bits *before* shifting so they don't leak into count.
            let count = ((header >> 22) & 0x3F) as usize;
            final_nodes = track.snapshots[i + 1..i + 1 + count]
                .iter()
                .map(|&n| {
                    (
                        ((n >> 20) & 0x3FF) as i32,
                        (((n >> 10) & 0x3FF) as i32 != 0).then_some(((n >> 10) & 0x3FF) as i32 - 1),
                        ((n & 0x3FF) as i32 != 0).then_some((n & 0x3FF) as i32 - 1),
                    )
                })
                .collect();
            i += 1 + count;
        }
        final_nodes
    }

    /// The root is the value no node points at.
    fn root_of(nodes: &[(i32, Option<i32>, Option<i32>)]) -> i32 {
        let mut is_child = std::collections::HashSet::new();
        for &(_, l, r) in nodes {
            if let Some(l) = l {
                is_child.insert(l);
            }
            if let Some(r) = r {
                is_child.insert(r);
            }
        }
        nodes
            .iter()
            .map(|&(v, _, _)| v)
            .find(|v| !is_child.contains(v))
            .expect("exactly one root")
    }

    fn tree_height(nodes: &[(i32, Option<i32>, Option<i32>)]) -> usize {
        fn walk(nodes: &[(i32, Option<i32>, Option<i32>)], value: i32) -> usize {
            let node = nodes.iter().find(|n| n.0 == value).expect("node exists");
            1 + node
                .1
                .map(|l| walk(nodes, l))
                .unwrap_or(0)
                .max(node.2.map(|r| walk(nodes, r)).unwrap_or(0))
        }
        walk(nodes, root_of(nodes))
    }

    fn in_order(nodes: &[(i32, Option<i32>, Option<i32>)]) -> Vec<i32> {
        fn walk(nodes: &[(i32, Option<i32>, Option<i32>)], value: i32, out: &mut Vec<i32>) {
            let node = nodes.iter().find(|n| n.0 == value).expect("node exists");
            if let Some(l) = node.1 {
                walk(nodes, l, out);
            }
            out.push(value);
            if let Some(r) = node.2 {
                walk(nodes, r, out);
            }
        }
        let mut out = Vec::new();
        walk(nodes, root_of(nodes), &mut out);
        out
    }

    #[test]
    fn both_tracks_record_every_insert() {
        let values = [5, 3, 8, 1, 4, 7, 9, 2, 6];
        let (bst, avl) = record_trees(&values);
        assert_eq!(bst.snapshot_count(), values.len());
        assert_eq!(avl.snapshot_count(), values.len());
        assert_eq!(replay(&bst).len(), values.len());
        assert_eq!(replay(&avl).len(), values.len());
    }

    #[test]
    fn bst_sorted_input_degrades_to_chain() {
        let values: Vec<i32> = (0..20).collect();
        let (bst, _avl) = record_trees(&values);
        let nodes = replay(&bst);
        // Height of a chain is n.
        assert_eq!(tree_height(&nodes), values.len());
    }

    #[test]
    fn avl_stays_balanced_even_sorted() {
        let values: Vec<i32> = (0..50).collect();
        let (_bst, avl) = record_trees(&values);
        let nodes = replay(&avl);
        let h = tree_height(&nodes);
        // AVL guarantee: height <= ~1.44 * log2(n); assert a generous bound.
        let bound = 2.0 * (values.len() as f64).log2() + 1.0;
        assert!(h as f64 <= bound, "AVL height {h} exceeds {bound}");
        // In-order must be sorted regardless.
        assert_eq!(in_order(&nodes), values);
    }

    #[test]
    fn random_input_bst_vs_avl() {
        let values = [42, 17, 93, 5, 28, 71, 3, 12, 55, 88];
        let (bst, avl) = record_trees(&values);
        let bst_nodes = replay(&bst);
        let avl_nodes = replay(&avl);
        // Same values, same in-order — only the shape differs.
        assert_eq!(in_order(&bst_nodes), in_order(&avl_nodes));
        assert!(tree_height(&avl_nodes) <= tree_height(&bst_nodes));
    }

    #[test]
    fn every_snapshot_is_a_valid_tree() {
        // Fuzz: for many random orders, EVERY snapshot must decode to a
        // single-rooted tree whose in-order is sorted.
        let mut state: u64 = 0x1234_5678_9abc_def0;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 33) as usize
        };
        for _ in 0..20 {
            let n = 10 + next() % 40;
            let mut values: Vec<i32> = (0..n as i32).collect();
            for i in (1..values.len()).rev() {
                let j = next() % (i + 1);
                values.swap(i, j);
            }
            let (bst, avl) = record_trees(&values);
            for track in [&bst, &avl] {
                let mut i = 0;
                let mut snapshot_no = 0;
                while i < track.snapshots.len() {
                    let header = track.snapshots[i];
                    let count = ((header >> 22) & 0x3F) as usize;
                    let nodes: Vec<(i32, Option<i32>, Option<i32>)> = track.snapshots
                        [i + 1..i + 1 + count]
                        .iter()
                        .map(|&n| {
                            (
                                ((n >> 20) & 0x3FF) as i32,
                                (((n >> 10) & 0x3FF) as i32 != 0)
                                    .then_some(((n >> 10) & 0x3FF) as i32 - 1),
                                ((n & 0x3FF) as i32 != 0).then_some((n & 0x3FF) as i32 - 1),
                            )
                        })
                        .collect();
                    assert_eq!(
                        in_order(&nodes),
                        {
                            let mut sorted: Vec<i32> = nodes.iter().map(|n| n.0).collect();
                            sorted.sort_unstable();
                            sorted
                        },
                        "snapshot {snapshot_no} not sorted"
                    );
                    i += 1 + count;
                    snapshot_no += 1;
                }
            }
        }
    }
}
