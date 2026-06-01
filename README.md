# tracescope
A tracing and performance-analysis toolkit in Rust. Instrument code with nested spans, then analyze collected traces to produce flamegraph trees, latency percentiles (p50/p95/p99), critical-path breakdowns, and bottleneck reports. A pure analysis core ingests trace data from any source, with a low-overhead span collector and a CLI.
