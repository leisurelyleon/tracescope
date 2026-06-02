# 3. Linear-interpolation percentiles

- Status: Accepted
- Date: 2026-06

## Context

Several percentile definitions exist (nearest-rank, various interpolations).
Choosing and documenting one matters: a p99 computed inconsistently is
misleading.

## Decision

Use linear interpolation between closest ranks (the method NumPy calls "linear"
and Excel calls PERCENTILE.INC): for `n` sorted values and percentile `p`,
`rank = (p/100)*(n-1)`, interpolating between the values at floor and ceil of
the rank.

## Consequences

- Exact at the extremes (p0 = min, p100 = max) and smoothly interpolated within.
- Matches widely-used tools, so results are comparable.
- The formula is documented in code and verified against known values.
