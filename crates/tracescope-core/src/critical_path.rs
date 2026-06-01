//! Critical-path analysis.
//!
//! The critical path is the chain of spans that gates total completion: starting
//! at the longest-running root, at each level we descend into the child that
//! *finishes last* (greatest end time), since that is what its parent waited on.
//! Ties break toward the smaller span id for determinism.

use std::cmp::Reverse;

use serde::Serialize;

use crate::trace::Trace;

/// One span on the critical path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CriticalSpan {
    pub id: u64,
    pub name: String,
    pub duration_ns: u64,
}

/// Computes the critical path through the trace.
pub fn critical_path(trace: &Trace) -> Vec<CriticalSpan> {
    let by_id = trace.index_by_id();
    let child_map = trace.child_map();

    // The governing root is the longest-running one (ties: smaller id).
    let Some(mut current) = trace
        .roots()
        .into_iter()
        .max_by_key(|s| (s.duration_ns(), Reverse(s.id)))
    else {
        return Vec::new();
    };

    let mut path = Vec::new();
    loop {
        path.push(CriticalSpan {
            id: current.id,
            name: current.name.clone(),
            duration_ns: current.duration_ns(),
        });

        // Descend into the last-finishing child (ties: smaller id).
        let next = child_map
            .get(&current.id)
            .into_iter()
            .flatten()
            .filter_map(|cid| by_id.get(cid).copied())
            .max_by_key(|s| (s.end_ns, Reverse(s.id)));

        match next {
            Some(child) => current = child,
            None => break,
        }
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    fn sample() -> Trace {
        Trace::new(vec![
            Span::new(1, "main", 0, 100, None),
            Span::new(2, "work", 0, 60, Some(1)),
            Span::new(3, "io", 60, 90, Some(1)),
            Span::new(4, "read", 60, 80, Some(3)),
        ])
    }

    #[test]
    fn follows_last_finishing_children() {
        // main's children: work ends 60, io ends 90 -> io. io's child read -> read.
        let path = critical_path(&sample());
        let names: Vec<&str> = path.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["main", "io", "read"]);
    }

    #[test]
    fn empty_trace_has_empty_path() {
        assert!(critical_path(&Trace::new(vec![])).is_empty());
    }
}
