//! A minimal async executor: `block_on`, built from scratch with only `std`.
//!
//! This is the async book's executor condensed. It demystifies the three
//! pieces that make `async`/`await` work:
//!
//! - **`Future`**: an async computation that returns `Poll::Pending` (not
//!   ready) or `Poll::Ready(output)` from `poll`. An `async fn` compiles down
//!   to a state machine implementing `Future`.
//! - **`Pin`**: polling needs a *stable address* — the future's internal state
//!   machine holds references across `poll` calls, so it must not move.
//!   `Box::pin` puts it on the heap and pins it.
//! - **`Waker`**: the bridge between "something happened" and "poll me again".
//!   `poll` receives a `Context` holding a `Waker`; the future stores it and
//!   calls `wake()` when an external event (I/O, timer, message) completes.
//!   A `Waker` is a raw pointer + **vtable** — the same dynamic-dispatch
//!   mechanism as `dyn Trait`.
//!
//! The executor itself is a trivial loop: poll; if `Pending`, park the thread
//! until the waker unparks it; repeat.
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::advanced::block_on;
//! use std::time::Duration;
//!
//! let output = block_on(async {
//!     rust_learning_and_dsa::advanced::TimerFuture::new(Duration::from_millis(5)).await;
//!     42
//! });
//! assert_eq!(output, 42);
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;
use std::time::Duration;

// ---------------------------------------------------------------------------
// The executor
// ---------------------------------------------------------------------------

/// Shared state between the executor loop and every waker clone.
struct ExecutorState {
    /// Set by `wake()`; cleared by the executor before parking.
    woken: AtomicBool,
    /// The thread to unpark.
    thread: thread::Thread,
}

/// Runs `future` to completion on the current thread, blocking it.
///
/// `async fn` blocks and `.await` expressions work inside — no runtime crate
/// needed, because the executor *is* this function.
pub fn block_on<F: Future>(future: F) -> F::Output {
    // Pin the future so it never moves between polls.
    let mut future = Box::pin(future);

    let state = Arc::new(ExecutorState {
        woken: AtomicBool::new(true), // poll once before parking
        thread: thread::current(),
    });

    // SAFETY: the raw waker holds an Arc<ExecutorState> and its vtable keeps
    // the refcount correct on clone/drop.
    let waker = unsafe {
        Waker::from_raw(RawWaker::new(
            Arc::into_raw(state.clone()).cast(),
            waker_vtable(),
        ))
    };
    let mut cx = Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                // Park until a waker says "poll again". The flag closes the
                // race between "woken just before parking" and the park call
                // itself: unpark is sticky, so a wake during the window still
                // releases us.
                state.woken.store(false, Ordering::SeqCst);
                if !state.woken.load(Ordering::SeqCst) {
                    thread::park();
                }
            }
        }
    }
}

/// Builds the waker vtable for `Arc<ExecutorState>`.
fn waker_vtable() -> &'static RawWakerVTable {
    // SAFETY: every function here operates on a valid Arc<ExecutorState>
    // produced by `block_on`; clone/drop keep the refcount balanced.
    &RawWakerVTable::new(
        |data| {
            let state = unsafe { Arc::from_raw(data.cast::<ExecutorState>()) };
            let cloned = Arc::clone(&state);
            // Re-package the new Arc; the original `state` refcount stays.
            let _ = state;
            RawWaker::new(Arc::into_raw(cloned).cast(), waker_vtable())
        },
        |data| {
            // wake() consumes one reference.
            let state = unsafe { Arc::from_raw(data.cast::<ExecutorState>()) };
            state.woken.store(true, Ordering::SeqCst);
            state.thread.unpark();
        },
        |data| {
            // wake_by_ref() does not consume.
            let state = unsafe { &*data.cast::<ExecutorState>() };
            state.woken.store(true, Ordering::SeqCst);
            state.thread.unpark();
        },
        |data| {
            let _ = unsafe { Arc::from_raw(data.cast::<ExecutorState>()) };
        },
    )
}

// ---------------------------------------------------------------------------
// Futures you can await
// ---------------------------------------------------------------------------

/// A future that completes after `duration`, waking the executor from a
/// background thread. The classic teaching example of a real wakeup source.
pub struct TimerFuture {
    inner: Arc<TimerState>,
}

