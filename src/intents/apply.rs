// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Apply Field-JSON v3.x intents against a parsed table's JSON record list.
//!
//! Today's surface focuses on iteminfo (sequential — no pabgh sister
//! file) and the four custom-item operations: `set`, `clone_record`,
//! `new_record`, `delete_record`. Storeinfo `array_append` is also
//! supported. Generalizing to every pabgh-bounded table requires the
//! pabgh-rebuild path, tracked under task #11.
//!
//! Apply pipeline:
//!
//! ```ignore
//! use dmm_parser::item_info::{parse_iteminfo_to_json, serialize_iteminfo_from_json};
//! use dmm_parser::intents::{IntentDoc, apply_resolved_intents};
//!
//! let body: Vec<u8> = std::fs::read("iteminfo.pabgb")?;
//! let mut records = parse_iteminfo_to_json(&body)?;
//!
//! let doc: IntentDoc = serde_json::from_slice(&manifest_bytes)?;
//! for (target, intents) in doc.flatten_targets() {
//!     if target == "iteminfo.pabgb" {
//!         apply_resolved_intents(&mut records, &intents)?;
//!     }
//! }
//!
//! let new_body = serialize_iteminfo_from_json(&records)?;
//! ```

use std::io;

use serde_json::Value;

use super::path::{array_append_at_path, set_value_at_path, PathError};
use super::types::{Intent, IntentResolveError, Patch, ResolvedIntentOp};

#[derive(Debug)]
pub enum ApplyError {
    /// The intent referenced a record (by `entry` string_key or numeric
    /// `key`) that doesn't exist in the parsed list.
    RecordNotFound { lookup: String },
    /// `clone_record` / `new_record` tried to create a record under a
    /// key that's already in use.
    DuplicateKey { key: i64 },
    /// Failed to resolve a JSON field path.
    Path(PathError),
    /// The intent's shape was incomplete or used an unknown op.
    Resolve(IntentResolveError),
    /// A record was missing the canonical `key` field — can't index it.
    MissingKeyField,
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::RecordNotFound { lookup } => {
                write!(f, "record not found: {}", lookup)
            }
            ApplyError::DuplicateKey { key } => {
                write!(f, "duplicate key {} — clone/new would overwrite an existing record", key)
            }
            ApplyError::Path(e) => write!(f, "{}", e),
            ApplyError::Resolve(e) => write!(f, "{}", e),
            ApplyError::MissingKeyField => write!(f, "record has no `key` field — cannot dispatch"),
        }
    }
}

impl std::error::Error for ApplyError {}

impl From<PathError> for ApplyError {
    fn from(e: PathError) -> Self { ApplyError::Path(e) }
}
impl From<IntentResolveError> for ApplyError {
    fn from(e: IntentResolveError) -> Self { ApplyError::Resolve(e) }
}

/// Per-intent outcome — one entry per intent in the input list.
#[derive(Debug, Clone)]
pub struct ApplyOutcome {
    pub op: String,
    pub status: ApplyStatus,
}

#[derive(Debug, Clone)]
pub enum ApplyStatus {
    Applied,
    Skipped(String),
}

/// Apply a list of JSON-faithful intents to `records` in-order. Collects
/// per-intent outcomes; aborts on the first hard error (`Path`,
/// `DuplicateKey`, etc.). Soft misses — a `set` against a non-existent
/// entry, for instance — are recorded as `Skipped` per the v3 spec
/// (which says "SKIP: entry '%s' not found in current game data").
pub fn apply_resolved_intents(
    records: &mut Vec<Value>,
    intents: &[Intent],
) -> Result<Vec<ApplyOutcome>, ApplyError> {
    let mut out = Vec::with_capacity(intents.len());
    for intent in intents {
        let resolved = intent.resolve_op()?;
        let op_name = op_name(&resolved);
        match apply_single(records, &resolved)? {
            Some(reason) => out.push(ApplyOutcome {
                op: op_name,
                status: ApplyStatus::Skipped(reason),
            }),
            None => out.push(ApplyOutcome {
                op: op_name,
                status: ApplyStatus::Applied,
            }),
        }
    }
    Ok(out)
}

