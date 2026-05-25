//! IDA-derived parser for `ActionRestrictionOrderInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::ActionRestrictionOrderInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  18
// Decoded by dmm-parser:   18
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _key
// ✅ _isBlocked (direct_15B, stream=15)
// ✅ _stringKey
// ✅ _ignoreThrow (direct_15B, stream=15)
// ✅ _ignoreCatch (direct_15B, stream=15)
// ✅ _delayKnockOut (direct_15B, stream=15)
// ✅ _delayDeadFatal (direct_15B, stream=15)
// ✅ _useAutoAttackThrow (direct_15B, stream=15)
// ✅ _useRandomHitDir (direct_15B, stream=15)
// ✅ _additiveSkill (direct_u64, stream=8)
// ✅ _endAiEventHash (direct_u32, stream=4)
// ✅ _startAiEventHash (direct_u32, stream=4)
// ✅ _actionRestrictionType (direct_15B, stream=15)
// ✅ _order (direct_u16, stream=2)
// ✅ _skillInfo (reader_4B, stream=4)
// ✅ _registTypeStatus (reader_4B, stream=4)
// ✅ _aiEventTagNameHash (direct_u32, stream=4)
// ✅ _spawnActionList (direct_u64, stream=8)

use crate::binary::*;
use crate::py_binary_struct;

// Hand-corrected: the auto-extractor saw spawn_action_list elements as
// [u8;12] but empirical sweep across all 29 vanilla elements shows the
// trailing u32 is always 0, with the first 8 bytes carrying real data.
// Promoted to (u64 + u32 reserved) for field-level JSON access.
py_binary_struct! {
    pub struct SpawnActionEntry {
        pub hash: u64,
        pub reserved: u32,
    }
}

py_binary_struct! {
    pub struct ActionRestrictionOrderInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub start_ai_event_hash: u32,
        pub end_ai_event_hash: u32,
        pub order: u16,
        pub action_restriction_type: u8,
        pub regist_type_status: u32,
        pub skill_info: u32,
        pub spawn_action_list: CArray<SpawnActionEntry>,
        pub ai_event_tag_name_hash: u32,
        pub ignore_catch: u8,
        pub ignore_throw: u8,
        pub delay_dead_fatal: u8,
        pub delay_knock_out: u8,
        pub use_random_hit_dir: u8,
        pub use_auto_attack_throw: u8,
        pub additive_skill: u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("actionrestrictionorderinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ActionRestrictionOrderInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "actionrestrictionorderinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ActionRestrictionOrderInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            ActionRestrictionOrderInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
