# Concurrency lab

Run from the repository root:

```text
cargo run -p concurrency_lab -- text README.md
cargo run -p concurrency_lab -- async
cargo test -p concurrency_lab
```

`text` compares sequential token counting with Rayon's parallel `fold` and `reduce`. Run it with a large file in release mode before comparing times. Both paths produce the same map; the parallel path avoids a mutex around every token.

`async` runs simulated waiting jobs with at most three live Tokio tasks. Some jobs time out, and results print in input order. Read `map_bounded` in `src/lib.rs` and its peak-concurrency test to see how the limit is enforced. This is waiting-task concurrency; it does not make CPU work faster.

Try changing the limit to 1 and 8. The tests show cancellation and task panic behavior. Then replace simulated sleeps with a real I/O operation, such as a local HTTP server or database call, while keeping the same bounded scheduler.
