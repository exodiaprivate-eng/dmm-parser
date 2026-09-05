// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! JSON-faithful types for the Field-JSON v3.x manifest format.
//!
//! Mirrors `FIELD_JSON_V3_SPEC.md` plus the v3.1 extensions documented in
//! `docs/CUSTOM_ITEM_CREATOR_V3_1.md`. Every field is `Option<T>` so a
//! single parse path handles every shape the spec allows:
//!
//!  - Legacy single-target (`target` + `intents` at the doc root)
//!  - v3.1 multi-target (`targets[].intents`)
//!  - The four intent ops: `set` (legacy), `clone_record`, `new_record`,
//!    `delete_record`. v3 also reserved `add_entry`; `array_append` is a
//!    v3.1 additive proposal handled here for storeinfo support.
//!
//! Parses straight off `serde_json::Value` to match the rest of the
//! codebase (see `py_binary_struct!` `to_json_dict` / `write_from_json_dict`
//! style — manual JSON walking, no `#[derive(Deserialize)]`).

use std::io;

use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct ModInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub note: Option<String>,
    pub category: Option<String>,
}

impl ModInfo {
    fn from_value(v: &Value) -> Self {
        let Some(obj) = v.as_object() else { return Self::default(); };
        let s = |k: &str| obj.get(k).and_then(|x| x.as_str()).map(|x| x.to_string());
        ModInfo {
            title: s("title"),
            author: s("author"),
            version: s("version"),
            description: s("description"),
            note: s("note"),
            category: s("category"),
        }
    }
}

/// One target block inside `targets[]` (v3.1 multi-target shape). Accepts
/// both `target` and the legacy alias `file` for the table identifier so
/// hand-written manifests using either key parse cleanly.
#[derive(Debug, Clone)]
pub struct Target {
    pub target: Option<String>,
    pub intents: Vec<Intent>,
}

/// A single edit operation. JSON-faithful — every field is optional and
/// the `op` discriminator is read at dispatch time. Shapes per op:
///
/// | op              | required                           | optional              |
/// |-----------------|------------------------------------|-----------------------|
/// | `set` (default) | `field`, `new`                     | `entry`, `key`        |
/// | `clone_record`  | `source_key`, `new_key`            | `patches`             |
/// | `new_record`    | `new_key`, (`template` or `data`)  | —                     |
/// | `delete_record` | `key`                              | —                     |
/// | `array_append`  | `field`, `value`                   | `entry`, `key`        |
/// | `add_entry`     | `key`, (`data` or `template`)      | `entry` (alias of new_record) |
#[derive(Debug, Clone, Default)]
pub struct Intent {
    pub op: Option<String>,

    // Lookup keys for set / array_append / delete_record.
    pub entry: Option<String>,
    pub key: Option<i64>,

    // set / array_append payload.
    pub field: Option<String>,
    pub new: Option<Value>,
    pub value: Option<Value>,

    // clone_record / new_record payload.
    pub source_key: Option<i64>,
    pub new_key: Option<i64>,
    pub patches: Option<Vec<Patch>>,
    pub template: Option<Value>,
    /// Legacy v3 spelling — `add_entry` used `data` for what v3.1 calls
    /// `template`. Accepted at parse time, normalized at resolve time.
    pub data: Option<Value>,
}

impl Intent {
    pub fn from_value(v: &Value) -> io::Result<Self> {
        let obj = v.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, format!("intent: expected object, got {}", type_name(v)))
        })?;
        let s = |k: &str| obj.get(k).and_then(|x| x.as_str()).map(|x| x.to_string());
        let i = |k: &str| obj.get(k).and_then(read_i64);
        let val = |k: &str| obj.get(k).cloned();
        let patches = match obj.get("patches") {
            None => None,
            Some(Value::Array(arr)) => {
                let mut out = Vec::with_capacity(arr.len());
                for p in arr {
                    out.push(Patch::from_value(p)?);
                }
                Some(out)
            }
            Some(other) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("intent.patches: expected array, got {}", type_name(other))));
            }
        };
        Ok(Intent {
            op: s("op"),
            entry: s("entry"),
            key: i("key"),
            field: s("field"),
            new: val("new"),
            value: val("value"),
            source_key: i("source_key"),
            new_key: i("new_key"),
            patches,
            template: val("template"),
            data: val("data"),
        })
    }
}