fn op_name(op: &ResolvedIntentOp) -> String {
    match op {
        ResolvedIntentOp::Set { .. } => "set".into(),
        ResolvedIntentOp::ArrayAppend { .. } => "array_append".into(),
        ResolvedIntentOp::CloneRecord { .. } => "clone_record".into(),
        ResolvedIntentOp::NewRecord { .. } => "new_record".into(),
        ResolvedIntentOp::DeleteRecord { .. } => "delete_record".into(),
    }
}

/// Returns `Some(reason)` for soft skips, `None` on success, or an
/// `Err(ApplyError)` for hard failures.
fn apply_single(
    records: &mut Vec<Value>,
    op: &ResolvedIntentOp,
) -> Result<Option<String>, ApplyError> {
    match op {
        ResolvedIntentOp::Set { entry, key, field, new } => {
            let Some(idx) = find_record_index(records, entry.as_deref(), *key) else {
                return Ok(Some(format!(
                    "set: record '{}' / key {:?} not found",
                    entry.as_deref().unwrap_or(""),
                    key
                )));
            };
            if let Some(reason) = blob_fallback_skip("set", &records[idx], entry.as_deref(), *key, field) {
                return Ok(Some(reason));
            }
            set_value_at_path(&mut records[idx], field, new.clone())?;
            Ok(None)
        }
        ResolvedIntentOp::ArrayAppend { entry, key, field, value } => {
            let Some(idx) = find_record_index(records, entry.as_deref(), *key) else {
                return Ok(Some(format!(
                    "array_append: record '{}' / key {:?} not found",
                    entry.as_deref().unwrap_or(""),
                    key
                )));
            };
            if let Some(reason) = blob_fallback_skip("array_append", &records[idx], entry.as_deref(), *key, field) {
                return Ok(Some(reason));
            }
            array_append_at_path(&mut records[idx], field, value.clone())?;
            Ok(None)
        }
        ResolvedIntentOp::CloneRecord { source_key, new_key, patches } => {
            if find_record_index(records, None, Some(*new_key)).is_some() {
                return Err(ApplyError::DuplicateKey { key: *new_key });
            }
            let src_idx = find_record_index(records, None, Some(*source_key)).ok_or(
                ApplyError::RecordNotFound {
                    lookup: format!("source_key={}", source_key),
                },
            )?;
            let mut clone = records[src_idx].clone();
            // Set the new key on the clone, then run patches.
            set_record_key(&mut clone, *new_key)?;
            apply_patches(&mut clone, patches)?;
            records.push(clone);
            Ok(None)
        }
        ResolvedIntentOp::NewRecord { new_key, template } => {
            if find_record_index(records, None, Some(*new_key)).is_some() {
                return Err(ApplyError::DuplicateKey { key: *new_key });
            }
            let mut new = template.clone();
            set_record_key(&mut new, *new_key)?;
            records.push(new);
            Ok(None)
        }
        ResolvedIntentOp::DeleteRecord { key } => {
            let Some(idx) = find_record_index(records, None, Some(*key)) else {
                return Ok(Some(format!("delete_record: key {} not found", key)));
            };
            records.remove(idx);
            Ok(None)
        }
    }
}

/// Guard against the silent-drop failure mode: when a typed table entry's
/// typed parse fails, the runtime falls back to an opaque `_blob_b64` blob
/// (marked `_blob_fallback: true`). Serialization writes that blob verbatim,
/// so ANY typed-field edit on it is a no-op — yet `set_value_at_path` would
/// happily insert a phantom JSON key and report success. That is exactly how
/// I Like Space's `default_slot_count` / `max_slot_count` edits silently
/// vanished on the live 1.09 inventory table before the GameCondition tag-402
/// parser drift was repaired.
///
/// Returns `Some(reason)` when the edit must be reported as `Skipped`: the
/// target is a blob-fallback entry and the field is anything other than a
/// whole-blob replacement (`_blob_b64`). This surfaces parser drift in the
/// mount log instead of pretending the edit landed.
fn blob_fallback_skip(
    op: &str,
    record: &Value,
    entry: Option<&str>,
    key: Option<i64>,
    field: &str,
) -> Option<String> {
    let is_fallback = record
        .get("_blob_fallback")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if is_fallback && field != "_blob_b64" {
        return Some(format!(
            "{}: entry '{}' / key {:?} fell to blob-fallback (typed parse failed — \
             parser likely out of date for this game build); field '{}' edit NOT applied",
            op,
            entry.unwrap_or(""),
            key,
            field
        ));
    }
    None
}

