//! Analysis-throughput benchmark.
//!
//! Measures how the pure analysis (`analyze`) scales with trace size. Builds
//! synthetic traces of increasing span counts and times a full analysis pass
//! (percentiles + flamegraph fold + critical path + bottleneck ranking).

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use tracescope_core::{analyze, Span, Trace};

/// Builds a trace: one root with `width` children, each child having one leaf.
/// Produces `1 + 2 * width` spans with a realistic two-level shape.
fn make_trace(width: u64) -> Trace {
    let mut spans = Vec::new();
    let total_end = (width + 1) * 100;
    spans.push(Span::new(0, "root", 0, total_end, None));

    let mut next_id = 1u64;
    for i in 0..width {
        let start = i * 100;
        let mid = start + 60;
        let end = start + 100;
        let child_id = next_id;
        next_id += 1;
        spans.push(Span::new(child_id, "child", start, end, Some(0)));

        let leaf_id = next_id;
        next_id += 1;
        spans.push(Span::new(leaf_id, "leaf", start, mid, Some(child_id)));
    }

    Trace::new(spans)
}

fn bench_analyze(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyze");

    for &width in &[100u64, 1_000, 10_000] {
        let trace = make_trace(width);
        group.bench_with_input(BenchmarkId::from_parameter(width), &trace, |b, t| {
            b.iter(|| {
                let report = analyze(black_box(t));
                black_box(report.span_count)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_analyze);
criterion_main!(benches);
