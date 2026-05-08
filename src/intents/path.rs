// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Field path resolver for Field-JSON v3.x intents.
//!
//! v3 spec defines paths as dot-separated names with optional `[index]`
//! suffixes — `enchant_data_list[0].enchant_stat_data.stat_list_static[2].value`.
//! This module parses that syntax into an ordered list of `PathSegment`s
//! and walks a `serde_json::Value` to read/write/append at the resolved
//! location.

use std::fmt;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    /// Object key — `obj["name"]`.
    Field(String),
    /// Array index — `arr[idx]`.
    Index(usize),
}

#[derive(Debug)]
pub enum PathError {
    /// Path was syntactically invalid (e.g. mismatched brackets).
    BadSyntax(String),
    /// A segment named a key the object doesn't contain.
    MissingField { path: String, segment: String },
    /// Tried to index something that isn't an array.
    NotArray { path: String, got: &'static str },
    /// Tried to access a field on something that isn't an object.
    NotObject { path: String, got: &'static str },
    /// Index out of bounds.
    IndexOutOfRange { path: String, index: usize, len: usize },
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::BadSyntax(p) => write!(f, "bad path syntax: {}", p),
            PathError::MissingField { path, segment } => {
                write!(f, "path '{}': field '{}' missing", path, segment)
            }
            PathError::NotArray { path, got } => {
                write!(f, "path '{}': expected array, got {}", path, got)
            }
            PathError::NotObject { path, got } => {
                write!(f, "path '{}': expected object, got {}", path, got)
            }
            PathError::IndexOutOfRange { path, index, len } => {
                write!(f, "path '{}': index {} out of range (len {})", path, index, len)
            }
        }
    }
}

impl std::error::Error for PathError {}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Parse a v3 field path into segments.
///
/// Splits on top-level `.` (not inside `[...]`) and breaks each `name[idx]`
/// chunk into a `Field` followed by an `Index`. Empty paths yield an empty
/// segment list (caller decides whether that's an error).
///
/// ```ignore
/// parse_path("enchant_data_list[0].value")
///   == [Field("enchant_data_list"), Index(0), Field("value")]
/// ```
pub fn parse_path(path: &str) -> Result<Vec<PathSegment>, PathError> {
    let mut out = Vec::new();
    if path.is_empty() {
        return Ok(out);
    }

    // Split on '.' but not inside brackets. Track bracket depth (always 0 or
    // 1 in practice — v3 doesn't nest [] inside []).
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let bytes = path.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth < 0 {
                    return Err(PathError::BadSyntax(path.to_string()));
                }
            }
            b'.' if depth == 0 => {
                process_chunk(&path[start..i], &mut out, path)?;
                start = i + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(PathError::BadSyntax(path.to_string()));
    }
    process_chunk(&path[start..], &mut out, path)?;
    Ok(out)
}

/// Convert one "name" or "name[i]" or "name[i][j]" chunk into segments.
/// Multiple bracketed indices on the same chunk are allowed (rare but
/// permitted by the syntax).
fn process_chunk(
    chunk: &str,
    out: &mut Vec<PathSegment>,
    full_path: &str,
) -> Result<(), PathError> {
    if chunk.is_empty() {
        return Err(PathError::BadSyntax(full_path.to_string()));
    }
    let mut rest = chunk;
    // Pull the leading field name (everything before the first '[').
    let name_end = rest.find('[').unwrap_or(rest.len());
    let name = &rest[..name_end];
    if name.is_empty() {
        return Err(PathError::BadSyntax(full_path.to_string()));
    }
    out.push(PathSegment::Field(name.to_string()));
    rest = &rest[name_end..];
    // Each subsequent "[idx]" group becomes an Index segment.
    while !rest.is_empty() {
        if !rest.starts_with('[') {
            return Err(PathError::BadSyntax(full_path.to_string()));
        }
        let close = rest.find(']').ok_or_else(|| PathError::BadSyntax(full_path.to_string()))?;
        let idx_str = &rest[1..close];
        let idx: usize = idx_str
            .parse()
            .map_err(|_| PathError::BadSyntax(full_path.to_string()))?;
        out.push(PathSegment::Index(idx));
        rest = &rest[close + 1..];
    }
    Ok(())
}

