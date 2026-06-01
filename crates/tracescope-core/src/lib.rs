//! Pure trace model and performance-analysis algorithms for `tracescope`.
//!
//! Everything here is a pure function over a [`Trace`]: percentiles, flamegraph
//! folding, critical-path analysis, and the aggregate [`AnalysisReport`]. No
//! I/O, no timing, no threads — so every metric is unit-tested against
//! hand-computed values.

pub mod critical_path;
pub mod error;
pub mod flamegraph;
pub mod percentile;
pub mod report;
pub mod span;
pub mod trace;

pub use critical_path::CriticalSpan;
pub use error::CoreError;
pub use flamegraph::FoldedStack;
pub use report::{analyze, AnalysisReport, Bottleneck};
pub use span::{Span, SpanId};
pub use trace::Trace;