/// A single patch applied to a clone_record's clone (and reused for
/// future `merge` op shapes).
#[derive(Debug, Clone)]
pub struct Patch {
    pub path: String,
    /// Defaults to `"set"` if absent. Future values: `"merge"`, `"remove"`,
    /// `"array_append"`.
    pub op: Option<String>,
    pub new: Value,
}

impl Patch {
    pub fn from_value(v: &Value) -> io::Result<Self> {
        let obj = v.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, format!("patch: expected object, got {}", type_name(v)))
        })?;
        let path = obj.get("path").and_then(|x| x.as_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "patch: missing 'path' string"))?
            .to_string();
        let op = obj.get("op").and_then(|x| x.as_str()).map(|x| x.to_string());
        let new = obj.get("new").cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "patch: missing 'new' value"))?;
        Ok(Patch { path, op, new })
    }
}

#[derive(Debug, Clone)]
pub struct IntentDoc {
    pub modinfo: ModInfo,
    pub format: u32,
    pub format_minor: Option<u32>,

    // Legacy single-target shape (v3.0).
    pub target: Option<String>,
    pub intents: Option<Vec<Intent>>,

    // v3.1 multi-target shape.
    pub targets: Option<Vec<Target>>,
}

impl IntentDoc {
    pub fn from_value(v: &Value) -> io::Result<Self> {
        let obj = v.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, format!("doc: expected object, got {}", type_name(v)))
        })?;
        let format = obj.get("format").and_then(read_u32)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "doc: missing or non-integer 'format'"))?;
        let format_minor = obj.get("format_minor").and_then(read_u32);
        let modinfo = obj.get("modinfo").map(ModInfo::from_value).unwrap_or_default();
        let target = obj.get("target").and_then(|x| x.as_str()).map(|x| x.to_string());
        let intents = match obj.get("intents") {
            None => None,
            Some(Value::Array(arr)) => {
                let mut out = Vec::with_capacity(arr.len());
                for it in arr {
                    out.push(Intent::from_value(it)?);
                }
                Some(out)
            }
            Some(other) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("doc.intents: expected array, got {}", type_name(other))));
            }
        };
        let targets = match obj.get("targets") {
            None => None,
            Some(Value::Array(arr)) => {
                let mut out = Vec::with_capacity(arr.len());
                for t in arr {
                    out.push(Self::parse_target(t)?);
                }
                Some(out)
            }
            Some(other) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("doc.targets: expected array, got {}", type_name(other))));
            }
        };
        Ok(IntentDoc { modinfo, format, format_minor, target, intents, targets })
    }

    fn parse_target(v: &Value) -> io::Result<Target> {
        let obj = v.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, format!("target: expected object, got {}", type_name(v)))
        })?;
        // Accept both `target` and legacy `file` for the table identifier.
        let target = obj.get("target")
            .or_else(|| obj.get("file"))
            .and_then(|x| x.as_str())
            .map(|x| x.to_string());
        let intents = match obj.get("intents") {
            None => Vec::new(),
            Some(Value::Array(arr)) => {
                let mut out = Vec::with_capacity(arr.len());
                for it in arr {
                    out.push(Intent::from_value(it)?);
                }
                out
            }
            Some(other) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("target.intents: expected array, got {}", type_name(other))));
            }
        };
        Ok(Target { target, intents })
    }

    pub fn from_slice(bytes: &[u8]) -> io::Result<Self> {
        let v: Value = serde_json::from_slice(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("json parse: {}", e)))?;
        Self::from_value(&v)
    }

    /// Normalize the legacy and v3.1 shapes into a single
    /// `(target_name, intents)` list. v3.1 wins if both shapes are
    /// present (per the spec). Targets without an explicit `target`
    /// string are skipped — the caller cannot dispatch them.
    pub fn flatten_targets(&self) -> Vec<(String, Vec<Intent>)> {
        if let Some(targets) = &self.targets {
            return targets.iter()
                .filter_map(|t| t.target.clone().map(|name| (name, t.intents.clone())))
                .collect();
        }
        if let (Some(target), Some(intents)) = (&self.target, &self.intents) {
            return vec![(target.clone(), intents.clone())];
        }
        Vec::new()
    }

    /// Detection per the v3 spec: format=3 AND a non-empty intent list
    /// somewhere (legacy or any target). Returns true only when the doc
    /// would actually do something on apply.
    pub fn is_field_json_v3(&self) -> bool {
        if self.format != 3 {
            return false;
        }
        if self.targets.as_ref().is_some_and(|ts| ts.iter().any(|t| !t.intents.is_empty())) {
            return true;
        }
        if self.intents.as_ref().is_some_and(|i| !i.is_empty()) {
            return true;
        }
        false
    }
}