/// Walk `target` along `path` and return a mutable reference to the leaf.
/// Returns an error if any intermediate segment is missing or has the
/// wrong type. The leaf itself is allowed to be missing for `Field`
/// segments — the caller decides whether to insert.
///
/// For internal use only; prefer [`set_value_at_path`] /
/// [`get_value_at_path`] / [`array_append_at_path`] in callers.
fn walk_to_parent<'a, 's>(
    target: &'a mut Value,
    segments: &'s [PathSegment],
    full_path: &str,
) -> Result<(&'a mut Value, &'s PathSegment), PathError> {
    if segments.is_empty() {
        return Err(PathError::BadSyntax(full_path.to_string()));
    }
    let (last, init) = segments.split_last().unwrap();
    let mut cur = target;
    for seg in init {
        cur = match seg {
            PathSegment::Field(name) => match cur {
                Value::Object(map) => map.get_mut(name).ok_or_else(|| PathError::MissingField {
                    path: full_path.to_string(),
                    segment: name.clone(),
                })?,
                other => {
                    return Err(PathError::NotObject {
                        path: full_path.to_string(),
                        got: type_name(other),
                    });
                }
            },
            PathSegment::Index(idx) => match cur {
                Value::Array(arr) => {
                    let len = arr.len();
                    arr.get_mut(*idx).ok_or_else(|| PathError::IndexOutOfRange {
                        path: full_path.to_string(),
                        index: *idx,
                        len,
                    })?
                }
                other => {
                    return Err(PathError::NotArray {
                        path: full_path.to_string(),
                        got: type_name(other),
                    });
                }
            },
        };
    }
    Ok((cur, last))
}

/// Set the value at `path` in `target`. Replaces the existing leaf or
/// inserts a new key if the leaf is a `Field` segment that doesn't exist
/// yet.
pub fn set_value_at_path(
    target: &mut Value,
    path: &str,
    new_value: Value,
) -> Result<(), PathError> {
    let segments = parse_path(path)?;
    if segments.is_empty() {
        return Err(PathError::BadSyntax(path.to_string()));
    }
    let (parent, last) = walk_to_parent(target, &segments, path)?;
    match last {
        PathSegment::Field(name) => match parent {
            Value::Object(map) => {
                map.insert(name.clone(), new_value);
                Ok(())
            }
            other => Err(PathError::NotObject {
                path: path.to_string(),
                got: type_name(other),
            }),
        },
        PathSegment::Index(idx) => match parent {
            Value::Array(arr) => {
                let len = arr.len();
                if *idx >= len {
                    return Err(PathError::IndexOutOfRange {
                        path: path.to_string(),
                        index: *idx,
                        len,
                    });
                }
                arr[*idx] = new_value;
                Ok(())
            }
            other => Err(PathError::NotArray {
                path: path.to_string(),
                got: type_name(other),
            }),
        },
    }
}

/// Read the value at `path`. The leaf must exist; missing fields produce
/// `MissingField` and out-of-range indices produce `IndexOutOfRange`.
pub fn get_value_at_path<'a>(
    target: &'a Value,
    path: &str,
) -> Result<&'a Value, PathError> {
    let segments = parse_path(path)?;
    let mut cur = target;
    for seg in &segments {
        cur = match seg {
            PathSegment::Field(name) => match cur {
                Value::Object(map) => map.get(name).ok_or_else(|| PathError::MissingField {
                    path: path.to_string(),
                    segment: name.clone(),
                })?,
                other => {
                    return Err(PathError::NotObject {
                        path: path.to_string(),
                        got: type_name(other),
                    });
                }
            },
            PathSegment::Index(idx) => match cur {
                Value::Array(arr) => {
                    let len = arr.len();
                    arr.get(*idx).ok_or_else(|| PathError::IndexOutOfRange {
                        path: path.to_string(),
                        index: *idx,
                        len,
                    })?
                }
                other => {
                    return Err(PathError::NotArray {
                        path: path.to_string(),
                        got: type_name(other),
                    });
                }
            },
        };
    }
    Ok(cur)
}

