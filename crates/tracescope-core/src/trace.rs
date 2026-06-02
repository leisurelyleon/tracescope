//! A trace: a collection of spans forming a parent/child forest.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::span::{Span, SpanId};

/// A trace is a flat list of spans linked by parent ids. Analysis treats it as a
/// forest of trees (one tree per parentless root).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub spans: Vec<Span>,
}

impl Trace {
    pub fn new(spans: Vec<Span>) -> Self {
        Self { spans }
    }

    pub fn from_json(text: &str) -> Result<Self, CoreError> {
        Ok(serde_json::from_str(text)?)
    }

    pub fn to_json(&self) -> Result<String, CoreError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn span_count(&self) -> usize {
        self.spans.len()
    }

    /// A span by id, if present.
    pub fn span(&self, id: SpanId) -> Option<&Span> {
        self.spans.iter().find(|s| s.id == id)
    }

    /// The total wall-clock span of the trace: latest end minus earliest start.
    pub fn total_duration_ns(&self) -> u64 {
        let min_start = self.spans.iter().map(|s| s.start_ns).min();
        let max_end = self.spans.iter().map(|s| s.end_ns).max();
        match (min_start, max_end) {
            (Some(start), Some(end)) => end.saturating_sub(start),
            _ => 0,
        }
    }

    /// The parentless root spans, sorted by id for determinism.
    pub fn roots(&self) -> Vec<&Span> {
        let mut roots: Vec<&Span> = self.spans.iter().filter(|s| s.parent.is_none()).collect();
        roots.sort_by_key(|s| s.id);
        roots
    }

    /// Map from span id to span reference.
    pub fn index_by_id(&self) -> HashMap<SpanId, &Span> {
        self.spans.iter().map(|s| (s.id, s)).collect()
    }

    /// Map from a parent id to its child ids, each child list sorted for
    /// deterministic traversal.
    pub fn child_map(&self) -> HashMap<SpanId, Vec<SpanId>> {
        let mut map: HashMap<SpanId, Vec<SpanId>> = HashMap::new();
        for span in &self.spans {
            if let Some(parent) = span.parent {
                map.entry(parent).or_default().push(span.id);
            }
        }
        for ids in map.values_mut() {
            ids.sort_unstable();
        }
        map
    }

    /// Validates structural integrity: unique ids, non-inverted intervals,
    /// existing parents, and absence of cycles.
    pub fn validate(&self) -> Result<(), CoreError> {
        let mut ids: HashSet<SpanId> = HashSet::new();
        for span in &self.spans {
            if !ids.insert(span.id) {
                return Err(CoreError::Validation(format!(
                    "duplicate span id {}",
                    span.id
                )));
            }
            if span.end_ns < span.start_ns {
                return Err(CoreError::Validation(format!(
                    "span {} ends before it starts",
                    span.id
                )));
            }
        }

        for span in &self.spans {
            if let Some(parent) = span.parent {
                if !ids.contains(&parent) {
                    return Err(CoreError::Validation(format!(
                        "span {} references missing parent {}",
                        span.id, parent
                    )));
                }
            }
        }

        // Cycle detection: walking any parent chain must terminate within the
        // number of spans.
        let by_id = self.index_by_id();
        for span in &self.spans {
            let mut steps = 0usize;
            let mut current = span.parent;
            while let Some(parent_id) = current {
                steps += 1;
                if steps > self.spans.len() {
                    return Err(CoreError::Validation(format!(
                        "cycle detected involving span {}",
                        span.id
                    )));
                }
                current = by_id.get(&parent_id).and_then(|p| p.parent);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Trace {
        Trace::new(vec![
            Span::new(1, "main", 0, 100, None),
            Span::new(2, "work", 0, 60, Some(1)),
            Span::new(3, "io", 60, 90, Some(1)),
            Span::new(4, "read", 60, 80, Some(3)),
        ])
    }

    #[test]
    fn json_roundtrip_preserves_spans() {
        let json = sample().to_json().unwrap();
        let back = Trace::from_json(&json).unwrap();
        assert_eq!(back.span_count(), 4);
        assert_eq!(back.span(3).unwrap().name, "io");
    }

    #[test]
    fn total_duration_spans_the_trace() {
        assert_eq!(sample().total_duration_ns(), 100);
    }

    #[test]
    fn roots_are_parentless() {
        let trace = sample();
        let roots = trace.roots();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, 1);
    }

    #[test]
    fn validate_accepts_well_formed_trace() {
        assert!(sample().validate().is_ok());
    }

    #[test]
    fn validate_rejects_missing_parent() {
        let trace = Trace::new(vec![Span::new(1, "a", 0, 10, Some(99))]);
        assert!(trace.validate().is_err());
    }

    #[test]
    fn validate_rejects_duplicate_id() {
        let trace = Trace::new(vec![
            Span::new(1, "a", 0, 10, None),
            Span::new(1, "b", 0, 5, None),
        ]);
        assert!(trace.validate().is_err());
    }

    #[test]
    fn validate_rejects_cycle() {
        let trace = Trace::new(vec![
            Span::new(1, "a", 0, 10, Some(2)),
            Span::new(2, "b", 0, 10, Some(1)),
        ]);
        assert!(trace.validate().is_err());
    }
}
