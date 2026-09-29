//! Advanced Rust topics: the parts of the language the data structures never
//! forced us to touch.
//!
//! | Module | Gap it fills |
//! |--------|--------------|
//! | [`cell`] | Interior mutability: `Cell` and `RefCell` from scratch |
//! | [`rc`] | Reference counting: `Rc` and `Weak` from scratch with raw pointers |
//! | [`errors`] | Real error handling: custom `Error` types + a `Result`-based CSV parser |
//! | [`trait_objects`] | Dynamic dispatch: `dyn Trait` vs generics |
//! | [`thread_pool`] | Concurrency: `Arc`, `Mutex`, channels, worker threads |
//! | [`block_on`] | Async: a minimal executor using `Pin`, `Waker`, `Context` |
//! | [`ffi`] | FFI: `extern "C"`, function pointers, `repr(C)` layout |
//! | [`macros`] | `macro_rules!`: declarative macros, including recursion |
//!
//! Each module is self-contained, documented, and unit-tested.

pub mod block_on;
pub mod cell;
pub mod errors;
pub mod ffi;
pub mod macros;
pub mod rc;
pub mod thread_pool;
pub mod trait_objects;

pub use block_on::{TimerFuture, YieldMany, block_on, sleep_then_double, sum_after_yields};
pub use cell::{Cell, RefCell};
pub use errors::{CsvError, parse_csv};
pub use ffi::{Point, add_i32, c_strlen, qsort_points};
pub use rc::{Rc, Weak};
pub use thread_pool::ThreadPool;