/// Resolved intent op — strongly typed, ready to apply. Use
/// [`Intent::resolve_op`] to convert a JSON-faithful `Intent` into one
/// of these.
#[derive(Debug, Clone)]
pub enum ResolvedIntentOp {
    /// Replace a field at `field` on the record matching `entry` /
    /// `key`. Default op when the manifest's `op` field is missing.
    Set {
        entry: Option<String>,
        key: Option<i64>,
        field: String,
        new: Value,
    },
    /// Append `value` to the array at `field` on the record matching
    /// `entry` / `key`. v3.1 additive.
    ArrayAppend {
        entry: Option<String>,
        key: Option<i64>,
        field: String,
        value: Value,
    },
    /// Add every element of `values` that the array at `field` does not
    /// already hold (by equality); skip the rest. `list_union` in the manifest.
    /// Additive and order-independent: two mods widening the same allow-list
    /// (a skill's `carray_u16`) both land whichever applies last, where two
    /// whole-list `set`s keep only the last writer's entries.
    ArrayUnion {
        entry: Option<String>,
        key: Option<i64>,
        field: String,
        values: Vec<Value>,
    },
    /// Clone the record with `source_key` under `new_key`, then apply
    /// `patches` to the clone.
    CloneRecord {
        source_key: i64,
        new_key: i64,
        patches: Vec<Patch>,
    },
    /// Insert a brand-new record built from `template`, with `key` set
    /// to `new_key`.
    NewRecord {
        new_key: i64,
        template: Value,
    },
    /// Remove the record matching `key`.
    DeleteRecord { key: i64 },
}

#[derive(Debug)]
pub enum IntentResolveError {
    UnknownOp(String),
    MissingField { op: String, field: &'static str },
}

impl std::fmt::Display for IntentResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentResolveError::UnknownOp(op) => write!(f, "unknown intent op '{}'", op),
            IntentResolveError::MissingField { op, field } => {
                write!(f, "intent op '{}' missing required field '{}'", op, field)
            }
        }
    }
}

impl std::error::Error for IntentResolveError {}

