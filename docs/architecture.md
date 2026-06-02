# Architecture

`tracescope` is a Rust workspace implementing a tracing-and-analysis toolkit. It
separates the pure analysis algorithms from the runtime collection of spans, so
the interesting logic is deterministic and fully testable.

## Crates

```text
tracescope-core pure trace model + analysis: percentiles, flamegraph, critical path
tracescope-collect runtime collector: a pluggable Clock and an RAII span guard
tracescope-cli the tracescope binary (analyze a trace file, or run a demo)
```

## Pure core, runtime edge

`tracescope-core` is pure: a `Trace` is a flat list of `Span`s linked by parent
ids, and every analysis (`percentile`, `fold`, `critical_path`, `analyze`) is a
pure function over it. No clocks, no threads, no I/O — so each metric is tested
against hand-computed values.

`tracescope-collect` is the only crate that touches time. A `Clock` trait sits
between the collector and the system clock; a `ManualClock` makes collection
deterministic in tests and demos. The collector records spans into the same
`Trace` model the core analyzes.

## Analysis pipeline

1. **Percentiles** — p50/p95/p99 of span durations via linear interpolation.
2. **Flamegraph fold** — recursively accumulate each span's *self time* (its
   duration minus its children's) keyed by the root-to-node call stack.
3. **Critical path** — from the longest root, descend into the last-finishing
   child at each level: the chain that gates total completion.
4. **Bottlenecks** — self time aggregated by span name, ranked descending.

## Trace interchange

Traces serialize to JSON (see [`trace-format.md`](trace-format.md)), so traces
collected by one process can be analyzed by another, or by the CLI.
