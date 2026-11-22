#!/usr/bin/env bash
# Build the sample plugin, sign its archive contents, and produce the
# install archive.
#
# Steps:
#   1. Generate a fresh ed25519 keypair at `signing-key.pem` (no-op if it
#      already exists so re-runs don't churn the fixture signature).
#   2. `cargo build --release` — produces `target/release/libonq_sample_plugin.so`.
#   3. Build the deterministic signed payload and sign it with the ed25519
#      key: the exact `manifest.toml` bytes, a newline, then one line per
#      regular file in the archive (excluding `signature.sig`):
#      `<lowercase-hex sha256>  <relative path>\n`, sorted by relative
#      path bytes. The install pipeline (`onq_core::plugin_install`)
#      verifies the signature over this same payload.
#   4. Emit the 64-byte raw signature to `signature.sig`.
#   5. Bundle `manifest.toml` + `signature.sig` + the `.so` into a
#      `.tar.gz` archive consumable by `plugin_install::install`.
#
# Usage: ./build-and-sign.sh

set -euo pipefail
cd "$(dirname "$0")"

KEY="signing-key.pem"
PUB="signing-key.pub"
# Cargo normalizes the crate name `onq-sample-plugin` (hyphens →
# underscores) for the cdylib output filename.
SO="target/release/libonq_sample_plugin.so"

# 1. Keypair (idempotent: keep existing key so signature stays stable).
if [[ ! -f "$KEY" ]]; then
    echo "==> generating ed25519 keypair at $KEY"
    openssl genpkey -algorithm ed25519 -out "$KEY" 2>/dev/null
    # Extract the raw 32-byte public key for verification helpers.
    openssl pkey -in "$KEY" -pubout -out "$PUB" 2>/dev/null
fi

# 2. Build the cdylib.
echo "==> cargo build --release"
cargo build --release

if [[ ! -f "$SO" ]]; then
    echo "ERROR: expected $SO after build, but it does not exist" >&2
    exit 1
fi

# 3. Build the signed payload. Files are hashed in byte-sorted order of
#    their relative path; `signature.sig` is excluded because it carries
#    the signature itself.
echo "==> building the signed payload"
PAYLOAD="$(mktemp)"
trap 'rm -f "$PAYLOAD"' EXIT
{
    cat manifest.toml
    printf '\n'
    files=("manifest.toml" "$SO")
    IFS=$'\n' sorted=($(printf '%s\n' "${files[@]}" | LC_ALL=C sort))
    unset IFS
    for f in "${sorted[@]}"; do
        hex=$(sha256sum "$f" | cut -d' ' -f1)
        printf '%s  %s\n' "$hex" "$f"
    done
} > "$PAYLOAD"

#    openssl pkeyutl -sign on ed25519 emits a raw 64-byte signature (R || S)
#    which matches what `onq_core::signature::verify` expects
#    (ed25519_dalek's `Signature::from_slice`).
echo "==> signing the payload with ed25519"
openssl pkeyutl -sign \
    -inkey "$KEY" \
    -in "$PAYLOAD" \
    -out signature.sig

# 4. Sanity-check the signature locally before bundling, so a botched key
#    fails fast in the build script rather than at install time.
echo "==> verifying signature.sig against the signed payload"
openssl pkeyutl -verify \
    -inkey "$KEY" \
    -in "$PAYLOAD" \
    -sigfile signature.sig

# 5. Bundle for plugin_install::install. Layout mirrors what the
#    plugin_install test fixture expects (manifest.toml + signature.sig
#    + the .so binary).
echo "==> packaging sample-plugin.tar.gz"
tar czf sample-plugin.tar.gz \
    manifest.toml \
    signature.sig \
    "$SO"

echo "==> done."
echo "    bundle:  $(pwd)/sample-plugin.tar.gz"
echo "    signature: $(pwd)/signature.sig (raw ed25519, 64 bytes)"
