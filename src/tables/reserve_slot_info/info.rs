//! Full Tier 1 — every wire read decoded.
//!
//! Reader: `sub_1410F6600` in CrimsonDesert.exe (Win build).
//!
//! All helpers are non-polymorphic single-shot or CArray hash-lookups
//! at qword_145F0DAxx / qword_145F0E9C0 / qword_145F0DA40 / qword_145F0DA20.
//! Raw wire u16/u32 round-trip directly.
//!
//! Wire reads, in order. Canonical names verified against the Mac
//! binary's pa::ReserveSlotInfo parser (`sub_101889F14` in
//! `CrimsonDesert_Steam`) via the Korean error strings at
//! 0x1073b0ee4..0x1073b139d. Wire-type widths confirmed by
//! decompiling each CArray reader (see notes below):
//!   1. u32 key                                  (_key)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. u64 time_limit                           (_timeLimit)
//!   5. u32 cool_time                            (_coolTime)
//!   6. u32 auto_use_item_info                   (_autoUseItemInfo)
//!   7. u32 convert_item_info                    (_convertItemInfo)
//!   8. CArray<ReserveSlotPairA> fill_data_list  (_fillDataList,
//!      element: u32 lookup + u64)
//!   9. CString memo                             (_memo)
//!  10. u8 reserve_slot_type                     (_reserveSlotType)
//!  11. u8 using_type                            (_usingType)
//!  12. CArray<u16> enable_tribe_list            (_enableTribeList,
//!      Mac reader sub_1018BB0A4: wire u16 hash. Pre-2026-05-12
//!      dmm-parser had this as CArray<u32>, doubling the per-record
//!      drift for any record with tribe_count > 0 — the root cause
//!      of the "1.06 first record OK, later records fail at random
//!      offsets" symptom NattKh saw before their v1.1.9 fix.)
//!  13. CArray<u16> enable_vehicle_list          (_enableVehicleList,
//!      Mac reader sub_10104DC90: wire u16 hash)
//!  14. CArray<u8>  enable_mercenary_list        (_enableMercenaryList,
//!      Mac reader sub_10117EDF4: wire u8 — pre-2026-05-12 dmm-parser
//!      was MISSING this field entirely, even though Mac canonical
//!      has it. Restoring it brings dmm-parser to Mac parity.)
//!  15. CArray<ReserveSlotPairB> enable_special_name_hash_list
//!      (_enableSpecialNameHashList, Mac reader sub_1018BB2FC. NattKh
//!      CGM v1.1.9 claimed 1.06 removed this field, but 1.06 fixture
//!      roundtrip confirms it's still present — empty CArray for all
//!      27 vanilla entries.)
//!  16. CArray<u16> target_item_group_list       (_targetItemGroupList)
//!  17. u32 send_gimmick_event_key_for_slot_data_changed
//!      (_sendGimmickEventKeyForSlotDataChanged)
//!  18. u8 is_self_player_only                   (_isSelfPlayerOnly)
//!
//! VERIFIED 2026-05-12 against live 1.06 fixture: 27 entries, 3383
//! bytes, byte-identical roundtrip.
//!
//! ─── 2026-06-05 — 1.10 update (Mac parser sub_10190E7B0) ──────────────────
//! The list section changed in 1.10 (net +4 bytes per record):
//!   - `_enableVehicleList` REMOVED (no vehicle reader in sub_10190E7B0).
//!   - `_enableReserveSlotList` ADDED after `_enableSpecialNameHashList`
//!     (Mac reader sub_101942F58 @a2+120 — u32-count, ReserveSlotKey u32 wire).
//!   - `_reserveSlotTargetList` ADDED after `_enableItemGroupList`
//!     (same reader sub_101942F58 @a2+152).
//!   - trailing `unk_trailing_108` is really `_restoreOnRetry` (@a2+173, u8).
//! All six lists are empty (count=0) in vanilla 1.10, so the change is +4
//! bytes/record (one extra empty CArray). Byte-exact roundtrip vs the 1.10
//! fixture confirms the layout.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::ReserveSlotInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  18 (Mac binary == 1.06)
// Decoded by dmm-parser:   18 (1.06 fixture roundtrip verified)
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _sendGimmickEventKeyForSlotDataChanged (direct_u32, stream=4)
// ✅ _targetItemGroupList (reader_2B, stream=2)
// ✅ _isSelfPlayerOnly
// ✅ _fillDataList
// ✅ _convertItemInfo (reader_4B, stream=4)
// ✅ _reserveSlotType (direct_13B, stream=13)
// ✅ _memo
// ✅ _enableTribeList
// ✅ _usingType (direct_13B, stream=13)
// ✅ _enableSpecialNameHashList
// ✅ _enableVehicleList (reader_2B, stream=2)
// ✅ _stringKey
// ✅ _key
// ✅ _timeLimit (direct_u64, stream=8)
// ✅ _isBlocked (direct_13B, stream=13)
// ✅ _autoUseItemInfo (reader_4B, stream=4)
// ✅ _coolTime (direct_u32, stream=4)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct ReserveSlotPairA {
        pub lookup: u32,
        pub raw_bytes: u64,
    }
}

