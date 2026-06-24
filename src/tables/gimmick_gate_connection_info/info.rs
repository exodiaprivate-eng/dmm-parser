//! Full Tier 1 — every wire read decoded.
//!
//! Reader: `sub_1410E5470` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings —
//! 1.3.5 audit re-mapped placeholder names like `lookup_a..e`,
//! `trailing` to canonical):
//!   1. u32 key                                  (_key)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. u32 material_item_info                   (_materialItemInfo,
//!      sub_1410FF5C0 → qword_145F0DA00)
//!   5. u32 result_item_info                     (_resultItemInfo,
//!      sub_1410FF5C0 → qword_145F0DA00)
//!   6. u32 knowledge_info                       (_knowledgeInfo,
//!      sub_1411006D0 → qword_145F0DA28)
//!   7. u32 src_gate_info                        (_srcGateInfo,
//!      inline → qword_145F24D68)
//!   8. u32 dest_gate_info                       (_destGateInfo,
//!      inline → qword_145F24D68)
//!   9. u8 push_knowledge_to_gimmick             (_pushKnowledgeToGimmick)
//!
//! All helpers are non-polymorphic single-shot u32 lookups; raw wire
//! u32 round-trips. No CArrays, no COptional.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::GimmickGateConnectionInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  9
// Decoded by dmm-parser:   9
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _pushKnowledgeToGimmick (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _key
// ✅ _materialItemInfo (reader_4B, stream=4)
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _knowledgeInfo (reader_4B, stream=4)
// ✅ _resultItemInfo (reader_4B, stream=4)
// ✅ _destGateInfo (direct_u8, stream=1)
// ✅ _srcGateInfo

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct GimmickGateConnectionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub material_item_info: u32,
        pub result_item_info: u32,
        pub knowledge_info: u32,
        pub src_gate_info: u32,
        pub dest_gate_info: u32,
        pub push_knowledge_to_gimmick: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("gimmickgateconnection.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GimmickGateConnectionInfo::read_from(&data, &mut c)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e, "entry {} key=0x{:x}: cursor at {} expected {}", i, k, c, e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gimmickgateconnection roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: pabgh not found");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut c = *start;
            let item = GimmickGateConnectionInfo::read_from(&data, &mut c).unwrap();
            assert_eq!(c, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            GimmickGateConnectionInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
