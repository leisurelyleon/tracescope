//! The runtime span collector for `tracescope`.
//!
//! Instrument code with [`Collector::span`], which returns an RAII guard that
//! times a span for its lexical scope. Timing comes from a pluggable [`Clock`];
//! a [`ManualClock`] makes collection fully deterministic for tests.

pub mod clock;
pub mod collector;
pub mod guard;

pub use clock::{Clock, ManualClock, SystemClock};
pub use collector::Collector;
pub use guard::SpanGuard;