py_binary_struct! {
    pub struct ReserveSlotPairB {
        pub lookup_a: u32,
        pub lookup_b: u32,
    }
}

py_binary_struct! {
    pub struct ReserveSlotInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub time_limit: u64,
        pub cool_time: u32,
        pub auto_use_item_info: u32,
        pub convert_item_info: u32,
        pub fill_data_list: CArray<ReserveSlotPairA>,
        pub memo: CString<'a>,
        pub reserve_slot_type: u8,
        pub using_type: u8,
        // FIX 2026-05-12: wire u16 per element (Mac reader sub_101942960 @a2+72).
        pub enable_tribe_list: CArray<u16>,
        // 1.10 REMOVED _enableVehicleList: the 1.10 Mac parser sub_10190E7B0 has
        // NO vehicle reader between tribe (@a2+72) and mercenary (@a2+88). Present
        // in 1.06 (old reader sub_10104DC90) but dropped in 1.10.
        // Mac reader sub_1011DCCCC @a2+88 (wire u8 — mercenary index).
        pub enable_mercenary_list: CArray<u8>,
        // Mac reader sub_101942BB8 @a2+104. Element ReserveSlotPairB {u32,u32} = 8B.
        // Empty (count=0) for all vanilla entries.
        pub enable_special_name_hash_list: CArray<ReserveSlotPairB>,
        // NEW 1.10: _enableReserveSlotList. Mac reader sub_101942F58 @a2+120:
        // u32 count, then each element a ReserveSlotKey (u32 wire, resolved to a
        // u16 index in memory). Empty for all vanilla entries.
        pub enable_reserve_slot_list: CArray<u32>,
        // _enableItemGroupList. Mac reader sub_1010AA830 @a2+136 (wire u16).
        pub target_item_group_list: CArray<u16>,
        // NEW 1.10: _reserveSlotTargetList. Same reader as _enableReserveSlotList
        // (sub_101942F58 @a2+152) — u32-wire ReserveSlotKey elements. Empty in vanilla.
        pub reserve_slot_target_list: CArray<u32>,
        // _sendGimmickEventKeyForSlotDataChanged. Mac reader sub_1010AEEC4 @a2+168 (u32).
        pub send_gimmick_event_key_for_slot_data_changed: u32,
        // _isSelfPlayerOnly. Mac @a2+172 (u8).
        pub is_self_player_only: u8,
        // _restoreOnRetry (1.10). Mac sub_10190E7B0 @a2+173 (u8). Was previously
        // mislabeled `unk_trailing_108` — the Korean error string at 0x10190eaa0
        // names it _restoreOnRetry (a Re-Blockade / retry feature flag).
        pub restore_on_retry: u8,
    }
}

// 2026-05-12 follow-up: the partial fix shipped in commit 2df98d2
// removed enable_special_name_hash_list (correct per NattKh CGM 1.1.9
// notes) but left two latent bugs that caused "first record OK, later
// records fail at offset 0x79c" on 1.06:
//   (a) enable_tribe_list was CArray<u32> (4 bytes/elem) but the Mac
//       reader sub_1018BB0A4 confirms wire u16 (2 bytes/elem). Each
//       record with tribe_count > 0 drifted 2*tribe_count bytes.
//   (b) enable_mercenary_list (Mac mem offset 104, reader sub_10117EDF4,
//       wire u8) was missing from the struct entirely.
// Both fixed in this commit by IDA decompile of the Mac binary
// reserveslot parser sub_101889F14.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("reserveslot.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                ReserveSlotInfo::read_from(&data, &mut c)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e, "entry {} key=0x{:x}: cursor at {} expected {}", i, k, c, e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "reserveslot roundtrip mismatch");
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
            let item = ReserveSlotInfo::read_from(&data, &mut c).unwrap();
            assert_eq!(c, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            ReserveSlotInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
