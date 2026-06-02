# 4. Clock abstraction for deterministic timing

- Status: Accepted
- Date: 2026-06

## Context

The collector must time spans, but tests and demos that depend on the wall clock
are flaky and slow (they need real sleeps and tolerate jitter).

## Decision

Define a `Clock` trait. `SystemClock` reads the real monotonic clock for
production; `ManualClock` is advanced explicitly by tests and demos for exact,
reproducible timings.

## Consequences

- Collection is testable with precise, deterministic durations — no sleeps.
- The demo produces stable, inspectable output.
- Production timing uses the real monotonic clock with no overhead from the
  abstraction beyond a trait call.
