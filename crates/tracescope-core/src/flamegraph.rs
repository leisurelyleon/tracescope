//! Flamegraph aggregation: fold the trace tree into per-stack self-time.
//!
//! A span's *self time* is its own duration minus the time accounted for by its
//! direct children. Folding accumulates self time keyed by the call stack (the
//! sequence of span names from a root down to the span), so repeated stacks
//! aggregate — exactly the structure a flamegraph renders.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::span::{Span, SpanId};
use crate::trace::Trace;

/// One folded stack: a root-to-node name path and its accumulated self time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FoldedStack {
    pub stack: Vec<String>,
    pub self_ns: u64,
}

/// Folds a trace into its flamegraph stacks, ordered deterministically.
pub fn fold(trace: &Trace) -> Vec<FoldedStack> {
    let by_id = trace.index_by_id();
    let child_map = trace.child_map();
    let mut acc: BTreeMap<Vec<String>, u64> = BTreeMap::new();

    for root in trace.roots() {
        fold_span(root, &[], &by_id, &child_map, &mut acc);
    }

    acc.into_iter()
        .map(|(stack, self_ns)| FoldedStack { stack, self_ns })
        .collect()
}

fn fold_span(
    span: &Span,
    prefix: &[String],
    by_id: &HashMap<SpanId, &Span>,
    child_map: &HashMap<SpanId, Vec<SpanId>>,
    acc: &mut BTreeMap<Vec<String>, u64>,
) {
    let mut stack = prefix.to_vec();
    stack.push(span.name.clone());

    let child_ids = child_map.get(&span.id);
    let children_total: u64 = child_ids
        .into_iter()
        .flatten()
        .filter_map(|cid| by_id.get(cid).copied())
        .map(Span::duration_ns)
        .sum();
    let self_ns = span.duration_ns().saturating_sub(children_total);

    *acc.entry(stack.clone()).or_default() += self_ns;

    for cid in child_ids.into_iter().flatten() {
        if let Some(child) = by_id.get(cid).copied() {
            fold_span(child, &stack, by_id, child_map, acc);
        }
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

    fn self_of(folded: &[FoldedStack], path: &[&str]) -> Option<u64> {
        let target: Vec<String> = path.iter().map(|s| (*s).to_string()).collect();
        folded.iter().find(|f| f.stack == target).map(|f| f.self_ns)
    }

    #[test]
    fn self_times_sum_to_root_duration() {
        let folded = fold(&sample());
        // main: 100 - (work 60 + io 30) = 10
        assert_eq!(self_of(&folded, &["main"]), Some(10));
        // work: leaf -> 60
        assert_eq!(self_of(&folded, &["main", "work"]), Some(60));
        // io: 30 - (read 20) = 10
        assert_eq!(self_of(&folded, &["main", "io"]), Some(10));
        // read: leaf -> 20
        assert_eq!(self_of(&folded, &["main", "io", "read"]), Some(20));

        let total: u64 = folded.iter().map(|f| f.self_ns).sum();
        assert_eq!(total, 100);
    }
}
