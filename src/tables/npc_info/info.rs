//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410EBEB0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                                  (_key)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. u32 icon_path                            (_iconPath, read_u32_lookup_DA30)
//!   5. u16 store_info                           (_storeInfo, sub_141103610 → qword_145F15038)
//!   6. u32 coupon_item_info                     (_couponItemInfo, sub_1410FF5C0 → qword_145F0DA00)
//!   7. u32 npc_greet_friendly                   (_npcGreetFriendly)
//!   8. u32 npc_function_type_flag               (_npcFunctionTypeFlag)
//!   9. u32 shop_scenekey                        (_shopScenekey)
//!  10. u16 exchange_group_key                   (_exchangeGroupKey)
//!  11. LocalizableString exchange_button_text   (_exchangeButtonText)
//!  12. LocalizableString shop_name              (_shopName)
//!  13. LocalizableString interaction_name       (_interactionName)
//!  14. _dyeColorGroupDataList (sub_14110E340 → struct +136) ← TAIL STARTS HERE
//!  15. (body) _dyeTextureSetDataList, …
//!
//! Steps 1-13 are typed. The unknown helper sub_14110E340 wraps the
//! NpcInfo body proper; reopens cleanly when decoded.
//!
//! Helper: `sub_141103610` = single u16 lookup at qword_145F15038 (wire 2).

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct NpcInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub icon_path: u32,
        pub store_info: u16,
        pub coupon_item_info: u32,
        pub npc_greet_friendly: u32,
        pub npc_function_type_flag: u32,
        pub shop_scenekey: u32,
        pub exchange_group_key: u16,
        pub exchange_button_text: LocalizableString<'a>,
        pub shop_name: LocalizableString<'a>,
        pub interaction_name: LocalizableString<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\npcinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\npcinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                NpcInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "npcinfo roundtrip mismatch");
    }
}
