#!/usr/bin/env bash
# Rewrite crates/onq-core/tests/fixtures/compat-vault.tar.gz from the current
# mongreldb-core + onQ migrator. Run after intentional schema/engine changes.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Surface the resolved mongreldb-core version in the human manifest.
MONGRELDB_VER="$(
  python3 - <<'PY'
import re, pathlib
lock = pathlib.Path("Cargo.lock").read_text()
m = re.search(r'name = "mongreldb-core"\nversion = "([^"]+)"', lock)
print(m.group(1) if m else "unknown")
PY
)"

export ONQ_REGEN_COMPAT_FIXTURE=1
export ONQ_COMPAT_MONGRELDB_VERSION="$MONGRELDB_VER"

echo "Regenerating compat vault fixture (mongreldb-core $MONGRELDB_VER)..."
cargo test -p onq-core --test compat_vault regenerate_compat_fixture -- --ignored --nocapture

# Prove the freshly written bytes open under the same binary.
cargo test -p onq-core --test compat_vault -- --nocapture

echo "OK — commit crates/onq-core/tests/fixtures/compat-vault.{tar.gz,md}"
