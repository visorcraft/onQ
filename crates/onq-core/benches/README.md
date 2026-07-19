# onQ Criterion benchmarks

Performance baselines for the hot paths that the search palette and sync
worker hit every session. The numbers feed the perf-regression gate wired
in Task 6.9; if any metric regresses more than 10% the CI gate fails.

## Running

```bash
# All benches (slow — full Criterion sample size).
cargo bench -p onq-core

# Single bench file.
cargo bench --bench search_bench
cargo bench --bench merge_bench

# Quick smoke run during development.
cargo bench --bench search_bench -- --sample-size 10 --warm-up-time 1
```

HTML reports land in `target/criterion/` and are git-ignored; keep them
locally for diffing but never commit them.

## What's measured

| Bench file        | Function            | What it times                                                                                |
| ----------------- | ------------------- | -------------------------------------------------------------------------------------------- |
| `search_bench.rs` | `search_warm_p95`   | FM Contains pre-filter over the 1k fixture corpus (matches 100 prompts on `kw_t00_n0`).       |
| `merge_bench.rs`  | `merge_10kb_clean`  | Three-way merge on a ~10 KB markdown document with non-overlapping edits (clean auto-merge).  |

## Fixtures

- `onq_test_utils::fixtures::corpus_1k()` builds a fresh encrypted
  vault containing 10 topics × 100 deterministic prompts and returns
  `(Db, TempDir)`. The bench drops the `TempDir` after setup so cleanup is
  automatic. Reusing the helper from `onq-test-utils` keeps the
  benchmark corpus identical to the `search_ndcg` conformance test.

- The merge bench synthesises a ~10 KB markdown document in memory rather
  than loading a file fixture. That keeps `cargo bench` hermetic (no
  relative-path surprises when invoked from a worktree) and removes one
  source of drift between `tests/fixtures/` and the bench harness.

## Adding a new bench

1. Create `crates/onq-core/benches/<name>_bench.rs` with the
   standard Criterion skeleton (`criterion_group!` + `criterion_main!`).
2. Add a `[[bench]] name = "<name>_bench" harness = false` entry to
   `crates/onq-core/Cargo.toml`.
3. Re-use existing fixtures from `onq-test-utils` rather than
   inlining corpus builders.
4. After the bench runs cleanly, capture the median into
   `benches/baselines.json` so Task 6.9's regression gate has a target.
