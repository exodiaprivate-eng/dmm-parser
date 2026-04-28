//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410F4620` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                          (_key)
//!   2. CString string_key               (_stringKey)
//!   3. u8 is_blocked                    (_isBlocked)
//!   4. u8 quest_type                    (_questType)
//!   5. u8 quest_category                (_questCategory)
//!   6. LocalizableString name           (_name)
//!   7. LocalizableString desc           (_desc)
//!   8. _questGroupInfo (sub_141106F50 thunk → sub_14B99E300, deep)
//!      ← TAIL STARTS HERE
//!   9. (body) _factionInfo, _factionStateData, _branchData,
//!      _startPlayerList, _branchDataList, _executorQuestList,
//!      _gaugeList, …

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct QuestInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub quest_type: u8,
        pub quest_category: u8,
        pub name: LocalizableString<'a>,
        pub desc: LocalizableString<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\questinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\questinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                QuestInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "questinfo roundtrip mismatch");
    }
}
