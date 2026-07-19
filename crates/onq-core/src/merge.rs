use diffy::merge;

use crate::error::CoreResult;

pub enum MergeOutcome {
    Clean(String),
    /// `text` contains diffy conflict markers (<<<<<<< / ======= / >>>>>>>).
    /// The UI parses these and lets the user pick per-hunk resolutions.
    Conflicted {
        text: String,
    },
}

pub fn three_way(base: &str, ours: &str, theirs: &str) -> CoreResult<MergeOutcome> {
    match merge(base, ours, theirs) {
        Ok(merged) => Ok(MergeOutcome::Clean(merged)),
        Err(text) => Ok(MergeOutcome::Conflicted { text }),
    }
}

/// Count the number of conflict hunks in a diffy conflict-marker string.
pub fn count_conflicts(text: &str) -> usize {
    text.matches("<<<<<<<").count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_merge_non_overlapping() {
        let base = "line 1\nline 2\nline 3\nline 4\nline 5\n";
        let ours = "line 1\nOUR line 2\nline 3\nline 4\nline 5\n";
        let theirs = "line 1\nline 2\nline 3\nline 4\nTHEIRS line 5\n";
        let result = three_way(base, ours, theirs).unwrap();
        assert!(matches!(result, MergeOutcome::Clean(_)));
        if let MergeOutcome::Clean(text) = result {
            assert!(text.contains("OUR line 2"));
            assert!(text.contains("THEIRS line 5"));
        }
    }

    #[test]
    fn overlapping_edits_produce_conflict() {
        let base = "line 1\nline 2\nline 3\n";
        let ours = "line 1\nOUR change\nline 3\n";
        let theirs = "line 1\nTHEIRS change\nline 3\n";
        let result = three_way(base, ours, theirs).unwrap();
        match result {
            MergeOutcome::Conflicted { text } => {
                assert!(text.contains("<<<<<<<"));
                assert!(text.contains("======="));
                assert!(text.contains(">>>>>>>"));
                assert_eq!(count_conflicts(&text), 1);
            }
            _ => panic!("expected conflict"),
        }
    }

    #[test]
    fn identical_ours_theirs_no_conflict() {
        let base = "same\n";
        let ours = "OURS\n";
        let theirs = "OURS\n";
        let result = three_way(base, ours, theirs).unwrap();
        assert!(matches!(result, MergeOutcome::Clean(_)));
    }
}
