# 2. A pure analysis core

- Status: Accepted
- Date: 2026-06

## Context

Performance analysis (percentiles, flamegraph aggregation, critical path) is
subtle and easy to get wrong. Tangling it with timing or I/O would make it
hard to verify.

## Decision

Keep all analysis as pure functions over a `Trace` data model in
`tracescope-core`, with no clocks, threads, or I/O. Timing lives only in the
separate collector crate.

## Consequences

- Every metric is unit-tested against hand-computed known values.
- The analysis can run on traces from any source, including other processes.
- A subtly wrong metric is caught by tests rather than shipped.
