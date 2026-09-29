//! WASM visualizer: exposes the library's sorting algorithms to the browser
//! as a sequence of replayable steps.
//!
//! # How it works
//!
//! This crate is compiled with `--target wasm32-unknown-unknown` to a `cdylib`.
//! Every function here is a `#[unsafe(no_mangle)] pub extern "C"` export, so the
//! browser sees plain integer/pointer exports — no wasm-bindgen, no JS glue
//! generated for us. The browser talks to the wasm through its **linear
//! memory**: JS reads the arrays via `Int32Array(instance.exports.memory.buffer,
//! ptr, len)`.
//!
//! The sorts are *instrumented* versions of the library's algorithms
//! (same logic, plus a step recorder): every comparison, swap, and merge-write
//! is packed into a `u32` and stored. The browser then replays the steps to
//! animate the bars.
//!
//! # Step encoding
//!
//! `step = (phase << 28) | (a << 14) | b`, with `n <= 4000`:
//!
//! | phase | meaning |
//! |-------|---------|
//! | 0     | compare indices `a`, `b` |
//! | 1     | swap indices `a`, `b` |
//! | 2     | write value `b` at index `a` (merge sort) |

use std::cell::RefCell;

#[cfg(feature = "bindgen")]
mod bindgen_api;
mod tree_recorder;

/// Maximum array size; `a`/`b` fields have 14 bits each.
const MAX_N: usize = 4000;
/// Insertion-sort cutoff inside quicksort (mirrors the library).
const INSERTION_THRESHOLD: usize = 16;

/// A recorded algorithm run: the initial snapshot, the live (final) array,
/// and every step needed to replay from initial -> final.
struct Recorder {
    /// Snapshot taken before sorting; JS replays steps against this.
    initial: Vec<i32>,
    /// Live array, mutated while recording (ends sorted).
    data: Vec<i32>,
    /// Packed steps in chronological order.
    steps: Vec<u32>,
}

impl Recorder {
    fn record(&mut self, phase: u32, a: usize, b: usize) {
        debug_assert!(a < MAX_N && b < MAX_N, "index out of packing range");
        let step = ((phase & 0xF) << 28) | (((a as u32) & 0x3FFF) << 14) | ((b as u32) & 0x3FFF);
        self.steps.push(step);
    }

    fn cmp(&mut self, a: usize, b: usize) {
        self.record(0, a, b);
    }

    fn swap(&mut self, a: usize, b: usize) {
        self.record(1, a, b);
        self.data.swap(a, b);
    }

    fn write(&mut self, at: usize, value: i32) {
        self.record(2, at, value as usize);
        self.data[at] = value;
    }

    /// Instrumented quicksort: median-of-three pivot, Lomuto partition,
    /// insertion-sort cutoff — mirroring `algorithms::quicksort`.
    fn run_quicksort(&mut self) {
        self.qs(0, self.data.len());
    }

    fn qs(&mut self, lo: usize, hi: usize) {
        if hi - lo <= 1 {
            return;
        }
        if hi - lo <= INSERTION_THRESHOLD {
            for i in lo + 1..hi {
                let mut j = i;
                while j > lo {
                    self.cmp(j, j - 1);
                    if self.data[j] < self.data[j - 1] {
                        self.swap(j, j - 1);
                        j -= 1;
                    } else {
                        break;
                    }
                }
            }
            return;
        }
        // Median-of-three into the pivot slot (hi - 1).
        let mid = lo + (hi - lo) / 2;
        if self.data[lo] > self.data[mid] {
            self.swap(lo, mid);
        }
        if self.data[lo] > self.data[hi - 1] {
            self.swap(lo, hi - 1);
        }
        if self.data[mid] > self.data[hi - 1] {
            self.swap(mid, hi - 1);
        }
        self.swap(mid, hi - 1);

        // Lomuto partition (pivot parked at hi - 1).
        let mut i = lo;
        for j in lo..hi - 1 {
            self.cmp(j, hi - 1);
            if self.data[j] < self.data[hi - 1] {
                self.swap(i, j);
                i += 1;
            }
        }
        self.swap(i, hi - 1);

        if i > lo {
            self.qs(lo, i);
        }
        if i + 1 < hi {
            self.qs(i + 1, hi);
        }
    }

