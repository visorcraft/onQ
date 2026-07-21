//! Smart-folder DSL parser. The DSL is a whitespace-separated token stream
//! that compiles into a [`SearchQuery`] (see [`crate::search`]). It is the
//! execution half of the M5.3 CRUD work: smart folders store a DSL string,
//! and running one means parse → [`SearchQuery`] → `search` Tauri command.
//!
//! Token kinds:
//!
//! - `text:<phrase>`  — quoted phrase contributes to `q.text`; multiple
//!   `text:` tokens join with single spaces.
//! - `folder:<name>`  — exact folder match (`q.folder = Some(...)`).
//! - `tag:<name>`     — adds the tag to `q.tags_any`.
//! - `-tag:<name>`    — removes the tag from `q.tags_any` (no-op if absent).
//! - `favorite:true|false`, `locked:true|false` — pinned booleans.
//! - `char:>N`, `char:<N` — half-open char-count bounds.
//!
//! Anything that doesn't match a recognized prefix is rejected as
//! `CoreError::Other("unknown DSL token: …")`. This deliberately refuses
//! whitelisted-but-dangerous-looking operators (`exec:`, `script:` …) — the
//! parser never grants the DSL a side-channel to run anything.

use crate::error::{CoreError, CoreResult};
use crate::folder_path;
use crate::search::SearchQuery;

/// Parse a smart-folder DSL string into a [`SearchQuery`].
pub fn parse(input: &str) -> CoreResult<SearchQuery> {
    let mut q = SearchQuery::new("");
    let mut free_text: Vec<String> = vec![];
    let tokens = tokenize(input);
    for tok in &tokens {
        if let Some(rest) = tok.strip_prefix("text:") {
            free_text.push(unquote(rest));
        } else if let Some(rest) = tok.strip_prefix("folder:") {
            q.folder = Some(unquote(rest));
        } else if let Some(rest) = tok.strip_prefix("-tag:") {
            // Negation: drop a previously-pushed tag. No-op when the tag
            // wasn't included — the DSL is descriptive, not ordered.
            let needle = unquote(rest);
            q.tags_any.retain(|t| t != &needle);
        } else if let Some(rest) = tok.strip_prefix("tag:") {
            q.tags_any.push(unquote(rest));
        } else if tok == "favorite:true" {
            q.favorite = Some(true);
        } else if tok == "favorite:false" {
            q.favorite = Some(false);
        } else if tok == "locked:true" {
            q.locked = Some(true);
        } else if tok == "locked:false" {
            q.locked = Some(false);
        } else if let Some(rest) = tok.strip_prefix("char:>") {
            q.char_min = rest.parse().ok();
        } else if let Some(rest) = tok.strip_prefix("char:<") {
            q.char_max = rest.parse().ok();
        } else {
            return Err(CoreError::Other(format!("unknown DSL token: {tok}")));
        }
    }
    q.text = free_text.join(" ");
    Ok(q)
}

