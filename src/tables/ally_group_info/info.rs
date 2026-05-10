//! Hand-corrected: IDA-derived parser for `AllyGroupInfo.pabgb`.
//!
//! ─── v3.1 closure analysis (iter 75) ────────────────────────────────────
//! Cross-check via `sub_1410A21A0` (typeinfo→record-reader path per the
//! iter 57 typeinfo registry). Wire reads confirm the layout:
//!
//!   offset 0    4 bytes              → _key (u32)
//!   offset 8    CString              → _stringKey
//!   offset 16   1 byte               → _isBlocked
//!   offset 24   for i in 0..7 {       → 7 × CArray<u32>
//!                 read u32 count       (16-byte stride per element,
//!                 read N × u32         consuming 24..136)
//!               }
//!   offset 136  1 byte               → killer_detection_time (etc.)
//!   …           5 bytes of bools
//!   offset 142  sub_1410CC220        → interesting_condition (u16 hash)
//!   offset 144  sub_1410CC290        → add_on_ally_group_list
//!   offset 160  sub_1410CC290        → interesting_order_list
//!
//! NattKh schema lists `_relationTypeList` as a single canonical at this
//! position; the rust struct already unrolls the 7-iteration loop into
//! `relation_type_list_0..6`. Wire-level confirmation: **`_relationTypeList`
//! is a pure 1-to-7 wrapper** around the seven `relation_type_list_*`
//! rust fields. Closure path: 1-to-N alias entry. No new decoder work.
//!
//! Per IDA sub_1410D5BE0:
//!   - u32 key (sub_141BF6720 is internal storage helper, not a stream read)
//!   - CString string_key
//!   - u8 is_blocked
//!   - relation_type_list: 7-element fixed array of CArray<u32>
//!   - 5 u8 flags
//!   - u32 interesting_condition (sub_1410FF430 = u32 hash)
//!   - add_on_ally_group_list: CArray<u32> (sub_1410FF4A0 thunk)
//!   - interesting_order_list: CArray<u32> (sub_1410FF4A0 thunk)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct AllyGroupInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub relation_type_list_0: CArray<u32>,
        pub relation_type_list_1: CArray<u32>,
        pub relation_type_list_2: CArray<u32>,
        pub relation_type_list_3: CArray<u32>,
        pub relation_type_list_4: CArray<u32>,
        pub relation_type_list_5: CArray<u32>,
        pub relation_type_list_6: CArray<u32>,
        pub killer_detection_time: u8,
        pub apply_reporting: u8,
        pub is_wild: u8,
        pub is_main_ally_group: u8,
        pub is_intruder: u8,
        pub interesting_condition: u32,
        pub add_on_ally_group_list: CArray<u32>,
        pub interesting_order_list: CArray<u32>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/allygroupinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(AllyGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "allygroupinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(AllyGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            AllyGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