/// Apply each `Patch` to `record`. Defaults to `set` per
/// [`crate::intents::types::Patch`].
fn apply_patches(record: &mut Value, patches: &[Patch]) -> Result<(), ApplyError> {
    for p in patches {
        let op = p.op.as_deref().unwrap_or("set");
        match op {
            "set" => {
                set_value_at_path(record, &p.path, p.new.clone())?;
            }
            "array_append" => {
                array_append_at_path(record, &p.path, p.new.clone())?;
            }
            other => {
                return Err(ApplyError::Resolve(IntentResolveError::UnknownOp(
                    format!("patch.op={}", other),
                )));
            }
        }
    }
    Ok(())
}

/// Locate a record by `string_key` (preferred) or numeric `key`
/// (fallback). v3 spec: try entry-name first, fall back to key, return
/// nothing if neither matches.
pub fn find_record_index(
    records: &[Value],
    entry: Option<&str>,
    key: Option<i64>,
) -> Option<usize> {
    if let Some(entry_name) = entry.filter(|s| !s.is_empty()) {
        for (i, r) in records.iter().enumerate() {
            if let Some(sk) = r.get("string_key").and_then(|v| v.as_str())
                && sk == entry_name
            {
                return Some(i);
            }
        }
    }
    if let Some(k) = key {
        for (i, r) in records.iter().enumerate() {
            if record_key_matches(r, k) {
                return Some(i);
            }
        }
    }
    None
}

/// Read the canonical `key` field, accepting both integer and stringified
/// representations (different tables expose it as i64 / u64 / nested
/// `key.value`). Returns true if `record["key"] == k` by value, regardless
/// of which numeric subtype the JSON used.
fn record_key_matches(record: &Value, k: i64) -> bool {
    let Some(key_val) = lookup_record_key(record) else {
        return false;
    };
    if let Some(n) = key_val.as_i64() {
        return n == k;
    }
    if let Some(n) = key_val.as_u64() {
        return n as i64 == k;
    }
    if let Some(n) = key_val.as_f64() {
        return (n as i64) == k && n.fract() == 0.0;
    }
    false
}

/// Get the record's identifying key value. Most tables expose `key`
/// directly as a number; some (iteminfo) wrap it in `{"value": N}`
/// because `ItemKey` is a typed wrapper. We accept either shape.
fn lookup_record_key(record: &Value) -> Option<&Value> {
    let key = record.get("key")?;
    // Iteminfo's ItemKey serializes as `{ "value": N }`. Unwrap.
    if let Some(inner) = key.get("value")
        && key.is_object()
    {
        return Some(inner);
    }
    Some(key)
}

/// Set the record's key to `new_key`, preserving the existing JSON shape
/// (scalar number vs. `{"value": N}`).
fn set_record_key(record: &mut Value, new_key: i64) -> Result<(), ApplyError> {
    let obj = record.as_object_mut().ok_or(ApplyError::MissingKeyField)?;
    let existing = obj.get("key").ok_or(ApplyError::MissingKeyField)?;
    if existing.is_object() && existing.get("value").is_some() {
        obj.insert("key".into(), serde_json::json!({ "value": new_key }));
    } else {
        obj.insert("key".into(), serde_json::json!(new_key));
    }
    Ok(())
}

/// Apply intents end-to-end against an iteminfo.pabgb body. Iteminfo is
/// **sequential** — entry boundaries are encoded in the body itself, so
/// no pabgh sister file is needed.
///
/// Pipeline: `parse_iteminfo_to_json` → `apply_resolved_intents` →
/// `serialize_iteminfo_from_json`. Round-trips byte-perfect when no
/// intent mutates the records (verified by the existing
/// `test_full_roundtrip` test).
pub fn apply_intents_to_iteminfo(
    body: &[u8],
    intents: &[Intent],
) -> io::Result<(Vec<u8>, Vec<ApplyOutcome>)> {
    use crate::item_info::{parse_iteminfo_to_json, serialize_iteminfo_from_json};

    let mut records = parse_iteminfo_to_json(body)?;

    // Normalize v3.1 _camelCase field paths to rust snake_case BEFORE apply.
    let intents: Vec<Intent> = intents.iter().map(|i| {
        let mut clone = i.clone();
        normalize_intent_v3_1(&mut clone, "iteminfo");
        clone
    }).collect();

    let outcomes = apply_resolved_intents(&mut records, &intents).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("iteminfo apply: {}", e))
    })?;
    let new_body = serialize_iteminfo_from_json(&records)?;
    Ok((new_body, outcomes))
}

