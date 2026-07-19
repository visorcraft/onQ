#!/usr/bin/env bash
# Compute SHA256 of release artifacts for verification.
set -euo pipefail
for f in "$@"; do
  if command -v sha256sum >/dev/null; then
    sha256sum "$f"
  else
    shasum -a 256 "$f"
  fi
done
