// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser

//! Dual-shape JSON output for tier-0 / tier-1 mod-author surfaces.
//!
//! Tier 0 (canonical / "v3.1") uses real C++ field names recovered from the
//! game binary via IDA. Tier 1 (legacy / "v3") uses the placeholder
//! `_unkXXXX` names that shipped before each rename was confirmed.
//!
//! The Rust struct fields use the **canonical** names internally — the
//! `_unkXXXX` aliases are projected on output (when `JsonShape::V3` is
//! requested) and accepted on input (always, regardless of shape).
//!
//! This lets a single dmm-parser binary serve two consumers in parallel:
//!
//! * **DMM v3** — calls the parser without specifying a shape, gets the
//!   legacy `_unkXXXX` names verbatim. Existing mods round-trip
//!   identically.
//! * **DMM v2.0.0-beta** (and any future v3.1 consumer) — calls with
//!   `shape=V3_1` and receives canonical real-C++ names. Mods authored
//!   against the new names are forward-stable across all future
//!   field-naming work.
//!
//! ## Per-table alias tables
//!
//! Each binary table that has any `_unkXXXX` placeholders maintains a
//! `FIELD_ALIASES_V3` constant of `(canonical_name, v3_legacy_name)`
//! pairs. The constant lives next to the struct it describes (see
//! `src/binary/paatt_basedata.rs` for the canonical example).
//!
//! When `apply_v3_aliases` is called, every key in the JSON map that
//! matches a `canonical_name` entry gets renamed to its `v3_legacy_name`
//! counterpart. When `normalize_input_aliases` is called, the reverse
//! happens — any legacy key gets renamed to its canonical form so the
//! struct's typed read path (which uses canonical names) finds it.
//!
//! ## Initial state
//!
//! Every table's alias constant is currently empty `&[]`. The mechanism
//! is a no-op until the first canonical rename ships. That keeps this
//! commit a pure-scaffolding change with zero behavior diff for DMM v3.

use serde_json::{Map, Value};

/// Which JSON shape the consumer wants on output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonShape {
    /// Legacy DMM v3 names — `_unkXXXX` placeholders preserved for any
    /// field that hasn't been renamed yet. Default.
    V3,
    /// Canonical Tier 0 names from the game's C++ symbols. The "v3.1"
    /// surface for any future mod-manager build.
    V3_1,
}

impl JsonShape {
    /// Parse a shape string (`"v3"` or `"v3.1"`) into the enum. Used by the
    /// Python bindings so callers can pass strings rather than enum
    /// integer values.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "v3" | "V3" | "" => Ok(JsonShape::V3),
            "v3.1" | "V3.1" | "v3_1" | "V3_1" => Ok(JsonShape::V3_1),
            other => Err(format!(
                "unknown JsonShape `{}` — expected `v3` or `v3.1`",
                other
            )),
        }
    }
}

/// `(canonical_name, v3_legacy_name)` pairs for a single binary struct.
/// Empty for any struct where every field is already on its canonical name.
pub type FieldAliasTable = &'static [(&'static str, &'static str)];

/// Rewrite a JSON object's keys from canonical (T0) to v3 legacy names.
/// Used by `to_json_value_shaped(JsonShape::V3)` after the struct emits
/// its canonical-named map. No-op when `aliases` is empty.
pub fn apply_v3_aliases(map: &mut Map<String, Value>, aliases: FieldAliasTable) {
    for &(canonical, v3_legacy) in aliases {
        if let Some(v) = map.remove(canonical) {
            map.insert(v3_legacy.to_string(), v);
        }
    }
}

/// Look up a table's v3.1 alias table by dispatch-name. Returns `None` if
/// the table isn't indexed (e.g. tables with no extractable fields, or
/// table names that the v3.1 generator skipped). Callers should treat
/// `None` and `Some(&[])` identically — both are "no rename".
pub fn lookup_table_aliases_v3_1(
    table_name: &str,
) -> Option<FieldAliasTable> {
    crate::json_shape_table_registry::TABLE_FIELD_ALIASES_V3_1
        .iter()
        .find(|(name, _)| *name == table_name)
        .map(|(_, aliases)| *aliases)
}

/// Rewrite a JSON object's keys from Rust snake_case to Pearl Abyss
/// canonical `_camelCase` (the v3.1 surface). The `aliases` table here
/// uses `(rust_snake, _camelCase)` pairs — opposite semantics from
/// `apply_v3_aliases`, which uses `(canonical, v3_legacy_name)`.
///
/// v3 (snake_case) is the default Rust struct emit. v3.1 callers project
/// each key forward to its canonical C++ identifier.
pub fn apply_v3_1_aliases(map: &mut Map<String, Value>, aliases: FieldAliasTable) {
    for &(rust_snake, camel) in aliases {
        if let Some(v) = map.remove(rust_snake) {
            map.insert(camel.to_string(), v);
        }
    }
}

