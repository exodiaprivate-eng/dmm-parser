//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DFBA0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                            (_key)
//!   2. CString string_key                 (_stringKey)
//!   3. u8 is_blocked                      (_isBlocked)
//!   4. u8 interaction_type                (_interactionType)
//!   5. u8 interaction_show_ui_type        (_interactionShowUIType)
//!   6. u8 preemption_type                 (_preemptionType)
//!   7. LocalizableString interaction_name (_interactionName)
//!   8. u8 pivot_selection_target          (_pivotSelectionTarget)
//!   9. CArray<InteractionPivotData> interaction_pivot_list
//!      (sub_141E2BEB0 → 168 mem bytes / 28 wire fields per element)
//!      ← TAIL STARTS HERE
//!  10. (tail) _interactionConditionDataList — sub_141114DD0 →
//!      CArray<COptional<InteractionConditionData>>; per inner
//!      element: 2 GameCondition (sub_141103B30) + u8 + u32 lookup
//!      (sub_1410FF050) + u32 raw + u8 + u8. Reusable shape provided
//!      by `crate::binary::condition_pair::ConditionPairCArray`.
//!  11. (tail) u8 _autoInteractionType (mem a2+96)
//!  12. (tail) u16 _categoryInfo (sub_141103CA0, mem a2+98)
//!  13. (tail) u32 _inputKeyMapName (read_u32_lookup_DA30, mem a2+100)
//!  14. (tail) u8 _buttonClickType (mem a2+102)
//!  15. (tail) u8 _keyboardClickType (mem a2+103)
//!  16. (tail) 4× u8 unknown flags (mem a2+104..107)
//!  17. (tail) sub_141D8C6D0 _someSequencerDesc (232 mem bytes;
//!      26 wire fields incl. 1 GameCondition + 2 unknown helpers
//!      sub_1410F2F90 + sub_14110E010 — blocks Tier 1 promotion)
//!  18. (tail) u32 raw at a2+344
//!  19. (tail) u32 _someName lookup at a2+348 (read_u32_lookup_DA30)
//!  20. (tail) u16 lookup at a2+350 (sub_141100370)
//!  21. (tail) sub_141100E90 — CArray of 32-byte items (28 wire bytes
//!      per element: f32 + 3× 8-byte clusters), mem a2+352
//!  22. (tail) sub_141103C30 — u32 lookup, mem a2+368
//!  23. (tail) CArray<u32> at a2+376 (sub_1410FEF40)
//!  24. (tail) CString at a2+392
//!  25. (tail) CString at a2+400
//!  26. (tail) 10× u8 trailing flags at a2+408..417
//!
//! Steps 1-9 are typed. Field 10 (ConditionPairCArray) is decodable
//! via the shared helper but blocked: stream-mode GameConditionNode
//! reads through ConditionData variant 36 (and other anti-disassembly
//! variants) misinterpret bytes as CString lengths inside leaf
//! payloads. Field 17 (sub_141D8C6D0) blocks the rest until two
//! unknown helpers are decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

// InteractionPivotData — sub_141E2BEB0 inner, 168 mem bytes / 28 wire fields.
// Wire order: u32 key + 4× (CString-hash + Vec3) + 3× u32 raw + 4× u32 raw
// (v56 cluster) + 2× u32 raw (v57) + 2× u32 raw (v58) + 2× u32 raw +
// CString name + Vec3 + u32 + 2× u64 + u32 (faction_group_info lookup).
py_binary_struct! {
    pub struct InteractionPivotData<'a> {
        pub key: u32,
        pub tag_a: CString<'a>,
        pub vec_a: [f32; 3],
        pub tag_b: CString<'a>,
        pub vec_b: [f32; 3],
        pub tag_c: CString<'a>,
        pub vec_c: [f32; 3],
        pub tag_d: CString<'a>,
        pub vec_d: [f32; 3],
        pub raw_a: u32,
        pub raw_b: u32,
        pub raw_c: u32,
        pub vec_e: [f32; 3],     // v56 cluster: 4 raw u32s = 16 bytes
        pub raw_e_3: u32,
        pub raw_f: u32,
        pub raw_f_1: u32,
        pub raw_g: u32,
        pub raw_g_1: u32,
        pub raw_h: u32,
        pub raw_i: u32,
        pub name: CString<'a>,
        pub vec_f: [f32; 3],
        pub raw_j: u32,
        pub raw_k: u64,
        pub raw_l: u64,
        pub faction_group_info: u32,
    }
}

pabgh_typed_blob_table! {
    pub struct InteractionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub interaction_type: u8,
        pub interaction_show_ui_type: u8,
        pub preemption_type: u8,
        pub interaction_name: LocalizableString<'a>,
        pub pivot_selection_target: u8,
        pub interaction_pivot_list: CArray<InteractionPivotData<'a>>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB_PATH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\interactioninfo.pabgb";
    const PABGH_PATH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\interactioninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = InteractionInfo::read_with_size(&data, &mut c, e - s).unwrap_or_else(|er| panic!("entry {} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "interactioninfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH_PATH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = InteractionInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            InteractionInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
