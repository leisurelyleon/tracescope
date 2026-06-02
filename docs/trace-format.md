# Trace Format

A trace is a JSON object with a single `spans` array. Each span is an object:

```json
{
  "spans": [
    { "id": 0, "name": "handle_request", "start_ns": 0, "end_ns": 75 },
    { "id": 1, "name": "db_query", "start_ns": 5, "end_ns": 45, "parent": 0 },
    { "id": 2, "name": "render", "start_ns": 45, "end_ns": 70, "parent": 0 },
    { "id": 3, "name": "serialize", "start_ns": 60, "end_ns": 70, "parent": 2 }
  ]
}
```

## Fields

- `id` — unique unsigned integer span identifier.
- `name` — the span's label.
- `start_ns` / `end_ns` — start and end timestamps in nanoseconds; `end_ns`
  must be `>= start_ns`.
- `parent` — optional id of the enclosing span; omitted for root spans.

## Validity

A valid trace has unique ids, non-inverted intervals, parent references that
exist, and no cycles. The analyzer validates these before computing a report.