impl Intent {
    /// Convert a JSON-faithful intent into a strongly-typed operation.
    /// Defaults `op` to `"set"` when absent (matches the v3 spec).
    pub fn resolve_op(&self) -> Result<ResolvedIntentOp, IntentResolveError> {
        let op = self.op.as_deref().unwrap_or("set");
        match op {
            "set" => {
                let field = self.field.clone().ok_or(IntentResolveError::MissingField {
                    op: op.to_string(), field: "field",
                })?;
                let new = self.new.clone().ok_or(IntentResolveError::MissingField {
                    op: op.to_string(), field: "new",
                })?;
                Ok(ResolvedIntentOp::Set {
                    entry: self.entry.clone(),
                    key: self.key,
                    field,
                    new,
                })
            }
            "array_append" => {
                let field = self.field.clone().ok_or(IntentResolveError::MissingField {
                    op: op.to_string(), field: "field",
                })?;
                let value = self.value.clone().or_else(|| self.new.clone()).ok_or(
                    IntentResolveError::MissingField {
                        op: op.to_string(), field: "value",
                    },
                )?;
                Ok(ResolvedIntentOp::ArrayAppend {
                    entry: self.entry.clone(),
                    key: self.key,
                    field,
                    value,
                })
            }
            "list_union" => {
                let field = self.field.clone().ok_or(IntentResolveError::MissingField {
                    op: op.to_string(), field: "field",
                })?;
                let new = self.new.clone().or_else(|| self.value.clone()).ok_or(
                    IntentResolveError::MissingField {
                        op: op.to_string(), field: "new",
                    },
                )?;
                let values = match new {
                    Value::Array(a) => a,
                    v => vec![v],
                };
                Ok(ResolvedIntentOp::ArrayUnion {
                    entry: self.entry.clone(),
                    key: self.key,
                    field,
                    values,
                })
            }
            "clone_record" => {
                let source_key = self.source_key.ok_or(IntentResolveError::MissingField {
                    op: op.to_string(), field: "source_key",
                })?;
                let new_key = self.new_key.ok_or(IntentResolveError::MissingField {
                    op: op.to_string(), field: "new_key",
                })?;
                Ok(ResolvedIntentOp::CloneRecord {
                    source_key,
                    new_key,
                    patches: self.patches.clone().unwrap_or_default(),
                })
            }
            "new_record" | "add_entry" => {
                // Accept either spelling; v3 used add_entry, v3.1 prefers new_record.
                let new_key = self.new_key.or(self.key).ok_or(IntentResolveError::MissingField {
                    op: op.to_string(), field: "new_key",
                })?;
                let template = self.template.clone().or_else(|| self.data.clone()).ok_or(
                    IntentResolveError::MissingField {
                        op: op.to_string(), field: "template",
                    },
                )?;
                Ok(ResolvedIntentOp::NewRecord { new_key, template })
            }
            "delete_record" => {
                let key = self.key.ok_or(IntentResolveError::MissingField {
                    op: op.to_string(), field: "key",
                })?;
                Ok(ResolvedIntentOp::DeleteRecord { key })
            }
            other => Err(IntentResolveError::UnknownOp(other.to_string())),
        }
    }
}

/// Compute the v3 paloc key indices for a brand-new item key.
///
/// Per CUSTOM_ITEM_CREATOR_V3_1.md: every item's localized name lookup
/// uses category 0x07 with key = `(item_key << 32) | 0x70`, and the
/// description uses `(item_key << 32) | 0x71`. SWISS / DMM compute these
/// when the user picks a `new_key`; expose the formula here so v3.1
/// generators can reuse it without duplicating the bit math.
#[inline]
pub fn item_paloc_indices(item_key: u32) -> (u64, u64) {
    let base = (item_key as u64) << 32;
    (base | 0x70, base | 0x71)
}

// ── helpers ────────────────────────────────────────────────────────────────

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

fn read_i64(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() { return Some(n); }
    if let Some(n) = v.as_u64() { return i64::try_from(n).ok(); }
    if let Some(n) = v.as_f64() {
        if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            return Some(n as i64);
        }
    }
    None
}