    /// Instrumented mergesort: stable, top-down, scratch buffer.
    fn run_mergesort(&mut self) {
        let mut buffer = Vec::with_capacity(self.data.len() / 2);
        self.ms(0, self.data.len(), &mut buffer);
    }

    fn ms(&mut self, lo: usize, hi: usize, buffer: &mut Vec<i32>) {
        if hi - lo <= 1 {
            return;
        }
        let mid = lo + (hi - lo) / 2;
        self.ms(lo, mid, buffer);
        self.ms(mid, hi, buffer);

        buffer.clear();
        buffer.extend_from_slice(&self.data[lo..mid]);

        let mut i = 0; // cursor into the buffer (left half)
        let mut j = mid; // cursor into the right half
        let mut k = lo; // write head
        while i < buffer.len() && j < hi {
            self.cmp(lo + i, j);
            if buffer[i] <= self.data[j] {
                self.write(k, buffer[i]);
                i += 1;
            } else {
                self.write(k, self.data[j]);
                j += 1;
            }
            k += 1;
        }
        while i < buffer.len() {
            self.write(k, buffer[i]);
            i += 1;
            k += 1;
        }
        // If the left half ran out first, the untouched right remainder is
        // already in place (k == j).
    }

    /// Instrumented heapsort: Floyd heapify + extract.
    fn run_heapsort(&mut self) {
        let n = self.data.len();
        if n <= 1 {
            return;
        }
        for i in (0..n / 2).rev() {
            self.sift_down(i, n);
        }
        for end in (1..n).rev() {
            self.swap(0, end);
            self.sift_down(0, end);
        }
    }

    fn sift_down(&mut self, mut idx: usize, limit: usize) {
        loop {
            let left = idx * 2 + 1;
            if left >= limit {
                return;
            }
            let right = left + 1;
            let mut largest = idx;
            self.cmp(left, largest);
            if self.data[left] > self.data[largest] {
                largest = left;
            }
            if right < limit {
                self.cmp(right, largest);
                if self.data[right] > self.data[largest] {
                    largest = right;
                }
            }
            if largest == idx {
                return;
            }
            self.swap(idx, largest);
            idx = largest;
        }
    }

    /// Instrumented insertion sort — the O(n²) baseline for comparison.
    fn run_insertion_sort(&mut self) {
        for i in 1..self.data.len() {
            let mut j = i;
            while j > 0 {
                self.cmp(j, j - 1);
                if self.data[j] < self.data[j - 1] {
                    self.swap(j, j - 1);
                    j -= 1;
                } else {
                    break;
                }
            }
        }
    }
}

