# Contributing to onQ

Thanks for helping improve onQ.

## Development setup

onQ requires Node.js 24+, Rust stable, and the platform dependencies listed
in the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
git clone https://github.com/visorcraft/onQ
cd onQ
npm install
```

`mongreldb-core` resolves from crates.io; no sibling clone is required.

Run the desktop app:

```bash
npm run dev:app
```

## Before opening a pull request

Run the checks that cover your change:

```bash
npm run check
npm test
cargo test --workspace
```

Keep pull requests focused. Explain the user-visible result, include reproduction
steps for bug fixes, and add the smallest test that prevents a regression.

Agent and contributor working notes (layout, domain rules, credits regen) live in
[AGENTS.md](AGENTS.md).

## Reporting security issues

Do not open public issues for vulnerabilities. Follow the private reporting
instructions in [SECURITY.md](.github/SECURITY.md).
