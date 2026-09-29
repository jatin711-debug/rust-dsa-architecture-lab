//! Example: the advanced Rust topics — interior mutability, Rc/Weak, dyn
//! dispatch, error handling, threads, async, FFI, and macros.
//!
//! Run with: `cargo run --example advanced_rust`

use rust_learning_and_dsa::advanced::trait_objects::{Circle, Shape, describe, total_area_dyn};
use rust_learning_and_dsa::advanced::{Cell, Rc, RefCell, ThreadPool, block_on, parse_csv};
use rust_learning_and_dsa::{dynarray, hashmap, sum_of};
use std::time::Duration;

fn main() {
    println!("=== Interior mutability: Cell + RefCell ===");
    let cell = Cell::new(10);
    cell.set(20);
    println!("  Cell: {}", cell.get());
    let shared = RefCell::new(vec![1, 2, 3]);
    shared.borrow_mut().push(4);
    println!("  RefCell: {:?}", *shared.borrow());

    println!("=== Rc + Weak (from scratch) ===");
    let shared = Rc::new(String::from("shared"));
    let weak = Rc::downgrade(&shared);
    println!(
        "  strong: {}, weak: {}, value: {}",
        Rc::strong_count(&shared),
        Rc::weak_count(&shared),
        *shared
    );
    drop(shared);
    println!("  after drop, upgrade: {:?}", weak.upgrade());

    println!("=== Trait objects (dyn dispatch) ===");
    let circle = Circle { radius: 2.0 };
    let shapes: [&dyn Shape; 1] = [&circle];
    println!(
        "  total area (dyn): {:.2}, describe: {}",
        total_area_dyn(&shapes),
        describe(&circle)
    );

    println!("=== Error handling (Result + custom errors) ===");
    let csv = "name,age\nAlice,30\nBob,25\n";
    match parse_csv(csv) {
        Ok(records) => println!(
            "  parsed {} records, first: {:?}",
            records.len(),
            records[0]
        ),
        Err(e) => println!("  error: {e}"),
    }
    match parse_csv("a,b\n\"broken") {
        Ok(_) => println!("  unexpected success"),
        Err(e) => println!("  malformed CSV error: {e}"),
    }

    println!("=== Thread pool (Arc + Mutex + channels) ===");
    let pool = ThreadPool::new(4);
    let (tx, rx) = std::sync::mpsc::channel();
    for i in 0..8 {
        let tx = tx.clone();
        pool.execute(move || {
            let _ = tx.send(i * i);
        })
        .expect("pool alive");
    }
    drop(tx);
    let mut squares: Vec<i32> = rx.iter().collect();
    squares.sort_unstable();
    println!("  squares: {squares:?}");

    println!("=== Async: our own executor ===");
    let result = block_on(async {
        rust_learning_and_dsa::advanced::TimerFuture::new(Duration::from_millis(5)).await;
        sum_of!(1, 2, 3, 4)
    });
    println!("  block_on(async) -> {result}");

    println!("=== FFI: calling the C runtime ===");
    println!(
        "  c_strlen(\"hello\") = {}",
        rust_learning_and_dsa::advanced::c_strlen("hello")
    );
    let mut points = vec![
        rust_learning_and_dsa::advanced::Point::new(3.0, 3.0, 3),
        rust_learning_and_dsa::advanced::Point::new(1.0, 1.0, 1),
    ];
    rust_learning_and_dsa::advanced::qsort_points(&mut points);
    println!(
        "  qsort ids: {:?}",
        points.iter().map(|p| p.id).collect::<Vec<_>>()
    );

    println!("=== Macros ===");
    let arr = dynarray![1, 2, 3];
    let map = hashmap! { "rust" => 1, "go" => 2 };
    println!(
        "  dynarray!: len {}, hashmap!: {:?}",
        arr.len(),
        map.get(&"rust")
    );
}
