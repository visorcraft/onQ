#!/usr/bin/env bash
set -euo pipefail
fail() { echo "FAIL: $1" >&2; exit 1; }
node --version  | grep -qE 'v2[4-9]\.|v[3-9]' || fail "node >= 24 required"
rustc --version | grep -qE 'rustc 1\.(7[5-9]|[8-9][0-9]|[0-9]{3,})' || fail "rustc >= 1.75 required"
command -v npm >/dev/null || fail "npm required"
echo "Prerequisites OK."
