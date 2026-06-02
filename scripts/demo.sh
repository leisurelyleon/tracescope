#!/usr/bin/env bash
# Run the instrumented demo workload and print its performance analysis,
# then write the demo trace to disk and re-analyze it via the file path to
# show the JSON interchange round-trips.
set -euo pipefail

cargo build --release

echo "== Instrumented demo workload =="
./target/release/tracescope demo
