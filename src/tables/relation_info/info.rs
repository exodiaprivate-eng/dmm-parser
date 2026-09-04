
// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::RelationInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  11
// Decoded by dmm-parser:   11
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _detectRestrictCount (direct_u8, stream=1)
// ✅ _order (direct_u8, stream=1)
// ✅ _doCompleteNotPriorityActor (direct_u8, stream=1)
// ✅ _detectMemorizeTime (direct_u64, stream=8)
// ✅ _isDetectEventOnly (direct_u8, stream=1)
// ✅ _detectValueRatio (direct_u32, stream=4)
// ✅ _gimmickTagDataList
// ✅ _stringKey
// ✅ _key
// ✅ _relationReactionType (direct_u8, stream=1)
// ✅ _isBlocked (direct_u8, stream=1)

#![allow(clippy::doc_overindented_list_items)]
//! Hand-corrected: IDA-derived parser for `RelationInfo.pabgb`.
//!
//! Per IDA sub_1410F4C70 + sub_14110AA70:
//!   - 11 outer fields (u8 key, CString, then 9 more)
//!   - gimmick_tag_data_list element = { u32 hash + CArray<u32> + CArray<u32> }
//!     (NB: mac symbols list only 2 inner fields but binary reads 3 — last
//!      one is unnamed in current Mac depot)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct RelationGimmickTagData {
        pub gimmick_tag_hash: u32,
        pub spawn_reason_hash_list: CArray<u32>,
        pub extra_list: CArray<u32>,
    }
}

py_binary_struct! {
    /// 1.18 `_battleOverrideReactionList` element — 2 wire bytes.
    /// The 1.18 exe declares `BattleOverrideReaction` as a new 2-field type
    /// with exactly these names in this order, which matches the 2-byte
    /// element width: one u8 each.
    pub struct BattleOverrideReaction {
        pub target_ally_type: u8,
        pub relation_reaction_type: u8,
    }
}

py_binary_struct! {
    pub struct RelationInfo<'a> {
        pub key: u8,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub relation_reaction_type: u8,
        pub order: u8,
        pub detect_restrict_count: u8,
        pub detect_memorize_time: u64,
        pub do_complete_not_priority_actor: u8,
        pub detect_value_ratio: f32,
        pub is_detect_event_only: u8,
        // ── 2.01.00: `_disableNonBattleAccompanyReaction`, one u8 between
        // `_isDetectEventOnly` and `_gimmickTagDataList`. Every one of the 52 records
        // grew by exactly 1 byte, at offset 22 of record 0 — right here.
        pub disable_non_battle_accompany_reaction: u8,
        pub gimmick_tag_data_list: CArray<RelationGimmickTagData>,
        // ── 1.18.00: `_battleOverrideReactionList`, appended after the gimmick
        // tag list. 51 of 52 records grew by exactly 4 (count=0); record 0x16
        // grew by 6 — count=1 plus a 2-byte element, which fixes the element
        // width at 2.
        // The element split is not a guess: the 1.18 exe declares a new
        // `BattleOverrideReaction` type with exactly 2 fields, so 2 bytes = 2×u8.
        pub battle_override_reaction_list: CArray<BattleOverrideReaction>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("relationinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        // RelationInfo's pabgh uses u16 count + (u8 key + u32 offset) per entry.
        let Ok(pabgh) = std::fs::read(pabgb_path().with_extension("pabgh")) else {
            eprintln!("SKIP: missing pabgh fixture");
            return;
        };
        let count = u16::from_le_bytes(pabgh[0..2].try_into().unwrap()) as usize;
        let mut offsets = Vec::with_capacity(count.min(1 << 20));
        for i in 0..count {
            let pos = 2 + i * 5;
            let off = u32::from_le_bytes(pabgh[pos + 1..pos + 5].try_into().unwrap()) as usize;
            offsets.push(off);
        }
        offsets.sort();

        let mut items = Vec::new();
        for i in 0..offsets.len() {
            let mut o = offsets[i];
            let item = RelationInfo::read_from(&data, &mut o).unwrap();
            let next_off = if i + 1 < offsets.len() { offsets[i + 1] } else { data.len() };
            assert_eq!(o, next_off, "entry {} under/over-read: stopped at {} expected {}", i, o, next_off);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "relationinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Ok(pabgh) = std::fs::read(pabgb_path().with_extension("pabgh")) else {
            eprintln!("SKIP: missing pabgh fixture");
            return;
        };
        let count = u16::from_le_bytes(pabgh[0..2].try_into().unwrap()) as usize;
        let mut offsets = Vec::with_capacity(count.min(1 << 20));
        for i in 0..count {
            let pos = 2 + i * 5;
            let off = u32::from_le_bytes(pabgh[pos + 1..pos + 5].try_into().unwrap()) as usize;
            offsets.push(off);
        }
        offsets.sort();

        for (i, &start) in offsets.iter().enumerate() {
            let mut o = start;
            let item = RelationInfo::read_from(&data, &mut o).unwrap();
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            RelationInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
