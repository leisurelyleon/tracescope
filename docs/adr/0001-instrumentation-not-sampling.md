# 1. Instrumentation, not process sampling

- Status: Accepted
- Date: 2026-06

## Context

A profiler can work by sampling a running process's stack (requiring privileged,
OS-specific machinery: ptrace, perf counters, signal handlers, unwinding) or by
instrumentation (the code records its own spans). Sampling is powerful but hard
to test deterministically and fragile across environments.

## Decision

Build `tracescope` as an instrumentation-and-analysis toolkit. Code records
spans via a lightweight collector; the analysis runs over the collected data.

## Consequences

- The analysis — the algorithmically interesting part — is pure and testable.
- No privileged syscalls, so it runs anywhere without special permissions.
- It does not profile uninstrumented third-party code; that is an accepted limit.
