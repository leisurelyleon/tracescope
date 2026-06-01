# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial workspace scaffold: tracescope-core, tracescope-collect, tracescope-cli.

## [0.1.0] - TBD

### Added
- Span and trace-tree data model with a JSON interchange format.
- Correct latency percentile computation (p50/p95/p99).
- Flamegraph aggregation by folding the trace tree into self-time stacks.
- Critical-path and bottleneck analysis.
- A low-overhead span collector with an RAII guard and a pluggable clock.
- CLI to analyze trace files and run an instrumented demo workload.

[Unreleased]: https://github.com/leisurelyleon/tracescope/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/leisurelyleon/tracescope/releases/tag/v0.1.0