struct TimerState {
    completed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

impl TimerFuture {
    /// Creates a future that completes after `duration`. O(1).
    ///
    /// # Panics
    ///
    /// Never in practice; the internal mutex is only poisoned if a waker
    /// panics while holding it, which our wakers never do.
    #[must_use]
    pub fn new(duration: Duration) -> Self {
        let inner = Arc::new(TimerState {
            completed: AtomicBool::new(false),
            waker: Mutex::new(None),
        });
        let spawned = Arc::clone(&inner);
        thread::spawn(move || {
            thread::sleep(duration);
            spawned.completed.store(true, Ordering::SeqCst);
            // Wake whatever executor is polling us, if any.
            if let Some(waker) = spawned.waker.lock().expect("waker lock").take() {
                waker.wake();
            }
        });
        Self { inner }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.inner.completed.load(Ordering::SeqCst) {
            Poll::Ready(())
        } else {
            // Register this poll's waker so the timer thread can wake us.
            *self.inner.waker.lock().expect("waker lock") = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// A future that yields `n` times before completing. Each `poll` returns
/// `Pending` and asks to be re-polled — a simple CPU-bound coroutine.
pub struct YieldMany {
    remaining: usize,
}

impl YieldMany {
    /// Creates a future that yields `n` times. O(1).
    #[must_use]
    pub const fn new(n: usize) -> Self {
        Self { remaining: n }
    }
}

impl Future for YieldMany {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.remaining == 0 {
            Poll::Ready(())
        } else {
            self.remaining -= 1;
            // Ask the executor to poll us again (this makes the executor
            // spin; real I/O futures only wake on actual events).
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// A demo `async fn` — the compiler turns this into a state-machine `Future`.
///
/// # Examples
///
/// ```
/// use rust_learning_and_dsa::advanced::{block_on, sum_after_yields};
///
/// assert_eq!(block_on(sum_after_yields(5)), 10); // 0 + 1 + 2 + 3 + 4
/// ```
pub async fn sum_after_yields(n: usize) -> usize {
    let mut total = 0;
    for i in 0..n {
        YieldMany::new(2).await;
        total += i;
    }
    total
}

/// An `async fn` that sleeps between steps, showing real wakeups.
pub async fn sleep_then_double(value: u32, millis: u64) -> u32 {
    TimerFuture::new(Duration::from_millis(millis)).await;
    value * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_on_ready_future() {
        assert_eq!(block_on(async { 42 }), 42);
    }

    #[test]
    fn block_on_async_fn_with_yields() {
        assert_eq!(block_on(sum_after_yields(0)), 0);
        assert_eq!(block_on(sum_after_yields(5)), 10);
        assert_eq!(block_on(sum_after_yields(100)), 4950);
    }

    #[test]
    fn block_on_async_fn_with_timer() {
        let start = std::time::Instant::now();
        let result = block_on(sleep_then_double(21, 20));
        assert_eq!(result, 42);
        // The executor actually waited for the timer thread.
        assert!(start.elapsed() >= Duration::from_millis(20));
    }

    #[test]
    fn timer_future_completes() {
        let output = block_on(async {
            TimerFuture::new(Duration::from_millis(5)).await;
            TimerFuture::new(Duration::from_millis(5)).await;
            "done"
        });
        assert_eq!(output, "done");
    }

    #[test]
    fn nested_async_blocks() {
        // block_on inside block_on — each has its own park/unpark state.
        let outer = block_on(async {
            let inner = block_on(async { 1 + 1 });
            inner * 10
        });
        assert_eq!(outer, 20);
    }

    #[test]
    fn yields_are_actually_suspended() {
        // A yield future must go through Pending (polled more than once).
        let polls = block_on(async {
            let mut polls = 0;
            {
                let mut future = Box::pin(YieldMany::new(3));
                let (tx, rx) = std::sync::mpsc::channel();
                thread::spawn(move || {
                    let state = Arc::new(ExecutorState {
                        woken: AtomicBool::new(true),
                        thread: thread::current(),
                    });
                    // SAFETY: test-only waker with a valid Arc state.
                    let waker = unsafe {
                        Waker::from_raw(RawWaker::new(
                            Arc::into_raw(state.clone()).cast(),
                            waker_vtable(),
                        ))
                    };
                    let mut cx = Context::from_waker(&waker);
                    while future.as_mut().poll(&mut cx).is_pending() {
                        polls += 1;
                        waker.wake_by_ref();
                    }
                    let _ = tx.send(polls);
                });
                polls = rx.recv().expect("producer alive");
            }
            polls
        });
        // 3 yields + 1 final ready poll.
        assert_eq!(polls, 3);
    }
}