/// Normalize a single intent so that v3.1 PA-canonical `_camelCase` field
/// paths are translated to rust snake_case (the form the apply pipeline's
/// `set_value_at_path` walks against). Mutates the intent in place.
///
/// Translates BOTH `intent.field` AND every `patch.path` for record-level
/// ops (clone_record, new_record patches, etc.). Sub-segments (after `.`
/// or `[`) are preserved verbatim — only the LEADING field name maps to
/// the alias table, since aliases are top-level field renames only.
///
/// Callers that bypass `apply_intents_to_iteminfo` (DMM-BETA's
/// `dispatch_record_level_op`, Crimson Forge, third-party tooling) MUST
/// call this helper before forwarding intents to `apply_resolved_intents`,
/// otherwise v3.1-named intents silently insert junk keys instead of
/// updating the targeted field. The bug ΛεroωynX hit on 2026-05-11.
///
/// No-op if the table has no registered alias table (returns silently —
/// a non-v3.1 table just passes through).
pub fn normalize_intent_v3_1(intent: &mut Intent, table_name: &str) {
    let aliases = match crate::json_shape::lookup_table_aliases_v3_1(table_name) {
        Some(a) if !a.is_empty() => a,
        _ => return,
    };

    if let Some(field) = &intent.field {
        intent.field = Some(translate_v3_1_path_to_snake(field, aliases));
    }
    if let Some(patches) = &mut intent.patches {
        for p in patches.iter_mut() {
            p.path = translate_v3_1_path_to_snake(&p.path, aliases);
        }
    }
}

