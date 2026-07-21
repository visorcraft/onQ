//! Hierarchical project paths stored as slash-separated folder labels.
//!
//! Projects are free-form paths on prompt frontmatter (`folder: "Writing/Blog Posts"`).
//! The `folders` table registers the same paths so empty projects can exist.
//! Separators are `/` only; leading/trailing slashes and empty segments are rejected
//! after normalization.

use crate::error::{CoreError, CoreResult};

/// Maximum path depth (root counts as 1). Keeps the tree UI and rename cascade
/// bounded; nested deeper than this is almost always a modeling mistake.
pub const MAX_DEPTH: usize = 8;

/// Maximum length of a full path after normalization.
pub const MAX_PATH_LEN: usize = 200;

/// Normalize a project path: trim, collapse whitespace around segments, strip
/// surrounding `/`, reject empty / `.` / `..` segments and control characters.
pub fn normalize(raw: &str) -> CoreResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Other("project path cannot be empty".into()));
    }
    let mut parts: Vec<String> = Vec::new();
    for seg in trimmed.split('/') {
        let s = seg.trim();
        if s.is_empty() {
            continue;
        }
        if s == "." || s == ".." {
            return Err(CoreError::Other(
                "project path segments cannot be '.' or '..'".into(),
            ));
        }
        if s.chars()
            .any(|c| c.is_control() || c == '\0' || c == '"' || c == '\'')
        {
            return Err(CoreError::Other(
                "project path contains invalid characters".into(),
            ));
        }
        parts.push(s.to_string());
    }
    if parts.is_empty() {
        return Err(CoreError::Other("project path cannot be empty".into()));
    }
    if parts.len() > MAX_DEPTH {
        return Err(CoreError::Other(format!(
            "project path deeper than {MAX_DEPTH} levels"
        )));
    }
    let out = parts.join("/");
    if out.len() > MAX_PATH_LEN {
        return Err(CoreError::Other(format!(
            "project path longer than {MAX_PATH_LEN} characters"
        )));
    }
    Ok(out)
}

/// Parent path, if any. `"Writing/Blog"` → `Some("Writing")`; `"Writing"` → `None`.
pub fn parent(path: &str) -> Option<String> {
    let path = path.trim_matches('/');
    path.rsplit_once('/').map(|(p, _)| p.to_string())
}

/// True when `child` is `parent` or a descendant (`Writing/Blog` under `Writing`).
pub fn is_under(child: &str, ancestor: &str) -> bool {
    if child == ancestor {
        return true;
    }
    child.starts_with(ancestor) && child.as_bytes().get(ancestor.len()) == Some(&b'/')
}

/// If `path` is `old` or under `old`, return it rewritten under `new`.
/// Otherwise `None`.
pub fn rewrite_prefix(path: &str, old: &str, new: &str) -> Option<String> {
    if path == old {
        return Some(new.to_string());
    }
    if path.starts_with(old) && path.as_bytes().get(old.len()) == Some(&b'/') {
        return Some(format!("{new}{}", &path[old.len()..]));
    }
    None
}

/// Depth of a normalized path (1 = root segment).
pub fn depth(path: &str) -> usize {
    if path.is_empty() {
        0
    } else {
        path.bytes().filter(|&b| b == b'/').count() + 1
    }
}

/// Direct child name of `path` under `ancestor`, if `path` is a descendant.
/// e.g. child=`Writing/Blog/Drafts`, ancestor=`Writing` → `Some("Blog")`.
pub fn next_segment_under<'a>(path: &'a str, ancestor: &str) -> Option<&'a str> {
    if !is_under(path, ancestor) || path == ancestor {
        return None;
    }
    let rest = &path[ancestor.len() + 1..];
    Some(rest.split('/').next().unwrap_or(rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_collapses_slashes_and_trims() {
        assert_eq!(
            normalize("  Writing // Blog Posts / ").unwrap(),
            "Writing/Blog Posts"
        );
    }

    #[test]
    fn normalize_rejects_dot_segments() {
        assert!(normalize("Writing/../x").is_err());
        assert!(normalize(".").is_err());
    }

    #[test]
    fn normalize_rejects_quotes() {
        assert!(normalize(r#"Writing/"Blog""#).is_err());
        assert!(normalize("Writing/'x'").is_err());
    }

    #[test]
    fn is_under_matches_self_and_descendants() {
        assert!(is_under("Writing", "Writing"));
        assert!(is_under("Writing/Blog", "Writing"));
        assert!(!is_under("Writing2", "Writing"));
        assert!(!is_under("Writ", "Writing"));
    }

    #[test]
    fn rewrite_prefix_renames_tree() {
        assert_eq!(
            rewrite_prefix("Writing/Blog", "Writing", "Drafts").as_deref(),
            Some("Drafts/Blog")
        );
        assert_eq!(
            rewrite_prefix("Writing", "Writing", "Drafts").as_deref(),
            Some("Drafts")
        );
        assert!(rewrite_prefix("Coding", "Writing", "Drafts").is_none());
    }

    #[test]
    fn parent_and_depth() {
        assert_eq!(parent("Writing/Blog").as_deref(), Some("Writing"));
        assert_eq!(parent("Writing"), None);
        assert_eq!(depth("Writing/Blog/A"), 3);
    }
}
