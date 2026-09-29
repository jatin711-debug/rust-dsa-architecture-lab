# Rust Learning and DSA

A hands-on path from Rust ownership to data structures, algorithms, and application architecture. The core library is deliberately implemented from scratch for study. For applications, prefer the standard library collections unless a measured requirement justifies a custom implementation.

## Start here

Install a Rust toolchain compatible with `rust-version` in [Cargo.toml](Cargo.toml). From the repository root:

```text
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo run --example all_data_structures
cargo run --example advanced_rust
```

The workspace contains the core library, a WASM visualizer, and three small command line projects. The backend in `enterprise_backend/` has its own manifest and external PostgreSQL/Redis setup; run its checks from that directory when studying web services.

## Learning sequence

| Stage | Read and run | Learn | Change it yourself |
| --- | --- | --- | --- |
| 1. Rust basics | `src/linear/stack.rs`, `src/linear/queue.rs`, `projects/mini_grep/` | ownership, borrowing, `Option`, `Result`, iterators, buffered I/O | add a `--ignore-case` flag and test it |
| 2. Collections | `src/linear/dynamic_array.rs`, `src/map/mod.rs`, `src/tree/trie.rs` | allocation, generics, trait implementations, invariants | compare with `Vec`, `HashMap`, and `BTreeMap` |
| 3. Algorithms | `src/algorithms/`, `src/graph/mod.rs`, `src/union_find.rs` | complexity, graph traversal, binary search, dynamic programming | add an unweighted shortest path and test unreachable nodes |
| 4. Application design | `projects/route_planner/`, `wasm-visualizer/` | parsing boundaries, reusable library logic, WASM boundary | support named vertices in the route planner |
| 5. Advanced Rust | `src/advanced/` | interior mutability, `Rc`/`Weak`, threads, futures, FFI, unsafe contracts | document every invariant before changing unsafe code |
| 6. Concurrency | `projects/concurrency_lab/`, `src/advanced/thread_pool.rs` | CPU parallelism, bounded async tasks, timeouts, cancellation | replace timer jobs with local I/O and measure peak concurrency |
| 7. Services | `enterprise_backend/` | async runtime, HTTP routes, SQL transactions, row locks, batching, caching | run isolated database tests and add keyset pagination |

At each stage, read the public API and tests first. Predict behavior for empty input, duplicates, and invalid indices before running the tests. Then make one change and run the narrow test plus `cargo test --workspace`.

## Small projects

**Text search** streams a file without loading it all into memory:

```text
cargo run -p mini_grep -- rust README.md
```

**Route planner** parses a directed weighted graph and prints the cheapest path:

```text
cargo run -p route_planner -- projects/route_planner/sample.graph 0 4
```

Input format: first non-comment line is the vertex count. Each later line is `from to nonnegative_weight`. Vertex IDs run from `0` to `count - 1`.

**Concurrency lab** compares CPU parallel counting with bounded async waiting jobs:

```text
cargo run -p concurrency_lab -- text README.md
cargo run -p concurrency_lab -- async
cargo run -p concurrency_lab -- tcp
```

Follow [the concurrency and database chapter](docs/CONCURRENCY_AND_DATABASE.md) for the architecture, experiments, and live PostgreSQL test instructions.
GitHub CI runs the workspace checks, backend database integration tests, and frontend build on every push.

For measurements, use `cargo bench --bench core_benchmarks` in release mode. Benchmarks use identical shuffled tree input to compare BST and AVL insertion; a sorted plain BST has quadratic insertion time. Do not use debug-mode test runs as performance evidence.

## Architecture and current limits

The core library has no runtime dependencies. `projects/` holds small binaries with I/O at the boundary; algorithms remain reusable and testable in `src/`. The WASM crate is a workspace member because it consumes the core library. The backend is intentionally separate: its web, database, cache, and frontend dependencies add a different deployment and testing boundary.

Read [the codebase review](docs/CODEBASE_REVIEW.md) for concrete strengths, limitations, and next decisions. In particular, some educational containers use raw pointers or `UnsafeCell`; passing tests does not establish memory safety. The plain BST still has recursive removal and can overflow the stack on an adversarially deep tree. `Graph::dijkstra_*` uses `usize::MAX` as an unreachable sentinel, so paths with that exact cost are not representable.
