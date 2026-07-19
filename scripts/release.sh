#!/usr/bin/env bash
set -euo pipefail
# Bump version, update lockfiles, tag, push.
# Usage: scripts/release.sh 0.2.0
VERSION="${1:?usage: $0 VERSION}"
test -z "$(git status --porcelain)" || { echo "working tree dirty"; exit 1; }
git rev-parse --abbrev-ref HEAD | grep -q '^master$' || { echo "not on master"; exit 1; }
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" package.json
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" crates/onq-app/tauri.conf.json
cargo update -w
git add -A
git commit -m "chore: bump version to $VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin master --follow-tags
echo "Tagged v$VERSION. Release workflow will build artifacts."
