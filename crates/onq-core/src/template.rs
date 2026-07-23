//! Prompt template variables: `{{name}}` and `{{name|default}}`.
//!
//! Missing values render as the default when provided, otherwise empty string.
//! Use `\{\{` / `\}\}` to emit literal braces (backslash-escape).

use std::collections::{HashMap, HashSet};

use crate::error::CoreResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateField {
    pub name: String,
    pub default: Option<String>,
}

/// Extract unique template fields in order of first appearance.
pub fn parse_template(body: &str) -> Vec<TemplateField> {
    let mut fields = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            // skip escaped pair
            let _ = chars.next();
            continue;
        }
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            let mut inner = String::new();
            let mut closed = false;
            while let Some(n) = chars.next() {
                if n == '}' && chars.peek() == Some(&'}') {
                    chars.next();
                    closed = true;
                    break;
                }
                inner.push(n);
            }
            if !closed {
                continue;
            }
            let (name, default) = split_field(&inner);
            if name.is_empty() {
                continue;
            }
            if seen.insert(name.clone()) {
                fields.push(TemplateField { name, default });
            }
        }
    }
    fields
}

fn split_field(inner: &str) -> (String, Option<String>) {
    let trimmed = inner.trim();
    if let Some((name, def)) = trimmed.split_once('|') {
        (name.trim().to_string(), Some(def.trim().to_string()))
    } else {
        (trimmed.to_string(), None)
    }
}

/// Substitute template fields. Unknown fields use their default or empty.
pub fn render_template(body: &str, values: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(n) = chars.next() {
                out.push(n);
            }
            continue;
        }
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            let mut inner = String::new();
            let mut closed = false;
            while let Some(n) = chars.next() {
                if n == '}' && chars.peek() == Some(&'}') {
                    chars.next();
                    closed = true;
                    break;
                }
                inner.push(n);
            }
            if !closed {
                out.push_str("{{");
                out.push_str(&inner);
                continue;
            }
            let (name, default) = split_field(&inner);
            if let Some(v) = values.get(&name) {
                out.push_str(v);
            } else if let Some(d) = default {
                out.push_str(&d);
            }
            // else empty
            continue;
        }
        out.push(c);
    }
    out
}

/// Convenience: parse then no-op render validation.
pub fn preview_fields(body: &str) -> CoreResult<Vec<TemplateField>> {
    Ok(parse_template(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_and_default() {
        let fields = parse_template("Hello {{name}} from {{city|Paris}}");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "name");
        assert_eq!(fields[0].default, None);
        assert_eq!(fields[1].name, "city");
        assert_eq!(fields[1].default.as_deref(), Some("Paris"));
    }

    #[test]
    fn unique_order_preserved() {
        let fields = parse_template("{{a}} {{b}} {{a}}");
        assert_eq!(
            fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }

    #[test]
    fn render_missing_uses_default_or_empty() {
        let mut values = HashMap::new();
        values.insert("name".into(), "Ada".into());
        let out = render_template("{{name}} lives in {{city|London}} and {{x}}!", &values);
        assert_eq!(out, "Ada lives in London and !");
    }

    #[test]
    fn escape_literal_braces() {
        let out = render_template(r"use \{\{raw\}\} and {{v}}", &{
            let mut m = HashMap::new();
            m.insert("v".into(), "ok".into());
            m
        });
        assert_eq!(out, "use {{raw}} and ok");
    }
}