/// Append `value` to the array located at `path`. The leaf must already
/// exist and be an array.
pub fn array_append_at_path(
    target: &mut Value,
    path: &str,
    value: Value,
) -> Result<(), PathError> {
    let segments = parse_path(path)?;
    if segments.is_empty() {
        return Err(PathError::BadSyntax(path.to_string()));
    }
    let mut cur = target;
    for seg in &segments {
        cur = match seg {
            PathSegment::Field(name) => match cur {
                Value::Object(map) => map.get_mut(name).ok_or_else(|| PathError::MissingField {
                    path: path.to_string(),
                    segment: name.clone(),
                })?,
                other => {
                    return Err(PathError::NotObject {
                        path: path.to_string(),
                        got: type_name(other),
                    });
                }
            },
            PathSegment::Index(idx) => match cur {
                Value::Array(arr) => {
                    let len = arr.len();
                    arr.get_mut(*idx).ok_or_else(|| PathError::IndexOutOfRange {
                        path: path.to_string(),
                        index: *idx,
                        len,
                    })?
                }
                other => {
                    return Err(PathError::NotArray {
                        path: path.to_string(),
                        got: type_name(other),
                    });
                }
            },
        };
    }
    match cur {
        Value::Array(arr) => {
            arr.push(value);
            Ok(())
        }
        other => Err(PathError::NotArray {
            path: path.to_string(),
            got: type_name(other),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_simple_field() {
        assert_eq!(parse_path("cooltime").unwrap(), vec![PathSegment::Field("cooltime".into())]);
    }

    #[test]
    fn parse_nested() {
        assert_eq!(
            parse_path("drop_default_data.use_socket").unwrap(),
            vec![
                PathSegment::Field("drop_default_data".into()),
                PathSegment::Field("use_socket".into()),
            ],
        );
    }

    #[test]
    fn parse_indexed() {
        assert_eq!(
            parse_path("enchant_data_list[0].value").unwrap(),
            vec![
                PathSegment::Field("enchant_data_list".into()),
                PathSegment::Index(0),
                PathSegment::Field("value".into()),
            ],
        );
    }

    #[test]
    fn parse_double_index() {
        assert_eq!(
            parse_path("matrix[2][3]").unwrap(),
            vec![
                PathSegment::Field("matrix".into()),
                PathSegment::Index(2),
                PathSegment::Index(3),
            ],
        );
    }

    #[test]
    fn parse_full_v3_example() {
        let p = "enchant_data_list[0].enchant_stat_data.stat_list_static[2].value";
        assert_eq!(
            parse_path(p).unwrap(),
            vec![
                PathSegment::Field("enchant_data_list".into()),
                PathSegment::Index(0),
                PathSegment::Field("enchant_stat_data".into()),
                PathSegment::Field("stat_list_static".into()),
                PathSegment::Index(2),
                PathSegment::Field("value".into()),
            ],
        );
    }

    #[test]
    fn parse_rejects_bad_syntax() {
        assert!(parse_path("foo[").is_err());
        assert!(parse_path("foo]").is_err());
        assert!(parse_path("foo[abc]").is_err());
        assert!(parse_path(".bar").is_err());
        assert!(parse_path("foo..bar").is_err());
    }

    #[test]
    fn set_top_level() {
        let mut v = json!({ "a": 1 });
        set_value_at_path(&mut v, "a", json!(42)).unwrap();
        assert_eq!(v["a"], 42);
    }

    #[test]
    fn set_inserts_new_field() {
        let mut v = json!({ "a": 1 });
        set_value_at_path(&mut v, "b", json!("hi")).unwrap();
        assert_eq!(v["b"], "hi");
    }

    #[test]
    fn set_nested() {
        let mut v = json!({ "outer": { "inner": 1 } });
        set_value_at_path(&mut v, "outer.inner", json!(99)).unwrap();
        assert_eq!(v["outer"]["inner"], 99);
    }

    #[test]
    fn set_array_element() {
        let mut v = json!({ "arr": [10, 20, 30] });
        set_value_at_path(&mut v, "arr[1]", json!(99)).unwrap();
        assert_eq!(v["arr"][1], 99);
    }

    #[test]
    fn set_deep_path() {
        let mut v = json!({
            "enchant_data_list": [
                {
                    "enchant_stat_data": {
                        "stat_list_static": [
                            { "value": 0 },
                            { "value": 0 },
                        ]
                    }
                }
            ]
        });
        set_value_at_path(
            &mut v,
            "enchant_data_list[0].enchant_stat_data.stat_list_static[1].value",
            json!(999),
        ).unwrap();
        assert_eq!(
            v["enchant_data_list"][0]["enchant_stat_data"]["stat_list_static"][1]["value"],
            999,
        );
    }

    #[test]
    fn set_index_out_of_range() {
        let mut v = json!({ "arr": [1, 2] });
        let err = set_value_at_path(&mut v, "arr[5]", json!(0)).unwrap_err();
        assert!(matches!(err, PathError::IndexOutOfRange { .. }));
    }

    #[test]
    fn set_field_on_non_object_errors() {
        let mut v = json!({ "x": 1 });
        let err = set_value_at_path(&mut v, "x.y", json!(0)).unwrap_err();
        assert!(matches!(err, PathError::NotObject { .. }));
    }

    #[test]
    fn get_returns_leaf() {
        let v = json!({ "a": { "b": [10, 20] } });
        assert_eq!(get_value_at_path(&v, "a.b[1]").unwrap(), &json!(20));
    }

    #[test]
    fn array_append_basic() {
        let mut v = json!({ "items": [1, 2] });
        array_append_at_path(&mut v, "items", json!(3)).unwrap();
        assert_eq!(v["items"], json!([1, 2, 3]));
    }

    #[test]
    fn array_append_nested() {
        let mut v = json!({ "outer": { "inner": [] } });
        array_append_at_path(&mut v, "outer.inner", json!({"k": "v"})).unwrap();
        assert_eq!(v["outer"]["inner"], json!([{ "k": "v" }]));
    }
}
