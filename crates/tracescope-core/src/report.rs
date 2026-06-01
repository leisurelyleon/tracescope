//! The aggregate analysis report.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::critical_path::{critical_path as compute_critical_path, CriticalSpan};
use crate::flamegraph::{fold, FoldedStack};
use crate::percentile::percentile;
use crate::trace::Trace;

/// A named self-time contributor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Bottleneck {
    pub name: String,
    pub self_ns: u64,
}

/// The full analysis of a trace.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnalysisReport {
    pub span_count: usize,
    pub total_duration_ns: u64,
    pub p50_ns: f64,
    pub p95_ns: f64,
    pub p99_ns: f64,
    pub flame: Vec<FoldedStack>,
    pub critical_path: Vec<CriticalSpan>,
    pub bottlenecks: Vec<Bottleneck>,
}

impl AnalysisReport {
    /// A human-readable rendering of the report.
    pub fn format_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Spans: {}\n", self.span_count));
        out.push_str(&format!("Total duration: {} ns\n", self.total_duration_ns));
        out.push_str(&format!(
            "Latency p50/p95/p99: {:.1} / {:.1} / {:.1} ns\n",
            self.p50_ns, self.p95_ns, self.p99_ns
        ));

        out.push_str("\nTop bottlenecks (self time):\n");
        for bottleneck in self.bottlenecks.iter().take(5) {
            out.push_str(&format!("  {:<16} {} ns\n", bottleneck.name, bottleneck.self_ns));
        }

        out.push_str("\nCritical path:\n");
        for span in &self.critical_path {
            out.push_str(&format!("  {} ({} ns)\n", span.name, span.duration_ns));
        }

        out.push_str("\nFlamegraph (folded self time):\n");
        for stack in &self.flame {
            out.push_str(&format!("  {} {}\n", stack.stack.join(";"), stack.self_ns));
        }

        out
    }
}

/// Analyzes a trace, producing percentiles, a flamegraph, the critical path, and
/// ranked bottlenecks. Percentiles are computed over all span durations.
pub fn analyze(trace: &Trace) -> AnalysisReport {
    let durations: Vec<u64> = trace.spans.iter().map(|s| s.duration_ns()).collect();
    let flame = fold(trace);

    // Aggregate self time by span name to rank bottlenecks.
    let mut by_name: BTreeMap<String, u64> = BTreeMap::new();
    for stack in &flame {
        if let Some(name) = stack.stack.last() {
            *by_name.entry(name.clone()).or_default() += stack.self_ns;
        }
    }
    let mut bottlenecks: Vec<Bottleneck> = by_name
        .into_iter()
        .map(|(name, self_ns)| Bottleneck { name, self_ns })
        .collect();
    // Descending self time, then ascending name for stable ties.
    bottlenecks.sort_by(|a, b| b.self_ns.cmp(&a.self_ns).then_with(|| a.name.cmp(&b.name)));

    AnalysisReport {
        span_count: trace.span_count(),
        total_duration_ns: trace.total_duration_ns(),
        p50_ns: percentile(&durations, 50.0).unwrap_or(0.0),
        p95_ns: percentile(&durations, 95.0).unwrap_or(0.0),
        p99_ns: percentile(&durations, 99.0).unwrap_or(0.0),
        flame,
        critical_path: compute_critical_path(trace),
        bottlenecks,
    }
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
    fn reports_counts_and_totals() {
        let report = analyze(&sample());
        assert_eq!(report.span_count, 4);
        assert_eq!(report.total_duration_ns, 100);
    }

    #[test]
    fn ranks_largest_self_time_first() {
        let report = analyze(&sample());
        // self times: work 60, read 20, main 10, io 10 -> work leads.
        let top = report.bottlenecks.first().unwrap();
        assert_eq!(top.name, "work");
        assert_eq!(top.self_ns, 60);
    }

    #[test]
    fn format_text_includes_sections() {
        let text = analyze(&sample()).format_text();
        assert!(text.contains("Critical path"));
        assert!(text.contains("Top bottlenecks"));
    }
}
