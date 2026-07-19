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
    let rest = after_open
        .trim_start_matches('\n')
        .trim_start_matches("\r\n");
    let (fm, body) = rest
        .split_once("\n---")
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
        let raw = "---\nid: 01HXY3K9F2N8PQRSTVWXYZABCDE\ntitle: Test\ntags: [a, b]\nfavorite: true\nlocked: false\ncreated: 2026-07-18T10:30:00Z\nupdated: 2026-07-18T10:30:00Z\n---\n\nBody text";
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
}
