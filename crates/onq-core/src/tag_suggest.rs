fn tokenize(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Suggest vocabulary tags found as token sequences in `body`, ranked by frequency.
///
/// Tokens are alphanumeric runs compared case-insensitively. Equal-frequency
/// matches retain their order from `vocabulary`.
pub fn suggest_tags(body: &str, vocabulary: &[String], max_n: usize) -> Vec<String> {
    let body_tokens = tokenize(body);
    let mut matches: Vec<(usize, usize)> = vocabulary
        .iter()
        .enumerate()
        .filter_map(|(index, tag)| {
            let tag_tokens = tokenize(tag);
            if tag_tokens.is_empty() {
                return None;
            }

            let frequency = body_tokens
                .windows(tag_tokens.len())
                .filter(|window| *window == tag_tokens.as_slice())
                .count();
            (frequency > 0).then_some((index, frequency))
        })
        .collect();

    matches.sort_by(
        |(left_index, left_frequency), (right_index, right_frequency)| {
            right_frequency
                .cmp(left_frequency)
                .then_with(|| left_index.cmp(right_index))
        },
    );

    matches
        .into_iter()
        .take(max_n)
        .map(|(index, _)| vocabulary[index].clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::suggest_tags;

    #[test]
    fn extracts_case_insensitive_matches_ranked_by_frequency() {
        let vocabulary = vec!["Rust".into(), "database".into(), "testing".into()];

        let suggestions = suggest_tags(
            "Rust makes testing reliable. rust, rust, and DATABASE tooling.",
            &vocabulary,
            2,
        );

        assert_eq!(suggestions, vec!["Rust", "database"]);
    }

    #[test]
    fn matches_separator_bearing_vocabulary_tags() {
        let vocabulary = vec!["machine-learning".into(), "node.js".into()];

        let suggestions = suggest_tags("Machine-learning systems can use Node.js.", &vocabulary, 5);

        assert_eq!(suggestions, vec!["machine-learning", "node.js"]);
    }

    #[test]
    fn equal_frequencies_preserve_vocabulary_order() {
        let vocabulary = vec!["beta".into(), "alpha".into()];

        assert_eq!(
            suggest_tags("alpha beta", &vocabulary, 2),
            vec!["beta", "alpha"]
        );
    }

    #[test]
    fn zero_limit_returns_no_suggestions() {
        let vocabulary = vec!["rust".into()];

        assert!(suggest_tags("rust rust", &vocabulary, 0).is_empty());
    }

    #[test]
    fn limit_above_match_count_returns_all_matches() {
        let vocabulary = vec!["rust".into(), "database".into()];

        assert_eq!(suggest_tags("rust", &vocabulary, 10), vec!["rust"]);
    }

    #[test]
    fn empty_body_returns_no_suggestions() {
        let vocabulary = vec!["rust".into()];

        assert!(suggest_tags("", &vocabulary, 5).is_empty());
    }

    #[test]
    fn body_without_vocabulary_matches_returns_no_suggestions() {
        let vocabulary = vec!["rust".into(), "database".into()];

        assert!(suggest_tags("A prompt about gardening", &vocabulary, 5).is_empty());
    }
}
