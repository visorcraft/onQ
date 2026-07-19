#!/usr/bin/env bash
# Build the sample plugin, sign its manifest, and produce the install archive.
#
# Steps:
#   1. Generate a fresh ed25519 keypair at `signing-key.pem` (no-op if it
#      already exists so re-runs don't churn the fixture signature).
#   2. `cargo build --release` — produces `target/release/libsample_plugin.so`.
#   3. Sign the canonical `manifest.toml` bytes with the ed25519 key. The
#      install pipeline (`onq_core::plugin_install`) signs the
#      manifest text — not the raw archive — so the signer and verifier
#      agree on a deterministic payload.
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

# 3. Sign the canonical manifest text.
#    openssl pkeyutl -sign on ed25519 emits a raw 64-byte signature (R || S)
#    which matches what `onq_core::signature::verify` expects
#    (ed25519_dalek's `Signature::from_slice`).
echo "==> signing manifest.toml with ed25519"
openssl pkeyutl -sign \
    -inkey "$KEY" \
    -in manifest.toml \
    -out signature.sig

# 4. Sanity-check the signature locally before bundling, so a botched key
#    fails fast in the build script rather than at install time.
echo "==> verifying signature.sig against manifest.toml"
openssl pkeyutl -verify \
    -inkey "$KEY" \
    -in manifest.toml \
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