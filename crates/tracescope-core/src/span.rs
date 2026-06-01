//! The span: one timed unit of work.

use serde::{Deserialize, Serialize};

/// A span identifier.
pub type SpanId = u64;

/// A single span: a named interval `[start_ns, end_ns)` with an optional parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub id: SpanId,
    pub name: String,
    pub start_ns: u64,
    pub end_ns: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<SpanId>,
}

impl Span {
    pub fn new(
        id: SpanId,
        name: impl Into<String>,
        start_ns: u64,
        end_ns: u64,
        parent: Option<SpanId>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            start_ns,
            end_ns,
            parent,
        }
    }

    /// Wall-clock duration of the span, saturating at zero.
    pub fn duration_ns(&self) -> u64 {
        self.end_ns.saturating_sub(self.start_ns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_is_end_minus_start() {
        let span = Span::new(1, "x", 10, 35, None);
        assert_eq!(span.duration_ns(), 25);
    }

    #[test]
    fn duration_saturates_when_inverted() {
        let span = Span::new(1, "x", 50, 10, None);
        assert_eq!(span.duration_ns(), 0);
    }
}
