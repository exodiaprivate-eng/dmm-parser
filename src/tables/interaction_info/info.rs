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
//!   9. sub_141E2BEB0 → struct +64 (_interactionPivotList: CArray of
//!      168-byte composite items with 30+ helpers, way over budget)
//!      ← TAIL STARTS HERE
//!  10. (body) _interactionConditionDataList, _autoInteractionType,
//!      _categoryInfo, _inputKeyMapName, _buttonClickType,
//!      _keyboardClickType, … (sub_141D8C6D0 polymorphic
//!      GameCondition tree at struct +112)
//!
//! Steps 1-8 are typed. Step 9 onward is tail. The InteractionInfo
//! body is unusually deep (40+ wire reads, 5+ unknown helpers) and
//! includes the sub_141D8C6D0 polymorphic GameCondition descriptor.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

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
