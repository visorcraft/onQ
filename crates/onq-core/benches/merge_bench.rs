//! Criterion benchmark for three-way merge on a ~10 KB markdown document.
//!
//! Synthesises three non-overlapping edits to a 10 KB markdown file in
//! memory and times [`onq_core::merge::three_way`]. The diff is
//! constructed so the merge is clean (no conflict markers), which matches
//! the realistic auto-merge case the sync worker hits on every successful
//! pull. Conflict-heavy merges will be added as a separate bench variant
//! when the merge conflict UX (M7.x) lands.
//!
//! Why inline strings instead of `include_str!("../tests/fixtures/...")`:
//! keeping the fixture co-located with the bench removes one path-coupling
//! between `tests/` and `benches/` and means `cargo bench` works from any
//! working directory without a fixture-file churn git hook.

use criterion::{criterion_group, criterion_main, Criterion};
use onq_core::merge;

/// Approximate size of the synthetic markdown document under test (bytes).
const DOC_BYTES: usize = 10 * 1024;

/// One paragraph line in the synthetic markdown document. Hoisted to module
/// scope so both `fake_md` (which repeats it) and `build_triple` (which
/// locates/replaces the first and last occurrences) can reference the same
/// byte length when issuing `String::replace_range` calls.
const LINE: &str = "- item line with some filler text to bulk up the body\n";

/// Build a synthetic `n`-byte markdown document made of repeated
/// `LINE` paragraphs. Deterministic, no I/O, fast.
fn fake_md(n: usize) -> String {
    let mut out = String::with_capacity(n + LINE.len());
    while out.len() < n {
        out.push_str(LINE);
    }
    out
}

fn build_triple() -> (String, String, String) {
    // base: full document. `ours` edits the first half, `theirs` edits the
    // second half — diffy reports a clean merge because the edit ranges
    // don't overlap.
    let mut base = fake_md(DOC_BYTES);
    let mut ours = base.clone();
    let mut theirs = base.clone();

    // Edit 1 (ours, first half): replace the first paragraph with OURS-marker text.
    if let Some(idx) = ours.find(LINE) {
        let end = idx + LINE.len();
        ours.replace_range(idx..end, "- OURS: rewrote the first paragraph\n");
    }
    // Edit 2 (theirs, second half): replace the last paragraph with THEIRS-marker text.
    if let Some(last_idx) = theirs.rfind(LINE) {
        theirs.replace_range(
            last_idx..last_idx + LINE.len(),
            "- THEIRS: rewrote the last paragraph\n",
        );
    }
    // Trim trailing partial line so `base`/`ours`/`theirs` line counts stay
    // identical and diffy sees clean 3-way alignment.
    base.truncate(ours.len().min(theirs.len()));
    ours.truncate(base.len());
    theirs.truncate(base.len());

    (base, ours, theirs)
}

fn bench_merge(c: &mut Criterion) {
    let (base, ours, theirs) = build_triple();
    c.bench_function("merge_10kb_clean", |b| {
        b.iter(|| {
            let _out = merge::three_way(&base, &ours, &theirs).expect("clean merge");
        });
    });
}

criterion_group!(benches, bench_merge);
criterion_main!(benches);