/// Accept either rust_snake_case or _camelCase on input, normalize to
/// rust_snake_case (the form the typed read path expects). Used at the
/// entry point of `serialize_table_from_json_shaped` so v3.1 mod authors
/// can submit `_skeletonName` and v3 mod authors can submit `skeleton_name`
/// against the same parser.
pub fn normalize_input_aliases_v3_1(value: &mut Value, aliases: FieldAliasTable) {
    if let Value::Object(map) = value {
        for &(rust_snake, camel) in aliases {
            if map.contains_key(rust_snake) {
                continue; // caller used the snake form, leave alone
            }
            if let Some(v) = map.remove(camel) {
                map.insert(rust_snake.to_string(), v);
            }
        }
    }
}

/// Rewrite a JSON object's keys from v3 legacy to canonical (T0) names so
/// the struct's typed read path finds them. Called by `write_from_json`
/// at the entry point. Both names continue to round-trip identically;
/// this just rebrands incoming legacy keys to the canonical form before
/// per-field reads happen.
pub fn normalize_input_aliases(value: &mut Value, aliases: FieldAliasTable) {
    if let Value::Object(map) = value {
        for &(canonical, v3_legacy) in aliases {
            // Skip if the canonical key is already present — caller used
            // the new name, leave it alone.
            if map.contains_key(canonical) {
                continue;
            }
            if let Some(v) = map.remove(v3_legacy) {
                map.insert(canonical.to_string(), v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_shape_strings() {
        assert_eq!(JsonShape::from_str("v3").unwrap(), JsonShape::V3);
        assert_eq!(JsonShape::from_str("V3").unwrap(), JsonShape::V3);
        assert_eq!(JsonShape::from_str("").unwrap(), JsonShape::V3);
        assert_eq!(JsonShape::from_str("v3.1").unwrap(), JsonShape::V3_1);
        assert_eq!(JsonShape::from_str("V3.1").unwrap(), JsonShape::V3_1);
        assert_eq!(JsonShape::from_str("v3_1").unwrap(), JsonShape::V3_1);
        assert!(JsonShape::from_str("v4").is_err());
    }

    #[test]
    fn empty_alias_table_is_noop_on_output() {
        let mut v = json!({"foo": 1, "_unk0073": 2});
        let m = v.as_object_mut().unwrap();
        apply_v3_aliases(m, &[]);
        assert_eq!(m.get("foo"), Some(&json!(1)));
        assert_eq!(m.get("_unk0073"), Some(&json!(2)));
    }

    #[test]
    fn empty_alias_table_is_noop_on_input() {
        let mut v = json!({"foo": 1, "_unk0073": 2});
        normalize_input_aliases(&mut v, &[]);
        let m = v.as_object().unwrap();
        assert_eq!(m.get("foo"), Some(&json!(1)));
        assert_eq!(m.get("_unk0073"), Some(&json!(2)));
    }

    #[test]
    fn output_renames_canonical_to_v3_legacy() {
        let aliases: FieldAliasTable = &[("attack_impulse_level", "_unk0073")];
        let mut v = json!({"attack_impulse_level": 1, "weapon_key": 0xdead});
        apply_v3_aliases(v.as_object_mut().unwrap(), aliases);
        let m = v.as_object().unwrap();
        assert_eq!(m.get("_unk0073"), Some(&json!(1)));
        assert!(!m.contains_key("attack_impulse_level"));
        assert_eq!(m.get("weapon_key"), Some(&json!(0xdead)));
    }

    #[test]
    fn input_normalize_renames_v3_legacy_to_canonical() {
        let aliases: FieldAliasTable = &[("attack_impulse_level", "_unk0073")];
        let mut v = json!({"_unk0073": 1, "weapon_key": 0xdead});
        normalize_input_aliases(&mut v, aliases);
        let m = v.as_object().unwrap();
        assert_eq!(m.get("attack_impulse_level"), Some(&json!(1)));
        assert!(!m.contains_key("_unk0073"));
        assert_eq!(m.get("weapon_key"), Some(&json!(0xdead)));
    }

    #[test]
    fn input_normalize_prefers_canonical_when_both_present() {
        let aliases: FieldAliasTable = &[("attack_impulse_level", "_unk0073")];
        let mut v = json!({"attack_impulse_level": 99, "_unk0073": 1});
        normalize_input_aliases(&mut v, aliases);
        let m = v.as_object().unwrap();
        // Canonical wins, legacy is dropped to avoid ambiguity.
        assert_eq!(m.get("attack_impulse_level"), Some(&json!(99)));
        assert!(m.contains_key("_unk0073"));
    }

    #[test]
    fn round_trip_through_both_shapes_preserves_data() {
        let aliases: FieldAliasTable = &[("attack_impulse_level", "_unk0073")];

        // Start from a "canonical" (T0) JSON.
        let canonical = json!({"attack_impulse_level": 7, "weapon_key": 0x1234});

        // Project to V3 shape — should rename canonical → legacy.
        let mut v3 = canonical.clone();
        apply_v3_aliases(v3.as_object_mut().unwrap(), aliases);
        let v3_map = v3.as_object().unwrap();
        assert_eq!(v3_map.get("_unk0073"), Some(&json!(7)));

        // Now read the V3-shaped JSON back through the input normalizer.
        let mut back = v3.clone();
        normalize_input_aliases(&mut back, aliases);
        assert_eq!(back, canonical);
    }
}
