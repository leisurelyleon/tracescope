//! Clock abstraction: a real system clock and a deterministic manual clock.
//!
//! Abstracting the clock is what makes timing-dependent collection testable:
//! tests drive a `ManualClock` with explicit advances, so span durations are
//! exact and reproducible — no flaky `sleep`-based timing.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// A source of monotonically non-decreasing nanosecond timestamps.
pub trait Clock: Send + Sync {
    fn now_ns(&self) -> u64;
}

/// A real clock measuring nanoseconds since its creation.
pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_ns(&self) -> u64 {
        self.origin.elapsed().as_nanos() as u64
    }
}

/// A deterministic clock advanced explicitly by tests and demos.
#[derive(Default)]
pub struct ManualClock {
    now: AtomicU64,
}

impl ManualClock {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances the clock by `delta_ns`.
    pub fn advance(&self, delta_ns: u64) {
        self.now.fetch_add(delta_ns, Ordering::SeqCst);
    }

    /// Sets the clock to an absolute value.
    pub fn set(&self, value_ns: u64) {
        self.now.store(value_ns, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_ns(&self) -> u64 {
        self.now.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock_advances_and_sets() {
        let clock = ManualClock::new();
        assert_eq!(clock.now_ns(), 0);
        clock.advance(10);
        assert_eq!(clock.now_ns(), 10);
        clock.set(100);
        assert_eq!(clock.now_ns(), 100);
    }

    #[test]
    fn system_clock_is_non_decreasing() {
        let clock = SystemClock::new();
        let a = clock.now_ns();
        let b = clock.now_ns();
        assert!(b >= a);
    }
}
