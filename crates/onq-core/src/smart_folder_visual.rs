//! Visual smart-folder predicate model ↔ DSL codec.

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};
use crate::smart_folder_dsl;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VisualPredicate {
    pub field: String,
    pub op: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct VisualQuery {
    #[serde(default)]
    pub predicates: Vec<VisualPredicate>,
}

/// Convert visual predicates into DSL text accepted by [`smart_folder_dsl::parse`].
pub fn visual_to_dsl(visual: &VisualQuery) -> CoreResult<String> {
    let mut tokens = Vec::new();
    for p in &visual.predicates {
        let token = match (p.field.as_str(), p.op.as_str()) {
            ("tag", "is") | ("tag", "eq") => format!("tag:{}", quote_if_needed(&p.value)),
            ("tag", "not") => format!("-tag:{}", quote_if_needed(&p.value)),
            ("folder", "is") | ("folder", "eq") => {
                format!("folder:{}", quote_if_needed(&p.value))
            }
            ("favorite", "is") | ("favorite", "eq") => {
                let v = if p.value == "true" || p.value == "1" {
                    "true"
                } else {
                    "false"
                };
                format!("favorite:{v}")
            }
            ("locked", "is") | ("locked", "eq") => {
                let v = if p.value == "true" || p.value == "1" {
                    "true"
                } else {
                    "false"
                };
                format!("locked:{v}")
            }
            ("text", "contains") | ("text", "is") => {
                format!("text:{}", quote_if_needed(&p.value))
            }
            (field, op) => {
                return Err(CoreError::Other(format!(
                    "unsupported visual predicate {field}.{op}"
                )))
            }
        };
        tokens.push(token);
    }
    let dsl = tokens.join(" ");
    // Validate
    smart_folder_dsl::parse(&dsl)?;
    Ok(dsl)
}

/// Best-effort conversion from DSL tokens to visual predicates.
/// Returns `None` when the DSL cannot be fully represented.
pub fn dsl_to_visual(dsl: &str) -> Option<VisualQuery> {
    let mut predicates = Vec::new();
    for raw in dsl.split_whitespace() {
        let tok = raw.trim();
        if tok.is_empty() {
            continue;
        }
        if let Some(rest) = tok.strip_prefix("-tag:") {
            predicates.push(VisualPredicate {
                field: "tag".into(),
                op: "not".into(),
                value: unquote(rest),
            });
        } else if let Some(rest) = tok.strip_prefix("tag:") {
            predicates.push(VisualPredicate {
                field: "tag".into(),
                op: "is".into(),
                value: unquote(rest),
            });
        } else if let Some(rest) = tok.strip_prefix("folder:") {
            predicates.push(VisualPredicate {
                field: "folder".into(),
                op: "is".into(),
                value: unquote(rest),
            });
        } else if tok == "favorite:true" || tok == "favorite:false" {
            predicates.push(VisualPredicate {
                field: "favorite".into(),
                op: "is".into(),
                value: tok.trim_start_matches("favorite:").into(),
            });
        } else if tok == "locked:true" || tok == "locked:false" {
            predicates.push(VisualPredicate {
                field: "locked".into(),
                op: "is".into(),
                value: tok.trim_start_matches("locked:").into(),
            });
        } else {
            let rest = tok.strip_prefix("text:")?;
            predicates.push(VisualPredicate {
                field: "text".into(),
                op: "contains".into(),
                value: unquote(rest),
            });
        }
    }
    Some(VisualQuery { predicates })
}

fn quote_if_needed(value: &str) -> String {
    if value.contains(char::is_whitespace) || value.contains(':') {
        format!("\"{}\"", value.replace('"', ""))
    } else {
        value.to_string()
    }
}

fn unquote(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_round_trip_basic() {
        let visual = VisualQuery {
            predicates: vec![
                VisualPredicate {
                    field: "tag".into(),
                    op: "is".into(),
                    value: "writing".into(),
                },
                VisualPredicate {
                    field: "favorite".into(),
                    op: "is".into(),
                    value: "true".into(),
                },
            ],
        };
        let dsl = visual_to_dsl(&visual).unwrap();
        assert!(dsl.contains("tag:writing"));
        assert!(dsl.contains("favorite:true"));
        let back = dsl_to_visual(&dsl).unwrap();
        assert_eq!(back.predicates.len(), 2);
    }
}