fn read_u32(v: &Value) -> Option<u32> {
    let n = read_i64(v)?;
    u32::try_from(n).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_legacy_single_target() {
        let v = json!({
            "format": 3,
            "modinfo": { "title": "demo", "version": "1.0", "author": "x" },
            "target": "iteminfo.pabgb",
            "intents": [
                { "entry": "Foo", "key": 1, "field": "cooltime", "op": "set", "new": 1 }
            ]
        });
        let doc = IntentDoc::from_value(&v).unwrap();
        assert!(doc.is_field_json_v3());
        let flat = doc.flatten_targets();
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0, "iteminfo.pabgb");
        assert_eq!(flat[0].1.len(), 1);
        assert_eq!(flat[0].1[0].entry.as_deref(), Some("Foo"));
        assert_eq!(flat[0].1[0].field.as_deref(), Some("cooltime"));
    }

    #[test]
    fn parse_v3_1_multi_target() {
        let v = json!({
            "format": 3,
            "format_minor": 1,
            "modinfo": { "title": "demo", "version": "1.0", "author": "x" },
            "targets": [
                { "target": "iteminfo.pabgb", "intents": [
                    { "field": "cooltime", "new": 1, "key": 1 }
                ]},
                { "file": "buff_info.pabgb", "intents": [
                    { "key": 100, "field": "is_blocked", "new": 1 }
                ]}
            ]
        });
        let doc = IntentDoc::from_value(&v).unwrap();
        let flat = doc.flatten_targets();
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].0, "iteminfo.pabgb");
        assert_eq!(flat[1].0, "buff_info.pabgb");
    }

    #[test]
    fn parse_clone_record_patches() {
        let v = json!({
            "format": 3,
            "format_minor": 1,
            "target": "iteminfo.pabgb",
            "intents": [{
                "op": "clone_record",
                "source_key": 12345,
                "new_key": 999001,
                "patches": [
                    { "path": "item_name.default", "new": "Custom" },
                    { "path": "item_tier", "op": "set", "new": 5 },
                ]
            }]
        });
        let doc = IntentDoc::from_value(&v).unwrap();
        let flat = doc.flatten_targets();
        let intent = &flat[0].1[0];
        assert_eq!(intent.op.as_deref(), Some("clone_record"));
        assert_eq!(intent.source_key, Some(12345));
        assert_eq!(intent.new_key, Some(999001));
        let patches = intent.patches.as_ref().unwrap();
        assert_eq!(patches.len(), 2);
        assert_eq!(patches[0].path, "item_name.default");
        assert_eq!(patches[0].new, json!("Custom"));
    }

    #[test]
    fn resolve_set_default_op() {
        let intent = Intent {
            field: Some("cooltime".into()),
            new: Some(json!(1)),
            ..Default::default()
        };
        match intent.resolve_op().unwrap() {
            ResolvedIntentOp::Set { field, new, .. } => {
                assert_eq!(field, "cooltime");
                assert_eq!(new, json!(1));
            }
            other => panic!("expected Set, got {:?}", other),
        }
    }

    #[test]
    fn resolve_clone_record() {
        let intent = Intent {
            op: Some("clone_record".into()),
            source_key: Some(12345),
            new_key: Some(999001),
            patches: Some(vec![Patch {
                path: "item_name.default".into(),
                op: None,
                new: json!("Custom"),
            }]),
            ..Default::default()
        };
        match intent.resolve_op().unwrap() {
            ResolvedIntentOp::CloneRecord { source_key, new_key, patches } => {
                assert_eq!(source_key, 12345);
                assert_eq!(new_key, 999001);
                assert_eq!(patches.len(), 1);
                assert_eq!(patches[0].path, "item_name.default");
            }
            other => panic!("expected CloneRecord, got {:?}", other),
        }
    }

    #[test]
    fn resolve_new_record_accepts_legacy_add_entry() {
        let intent = Intent {
            op: Some("add_entry".into()),
            key: Some(999001),
            data: Some(json!({ "key": 999001, "string_key": "Custom" })),
            ..Default::default()
        };
        match intent.resolve_op().unwrap() {
            ResolvedIntentOp::NewRecord { new_key, .. } => assert_eq!(new_key, 999001),
            other => panic!("expected NewRecord, got {:?}", other),
        }
    }

    #[test]
    fn resolve_unknown_op_errors() {
        let intent = Intent {
            op: Some("burninate".into()),
            ..Default::default()
        };
        assert!(intent.resolve_op().is_err());
    }

    #[test]
    fn paloc_indices_match_formula() {
        let (name_idx, desc_idx) = item_paloc_indices(999001);
        assert_eq!(name_idx, (999001u64 << 32) | 0x70);
        assert_eq!(desc_idx, (999001u64 << 32) | 0x71);
        assert_eq!(desc_idx - name_idx, 1);
    }

    #[test]
    fn from_slice_round_trip() {
        let bytes = br#"{"format":3,"target":"iteminfo.pabgb","intents":[{"key":1,"field":"cooltime","new":1}]}"#;
        let doc = IntentDoc::from_slice(bytes).unwrap();
        assert!(doc.is_field_json_v3());
    }

    #[test]
    fn detection_rejects_empty_intents() {
        let v = json!({ "format": 3, "target": "iteminfo.pabgb", "intents": [] });
        let doc = IntentDoc::from_value(&v).unwrap();
        assert!(!doc.is_field_json_v3());
    }
}
