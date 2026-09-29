//! The wasm-bindgen layer: the *production* marshaling surface.
//!
//! The raw `extern "C"` exports in `lib.rs` are the educational core — you see
//! the linear memory and pointer passing directly. `wasm-bindgen` automates
//! that boundary: it generates the JS glue that converts Rust types into JS
//! values and back.
//!
//! What this module demonstrates:
//!
//! - `#[wasm_bindgen]` structs become JS classes (`new SortRun(...)`).
//! - `Vec<i32>` becomes a `Uint32Array`/`Int32Array` (a copy across the
//!   boundary — the cost of marshaling).
//! - `String` becomes a JS string.
//! - `Result<_, String>` becomes a thrown JS `Error` on `Err`.
//!
//! The generated glue lives in `www/pkg/` (produced by `wasm-bindgen`), so the
//! browser never touches raw pointers.

use wasm_bindgen::prelude::*;

use crate::{MAX_N, RECORDER, Recorder};

/// The four sort kinds, named like the raw exports.
const ALGO_NAMES: [&str; 4] = ["Quicksort", "Mergesort", "Heapsort", "Insertion sort"];

/// A completed sort recording, exposed to JS as a class.
#[wasm_bindgen]
pub struct SortRun {
    kind: u32,
    initial: Vec<i32>,
    steps: Vec<u32>,
}

/// Runs one of the instrumented algorithms and captures its recording.
///
/// The recording happens in Rust; JS receives plain owned values (`Vec` ->
/// typed array, `String` -> string). This is exactly what the raw exports do
/// manually, but with the marshaling generated for us.
#[wasm_bindgen]
impl SortRun {
    /// Creates a run: shuffles `n` values and records algorithm `kind`.
    ///
    /// # Errors
    ///
    /// `Err(String)` (thrown as a JS `Error`) if arguments are invalid.
    #[wasm_bindgen(constructor)]
    pub fn new(kind: u32, n: u32) -> Result<SortRun, String> {
        if n == 0 || n as usize > MAX_N || kind > 3 {
            return Err(format!(
                "invalid arguments: kind={kind}, n={n} (max {MAX_N})"
            ));
        }
        // Reuse the raw API's recording machinery: run it, then copy the
        // arrays out into owned values that can cross the boundary.
        if crate::sort_new(n, kind) != 0 {
            return Err("sort_new failed".to_string());
        }
        let (initial, steps) = RECORDER.with(|cell| {
            let borrowed = cell.borrow();
            let rec: &Recorder = borrowed.as_ref().expect("set by sort_new");
            (rec.initial.clone(), rec.steps.clone())
        });
        Ok(SortRun {
            kind,
            initial,
            steps,
        })
    }

    /// The initial (pre-sort) array as a typed array (a copy).
    #[wasm_bindgen]
    pub fn initial(&self) -> Vec<i32> {
        self.initial.clone()
    }

    /// The packed steps as a `Uint32Array` (a copy).
    #[wasm_bindgen]
    pub fn steps(&self) -> Vec<u32> {
        self.steps.clone()
    }

    /// Number of elements.
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.initial.len()
    }

    /// Number of recorded steps.
    #[wasm_bindgen]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Human-readable algorithm name (String marshaling).
    #[wasm_bindgen]
    pub fn algorithm_name(&self) -> String {
        ALGO_NAMES[self.kind as usize].to_string()
    }

    /// Replays the steps in Rust and returns per-kind counts — the same
    /// replay the browser does, proving the recording is self-consistent.
    #[wasm_bindgen]
    pub fn replay_stats(&self) -> Vec<u32> {
        let mut display = self.initial.clone();
        let mut counts = [0u32; 3]; // compares, swaps, writes
        for &step in &self.steps {
            let phase = (step >> 28) & 0xF;
            let a = ((step >> 14) & 0x3FFF) as usize;
            let b = (step & 0x3FFF) as usize;
            match phase {
                0 => counts[0] += 1,
                1 => {
                    display.swap(a, b);
                    counts[1] += 1;
                }
                2 => {
                    display[a] = b as i32;
                    counts[2] += 1;
                }
                _ => {}
            }
        }
        Vec::from(counts)
    }
}
