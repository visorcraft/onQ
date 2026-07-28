---
title: Build and test
description: Contributor setup, commands, validation matrix, conventions, dependency maintenance, docs, and releases.
---

# Build and test

onQ is a Node, Svelte, Tauri, and Rust workspace. The desktop shell requires
native platform dependencies in addition to npm and Cargo packages.

## Requirements

- Node.js 24 or newer
- npm with lockfile support
- Rust stable from `rust-toolchain.toml`
- Git
- Tauri 2 platform prerequisites

The Cargo workspace declares Rust 1.75 as its minimum package version, but
development and CI use current stable Rust.

## Clone and install

```bash
git clone https://github.com/visorcraft/onQ
cd onQ
npm install
```

CI uses `npm ci` for a clean lockfile-exact installation.

`mongreldb-core` resolves from crates.io. Do not add a sibling path or Git
checkout.

## Linux packages

Ubuntu CI installs:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

Use the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for
other platforms and distributions.

## Run

Desktop development:

```bash
npm run dev:app
```

Frontend-only Vite server:

```bash
npm run dev
```

The browser surface cannot complete Tauri `invoke` calls without mocks or the
desktop shell.

Production desktop bundle:

```bash
npm run build:app
```

Frontend production build:

```bash
npm run build
```

## Validation commands

### Frontend

```bash
npm run check
npm run lint
npm test
npm run build
```

### Browser tests

```bash
npm run test:a11y
npm run test:e2e
```

Install Playwright Chromium when needed:

```bash
npx playwright install --with-deps chromium
```

### Rust

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

### Documentation

```bash
npm run docs:dev
npm run docs:build
npm run docs:preview
```

### Storybook

```bash
npm run storybook
npm run build-storybook
```

### Performance and heavy tests

```bash
ONQ_RUN_HEAVY_TESTS=1 cargo test -p onq-core --test embed_real
cargo bench -p onq-core
bash scripts/check-perf-regression.sh
```

The heavy embedding test can download or load the real model. Normal tests use
mocks and should stay offline.

## CI

`.github/workflows/ci.yml` has two Ubuntu jobs.

Rust job:

1. install native Tauri dependencies;
2. `cargo fmt --all -- --check`;
3. clippy with warnings denied;
4. `cargo deny check`;
5. `cargo test --workspace`.

Frontend job:

1. Node 24 and `npm ci`;
2. Svelte check;
3. ESLint;
4. Vitest;
5. Vite production build;
6. VitePress documentation build;
7. Playwright Chromium accessibility tests.

Run the smallest checks that cover a change while iterating. Run the full
relevant lane before opening a pull request.

## Repository map

```text
src/                         Svelte 5 + TypeScript frontend
  lib/api/                   one Tauri invoke wrapper per domain
  lib/components/            screens, editor pieces, primitives, settings
  lib/stores/                application and vault-backed UI state
  lib/i18n/                  en, th, es, zh, fr, de
  lib/styles/                tokens, themes, glass, settings, motion
crates/
  onq-app/                   Tauri shell and IPC commands
  onq-core/                  vault, crypto, search, backup, plugins, sync
  onq-plugin-sdk/            ABI v1 declarations and export macro
  onq-test-utils/            shared Rust fixtures and mocks
docs/                        ADRs and plugin authoring source
docs-site/                   VitePress public documentation
tests/                       Playwright and sample plugin fixture
scripts/                     release, legal inventory, performance helpers
```

Read [Architecture](/architecture) before changing a cross-layer flow.

## Frontend conventions

- TypeScript strict mode.
- Svelte 5 runes such as `$state`, `$derived`, and `$props`.
- Tauri IPC only through `src/lib/api/*`.
- Keep wrappers aligned with the command registered in
  `crates/onq-app/src/lib.rs`.
- Add English i18n keys first, then mirror all six locale catalogs.
- Use existing design tokens and shared settings chrome.
- Colocate Vitest as `*.test.ts`.
- Mock Tauri at the invoke boundary.
- Preserve keyboard and screen-reader behavior for every UI change.

Storybook-only product components are intentional previews, not automatically
dead code.

## Rust conventions

- Run `cargo fmt`.
- Use `snake_case` functions/modules and `PascalCase` types.
- Return `CoreResult` and typed `thiserror` errors in `onq-core`.
- Return `Result<T, String>` at the Tauri IPC boundary.
- Do not use `unwrap()` or `expect()` on production IPC paths.
- Move synchronous MongrelDB, filesystem, model, and keychain work off async
  runtime threads where appropriate.
- Keep schema IDs stable and migrations monotonic.
- Keep workspace path dependency versions equal to the workspace version.

## Tests for behavior changes

Every behavior change needs the smallest test that would fail without it:

- Vitest for Svelte, stores, and frontend helpers;
- Rust unit tests for pure domain logic;
- Rust integration tests for cross-module storage behavior;
- Playwright for critical end-to-end or accessibility behavior.

Do not download the real embedding model in normal unit tests.

## Dependency and credits maintenance

After changing Rust or npm dependencies:

```bash
cargo update
npm update
bash scripts/regen-credits.sh
```

Commit the regenerated:

```text
crates/onq-app/legal/crates.json
crates/onq-app/legal/third-party.md
crates/onq-app/legal/npm-packages.json
crates/onq-app/legal/npm-third-party.md
```

`deny.toml` is the authoritative Rust license allow-list. Plugins must remain
GPL-3.0-only. Runtime OS libraries belong in acknowledgments and runtime
license documents, not generated language-package inventories.

## Versions

Release version must match:

```text
package.json
Cargo.toml [workspace.package]
crates/onq-app/tauri.conf.json
workspace path dependency versions
```

Use:

```bash
bash scripts/release.sh <version>
```

The script requires a clean `master` branch, updates versions and lockfiles,
creates a conventional version commit and annotated tag, and pushes both.

The release workflow builds:

- Apple Silicon macOS;
- x86-64 Linux;
- x86-64 Windows;
- signed updater artifacts and `latest.json`.

## Documentation changes

Public pages live in `docs-site/`. When behavior changes:

1. update the relevant guide;
2. update [Feature status](/feature-status) if an integration boundary moves;
3. keep README links current;
4. run `npm run docs:build`;
5. verify external statements against code or release workflow.

Documentation on `master` deploys through GitHub Pages.

## Pull requests

- Keep scope focused.
- Use Conventional Commits.
- Explain user-visible behavior.
- Include verification commands.
- Add screenshots for UI changes.
- Never commit passwords, recovery phrases, signing private keys, or vault
  data.
- Do not add generated-by footers, automated co-author trailers, or bot
  attribution.

Read
[`CONTRIBUTING.md`](https://github.com/visorcraft/onQ/blob/master/CONTRIBUTING.md)
for the short contributor contract and
[`SECURITY.md`](https://github.com/visorcraft/onQ/blob/master/.github/SECURITY.md)
for private vulnerability reporting.
