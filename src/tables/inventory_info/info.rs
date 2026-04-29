//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E05E0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u16 key                                  (_key, pabgh format 2)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. CArray<InventorySubA> pushable_item_type_list (_pushableItemTypeList,
//!      sub_141103FB0: each item = u16 lookup via sub_141100620 at
//!      qword_145F0DA20 + u8 = 3 wire bytes)
//!   5. CArray<InventorySubA> excluded_item_type_list (_excludedItemTypeList,
//!      same shape)
//!   6. _inventoryMoveDataList (sub_141114720 → struct +56, CArray of
//!      160-byte items via sub_1410E0460 + many sub-helpers)
//!      ← TAIL STARTS HERE
//!   7. (body) _defaultSlotCount, _maxSlotCount, _pushItemAlertUIText,
//!      _InventoryNameUIText, _keyGuideLocalStringInfo, _pushableCheckType,
//!      _npcUsableData, _isMoveableInventory, _needSaveSlotCount, …
//!
//! Inner InventorySubA fields (`lookup`, `byte_2`) kept as placeholders —
//! Mac error strings only leak the InventoryInfo-level field names, not
//! sub-struct semantics.

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct InventorySubA {
        pub lookup: u16,
        pub byte_2: u8,
    }
}

pabgh_typed_blob_table! {
    pub struct InventoryInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub pushable_item_type_list: CArray<InventorySubA>,
        pub excluded_item_type_list: CArray<InventorySubA>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\inventory.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\inventory.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                InventoryInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "inventory roundtrip mismatch");
    }
}
