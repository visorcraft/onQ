#!/usr/bin/env bash
# generate-latest-json.sh — build Tauri updater latest.json from a GitHub release.
#
# Usage:
#   scripts/generate-latest-json.sh [TAG] [OUT]
#
# Defaults: TAG = latest published tag (or v* from env RELEASE_TAG),
#           OUT = ./latest.json
#
# Requires: gh, jq
# The release must already have signed updater artifacts uploaded
# (.AppImage + .sig, .exe/.msi + .sig, .app.tar.gz + .sig).

set -euo pipefail

REPO="${GITHUB_REPOSITORY:-visorcraft/onQ}"
TAG="${1:-${RELEASE_TAG:-}}"
OUT="${2:-latest.json}"

if [[ -z "$TAG" ]]; then
  TAG=$(gh release view --repo "$REPO" --json tagName --jq .tagName)
fi
TAG="${TAG#v}"
# Accept both v1.1.0 and 1.1.0
RELEASE_TAG="v${TAG#v}"

meta=$(gh release view "$RELEASE_TAG" --repo "$REPO" --json tagName,publishedAt,body,assets)
version=$(echo "$meta" | jq -r '.tagName | ltrimstr("v")')
pub_date=$(echo "$meta" | jq -r '.publishedAt')
notes=$(echo "$meta" | jq -r '.body // ""')

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

# Download all .sig files so we can embed signatures.
mapfile -t sig_assets < <(echo "$meta" | jq -r '.assets[] | select(.name|endswith(".sig")) | .name')
if ((${#sig_assets[@]} == 0)); then
  echo "error: no .sig assets on $RELEASE_TAG" >&2
  exit 1
fi

(
  cd "$tmpdir"
  for name in "${sig_assets[@]}"; do
    gh release download "$RELEASE_TAG" --repo "$REPO" -p "$name" --clobber
  done
)

# Resolve download URL for a non-sig asset name.
asset_url() {
  local name="$1"
  echo "https://github.com/${REPO}/releases/download/${RELEASE_TAG}/${name}"
}

# Read signature text for asset basename (expects NAME.sig next to NAME).
sig_for() {
  local name="$1"
  local sig="$tmpdir/${name}.sig"
  if [[ ! -f "$sig" ]]; then
    echo "error: missing signature for $name" >&2
    return 1
  fi
  # .sig files are single-line base64-ish text; strip trailing newlines.
  tr -d '\r\n' <"$sig"
}

# Pick the first matching asset name from the release that has a .sig.
pick_asset() {
  local pattern="$1"
  echo "$meta" | jq -r --arg re "$pattern" '
    .assets[]
    | select(.name | test($re))
    | select(.name | endswith(".sig") | not)
    | .name
  ' | head -n1
}

platforms='{}'

add_platform() {
  local key="$1"
  local pattern="$2"
  local name
  name=$(pick_asset "$pattern")
  if [[ -z "$name" || "$name" == "null" ]]; then
    echo "warn: no asset for platform $key (pattern $pattern)" >&2
    return 0
  fi
  local url sig
  url=$(asset_url "$name")
  sig=$(sig_for "$name")
  platforms=$(jq -c --arg k "$key" --arg u "$url" --arg s "$sig" \
    '.[$k] = {url: $u, signature: $s}' <<<"$platforms")
  echo "platform $key -> $name" >&2
}

# Prefer AppImage / NSIS / macOS updater tarball (Tauri createUpdaterArtifacts).
add_platform "linux-x86_64" 'AppImage$'
add_platform "windows-x86_64" 'x64-setup\.exe$'
# Fallback MSI if NSIS missing
if ! jq -e '."windows-x86_64"' >/dev/null 2>&1 <<<"$platforms"; then
  add_platform "windows-x86_64" 'x64_en-US\.msi$'
fi
add_platform "darwin-aarch64" 'app\.tar\.gz$'

if [[ $(jq 'length' <<<"$platforms") -eq 0 ]]; then
  echo "error: no updater platforms found on $RELEASE_TAG" >&2
  exit 1
fi

jq -n \
  --arg version "$version" \
  --arg notes "$notes" \
  --arg pub_date "$pub_date" \
  --argjson platforms "$platforms" \
  '{
    version: $version,
    notes: $notes,
    pub_date: $pub_date,
    platforms: $platforms
  }' >"$OUT"

echo "wrote $OUT for $RELEASE_TAG" >&2
jq -c '{version, platforms: (.platforms|keys)}' "$OUT" >&2
