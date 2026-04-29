//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410FCD20` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u16 key                                  (_key, pabgh format 2)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. u32 exchange_item_info_for_buy           (_exchangeItemInfoForBuy,
//!      sub_1410FF5C0 → qword_145F0DA00)
//!   5. CArray<u32> exchange_item_info_list_for_sell
//!      (_exchangeItemInfoListForSell, sub_1410FFF10 → qword_145F0DA00)
//!   6. [u8; 8] sell_percents                    (_sellPercents)
//!   7. u8 store_type                            (_storeType)
//!   8. CArray<u64> price_increase_percent_list  (_priceIncreasePercentList,
//!      inline u32 count + N×u64)
//!   9. u32 sellable_character_condition_logic   (_sellableCharacterConditionLogic,
//!      sub_1410FF430 → qword_145F0E9C0)
//!  10. u32 reset_hour                           (_resetHour)
//!  11. u32 reset_day                            (_resetDay)
//!  12. u32 buyable_stock_count                  (_buyableStockCount)
//!  13. u32 sellable_stock_count                 (_sellableStockCount)
//!  14. u8 sellable_type                         (_sellableType)
//!  15. _stockDataList (inline CArray of 88-byte items via sub_1410FC8F0
//!      → struct +96) ← TAIL STARTS HERE
//!  16. (body) sub_1411002A0 16-byte slots, u32 + 3× u8 trailing
//!
//! Steps 1-14 are typed; everything from step 15 is in `tail_blob`.
//!
//! Helper: `sub_1410FFF10` = CArray<u32> hash-keyed at qword_145F0DA00.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct StoreInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub exchange_item_info_for_buy: u32,
        pub exchange_item_info_list_for_sell: CArray<u32>,
        pub sell_percents: [u8; 8],
        pub store_type: u8,
        pub price_increase_percent_list: CArray<u64>,
        pub sellable_character_condition_logic: u32,
        pub reset_hour: u32,
        pub reset_day: u32,
        pub buyable_stock_count: u32,
        pub sellable_stock_count: u32,
        pub sellable_type: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\storeinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\storeinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                StoreInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "storeinfo roundtrip mismatch");
    }
}
