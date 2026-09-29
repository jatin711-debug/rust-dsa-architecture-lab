# Codebase review and evolution plan

## What exists

The main crate already covers linear collections, linked lists, BST/AVL/trie, a heap, hash map, graph, union-find, sorting, and examples of interior mutability, reference counting, threads, async, macros, and FFI. Its unit tests exercise many edge cases. `wasm-visualizer` provides a useful second consumer of the library. `enterprise_backend` is a larger Axum/SQLx/Redis application with routes, repositories, domain types, and a frontend.

## Decisions made in this pass

| Decision | Reason | Tradeoff |
| --- | --- | --- |
| Keep the library dependency-free | Makes ownership and algorithm behavior easier to inspect | The custom containers are teaching tools, not replacements for `std` |
| Add small binaries as workspace members | Lets `cargo test --workspace` check realistic CLI boundaries | Adds separate manifests |
| Explicitly exclude the backend from the workspace | Core exercises remain fast, and its standalone manifest becomes usable | Backend checks are a separate command and need separate maintenance |
| Add `lower_bound` and `edit_distance` | Introduces sorted-input contracts and state-compression DP | Only two new algorithm families are covered so far |
| Make BST insertion and destruction iterative | Sorted input no longer overflows the stack in those operations | Plain BST remains O(n) tall; removal is still recursive |
| Track union-find set sizes at roots | Makes `set_size` match its advertised near-constant amortized cost | Requires one extra `usize` per element |
| Use checked path addition | An overflowing route cannot silently become a cheap route | `usize::MAX` remains reserved for unreachable |
| Run the tree benchmark on shuffled input | Avoid an accidental stack overflow while comparing typical insertion costs | `cargo test --all-targets` still executes Criterion smoke runs; use `cargo test --workspace` for routine checks |

## Risks and next engineering work

1. **Unsafe collection internals.** `DynamicArray`, the custom hash map, linked structures, `Rc`, and `RefCell` need invariant-focused review. Linux CI exposed a `RawWaker` reference-count error in the teaching executor; it is repaired and has an explicit count test. Review also corrected `Rc::from_raw` for aligned values, `Rc::get_mut` in the presence of weak references, and empty `Weak` allocation. For each remaining unsafe block, state allocation ownership, initialization, aliasing, and drop rules. Add Miri runs on a supported toolchain before treating them as reusable outside exercises.
2. **BST worst case.** Recursive removal and structure snapshots can still exhaust the stack on a deep tree. Convert them to iterative forms or explicitly cap input depth. Contrast the plain BST with AVL using the same sorted and shuffled inputs.
3. **Graph API.** Invalid vertex IDs currently panic in the core API. A service-facing graph layer should return typed errors; also consider `Option<usize>` distances to remove the sentinel and distinguish overflow.
4. **Backend boundary.** The backend now requires a JWT secret, restricts its CORS origin, fails startup if migrations fail, requires an admin role for product writes, and prevents public registration from selecting that role. It still needs a trusted admin provisioning path, service-backed integration tests, and deployment-specific configuration before deployment. Its presence alone does not imply production readiness.
5. **Build reproducibility.** CI now checks Rust formatting, Clippy, workspace tests, isolated PostgreSQL tests, and the frontend build. Decide separately whether generated WASM output should be committed or rebuilt in CI.
6. **Async and database learning.** The concurrency lab now contrasts CPU parallelism with bounded waiting tasks, and the backend has optional isolated SQLx tests for concurrent orders. Work through [the concurrency and database chapter](CONCURRENCY_AND_DATABASE.md), then add cancellation, retries, pagination, and query-plan measurements.

## Practice tasks, in order

1. Implement `upper_bound`, using the `lower_bound` tests as a model. State what happens with duplicates.
2. Extend edit distance to return one sequence of edits. Explain why the memory bound changes when reconstructing a path.
3. Add an unweighted shortest path to `Graph`, using BFS and a predecessor vector. Compare it with Dijkstra on unit-weight edges.
4. Make BST removal iterative, then test a long sorted tree. Explain how `Option::take` lets you relink owned nodes.
5. Add a small route-planner input type with named vertices, keeping file parsing outside `Graph`.
6. Benchmark custom structures against their standard library equivalents with identical input and report both input distribution and compiler profile.

Each task should include a failing behavior test first, an implementation, and a short note about complexity and invariants. Keep application parsing and printing out of the core library.
