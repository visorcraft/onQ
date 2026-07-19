#!/usr/bin/env bash
# check-perf-regression.sh — fail if any tracked benchmark regresses >10%
# vs the baseline numbers in the JSON file.
#
# Usage:
#   scripts/check-perf-regression.sh <baselines.json> <bench-output.txt>
#
# Recognized baseline keys (the suffix is the unit the value is expressed
# in; the script converts the median from Criterion's native unit to the
# baseline unit before comparing):
#
#   search_warm_p95_ms   → bench `search_warm_p95`   (FM Contains pre-filter)
#   merge_10kb_clean_us  → bench `merge_10kb_clean`  (3-way merge on 10 KB md)
#
# Criterion default output format:
#
#   <bench_name>                 time:   [<low> <unit> <median> <unit> <high> <unit>]
#
# The regex below captures the three (value, unit) pairs. We compare the
# median to `baseline * 1.10`. Benches listed in the JSON that aren't
# present in the output file are skipped silently (lets you run a subset
# of benches during development). A bench present in the output but
# missing from the JSON is logged as a warning but does not fail.

set -euo pipefail

BASELINE="${1:?usage: $0 BASELINE_JSON CURRENT_OUTPUT}"
CURRENT="${2:?usage: $0 BASELINE_JSON CURRENT_OUTPUT}"

if [[ ! -f "$BASELINE" ]]; then
    echo "FAIL: baseline file not found: $BASELINE" >&2
    exit 2
fi
if [[ ! -f "$CURRENT" ]]; then
    echo "FAIL: bench output file not found: $CURRENT" >&2
    exit 2
fi

python3 - "$BASELINE" "$CURRENT" <<'PYEOF'
import json
import re
import sys

baseline_path, current_path = sys.argv[1], sys.argv[2]
with open(baseline_path) as fh:
    baseline = json.load(fh)
with open(current_path) as fh:
    text = fh.read()


def to_unit(value: float, unit: str, target_unit: str) -> float:
    """Convert `value` expressed in `unit` to `target_unit`.

    Both Criterion's unit symbols (`ns`, `us`, `ms`, `s`) and the
    micro-symbol `µs` are accepted; we normalise `µs` → `us` first.
    """
    u = unit.replace("µ", "u")
    to_ns = {
        "ns": 1.0,
        "us": 1_000.0,
        "ms": 1_000_000.0,
        "s": 1_000_000_000.0,
    }.get(u)
    if to_ns is None:
        sys.exit(f"unknown unit in bench output: {unit!r}")
    value_ns = value * to_ns
    target_to_ns = {
        "ns": 1.0,
        "us": 1_000.0,
        "ms": 1_000_000.0,
        "s": 1_000_000_000.0,
    }[target_unit]
    return value_ns / target_to_ns


# Baseline key → (Criterion bench name, unit the baseline is expressed in).
BENCHES = [
    ("search_warm_p95_ms", "search_warm_p95", "ms"),
    ("merge_10kb_clean_us", "merge_10kb_clean", "us"),
]

# Match `<name> time: [<low> <unit> <med> <unit> <high> <unit>]`. The
# regex tolerates variable whitespace, scientific notation in the value,
# and any unit suffix Criterion emits (`ns`, `us`, `µs`, `ms`, `s`).
pattern = re.compile(
    r"\b(?P<name>[A-Za-z0-9_]+)\s+time:\s*"
    r"\[\s*(?P<low>[\d.eE+-]+)\s+(?P<lowu>\S+)\s+"
    r"(?P<med>[\d.eE+-]+)\s+(?P<medu>\S+)\s+"
    r"(?P<high>[\d.eE+-]+)\s+(?P<highu>\S+)\s*\]"
)

# Last match wins when Criterion re-runs the same bench in a single
# session (it prints a `time:` line per measurement run). We only have
# one run here, but this keeps the gate idempotent under retries.
medians: dict[str, tuple[float, str]] = {}
for m in pattern.finditer(text):
    medians[m.group("name")] = (float(m.group("med")), m.group("medu"))

if not medians:
    sys.exit(
        f"no benchmark `time: [...]` lines found in {current_path}; "
        "is the output from a Criterion run?"
    )

failures: list[str] = []
for key, bench_name, target_unit in BENCHES:
    if key not in baseline:
        print(f"skip {bench_name}: no baseline for {key}")
        continue
    if bench_name not in medians:
        print(f"skip {bench_name}: not present in {current_path}")
        continue
    med, unit = medians[bench_name]
    med_in_target = to_unit(med, unit, target_unit)
    base = baseline[key]
    threshold = base * 1.10
    pct = (med_in_target - base) / base * 100.0
    if med_in_target > threshold:
        failures.append(
            f"REGRESSION: {bench_name} {med_in_target:.3f}{target_unit} > "
            f"baseline {base}{target_unit} * 1.10 = {threshold:.3f}{target_unit} "
            f"({pct:+.1f}%)"
        )
    else:
        print(
            f"{bench_name}: {med_in_target:.3f}{target_unit} "
            f"(baseline {base}{target_unit}, threshold "
            f"{threshold:.3f}{target_unit}, {pct:+.1f}%) — OK"
        )

# Warn about output benches with no baseline (forward-compat: a new bench
# shows up here before someone commits a baseline for it).
for name in sorted(medians):
    if not any(name == bn for _, bn, _ in BENCHES):
        print(f"warn: bench {name!r} has no baseline entry in {baseline_path}")

if failures:
    print("\n".join(failures), file=sys.stderr)
    sys.exit(1)

print("perf gate: OK")
PYEOF