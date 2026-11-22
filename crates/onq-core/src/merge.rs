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

/// Count conflict-hunk marker lines in a diffy conflict-marker string.
///
/// Counts only column-0 `<<<<<<<` marker lines (`<<<<<<<` or `<<<<<<< {label}`,
/// mirroring diffy's output format). `<<<<<<<` appearing mid-line in prompt
/// content is ignored.
pub fn count_conflicts(text: &str) -> usize {
    text.lines()
        .filter(|line| *line == "<<<<<<<" || line.starts_with("<<<<<<< "))
        .count()
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
    fn count_conflicts_ignores_mid_line_markers() {
        assert_eq!(
            count_conflicts("explain <<<<<<< HEAD in git merges\nplain line\n"),
            0
        );
    }

    #[test]
    fn count_conflicts_counts_labeled_marker_lines() {
        let text = "<<<<<<< ours\nx\n=======\ny\n>>>>>>> theirs\n";
        assert_eq!(count_conflicts(text), 1);
        let with_noise = format!("{}content line with ... >>>>>>> tail\n", text);
        assert_eq!(count_conflicts(&with_noise), 1);
    }

    #[test]
    fn count_conflicts_unlabeled_marker_line() {
        assert_eq!(count_conflicts("<<<<<<<\n"), 1);
    }

    #[test]
    fn count_conflicts_empty_is_zero() {
        assert_eq!(count_conflicts(""), 0);
    }

    #[test]
    fn multi_hunk_conflict_counts_two() {
        let base: String = (1..=12).map(|i| format!("line {}\n", i)).collect();
        let ours = base
            .lines()
            .map(|l| match l {
                "line 2" => "OUR line 2".to_string(),
                "line 12" => "OUR line 12".to_string(),
                _ => l.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let theirs = base
            .lines()
            .map(|l| match l {
                "line 2" => "THEIRS line 2".to_string(),
                "line 12" => "THEIRS line 12".to_string(),
                _ => l.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let result = three_way(&base, &ours, &theirs).unwrap();
        match result {
            MergeOutcome::Conflicted { text } => {
                assert_eq!(count_conflicts(&text), 2);
                let with_noise = format!("{}content line with <<<<<<< noise\n", text);
                assert_eq!(count_conflicts(&with_noise), 2);
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
