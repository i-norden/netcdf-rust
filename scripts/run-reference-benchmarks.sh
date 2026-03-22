#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python_bin="${PYTHON:-python3}"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target/reference-compat}"
export BENCH_THREAD_LIST="${BENCH_THREAD_LIST:-1,2,4,8}"
export BENCH_HOT_OPS_PER_THREAD="${BENCH_HOT_OPS_PER_THREAD:-256}"

benchmark_summary_path="${BENCHMARK_SUMMARY_PATH:-$repo_root/benchmark-summary.md}"

"$repo_root/scripts/generate-fixtures.sh"

(
  cd "$repo_root"
  cargo bench -p netcdf-reader --bench compare_georust -- --noplot
)

"$python_bin" "$repo_root/scripts/criterion_summary.py" \
  --criterion-root "$CARGO_TARGET_DIR/criterion" \
  --speedup \
  > "$benchmark_summary_path"