// The single active recording. `thread_local` is safe and idiomatic here:
// wasm is single-threaded, and each JS call happens on the same thread.
thread_local! {
    static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

// The recorded tree tracks plus the sequence length: (bst, avl, n).
thread_local! {
    static TREE_STATE: RefCell<Option<(tree_recorder::TreeTrack, tree_recorder::TreeTrack, u32)>> =
        const { RefCell::new(None) };
}

fn with_recorder<R>(f: impl FnOnce(&Recorder) -> R) -> Option<R> {
    RECORDER.with(|cell| cell.borrow().as_ref().map(f))
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

/// Starts a new recording: creates a shuffled array of `n` values and runs
/// algorithm `kind` over it, recording every step.
///
/// `kind`: 0 = quicksort, 1 = mergesort, 2 = heapsort, 3 = insertion sort.
///
/// Returns `0` on success, `-1` on invalid arguments.
#[unsafe(no_mangle)]
pub extern "C" fn sort_new(n: u32, kind: u32) -> i32 {
    let n = n as usize;
    if n == 0 || n > MAX_N || kind > 3 {
        return -1;
    }

    // Shuffled values 0..n (deterministic xorshift, no external deps).
    let mut data: Vec<i32> = (0..n as i32).collect();
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15 ^ ((n as u64).wrapping_mul(0x2545_f491_4f6c_dd1d));
    for i in (1..n).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = ((state >> 33) as usize) % (i + 1);
        data.swap(i, j);
    }

    let initial = data.clone();
    let mut rec = Recorder {
        initial,
        data,
        steps: Vec::new(),
    };
    match kind {
        0 => rec.run_quicksort(),
        1 => rec.run_mergesort(),
        2 => rec.run_heapsort(),
        _ => rec.run_insertion_sort(),
    }

    RECORDER.with(|cell| *cell.borrow_mut() = Some(rec));
    0
}

/// Pointer to the initial (pre-sort) array in wasm linear memory.
///
/// # Safety for the caller
///
/// The returned pointer is only valid until the next `sort_new`; read it via
/// `new Int32Array(memory.buffer, ptr, len)`.
#[unsafe(no_mangle)]
pub extern "C" fn sort_initial_ptr() -> *const i32 {
    with_recorder(|r| r.initial.as_ptr()).unwrap_or(std::ptr::null())
}

/// Pointer to the final (sorted) array.
#[unsafe(no_mangle)]
pub extern "C" fn sort_final_ptr() -> *const i32 {
    with_recorder(|r| r.data.as_ptr()).unwrap_or(std::ptr::null())
}

/// Length of the arrays (initial and final are the same length).
#[unsafe(no_mangle)]
pub extern "C" fn sort_len() -> u32 {
    with_recorder(|r| r.data.len() as u32).unwrap_or(0)
}

/// Pointer to the packed step array.
#[unsafe(no_mangle)]
pub extern "C" fn sort_steps_ptr() -> *const u32 {
    with_recorder(|r| r.steps.as_ptr()).unwrap_or(std::ptr::null())
}

/// Number of recorded steps.
#[unsafe(no_mangle)]
pub extern "C" fn sort_steps_len() -> u32 {
    with_recorder(|r| r.steps.len() as u32).unwrap_or(0)
}

/// Releases the current recording.
#[unsafe(no_mangle)]
pub extern "C" fn sort_free() {
    RECORDER.with(|cell| *cell.borrow_mut() = None);
}

// ---------------------------------------------------------------------------
// Tree exports (raw, same style as the sort exports)
// ---------------------------------------------------------------------------

/// Maximum tree size (values are 0..n, packed into 10-bit fields).
const MAX_TREE_N: u32 = 50;

/// Records both a plain BST and an AVL for the same sequence of `n` values.
/// `sorted` = 1 uses 0..n (worst case for the BST); 0 shuffles.
///
/// Returns `0` on success, `-1` on invalid arguments.
#[unsafe(no_mangle)]
pub extern "C" fn tree_init(n: u32, sorted: u32) -> i32 {
    if !(2..=MAX_TREE_N).contains(&n) || sorted > 1 {
        return -1;
    }
    let values: Vec<i32> = if sorted == 1 {
        (0..n as i32).collect()
    } else {
        // Deterministic shuffle of 0..n.
        let mut data: Vec<i32> = (0..n as i32).collect();
        let mut state: u64 =
            0x9e37_79b9_7f4a_7c15 ^ ((n as u64).wrapping_mul(0x2545_f491_4f6c_dd1d));
        for i in (1..n as usize).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let j = ((state >> 33) as usize) % (i + 1);
            data.swap(i, j);
        }
        data
    };
    let (bst, avl) = tree_recorder::record_trees(&values);
    let n = values.len() as u32;
    TREE_STATE.with(|cell| *cell.borrow_mut() = Some((bst, avl, n)));
    0
}

/// Number of nodes in the current recording (same for both tracks).
#[unsafe(no_mangle)]
pub extern "C" fn tree_nodes() -> u32 {
    TREE_STATE
        .with(|cell| cell.borrow().as_ref().map(|(_, _, n)| *n))
        .unwrap_or(0)
}

