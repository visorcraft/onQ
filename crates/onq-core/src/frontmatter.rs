use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ulid::PromptId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frontmatter {
    pub id: PromptId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub locked: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

const DELIM: &str = "---";

pub fn split(raw: &str) -> Result<(&str, &str), crate::error::CoreError> {
    let trimmed = raw.trim_start_matches('\u{feff}');
    let after_open = trimmed
        .strip_prefix(DELIM)
        .ok_or_else(|| crate::error::CoreError::Frontmatter("missing opening ---".into()))?;
    match after_open.chars().next() {
        None | Some('\n') | Some('\r') => {}
        Some(_) => {
            return Err(crate::error::CoreError::Frontmatter(
                "missing opening ---".into(),
            ));
        }
    }
    let rest = after_open
        .trim_start_matches('\n')
        .trim_start_matches("\r\n");
    let mut search_from = 0;
    let mut accepted = None;
    while let Some(offset) = rest[search_from..].find("\n---") {
        let i = search_from + offset;
        let tail = &rest[i + "\n---".len()..];
        let bytes = tail.as_bytes();
        let mut j = 0;
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
            j += 1;
        }
        let is_closer = j == bytes.len() || bytes[j] == b'\n' || bytes[j] == b'\r';
        if is_closer {
            accepted = Some((&rest[..i], &tail[j..]));
            break;
        }
        search_from = i + 1;
    }
    let (fm, body) = accepted
        .ok_or_else(|| crate::error::CoreError::Frontmatter("missing closing ---".into()))?;
    let body = body.trim_start_matches('\n').trim_start_matches("\r\n");
    Ok((fm, body))
}

pub fn parse(raw: &str) -> Result<(Frontmatter, String), crate::error::CoreError> {
    let (fm_str, body) = split(raw)?;
    let fm: Frontmatter = serde_yaml::from_str(fm_str)
        .map_err(|e| crate::error::CoreError::Frontmatter(e.to_string()))?;
    Ok((fm, body.to_string()))
}

pub fn render(fm: &Frontmatter, body: &str) -> String {
    let yaml = serde_yaml::to_string(fm).expect("frontmatter serialization");
    format!("---\n{}---\n\n{}", yaml, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_roundtrip() {
        let raw = "---\nid: 01HXY3K9F2N8PQRSTVWXYZABCD\ntitle: Test\ntags: [a, b]\nfavorite: true\nlocked: false\ncreated: 2026-07-18T10:30:00Z\nupdated: 2026-07-18T10:30:00Z\n---\n\nBody text";
        let (fm, body) = parse(raw).unwrap();
        assert_eq!(fm.title, "Test");
        assert_eq!(fm.tags, vec!["a", "b"]);
        assert_eq!(body, "Body text");
        let rendered = render(&fm, &body);
        let (fm2, body2) = parse(&rendered).unwrap();
        assert_eq!(fm, fm2);
        assert_eq!(body, body2);
    }
    #[test]
    fn parse_rejects_missing_delim() {
        assert!(parse("not yaml").is_err());
    }
    #[test]
    fn split_rejects_longer_opening_fence() {
        let err = split("----\ntitle: T\n---\nbody").unwrap_err();
        assert!(err.to_string().contains("missing opening ---"));
    }
    #[test]
    fn split_rejects_opening_fence_with_suffix() {
        let err = split("--- text\nfoo: 1\n---\nbody").unwrap_err();
        assert!(err.to_string().contains("missing opening ---"));
    }
    #[test]
    fn parse_accepts_closer_with_trailing_space() {
        let raw = "---\nid: 01HXY3K9F2N8PQRSTVWXYZABCD\ntitle: T\ncreated: 2026-07-18T10:30:00Z\nupdated: 2026-07-18T10:30:00Z\n--- \nbody";
        let (fm, body) = parse(raw).unwrap();
        assert_eq!(fm.title, "T");
        assert_eq!(body, "body");
    }
    #[test]
    fn split_consumes_trailing_space_in_closer() {
        let (fm, body) = split("---\ntitle: T\n--- \nbody").unwrap();
        assert_eq!(fm, "title: T");
        assert_eq!(body, "body");
    }
    #[test]
    fn parse_rejects_four_dash_closer() {
        let err = parse("---\ntitle: T\n----\nbody").unwrap_err();
        assert!(err.to_string().contains("missing closing ---"));
    }
    #[test]
    fn split_skips_interior_partial_fence() {
        let raw = "---\ntitle: T\n---embedded\nreal: 1\n---\nbody";
        let (fm, body) = split(raw).unwrap();
        assert!(fm.contains("---embedded"));
        assert_eq!(body, "body");
    }
    #[test]
    fn parse_skips_interior_partial_fence_in_multiline_value() {
        let raw = "---\nid: 01HXY3K9F2N8PQRSTVWXYZABCD\ntitle: |\n  line 1\n---embedded\ncreated: 2026-07-18T10:30:00Z\nupdated: 2026-07-18T10:30:00Z\n---\nbody";
        let (fm, body) = split(raw).unwrap();
        assert!(fm.contains("---embedded"));
        assert_eq!(body, "body");
    }
    #[test]
    fn parse_roundtrip_empty_body() {
        let raw = "---\nid: 01HXY3K9F2N8PQRSTVWXYZABCD\ntitle: Test\ncreated: 2026-07-18T10:30:00Z\nupdated: 2026-07-18T10:30:00Z\n---\n\n";
        let (fm, body) = parse(raw).unwrap();
        assert_eq!(body, "");
        let rendered = render(&fm, "");
        let (fm2, body2) = parse(&rendered).unwrap();
        assert_eq!(fm, fm2);
        assert_eq!(body2, "");
    }
}
