//! A small thread pool — the canonical exercise for real concurrency.
//!
//! It exercises everything the data structures couldn't:
//!
//! - **`Arc<Mutex<Receiver>>`**: the job queue is shared between all workers;
//!   `Mutex` makes the `Receiver` safe to share (it is `Send` but not `Sync`),
//!   and `Arc` gives shared ownership across threads.
//! - **Channels** (`std::sync::mpsc`): the *sender* is the pool, the
//!   *receiver* is wrapped in the shared mutex. When every sender drops, the
//!   channel closes and workers exit — that's the graceful-shutdown signal.
//! - **`Box<dyn FnOnce() + Send + 'static>`**: a trait object as a job, so any
//!   closure that can be sent to another thread can be executed.
//! - **`Send` / `Sync` in practice**: the pool must be `Send` (its parts are),
//!   and `execute` must accept only `Send` jobs.
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::advanced::ThreadPool;
//! use std::sync::mpsc;
//!
//! let pool = ThreadPool::new(4);
//! let (tx, rx) = mpsc::channel();
//! for i in 0..8 {
//!     let tx = tx.clone();
//!     pool.execute(move || {
//!         let _ = tx.send(i * i);
//!     });
//! }
//! drop(tx);
//! let mut squares: Vec<i32> = rx.iter().collect();
//! squares.sort_unstable();
//! assert_eq!(squares, vec![0, 1, 4, 9, 16, 25, 36, 49]);
//! ```

use std::fmt;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// A unit of work: any closure that can be moved to another thread.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Error returned when submitting a job to a pool that has shut down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolError;

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "thread pool is shut down")
    }
}

impl std::error::Error for PoolError {}

/// A fixed-size pool of worker threads.
///
/// Jobs submitted with [`ThreadPool::execute`] run on whichever worker picks
/// them up. Dropping the pool finishes pending jobs and joins the workers.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    /// Creates a pool with `size` worker threads. O(size).
    ///
    /// # Panics
    ///
    /// Panics if `size == 0`.
    #[must_use]
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "thread pool needs at least one worker");

        let (sender, receiver) = mpsc::channel();
        // One receiver, many consumers: every worker needs a handle, so the
        // receiver lives behind Arc<Mutex<...>>.
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Submits a job for execution by some worker. O(1) queueing.
    ///
    /// # Errors
    ///
    /// Returns [`PoolError`] if the pool is already shutting down (all workers
    /// gone). Defensive: with `&self`, the pool outlives its sender, so this
    /// cannot occur through the public API.
    ///
    /// # Panics
    ///
    /// Never in practice: the sender is `Some` while the pool is alive.
    pub fn execute<F>(&self, job: F) -> Result<(), PoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .as_ref()
            .expect("sender exists while pool is alive")
            .send(Box::new(job))
            .map_err(|_| PoolError)
    }

    /// Number of worker threads. O(1).
    #[must_use]
    pub fn size(&self) -> usize {
        self.workers.len()
    }
}

impl fmt::Debug for ThreadPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThreadPool")
            .field("workers", &self.workers.len())
            .finish_non_exhaustive()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Dropping the sender closes the channel. Workers see `Err` from
        // `recv` and exit their loop; we then join them all.
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                // SAFETY of semantics: joining is safe; the worker exits
                // because the channel closed.
                let _ = handle.join();
            }
        }
    }
}

/// One worker: a thread that pulls jobs off the shared queue forever.
struct Worker {
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(_id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                // Holding the lock only for `recv` (not while running the job)
                // keeps contention low and lets workers run jobs concurrently.
                let job = receiver.lock().expect("receiver mutex poisoned").recv();
                match job {
                    Ok(job) => job(),
                    // Channel closed: all senders dropped, pool is shutting down.
                    Err(_) => break,
                }
            }
        });
        Self {
            handle: Some(handle),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_jobs_and_returns_results() {
        let pool = ThreadPool::new(4);
        let (tx, rx) = mpsc::channel();
        for i in 0..32 {
            let tx = tx.clone();
            pool.execute(move || {
                let _ = tx.send(i);
            })
            .expect("pool alive");
        }
        drop(tx);
        let results: Vec<usize> = rx.iter().collect();
        assert_eq!(results.len(), 32);
        // Every job ran exactly once.
        let mut sorted = results.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..32).collect::<Vec<_>>());
    }

    #[test]
    fn jobs_run_concurrently() {
        let pool = ThreadPool::new(4);
        let (tx, rx) = mpsc::channel();
        let start = std::time::Instant::now();
        for _ in 0..4 {
            let tx = tx.clone();
            pool.execute(move || {
                thread::sleep(std::time::Duration::from_millis(200));
                let _ = tx.send(());
            })
            .expect("pool alive");
        }
        drop(tx);
        assert_eq!(rx.iter().count(), 4);
        // 4 jobs x 200ms on 4 workers should take ~200ms, not ~800ms.
        assert!(start.elapsed() < std::time::Duration::from_millis(600));
    }

    #[test]
    fn execute_while_alive_succeeds() {
        let pool = ThreadPool::new(1);
        assert_eq!(pool.execute(|| {}), Ok(()));
        // The error path (`PoolError`) is defensive: with `execute(&self)`,
        // the pool always outlives its sender, so `send` can only fail if the
        // receiver side vanished — which cannot happen through the public API.
        // We still verify the mapping on a manually closed channel:
        let (tx, rx) = mpsc::channel::<Job>();
        drop(rx);
        let result = tx.send(Box::new(|| {})).map_err(|_| PoolError);
        assert_eq!(result, Err(PoolError));
    }

    #[test]
    fn zero_workers_panics() {
        let result = std::panic::catch_unwind(|| ThreadPool::new(0));
        assert!(result.is_err());
    }

    #[test]
    fn pool_drop_joins_workers() {
        let pool = ThreadPool::new(2);
        let (tx, rx) = mpsc::channel();
        pool.execute(move || {
            let _ = tx.send("done");
        })
        .expect("pool alive");
        // Dropping the pool must let the pending job finish and the worker
        // threads exit cleanly (join succeeds).
        drop(pool);
        assert_eq!(rx.recv(), Ok("done"));
    }

    #[test]
    fn many_jobs_short_lived_pools() {
        // Repeatedly create/drop pools: no panics, no leaked threads (the
        // test process exiting cleanly is the assertion).
        for _ in 0..50 {
            let pool = ThreadPool::new(3);
            for i in 0..10 {
                let _ = pool.execute(move || {
                    std::hint::black_box(i);
                });
            }
        }
    }
}
