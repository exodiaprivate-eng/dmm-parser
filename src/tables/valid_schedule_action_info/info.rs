//! Tier 1 — fully typed parser.
//!
//! Reader (Mac CrimsonDesert_Steam):
//!   - Entry-level: `sub_101855C70` at 0x101855C70
//!   - _convertingData (NighScheduleConvertingConditionData): `sub_101855B24` at 0x101855B24
//!
//! NOTE on `_jobList`: the Mac NighScheduleConvertingConditionData reader
//! `sub_1018743E8` reads each job element via a `JobKey` reader where the key
//! local is `unsigned __int16` (vs `int` for Tribe / EquipType / Faction).
//! The wire element width is therefore u16, not u32. Mismatching this was the
//! decisive bug that hid every previous Tier 1 attempt: vanilla entry 22
//! (key 0xfd87061c) was the only entry whose tail exposed the byte-shift,
//! everything else aligned by accident with empty arrays.
//!
//! StaticInfoWrapper template signatures (Mac demangled symbols):
//!   - tribe:   StaticInfoWrapper<TribeInfoKey, TribeInfo, TribeInfoManager, u16>     → key u32
//!   - job:     StaticInfoWrapper<JobKey, JobInfo, JobInfoManager, u16>               → key **u16**
//!   - equip:   StaticInfoWrapper<EquipTypeKey, EquipTypeInfo, EquipTypeInfoManager, u16> → key u32
//!   - faction: StaticInfoWrapper<FactionKey, FactionInfo, FactionInfoManager, u16>   → key u32
//!
//! Wire layout for ValidScheduleActionInfo (sub_101855C70, in order):
//!   1. u32 key
//!   2. CString string_key
//!   3. u8  is_blocked
//!   4. CArray<u32> action_name_hash_list
//!   5. u8  type_
//!   6. CArray<CString> keyword_lower_string_list
//!   7. NighScheduleConvertingConditionData converting_data
//!
//! Wire layout for NighScheduleConvertingConditionData (sub_101855B24):
//!   1. CArray<CString> action_keyword_list
//!   2. CArray<CString> tag_list
//!   3. CArray<u32> tribe_list
//!   4. CArray<u16> job_list                      ← u16, not u32
//!   5. CArray<u32> equip_type_list
//!   6. CArray<u32> faction_list
//!   7. [u8;16] action_attribute_flag
//!   8. CArray<NighScheduleConvertingConditionData> and_condition_data_list


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::ValidScheduleActionInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  6
// Decoded by dmm-parser:   6
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _type (direct_u8, stream=1)
// ✅ _actionNameHashList (reader_4B, stream=4)
// ✅ _keywordLowerStringList
// ✅ _key

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct NighScheduleConvertingConditionData<'a> {
        pub action_keyword_list: CArray<CString<'a>>,
        pub tag_list: CArray<CString<'a>>,
        pub tribe_list: CArray<u32>,
        pub job_list: CArray<u16>,
        pub equip_type_list: CArray<u32>,
        pub faction_list: CArray<u32>,
        // Split 16-byte flag field into 2× u64 for field-level scalar access.
        pub action_attribute_flag_low: u64,
        pub action_attribute_flag_high: u64,
        pub and_condition_data_list: CArray<NighScheduleConvertingConditionData<'a>>,
    }
}

py_binary_struct! {
    pub struct ValidScheduleActionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub action_name_hash_list: CArray<u32>,
        pub type_: u8,
        pub keyword_lower_string_list: CArray<CString<'a>>,
        pub converting_data: NighScheduleConvertingConditionData<'a>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("validscheduleaction.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = ValidScheduleActionInfo::read_from(&data, &mut c)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e, "entry {} k=0x{:x} consumed {} of {} bytes", i, k, c - s, e - s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "validscheduleaction roundtrip mismatch");
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
            let item = ValidScheduleActionInfo::read_from(&data, &mut c).unwrap();
            assert_eq!(c, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            ValidScheduleActionInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