/// Flat stream length (u32 words) for a track (0 = BST, 1 = AVL).
///
/// This is the size to pass when creating the `Uint32Array` view over
/// [`tree_steps_ptr`]; the stream is `snapshots * (1 + nodes)` words.
#[unsafe(no_mangle)]
pub extern "C" fn tree_steps_len(track: u32) -> u32 {
    TREE_STATE
        .with(|cell| {
            cell.borrow().as_ref().map(|(bst, avl, _)| {
                let t = if track == 0 { bst } else { avl };
                t.steps().len() as u32
            })
        })
        .unwrap_or(0)
}

/// Pointer to the flat snapshot stream for a track (0 = BST, 1 = AVL).
///
/// Valid until the next `tree_init`; see the module docs for the packing.
#[unsafe(no_mangle)]
pub extern "C" fn tree_steps_ptr(track: u32) -> *const u32 {
    TREE_STATE
        .with(|cell| {
            cell.borrow().as_ref().map(|(bst, avl, _)| {
                let t = if track == 0 { bst } else { avl };
                t.steps().as_ptr()
            })
        })
        .unwrap_or(std::ptr::null())
}

/// Releases the current tree recording.
#[unsafe(no_mangle)]
pub extern "C" fn tree_free() {
    TREE_STATE.with(|cell| *cell.borrow_mut() = None);
}

// ---------------------------------------------------------------------------
// Tests (host side)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Replays the recorded steps in JS fashion and verifies the result is
    /// sorted — the same check the browser performs, in pure Rust.
    fn replay_and_verify(kind: u32) {
        RECORDER.with(|cell| *cell.borrow_mut() = None);
        assert_eq!(sort_new(200, kind), 0);
        let steps: Vec<u32> = with_recorder(|r| r.steps.clone()).unwrap();
        let initial: Vec<i32> = with_recorder(|r| r.initial.clone()).unwrap();
        let final_data: Vec<i32> = with_recorder(|r| r.data.clone()).unwrap();

        let mut replay = initial;
        for &step in &steps {
            let phase = (step >> 28) & 0xF;
            let a = ((step >> 14) & 0x3FFF) as usize;
            let b = (step & 0x3FFF) as usize;
            match phase {
                0 => {} // compare: no mutation
                1 => replay.swap(a, b),
                2 => replay[a] = b as i32,
                _ => panic!("bad phase {phase}"),
            }
        }
        assert_eq!(replay, final_data, "replay diverged for kind {kind}");
        assert!(
            replay.windows(2).all(|w| w[0] <= w[1]),
            "not sorted: kind {kind}"
        );
        assert_eq!(replay.len(), 200);
    }

    #[test]
    fn quicksort_records_replayable_steps() {
        replay_and_verify(0);
    }

    #[test]
    fn mergesort_records_replayable_steps() {
        replay_and_verify(1);
    }

    #[test]
    fn heapsort_records_replayable_steps() {
        replay_and_verify(2);
    }

    #[test]
    fn insertion_records_replayable_steps() {
        replay_and_verify(3);
    }

    #[test]
    fn invalid_arguments_rejected() {
        assert_eq!(sort_new(0, 0), -1);
        assert_eq!(sort_new(MAX_N as u32 + 1, 0), -1);
        assert_eq!(sort_new(10, 99), -1);
    }

    #[test]
    fn step_packing_roundtrip() {
        RECORDER.with(|cell| *cell.borrow_mut() = None);
        assert_eq!(sort_new(100, 0), 0);
        let steps: Vec<u32> = with_recorder(|r| r.steps.clone()).unwrap();
        assert!(!steps.is_empty());
        for &step in &steps {
            let a = ((step >> 14) & 0x3FFF) as usize;
            let b = (step & 0x3FFF) as usize;
            assert!(a < 100 && b < 100, "index out of bounds: {a}, {b}");
        }
    }
}
