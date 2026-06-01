//! End-to-end integration: instrument a workload with the collector (driven by
//! a deterministic clock), then analyze the resulting trace and assert the
//! analysis matches the known, hand-computed shape of the workload.

use tracescope_collect::{Collector, ManualClock};
use tracescope_core::analyze;

/// Builds the same deterministic workload the CLI demo uses, returning its
/// analyzed report. Timeline (all in ns):
///   handle_request [0, 75)
///     db_query     [5, 45)      -> 40 self
///     render       [45, 70)
///       serialize  [60, 70)     -> 10 self
fn run_workload() -> tracescope_core::AnalysisReport {
    let collector = Collector::new(ManualClock::new());
    {
        let _request = collector.span("handle_request");
        collector.clock().advance(5);
        {
            let _db = collector.span("db_query");
            collector.clock().advance(40);
        }
        {
            let _render = collector.span("render");
            collector.clock().advance(15);
            {
                let _serialize = collector.span("serialize");
                collector.clock().advance(10);
            }
        }
        collector.clock().advance(5);
    }
    let trace = collector.finish();
    trace.validate().expect("collected trace should be structurally valid");
    analyze(&trace)
}

#[test]
fn collected_workload_has_expected_shape() {
    let report = run_workload();
    assert_eq!(report.span_count, 4);
    assert_eq!(report.total_duration_ns, 75);
}

#[test]
fn db_query_is_the_top_bottleneck() {
    let report = run_workload();
    // Self times: db_query 40, render 15 (25 - serialize 10), serialize 10,
    // handle_request 10 (75 - 40 - 25). db_query leads.
    let top = report.bottlenecks.first().expect("expected at least one bottleneck");
    assert_eq!(top.name, "db_query");
    assert_eq!(top.self_ns, 40);
}

#[test]
fn critical_path_follows_last_finishing_children() {
    let report = run_workload();
    // root -> render (ends 70, after db_query's 45) -> serialize (ends 70).
    let names: Vec<&str> = report.critical_path.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["handle_request", "render", "serialize"]);
}

#[test]
fn flame_self_times_sum_to_total() {
    let report = run_workload();
    let total: u64 = report.flame.iter().map(|s| s.self_ns).sum();
    assert_eq!(total, report.total_duration_ns);
}
