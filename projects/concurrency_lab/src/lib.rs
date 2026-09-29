//! Two kinds of concurrency with different jobs:
//! - Rayon uses CPU workers to process independent data chunks.
//! - Tokio schedules tasks that spend most of their time waiting.

use rayon::prelude::*;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::task::{JoinError, JoinSet};

pub mod network;

/// Count whitespace-separated, lowercase tokens on the caller's thread.
/// O(n) time for n input bytes, excluding hash and allocation constants.
pub fn count_words_sequential(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// Count tokens on Rayon workers. Each worker owns its map during `fold`;
/// `reduce` combines maps after work finishes, avoiding a shared mutex on
/// every word. Parallel overhead may make this slower for small input.
pub fn count_words_parallel(text: &str) -> HashMap<String, usize> {
    text.par_lines()
        .fold(HashMap::new, |mut counts, line| {
            for word in line.split_whitespace() {
                *counts.entry(word.to_lowercase()).or_insert(0) += 1;
            }
            counts
        })
        .reduce(HashMap::new, |mut left, right| {
            for (word, count) in right {
                *left.entry(word).or_insert(0) += count;
            }
            left
        })
}

/// Apply an async operation with at most `limit` live tasks. Results retain
/// input order even when tasks finish out of order. Unlike spawning every
/// request up front, the task set and scheduler state stay O(limit).
///
/// Dropping the returned future drops `JoinSet`, which aborts its child tasks.
/// A spawned task panic returns a `JoinError` and aborts the remaining tasks.
/// Cancellation is cooperative: a task must reach an `.await` to stop.
///
/// # Panics
/// Panics if `limit` is zero.
pub async fn map_bounded<I, O, F, Fut>(
    items: Vec<I>,
    limit: usize,
    operation: F,
) -> Result<Vec<O>, JoinError>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Fn(I) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = O> + Send + 'static,
{
    assert!(limit > 0, "concurrency limit must be positive");
    let count = items.len();
    let mut pending = items.into_iter().enumerate();
    let mut results: Vec<Option<O>> = std::iter::repeat_with(|| None).take(count).collect();
    let operation = Arc::new(operation);
    let mut tasks = JoinSet::new();

    for _ in 0..limit.min(count) {
        let (index, item) = pending.next().expect("initial task exists");
        let operation = Arc::clone(&operation);
        tasks.spawn(async move { (index, operation(item).await) });
    }

    while let Some(joined) = tasks.join_next().await {
        let (index, output) = match joined {
            Ok(value) => value,
            Err(error) => {
                tasks.abort_all();
                return Err(error);
            }
        };
        results[index] = Some(output);
        if let Some((next_index, item)) = pending.next() {
            let operation = Arc::clone(&operation);
            tasks.spawn(async move { (next_index, operation(item).await) });
        }
    }

    Ok(results
        .into_iter()
        .map(|value| value.expect("one result per input"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn parallel_matches_sequential_for_multiple_input_shapes() {
        for text in ["", "Rust rust RUST", "a\nb b\nc c c", "naïve naïve café"] {
            assert_eq!(count_words_parallel(text), count_words_sequential(text));
        }
        let large = "Rust async parallel database\n".repeat(10_000);
        assert_eq!(count_words_parallel(&large), count_words_sequential(&large));
    }

    #[tokio::test]
    async fn bounded_tasks_preserve_order_and_limit() {
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let results = map_bounded((0..12).collect(), 3, {
            let active = Arc::clone(&active);
            let peak = Arc::clone(&peak);
            move |item| {
                let active = Arc::clone(&active);
                let peak = Arc::clone(&peak);
                async move {
                    let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(2 * (12 - item))).await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    item * item
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(results, (0..12).map(|x| x * x).collect::<Vec<_>>());
        assert!(peak.load(Ordering::SeqCst) <= 3);
        assert!(peak.load(Ordering::SeqCst) > 1);
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn empty_input_never_invokes_operation() {
        let values: Vec<i32> = map_bounded(Vec::<i32>::new(), 2, |_| async { 1 })
            .await
            .unwrap();
        assert!(values.is_empty());
    }

    #[tokio::test]
    async fn timeout_is_an_operation_result() {
        let values = map_bounded(vec![0_u64, 100], 2, |millis| async move {
            tokio::time::timeout(
                Duration::from_millis(10),
                tokio::time::sleep(Duration::from_millis(millis)),
            )
            .await
            .is_ok()
        })
        .await
        .unwrap();
        assert_eq!(values, vec![true, false]);
    }

    #[tokio::test]
    async fn dropping_outer_task_cancels_children() {
        struct DropSignal(Arc<AtomicUsize>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let started = Arc::new(AtomicUsize::new(0));
        let dropped = Arc::new(AtomicUsize::new(0));
        let task = tokio::spawn(map_bounded(vec![0, 1, 2], 2, {
            let started = Arc::clone(&started);
            let dropped = Arc::clone(&dropped);
            move |_| {
                let started = Arc::clone(&started);
                let dropped = Arc::clone(&dropped);
                async move {
                    let _signal = DropSignal(dropped);
                    started.fetch_add(1, Ordering::SeqCst);
                    std::future::pending::<()>().await;
                }
            }
        }));
        tokio::time::timeout(Duration::from_secs(1), async {
            while started.load(Ordering::SeqCst) < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        tokio::time::timeout(Duration::from_secs(1), async {
            while dropped.load(Ordering::SeqCst) < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(started.load(Ordering::SeqCst), 2);
        assert_eq!(dropped.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn child_panic_returns_join_error() {
        let result = map_bounded(vec![0], 1, |_| async { panic!("simulated task failure") }).await;
        assert!(result.unwrap_err().is_panic());
    }
}
