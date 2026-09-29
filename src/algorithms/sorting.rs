//! Sorting algorithms.
//!
//! # Comparison
//!
//! | Algorithm | Average | Worst | Stable | In-place | Extra memory |
//! |-----------|---------|-------|--------|----------|--------------|
//! | `insertion_sort` | O(n²) | O(n²) | yes | yes | O(1) |
//! | `selection_sort` | O(n²) | O(n²) | no | yes | O(1) |
//! | `bubble_sort`    | O(n²) | O(n²) | yes | yes | O(1) |
//! | `quicksort`      | O(n log n) | O(n²)* | no | yes | O(log n) stack |
//! | `mergesort`      | O(n log n) | O(n log n) | yes | no | O(n) buffer |
//! | `heapsort`       | O(n log n) | O(n log n) | no | yes | O(1) |
//!
//! \* Quicksort's worst case is avoided in practice by the median-of-three
//! pivot, three-way partitioning (duplicates), and an insertion-sort cutoff —
//! but adversarial inputs can still trigger it. `std::slice::sort_unstable`
//! uses pattern-defeating quicksort, which detects such cases and switches
//! strategy.
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::algorithms::{heapsort, mergesort, quicksort};
//!
//! let mut a = vec![3, 1, 2];
//! quicksort(&mut a);
//! assert_eq!(a, vec![1, 2, 3]);
//!
//! let mut b = vec![3, 1, 2];
//! mergesort(&mut b);
//! assert_eq!(b, vec![1, 2, 3]);
//! ```

use std::cmp::Ordering;

/// Sub-arrays at or below this size are finished with insertion sort, which
/// has lower constant factors than the divide-and-conquer sorts.
const INSERTION_SORT_THRESHOLD: usize = 16;

// ---------------------------------------------------------------------------
// O(n^2) educational sorts
// ---------------------------------------------------------------------------

/// Insertion sort: repeatedly insert the next element into the sorted prefix.
/// O(n²) worst case, O(n) on nearly-sorted input. Stable.
pub fn insertion_sort<T: Ord>(slice: &mut [T]) {
    for i in 1..slice.len() {
        // Slide `slice[i]` left until it sits in sorted position.
        let mut j = i;
        while j > 0 && slice[j] < slice[j - 1] {
            slice.swap(j, j - 1);
            j -= 1;
        }
    }
}

/// Selection sort: repeatedly find the minimum of the unsorted suffix and
/// swap it to the front. O(n²) always, even on sorted input. Not stable.
pub fn selection_sort<T: Ord>(slice: &mut [T]) {
    for i in 0..slice.len() {
        // Find the index of the smallest element in [i, len).
        let mut min = i;
        for j in (i + 1)..slice.len() {
            if slice[j] < slice[min] {
                min = j;
            }
        }
        slice.swap(i, min);
    }
}