/// Translate the LEADING field name of a path from v3.1 _camelCase to
/// rust snake_case using the table's alias table. Only the FIRST segment
/// is rewritten — sub-struct fields keep their snake form (which is what
/// the iteminfo struct definition uses internally).
///
/// Examples (with iteminfo's alias table):
///   "_prefabDataList"             -> "prefab_data_list"
///   "_prefabDataList[0]"          -> "prefab_data_list[0]"
///   "_prefabDataList[0].something" -> "prefab_data_list[0].something"
///   "prefab_data_list"            -> "prefab_data_list"  (unchanged)
fn translate_v3_1_path_to_snake(path: &str, aliases: &[(&str, &str)]) -> String {
    // Split off the first path segment (everything before the first '.' or '[').
    let head_end = path.find(|c: char| c == '.' || c == '[').unwrap_or(path.len());
    let head = &path[..head_end];
    let rest = &path[head_end..];

    for &(snake, camel) in aliases {
        if head == camel {
            return format!("{}{}", snake, rest);
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intents::types::Patch;
    use serde_json::json;

    fn fake_records() -> Vec<Value> {
        vec![
            json!({
                "key": 100,
                "string_key": "Sword_Of_Vanilla",
                "cooltime": 30,
                "max_stack_count": 1,
            }),
            json!({
                "key": 200,
                "string_key": "Apple",
                "cooltime": 0,
                "max_stack_count": 99,
            }),
        ]
    }

    #[test]
    fn v3_1_path_translation() {
        // Mock alias table for testing.
        let aliases: &[(&str, &str)] = &[
            ("prefab_data_list", "_prefabDataList"),
            ("max_stack_count", "_maxStackCount"),
        ];
        // _camelCase head → snake_case head, rest preserved.
        assert_eq!(translate_v3_1_path_to_snake("_prefabDataList", aliases),
                   "prefab_data_list");
        assert_eq!(translate_v3_1_path_to_snake("_prefabDataList[0]", aliases),
                   "prefab_data_list[0]");
        assert_eq!(translate_v3_1_path_to_snake("_prefabDataList[0].sub_field", aliases),
                   "prefab_data_list[0].sub_field");
        assert_eq!(translate_v3_1_path_to_snake("_prefabDataList.sub", aliases),
                   "prefab_data_list.sub");
        // snake form passes through unchanged.
        assert_eq!(translate_v3_1_path_to_snake("prefab_data_list", aliases),
                   "prefab_data_list");
        assert_eq!(translate_v3_1_path_to_snake("prefab_data_list[0]", aliases),
                   "prefab_data_list[0]");
        // Unknown name: pass through (caller handles "field not found").
        assert_eq!(translate_v3_1_path_to_snake("_unknownField", aliases),
                   "_unknownField");
    }

    #[test]
    fn iteminfo_aliases_are_wired() {
        // Regression: ensure the iteminfo alias table is registered in
        // TABLE_FIELD_ALIASES_V3_1 and reachable from apply_intents_to_iteminfo.
        let aliases = crate::json_shape::lookup_table_aliases_v3_1("iteminfo");
        assert!(aliases.is_some(), "iteminfo not in TABLE_FIELD_ALIASES_V3_1");
        let aliases = aliases.unwrap();
        assert!(!aliases.is_empty(), "iteminfo alias table is empty");
        // Sanity: prefab_data_list should map to _prefabDataList per
        // PA naming convention (the field used by transmog mods).
        let has_prefab = aliases.iter().any(|(s, _)| *s == "prefab_data_list");
        assert!(has_prefab, "prefab_data_list missing from iteminfo aliases");
    }

    #[test]
    fn set_by_entry_name() {
        let mut records = fake_records();
        let intent = Intent {
            entry: Some("Sword_Of_Vanilla".into()),
            field: Some("cooltime".into()),
            new: Some(json!(1)),
            ..Default::default()
        };
        let outcomes = apply_resolved_intents(&mut records, &[intent]).unwrap();
        assert!(matches!(outcomes[0].status, ApplyStatus::Applied));
        assert_eq!(records[0]["cooltime"], 1);
    }

    #[test]
    fn set_by_key_fallback() {
        let mut records = fake_records();
        let intent = Intent {
            entry: Some("Wrong_Name".into()),
            key: Some(200),
            field: Some("cooltime".into()),
            new: Some(json!(99)),
            ..Default::default()
        };
        apply_resolved_intents(&mut records, &[intent]).unwrap();
        assert_eq!(records[1]["cooltime"], 99);
    }

    #[test]
    fn set_missing_record_is_skipped() {
        let mut records = fake_records();
        let intent = Intent {
            entry: Some("Does_Not_Exist".into()),
            field: Some("cooltime".into()),
            new: Some(json!(1)),
            ..Default::default()
        };
        let outcomes = apply_resolved_intents(&mut records, &[intent]).unwrap();
        assert!(matches!(outcomes[0].status, ApplyStatus::Skipped(_)));
        // Records unchanged.
        assert_eq!(records[0]["cooltime"], 30);
    }

    #[test]
    fn set_on_blob_fallback_entry_is_skipped_not_silently_dropped() {
        // Regression: a typed table entry whose typed parse failed becomes a
        // `_blob_fallback` blob. A field `set` on it serializes to a no-op
        // (the raw blob is written verbatim), but the old code reported it as
        // Applied — exactly how I Like Space's slot-count edits vanished on
        // the live 1.09 inventory table. The guard must report Skipped.
        let mut records = vec![json!({
            "key": 2,
            "_blob_b64": "AAAA",
            "_blob_fallback": true,
        })];
        let intent = Intent {
            entry: None,
            key: Some(2),
            field: Some("default_slot_count".into()),
            new: Some(json!(700)),
            ..Default::default()
        };
        let outcomes = apply_resolved_intents(&mut records, &[intent]).unwrap();
        match &outcomes[0].status {
            ApplyStatus::Skipped(reason) => assert!(
                reason.contains("blob-fallback"),
                "skip reason should mention blob-fallback, got: {reason}"
            ),
            ApplyStatus::Applied => panic!("blob-fallback field edit must NOT report Applied"),
        }
        // The phantom field must NOT have been inserted.
        assert!(records[0].get("default_slot_count").is_none());
    }

    #[test]
    fn clone_record_creates_new_with_patches() {
        let mut records = fake_records();
        let intent = Intent {
            op: Some("clone_record".into()),
            source_key: Some(100),
            new_key: Some(999001),
            patches: Some(vec![
                Patch { path: "string_key".into(), op: None, new: json!("Custom_Sword") },
                Patch { path: "cooltime".into(), op: None, new: json!(1) },
                Patch { path: "max_stack_count".into(), op: None, new: json!(999) },
            ]),
            ..Default::default()
        };
        apply_resolved_intents(&mut records, &[intent]).unwrap();
        assert_eq!(records.len(), 3, "clone added a new record");
        let cloned = &records[2];
        assert_eq!(cloned["key"], 999001);
        assert_eq!(cloned["string_key"], "Custom_Sword");
        assert_eq!(cloned["cooltime"], 1);
        assert_eq!(cloned["max_stack_count"], 999);
        // Source unchanged.
        let src = &records[0];
        assert_eq!(src["key"], 100);
        assert_eq!(src["cooltime"], 30);
    }

    #[test]
    fn clone_record_rejects_duplicate_key() {
        let mut records = fake_records();
        let intent = Intent {
            op: Some("clone_record".into()),
            source_key: Some(100),
            new_key: Some(200),  // already taken
            ..Default::default()
        };
        let err = apply_resolved_intents(&mut records, &[intent]).unwrap_err();
        assert!(matches!(err, ApplyError::DuplicateKey { .. }));
    }

    #[test]
    fn clone_record_missing_source() {
        let mut records = fake_records();
        let intent = Intent {
            op: Some("clone_record".into()),
            source_key: Some(99999),
            new_key: Some(999001),
            ..Default::default()
        };
        let err = apply_resolved_intents(&mut records, &[intent]).unwrap_err();
        assert!(matches!(err, ApplyError::RecordNotFound { .. }));
    }

    #[test]
    fn new_record_inserts_template() {
        let mut records = fake_records();
        let intent = Intent {
            op: Some("new_record".into()),
            new_key: Some(999500),
            template: Some(json!({
                "key": 0,
                "string_key": "From_Scratch",
                "cooltime": 5,
                "max_stack_count": 1,
            })),
            ..Default::default()
        };
        apply_resolved_intents(&mut records, &[intent]).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[2]["key"], 999500);
        assert_eq!(records[2]["string_key"], "From_Scratch");
    }

    #[test]
    fn delete_record_removes_by_key() {
        let mut records = fake_records();
        let intent = Intent {
            op: Some("delete_record".into()),
            key: Some(200),
            ..Default::default()
        };
        apply_resolved_intents(&mut records, &[intent]).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["key"], 100);
    }

    #[test]
    fn delete_record_missing_is_skipped() {
        let mut records = fake_records();
        let intent = Intent {
            op: Some("delete_record".into()),
            key: Some(99999),
            ..Default::default()
        };
        let outcomes = apply_resolved_intents(&mut records, &[intent]).unwrap();
        assert!(matches!(outcomes[0].status, ApplyStatus::Skipped(_)));
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn array_append_extends_list() {
        let mut records = vec![json!({
            "key": 1,
            "string_key": "Vendor_A",
            "items": [
                { "item_key": 100, "price": 10 },
                { "item_key": 101, "price": 20 },
            ],
        })];
        let intent = Intent {
            op: Some("array_append".into()),
            key: Some(1),
            field: Some("items".into()),
            value: Some(json!({ "item_key": 999001, "price": 9999 })),
            ..Default::default()
        };
        apply_resolved_intents(&mut records, &[intent]).unwrap();
        let items = records[0]["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[2]["item_key"], 999001);
    }

    /// End-to-end on a real iteminfo.pabgb fixture if available. Parses,
    /// clones a known item under a brand-new key, mutates a few fields
    /// via patches, serializes, re-parses, and asserts the new entry is
    /// present. Skips when no fixture is available so CI without the
    /// game data stays green.
    #[test]
    fn apply_intents_to_iteminfo_clone_record_real_fixture() {
        use crate::intents::apply_intents_to_iteminfo;
        use crate::item_info::parse_iteminfo_to_json;

        // Reuse the fixture lookup convention from lib.rs's tests.
        let candidates = [
            std::env::var("DMM_PARSER_ITEMINFO_PATH").ok(),
            Some("/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/iteminfo.pabgb".into()),
            Some(r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-1\iteminfo.pabgb".into()),
        ];
        let body = candidates
            .iter()
            .flatten()
            .find_map(|p| std::fs::read(p).ok());
        let Some(body) = body else {
            eprintln!("SKIP apply_intents_to_iteminfo_clone_record_real_fixture: no fixture");
            return;
        };

        let original_records = parse_iteminfo_to_json(&body).expect("parse iteminfo");
        assert!(!original_records.is_empty(), "iteminfo fixture is empty?");

        // Pick the first record's key as the donor. Iteminfo wraps `key`
        // as `{"value": N}`, so unwrap.
        let donor = &original_records[0];
        let donor_key = donor
            .get("key")
            .and_then(|k| k.get("value").or(Some(k)))
            .and_then(|v| v.as_i64())
            .expect("donor key");
        let donor_string_key = donor
            .get("string_key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Pick a new_key that isn't currently in use. Walk records and
        // bump the candidate until it's unique.
        let used: std::collections::HashSet<i64> = original_records
            .iter()
            .filter_map(|r| {
                r.get("key")
                    .and_then(|k| k.get("value").or(Some(k)))
                    .and_then(|v| v.as_i64())
            })
            .collect();
        let mut new_key: i64 = 999_001;
        while used.contains(&new_key) {
            new_key += 1;
        }

        let intent = Intent {
            op: Some("clone_record".into()),
            source_key: Some(donor_key),
            new_key: Some(new_key),
            patches: Some(vec![Patch {
                path: "string_key".into(),
                op: None,
                new: json!(format!("Custom_Clone_{}", new_key)),
            }]),
            ..Default::default()
        };

        let (new_body, outcomes) = apply_intents_to_iteminfo(&body, &[intent]).expect("apply");
        assert_eq!(outcomes.len(), 1);
        assert!(matches!(outcomes[0].status, ApplyStatus::Applied));

        // Re-parse the new body and verify the cloned entry appears.
        let new_records = parse_iteminfo_to_json(&new_body).expect("re-parse");
        assert_eq!(
            new_records.len(),
            original_records.len() + 1,
            "expected one additional record"
        );
        let cloned = new_records.last().expect("clone is appended at end");
        let cloned_key = cloned
            .get("key")
            .and_then(|k| k.get("value").or(Some(k)))
            .and_then(|v| v.as_i64());
        assert_eq!(cloned_key, Some(new_key));
        assert_eq!(
            cloned.get("string_key").and_then(|v| v.as_str()),
            Some(format!("Custom_Clone_{}", new_key).as_str()),
        );
        // Donor untouched.
        let donor_after = &new_records[0];
        let donor_key_after = donor_after
            .get("key")
            .and_then(|k| k.get("value").or(Some(k)))
            .and_then(|v| v.as_i64());
        assert_eq!(donor_key_after, Some(donor_key));
        assert_eq!(
            donor_after.get("string_key").and_then(|v| v.as_str()),
            Some(donor_string_key.as_str()),
        );
    }

    /// Round-trip preservation: applying an empty intent list should
    /// produce byte-identical output to the input. Strong sanity check
    /// that the parse → serialize chain doesn't lose data.
    #[test]
    fn apply_empty_intents_is_byte_perfect() {
        use crate::intents::apply_intents_to_iteminfo;

        let candidates = [
            std::env::var("DMM_PARSER_ITEMINFO_PATH").ok(),
            Some("/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/iteminfo.pabgb".into()),
            Some(r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-1\iteminfo.pabgb".into()),
        ];
        let body = candidates
            .iter()
            .flatten()
            .find_map(|p| std::fs::read(p).ok());
        let Some(body) = body else {
            eprintln!("SKIP apply_empty_intents_is_byte_perfect: no fixture");
            return;
        };
        let (new_body, outcomes) = apply_intents_to_iteminfo(&body, &[]).expect("apply");
        assert!(outcomes.is_empty());
        assert_eq!(new_body.len(), body.len(), "size diverged");
        assert_eq!(new_body, body, "bytes diverged");
    }

    #[test]
    fn iteminfo_keyed_value_object_shape() {
        // Iteminfo serializes its ItemKey as `{"value": N}` rather than a
        // scalar. Verify lookup + clone preserve that shape.
        let mut records = vec![
            json!({
                "key": { "value": 100 },
                "string_key": "Sword",
                "cooltime": 30,
            }),
        ];
        let intent = Intent {
            op: Some("clone_record".into()),
            source_key: Some(100),
            new_key: Some(999001),
            patches: Some(vec![Patch {
                path: "cooltime".into(),
                op: None,
                new: json!(1),
            }]),
            ..Default::default()
        };
        apply_resolved_intents(&mut records, &[intent]).unwrap();
        assert_eq!(records.len(), 2);
        // Clone preserves the {"value": N} wrapper shape.
        assert_eq!(records[1]["key"], json!({ "value": 999001 }));
        assert_eq!(records[1]["cooltime"], 1);
    }
}
