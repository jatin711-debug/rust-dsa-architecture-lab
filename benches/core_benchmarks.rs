//! Criterion benchmarks for the core data structures.
//!
//! Run with: `cargo bench --bench core_benchmarks`

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rust_learning_and_dsa::algorithms::{heapsort, insertion_sort, mergesort, quicksort};
use rust_learning_and_dsa::heap::BinaryHeap;
use rust_learning_and_dsa::linear::{DynamicArray, Queue, Stack};
use rust_learning_and_dsa::map::HashMap;
use rust_learning_and_dsa::tree::{AvlTree, BinarySearchTree};

const N: usize = 10_000;

fn bench_dynamic_array(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamic_array");
    group.bench_function("push_10k", |b| {
        b.iter(|| {
            let mut arr = DynamicArray::new();
            for i in 0..N {
                arr.push(i);
            }
            std::hint::black_box(arr.len());
        })
    });
    group.bench_function("index_sum_10k", |b| {
        let arr: DynamicArray<usize> = (0..N).collect();
        b.iter(|| {
            let mut sum = 0usize;
            for i in 0..N {
                sum += arr[i];
            }
            std::hint::black_box(sum);
        })
    });
    group.finish();
}

fn bench_stack_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack_queue");
    group.bench_function("stack_push_pop_10k", |b| {
        b.iter(|| {
            let mut stack = Stack::new();
            for i in 0..N {
                stack.push(i);
            }
            while stack.pop().is_some() {}
        })
    });
    group.bench_function("queue_enqueue_dequeue_10k", |b| {
        b.iter(|| {
            let mut queue = Queue::new();
            for i in 0..N {
                queue.enqueue(i);
            }
            while queue.dequeue().is_some() {}
        })
    });
    group.finish();
}

fn bench_hash_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_map");
    group.bench_function("insert_10k", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..N {
                map.insert(i, i * 2);
            }
            std::hint::black_box(map.len());
        })
    });
    group.bench_function("get_10k", |b| {
        let map: HashMap<usize, usize> = (0..N).map(|i| (i, i * 2)).collect();
        b.iter(|| {
            let mut sum = 0usize;
            for i in 0..N {
                sum += map.get(&i).copied().unwrap_or(0);
            }
            std::hint::black_box(sum);
        })
    });
    group.finish();
}

fn bench_trees(c: &mut Criterion) {
    // A plain BST degenerates on sorted input; use the same permutation for
    // both trees when comparing typical insertion cost.
    let input = shuffled(N);
    let mut group = c.benchmark_group("trees");
    group.bench_function("bst_insert_10k", |b| {
        b.iter(|| {
            let mut tree = BinarySearchTree::new();
            for &i in &input {
                tree.insert(i);
            }
            std::hint::black_box(tree.len());
        })
    });
    group.bench_function("avl_insert_10k", |b| {
        b.iter(|| {
            let mut tree = AvlTree::new();
            for &i in &input {
                tree.insert(i);
            }
            std::hint::black_box(tree.len());
        })
    });
    group.finish();
}

fn bench_heap(c: &mut Criterion) {
    let mut group = c.benchmark_group("heap");
    group.bench_function("push_pop_10k", |b| {
        b.iter(|| {
            let mut heap = BinaryHeap::new();
            for i in 0..N {
                heap.push(i);
            }
            while heap.pop().is_some() {}
        })
    });
    group.finish();
}

fn bench_sizes(c: &mut Criterion) {
    // Show how one operation scales across input sizes.
    let mut group = c.benchmark_group("scaling");
    for size in [1_000usize, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::new("hashmap_insert", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut map = HashMap::new();
                    for i in 0..size {
                        map.insert(i, i);
                    }
                    std::hint::black_box(map.len());
                })
            },
        );
    }
    group.finish();
}

/// Deterministic pseudo-random permutation of `0..n` (no external deps).
fn shuffled(n: usize) -> Vec<usize> {
    let mut data: Vec<usize> = (0..n).collect();
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    for i in (1..n).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = ((state >> 33) as usize) % (i + 1);
        data.swap(i, j);
    }
    data
}

fn bench_sorting(c: &mut Criterion) {
    let n = 10_000;
    let random = shuffled(n);
    let sorted: Vec<usize> = (0..n).collect();
    let reverse: Vec<usize> = (0..n).rev().collect();
    let few_distinct: Vec<usize> = (0..n).map(|i| i % 4).collect();

    // Our implementations vs the standard library on random data.
    let mut group = c.benchmark_group("sorting_random_10k");
    for (name, sort) in [
        ("ours_quicksort", quicksort as fn(&mut [usize])),
        ("ours_mergesort", mergesort),
        ("ours_heapsort", heapsort),
        ("ours_insertion", insertion_sort),
        ("std_sort", |s: &mut [usize]| s.sort()),
        ("std_sort_unstable", |s: &mut [usize]| s.sort_unstable()),
    ] {
        group.bench_function(name, |b| {
            b.iter_batched(
                || random.clone(),
                |mut data| sort(&mut data),
                criterion::BatchSize::SmallInput,
            )
        });
    }
    group.finish();

    // Quicksort behavior on adversarial inputs.
    let mut group = c.benchmark_group("quicksort_inputs_10k");
    for (name, input) in [
        ("random", &random),
        ("sorted", &sorted),
        ("reverse", &reverse),
        ("few_distinct", &few_distinct),
    ] {
        group.bench_function(name, |b| {
            b.iter_batched(
                || input.clone(),
                |mut data| quicksort(&mut data),
                criterion::BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_dynamic_array,
    bench_stack_queue,
    bench_hash_map,
    bench_trees,
    bench_heap,
    bench_sorting,
    bench_sizes
);
criterion_main!(benches);
