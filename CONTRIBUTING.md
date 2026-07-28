# Contributing to onQ

Thanks for improving onQ. Keep changes focused, preserve local-first behavior,
and add the smallest test that proves each behavior change.

## Before you start

Read:

- [AGENTS.md](AGENTS.md) for the repository map and domain rules
- [public development guide](https://visorcraft.github.io/onQ/development)
- [architecture guide](https://visorcraft.github.io/onQ/architecture)
- [feature status](https://visorcraft.github.io/onQ/feature-status)
- [security policy](.github/SECURITY.md) before reporting a vulnerability

## Prerequisites

- Node.js 24+
- Rust stable, selected by `rust-toolchain.toml`
- [Tauri platform dependencies](https://v2.tauri.app/start/prerequisites/)

Clone and install:

```bash
git clone https://github.com/visorcraft/onQ
cd onQ
npm install
```

`mongreldb-core` resolves from crates.io. Do not add a sibling path or Git
dependency.

## Run locally

Desktop app:

```bash
npm run dev:app
```

Frontend only:

```bash
npm run dev
```

Public documentation:

```bash
npm run docs:dev
```

Storybook:

```bash
npm run storybook
```

Use a disposable vault for development. Never commit real prompts, recovery
phrases, passwords, keychain exports, or signing private keys.

## Repository layout

```text
src/                         Svelte 5 + TypeScript UI
  lib/api/                   Tauri invoke wrappers
  lib/components/            Screens and primitives
  lib/i18n/                  Six locales
  lib/stores/                UI state
crates/
  onq-app/                   Tauri shell and IPC
  onq-core/                  Domain and persistence logic
  onq-plugin-sdk/            Native plugin ABI
  onq-test-utils/            Shared test helpers
docs/                        ADRs and plugin authoring
docs-site/                   VitePress public documentation
tests/                       Playwright and plugin fixtures
scripts/                     Release, legal, and performance tools
```

## Change rules

### Frontend

- Use Svelte 5 runes and strict TypeScript.
- Put all Tauri calls behind `src/lib/api/*`.
- Add English i18n keys first, then update Thai, Spanish, Chinese, French,
  and German.
- Reuse CSS tokens and existing components.
- Colocate Vitest files as `*.test.ts`.

### Rust

- Return `CoreResult` and domain errors from `onq-core`.
- Return `Result<T, String>` at Tauri IPC boundaries.
- Avoid `unwrap()` and `expect()` on non-test IPC paths.
- Keep schema changes in `schema.rs` and migrations in `migrate.rs`.
- Keep workspace path-dependency versions equal to the workspace version.

### Product boundaries

- No cloud account or telemetry.
- No proprietary SDK or non-GPL plugin license.
- Prompt files remain portable Markdown.
- The encrypted index is derived and disposable.
- Hugging Face model download remains explicit and optional.
- Do not present scaffolded sync or plugin paths as shipped behavior.

## Tests

Run checks proportional to the change.

Frontend:

```bash
npm run check
npm run lint
npm test
npm run build
```

Rust:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

Desktop and browser:

```bash
npm run build:app
npm run test:a11y
npm run test:e2e
```

Documentation:

```bash
npm run docs:build
```

Heavy embedding and performance checks are opt-in:

```bash
ONQ_RUN_HEAVY_TESTS=1 cargo test -p onq-core --test embed_real
cargo bench -p onq-core
bash scripts/check-perf-regression.sh
```

CI runs formatting, Clippy with warnings denied, cargo-deny, the Rust
workspace tests, frontend checks, lint, tests, application and documentation
builds, and Playwright accessibility coverage.

## Documentation changes

Public user documentation lives in `docs-site/`. Update it whenever behavior,
settings, disk layout, security boundaries, or platform support changes.

- Keep `docs-site/.vitepress/config.mts` navigation in sync with new pages.
- State partial features plainly.
- Use exact UI labels and paths.
- Prefer links to stable public pages over source line numbers.
- Run `npm run docs:build` to catch broken internal links.

Developer-only plugin details live in `docs/plugins/README.md`. Architecture
decisions belong in `docs/adr/`.

## Dependencies and credits

After changing Rust or npm dependencies:

```bash
cargo update
npm update
bash scripts/regen-credits.sh
```

Commit the regenerated legal inventories:

```text
crates/onq-app/legal/crates.json
crates/onq-app/legal/third-party.md
crates/onq-app/legal/npm-packages.json
crates/onq-app/legal/npm-third-party.md
```

Do not edit generated inventories by hand.

## Commits

Use Conventional Commits:

```text
feat: add ...
fix(search): correct ...
docs: explain ...
test: cover ...
refactor: simplify ...
chore: update ...
```

Use an imperative subject. Keep it near 50 characters when practical. Do not
add generated-by footers, bot attribution, or AI co-author trailers.

## Pull requests

Include:

- the user-visible result;
- the reason for the change;
- validation commands and results;
- reproduction steps for a bug fix;
- screenshots for visible UI work;
- format or migration notes when stored data changes.

Keep unrelated cleanup out of the same pull request.

## Security reports

Do not open a public issue for a vulnerability. Follow the private reporting
instructions in [SECURITY.md](.github/SECURITY.md).
