# tracescope

> A tracing and performance-analysis toolkit: spans, flamegraphs, percentiles, and critical-path analysis.

`tracescope` helps you understand where time goes. You instrument code with
nested spans; the collector records them into a trace tree; and the analysis
engine turns that tree into the things a profiler actually reports: flamegraph
stacks, latency percentiles (p50/p95/p99), the critical path that determines
total duration, and the largest self-time bottlenecks.

## Approach

`tracescope` is an instrumentation-and-analysis toolkit, not a process-attaching
sampler. You add spans to your code (cheaply, via an RAII guard), and the
analysis runs over the collected trace data. This keeps the interesting part —
the analysis algorithms — pure, deterministic, and fully testable, and avoids
any privileged, OS-specific machinery.

## Architecture

```
tracescope-core      pure trace model + analysis: percentiles, flamegraph, critical path
tracescope-collect   the runtime collector: a pluggable clock and an RAII span guard
tracescope-cli       the binary: analyze a trace file, or run an instrumented demo
```

The analysis core is pure functions over a trace model, so every metric is unit-
tested against known values. The collector's clock is abstracted behind a trait,
with a deterministic fake, so timing-dependent code is testable without flaky
sleeps. See [`docs/architecture.md`](docs/architecture.md) and
[`docs/trace-format.md`](docs/trace-format.md).

## Build & Test

```bash
cargo build
cargo test
```

## Run

```bash
# Run an instrumented demo workload and print the analysis report
cargo run -p tracescope-cli -- demo

# Analyze a trace file (JSON)
cargo run -p tracescope-cli -- analyze trace.json
```

## License

MIT — see [LICENSE](LICENSE).
