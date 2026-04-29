//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410EDA00` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                  (_key)
//!   2. CString string_key       (_stringKey)
//!   3. u8 is_blocked            (_isBlocked)
//!   4. u16 craft_tool_info      (_craftToolInfo, sub_141105A10 →
//!      qword_145F15028 lookup)
//!   5. u8 item_consume_type     (_itemConsumeType)
//!      ← TAIL STARTS HERE
//!   6. _conditionList, _needKnowledgeInfo, _craftTagName,
//!      _isFromItemInfo, _isResultItemForWarehouse, _isWithSealedItem,
//!      _isApplyEnchantLevel, _isMaterialItemOnlySameItemNo,
//!      _isAllowMaterialItemSelfSame, _fixedMaterialDataList, …
//!
//! Steps 1-5 are typed. The body has many more fields with unknown
//! helpers; reopens cleanly when decoded.
//!
//! Helper: `sub_141105A10` = u16 lookup at qword_145F15028.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct MultiChangeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub craft_tool_info: u16,
        pub item_consume_type: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\multichangeinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\multichangeinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                MultiChangeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "multichangeinfo roundtrip mismatch");
    }
}