/// Bubble sort: repeatedly swap adjacent inverted pairs until no swaps remain.
/// O(n²) worst case, O(n) on already-sorted input (early exit). Stable.
pub fn bubble_sort<T: Ord>(slice: &mut [T]) {
    let n = slice.len();
    if n <= 1 {
        return;
    }
    let mut swapped = true;
    let mut sorted_len = 0;
    while swapped {
        swapped = false;
        sorted_len += 1;
        for j in 0..n - sorted_len {
            if slice[j] > slice[j + 1] {
                slice.swap(j, j + 1);
                swapped = true;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Quicksort
// ---------------------------------------------------------------------------

/// In-place quicksort with three production-grade mitigations:
///
/// 1. **Median-of-three pivot** — guards against sorted/reverse-sorted input.
/// 2. **Three-way partitioning** — all elements equal to the pivot land in the
///    middle, so duplicate-heavy input is sorted in O(n), not O(n²).
/// 3. **Tail-call elimination on the larger half** — recursion depth stays
///    O(log n) even on adversarial input, so no stack overflow.
///
/// Small sub-arrays fall back to insertion sort for constant-factor wins.
/// Not stable. Average O(n log n), worst O(n²).
pub fn quicksort<T: Ord>(slice: &mut [T]) {
    if slice.len() <= 1 {
        return;
    }
    quicksort_impl(slice, 0, slice.len());
}

/// Sorts `slice[lo..hi]`. The outer `loop` is the tail call: we recurse into
/// the smaller half and continue looping on the larger one.
fn quicksort_impl<T: Ord>(slice: &mut [T], mut lo: usize, mut hi: usize) {
    loop {
        if hi - lo <= 1 {
            return;
        }
        if hi - lo <= INSERTION_SORT_THRESHOLD {
            insertion_sort(&mut slice[lo..hi]);
            return;
        }

        // Median-of-three: order slice[lo], slice[mid], slice[hi-1] so the
        // median lands at `lo` and becomes the pivot.
        let mid = lo + (hi - lo) / 2;
        if slice[lo] > slice[mid] {
            slice.swap(lo, mid);
        }
        if slice[lo] > slice[hi - 1] {
            slice.swap(lo, hi - 1);
        }
        if slice[mid] > slice[hi - 1] {
            slice.swap(mid, hi - 1);
        }
        slice.swap(lo, mid);

        // Three-way partition. Invariants while scanning:
        //   slice[lo+1..lt] < pivot, slice[lt..i] == pivot, slice[gt..hi] > pivot
        // The pivot stays at `lo` for the whole scan, so every comparison is
        // against the same, never-mutated element.
        let mut lt = lo + 1;
        let mut gt = hi;
        let mut i = lo + 1;
        while i < gt {
            match slice[i].cmp(&slice[lo]) {
                Ordering::Less => {
                    slice.swap(i, lt);
                    lt += 1;
                    i += 1;
                }
                Ordering::Greater => {
                    gt -= 1;
                    slice.swap(i, gt);
                }
                Ordering::Equal => i += 1,
            }
        }
        // Park the pivot at the end of the `<` zone: [lo, lt-1) < pivot,
        // [lt-1, gt) == pivot, [gt, hi) > pivot.
        slice.swap(lo, lt - 1);

        // Recurse on the smaller half; loop on the larger half.
        let small_end = lt - 1; // exclusive end of the `<` zone
        if (small_end - lo) < (hi - gt) {
            quicksort_impl(slice, lo, small_end);
            lo = gt;
        } else {
            quicksort_impl(slice, gt, hi);
            hi = small_end;
        }
    }
}

// ---------------------------------------------------------------------------
// Mergesort
// ---------------------------------------------------------------------------

/// Stable, top-down mergesort with a single scratch buffer allocated once.
/// Guaranteed O(n log n); uses O(n) extra memory.
///
/// Requires `T: Clone` because the stable merge duplicates the left half into
/// the scratch buffer. (`std::slice::sort` avoids this bound with internal
/// unsafe moves — a good follow-up exercise.)
pub fn mergesort<T: Ord + Clone>(slice: &mut [T]) {
    if slice.len() <= 1 {
        return;
    }
    let mut buffer = Vec::with_capacity(slice.len() / 2);
    mergesort_impl(slice, &mut buffer);
}

fn mergesort_impl<T: Ord + Clone>(slice: &mut [T], buffer: &mut Vec<T>) {
    let n = slice.len();
    if n <= 1 {
        return;
    }
    let mid = n / 2;
    mergesort_impl(&mut slice[..mid], buffer);
    mergesort_impl(&mut slice[mid..], buffer);
    merge(slice, mid, buffer);
}

/// Merges the two sorted halves `slice[..mid]` and `slice[mid..]` in place.
///
/// The left half is copied into `buffer` (via `Clone`). We then walk three
/// cursors: `i` into the buffer, `j` into the right half, `k` the write head.
/// Taking from the right uses `swap` so values are *moved*, never copied.
///
/// Invariant: `k <= j` at all times, so the swap never tramples a value that
/// is still needed (the garbage left behind at `j` is already consumed).
fn merge<T: Ord + Clone>(slice: &mut [T], mid: usize, buffer: &mut Vec<T>) {
    buffer.clear();
    buffer.extend_from_slice(&slice[..mid]);

    let mut i = 0; // cursor into buffer (left half)
    let mut j = mid; // cursor into the right half
    let mut k = 0; // write head

    while i < mid && j < slice.len() {
        if buffer[i] <= slice[j] {
            // Take from the left: the old value at `k` is already in the buffer.
            slice[k] = buffer[i].clone();
            i += 1;
        } else {
            // Take from the right: swap moves it into place without a copy.
            slice.swap(k, j);
            j += 1;
        }
        k += 1;
    }
    // Left half exhausted (or right half exhausted): flush the buffer.
    while i < mid {
        slice[k] = buffer[i].clone();
        i += 1;
        k += 1;
    }
    // If the left half was exhausted first, `k == j` and the untouched right
    // remainder is already in its final position.
}

// ---------------------------------------------------------------------------
// Heapsort
// ---------------------------------------------------------------------------

/// In-place heapsort: build a max-heap with Floyd's O(n) heapify, then
/// repeatedly swap the root with the end and sift down. Guaranteed
/// O(n log n), O(1) extra memory. Not stable.
pub fn heapsort<T: Ord>(slice: &mut [T]) {
    let n = slice.len();
    if n <= 1 {
        return;
    }
    // Build the max-heap: sift down from the deepest parents to the root.
    for i in (0..n / 2).rev() {
        sift_down(slice, i, n);
    }
    // Extract: move the max to the end, then restore the heap.
    for end in (1..n).rev() {
        slice.swap(0, end);
        sift_down(slice, 0, end);
    }
}

/// Restores the max-heap property for `slice[idx..limit]` after `idx` may have
/// violated it (children of `idx` are already valid heaps).
fn sift_down<T: Ord>(slice: &mut [T], mut idx: usize, limit: usize) {
    loop {
        let left = 2 * idx + 1;
        if left >= limit {
            break;
        }
        let right = left + 1;
        let mut largest = idx;
        if slice[left] > slice[largest] {
            largest = left;
        }
        if right < limit && slice[right] > slice[largest] {
            largest = right;
        }
        if largest == idx {
            break;
        }
        slice.swap(idx, largest);
        idx = largest;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Sorts that work with any `Ord` (no `Clone`).
    const PLAIN_SORTS: &[fn(&mut [i32])] = &[
        insertion_sort,
        selection_sort,
        bubble_sort,
        quicksort,
        heapsort,
    ];

    /// All sorts, including the `Clone`-bound mergesort.
    fn all_sorts() -> Vec<fn(&mut [i32])> {
        let mut sorts: Vec<fn(&mut [i32])> = PLAIN_SORTS.to_vec();
        sorts.push(mergesort);
        sorts
    }

    fn assert_sorts_eq(input: &[i32]) {
        let mut expected = input.to_vec();
        expected.sort_unstable();
        for sort in all_sorts() {
            let mut data = input.to_vec();
            sort(&mut data);
            assert_eq!(data, expected, "sort mismatch on {input:?}");
        }
    }

    #[test]
    fn empty_and_single() {
        assert_sorts_eq(&[]);
        assert_sorts_eq(&[42]);
    }

    #[test]
    fn small_cases() {
        assert_sorts_eq(&[3, 1, 2]);
        assert_sorts_eq(&[5, 4, 3, 2, 1]);
        assert_sorts_eq(&[1, 2, 3, 4, 5]);
        assert_sorts_eq(&[2, 2, 2, 2]);
    }

    #[test]
    fn duplicates_and_negative() {
        assert_sorts_eq(&[0, -5, 3, -5, 0, 2, -1, 3, 3, 0]);
        assert_sorts_eq(&[7, 7, 7, 1, 7, 7, 2, 7]);
    }

    #[test]
    fn randomized_against_std() {
        // Deterministic pseudo-random generator (no external deps in tests).
        // The u64 -> i32 narrowing is intentional: we only need a bounded
        // pseudo-random spread, and the generator is deterministic.
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let next = || {
            static mut STATE: u64 = 0x1234_5678_9abc_def0;
            // SAFETY: single-threaded test; the state is only touched here.
            unsafe {
                STATE = STATE
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                (STATE >> 33) as i32 % 1000 - 500
            }
        };
        for _ in 0..50 {
            let data: Vec<i32> = (0..500).map(|_| next()).collect();
            assert_sorts_eq(&data);
        }
    }

    #[test]
    fn quicksort_handles_duplicate_heavy_input() {
        // Three-way partitioning must not degrade on few distinct values.
        let mut data = vec![1u8; 100_000];
        data.extend([0, 2]);
        let mut expected = data.clone();
        expected.sort_unstable();
        quicksort(&mut data);
        assert_eq!(data, expected);
    }

    #[test]
    fn quicksort_handles_sorted_and_reverse() {
        let mut data: Vec<usize> = (0..100_000).collect();
        let mut expected = data.clone();
        expected.sort_unstable();
        quicksort(&mut data);
        assert_eq!(data, expected);

        let mut rev: Vec<usize> = (0..100_000).rev().collect();
        quicksort(&mut rev);
        assert_eq!(rev, expected);
    }

    #[test]
    fn large_random_all_sorts() {
        // 5_000 elements: enough to catch subtle bugs while keeping the O(n²)
        // sorts (insertion/selection/bubble) fast in debug builds.
        // `0..5_000` defaults to i32, so the whole expression is i32 math.
        let data: Vec<i32> = (0..5_000).map(|i| (i * 7919 % 50_000) - 25_000).collect();
        let mut expected = data.clone();
        expected.sort_unstable();
        for sort in all_sorts() {
            let mut d = data.clone();
            sort(&mut d);
            assert_eq!(d, expected);
        }
    }

    #[test]
    fn mergesort_is_stable() {
        // Sort (key, tag) pairs by key only; stability means equal keys keep
        // their original relative order.
        let pairs: Vec<(i32, usize)> = vec![(2, 0), (1, 1), (2, 2), (1, 3), (3, 4), (2, 5), (1, 6)];
        let mut data = pairs.clone();
        mergesort(&mut data);
        let keys: Vec<i32> = data.iter().map(|&(k, _)| k).collect();
        assert_eq!(keys, vec![1, 1, 1, 2, 2, 2, 3]);
        // Within each key group, tags must appear in insertion order.
        let mut last_key = i32::MIN;
        let mut last_tag = 0;
        for (key, tag) in data {
            if key == last_key {
                assert!(tag > last_tag, "unstable: {key} tag {tag} after {last_tag}");
            }
            last_key = key;
            last_tag = tag;
        }
    }

    #[test]
    fn insertion_sort_linear_on_nearly_sorted() {
        // Just a sanity check that insertion sort doesn't blow up; the O(n)
        // best case is a performance property, not asserted here.
        let mut data: Vec<i32> = (0..5_000).collect();
        data[2_500] = -1; // single inversion
        let mut expected = data.clone();
        expected.sort_unstable();
        insertion_sort(&mut data);
        assert_eq!(data, expected);
    }

    #[test]
    fn strings_and_structs() {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct Point {
            x: i32,
            y: i32,
        }

        let mut words = vec!["pear", "apple", "orange", "banana"];
        quicksort(&mut words);
        assert_eq!(words, vec!["apple", "banana", "orange", "pear"]);

        let mut points = vec![
            Point { x: 3, y: 1 },
            Point { x: 1, y: 5 },
            Point { x: 2, y: 2 },
        ];
        mergesort(&mut points);
        assert_eq!(
            points,
            vec![
                Point { x: 1, y: 5 },
                Point { x: 2, y: 2 },
                Point { x: 3, y: 1 },
            ]
        );
    }
}