/// Split the DSL into tokens. Whitespace is the separator, but content
/// inside a matching pair of `"` or `'` quotes is kept intact so that
/// `text:"api errors"` survives as a single token. Opening quotes are
/// greedy: an unmatched quote swallows the rest of the input. The single
/// character used to open must close — `"a'b"` is not a `"…"` token.
fn tokenize(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote: Option<char> = None;
    for c in input.chars() {
        match (c, in_quote) {
            (q @ ('"' | '\''), None) => {
                in_quote = Some(q);
                cur.push(q);
            }
            (q, Some(open)) if q == open => {
                in_quote = None;
                cur.push(q);
            }
            (ch, None) if ch.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            (ch, _) => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Strip a single layer of `"…"` or `'…'` if both ends match. We don't try
/// to be smarter about embedded quotes — the DSL deliberately uses only
/// one quoting style per operand.
fn unquote(s: &str) -> String {
    s.trim_matches('"').trim_matches('\'').to_string()
}

/// Rewrite every `folder:` operand under `old` to the corresponding path
/// under `new` (same prefix rules as [`folder_path::rewrite_prefix`]).
/// Non-folder tokens are left unchanged. Used when a project is renamed so
/// saved smart-folder queries keep matching.
pub fn rewrite_folder_paths(dsl: &str, old: &str, new: &str) -> String {
    let tokens = tokenize(dsl);
    let mut out = Vec::with_capacity(tokens.len());
    for tok in tokens {
        if let Some(rest) = tok.strip_prefix("folder:") {
            let name = unquote(rest);
            if let Some(rewritten) = folder_path::rewrite_prefix(&name, old, new) {
                let quoted = rest.starts_with('"')
                    || rest.starts_with('\'')
                    || rewritten.contains(char::is_whitespace);
                if quoted {
                    out.push(format!("folder:\"{}\"", rewritten.replace('"', "")));
                } else {
                    out.push(format!("folder:{rewritten}"));
                }
            } else {
                out.push(tok);
            }
        } else {
            out.push(tok);
        }
    }
    out.join(" ")
}

/// Remove every `folder:` token whose path is under `ancestor` (self or
/// descendant). Used when a project is deleted so saved searches do not keep
/// filtering on a dead path. If the DSL becomes empty, returns a filter that
/// matches nothing (`char:<-1`) so the smart folder does not silently become
/// "all prompts".
pub fn strip_folder_paths_under(dsl: &str, ancestor: &str) -> String {
    let tokens = tokenize(dsl);
    let mut out = Vec::with_capacity(tokens.len());
    for tok in tokens {
        if let Some(rest) = tok.strip_prefix("folder:") {
            let name = unquote(rest);
            if folder_path::is_under(&name, ancestor) {
                continue;
            }
            out.push(tok);
        } else {
            out.push(tok);
        }
    }
    if out.is_empty() {
        // Impossible char range — char_count is always >= 0.
        "char:<-1".to_string()
    } else {
        out.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_dsl() {
        let q =
            parse(r#"text:"api errors" folder:work tag:python -tag:wip favorite:true"#).unwrap();
        assert_eq!(q.text, "api errors");
        assert_eq!(q.folder.as_deref(), Some("work"));
        assert_eq!(q.tags_any, vec!["python"]);
        assert!(q.favorite == Some(true));
    }

    #[test]
    fn rejects_unknown_operator() {
        assert!(parse(r#"exec:"rm -rf /""#).is_err());
    }

    #[test]
    fn parses_char_range() {
        let q = parse("char:>100 char:<1000").unwrap();
        assert_eq!(q.char_min, Some(100));
        assert_eq!(q.char_max, Some(1000));
    }

    #[test]
    fn rewrite_folder_paths_renames_operands() {
        let dsl = r#"folder:Writing tag:x folder:"Writing/Blog Posts" text:"keep""#;
        let out = rewrite_folder_paths(dsl, "Writing", "Drafts");
        assert!(out.contains("folder:Drafts"));
        assert!(out.contains(r#"folder:"Drafts/Blog Posts""#));
        assert!(out.contains("tag:x"));
        assert!(out.contains(r#"text:"keep""#));
        // Unrelated root stays put.
        assert_eq!(
            rewrite_folder_paths("folder:Coding", "Writing", "Drafts"),
            "folder:Coding"
        );
    }

    #[test]
    fn strip_folder_paths_under_removes_matching_tokens() {
        let dsl = r#"folder:Doomed tag:x folder:"Doomed/Child" folder:Keep"#;
        let out = strip_folder_paths_under(dsl, "Doomed");
        assert!(out.contains("tag:x"));
        assert!(out.contains("folder:Keep"));
        assert!(!out.contains("Doomed"));
        assert_eq!(
            strip_folder_paths_under("folder:Doomed", "Doomed"),
            "char:<-1"
        );
    }
}
