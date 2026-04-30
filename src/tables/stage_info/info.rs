//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_10183756C` at 0x10183756C
//! (size 0xa00, ~2.5 KB). 25 MB pabgb / largest table in the set.
//! Korean error strings leak ~50 field names — the tail is dominated
//! by polymorphic `SequencerDesc` (sub_10109D1F4, +120, stride 232)
//! and many `_spawn*Faction*` / `*ConditionList` polymorphic blobs.
//!
//! Wire reads, in order:
//!   1. u32 key                  (sub_100F193A0)
//!   2. CString string_key       (sub_1006B3F50, struct +8)
//!   3. u8 is_blocked            (sub_1006B3CC0, struct +16)
//!   4. LocalizableString name   (sub_1006D8484, struct +24, stride 32)
//!   5. LocalizableString stage_desc    (sub_1006D8484, struct +56)
//!   6. LocalizableString complete_log  (sub_1006D8484, struct +88)
//!      ← TAIL STARTS HERE
//!   7. (tail) _sequencerDesc          (sub_10109D1F4, +120, stride 232)
//!      — POLYMORPHIC SequencerStageChartDesc family (sub_141D8C6D0
//!      tree on Win)
//!   8. (tail) _spawnFactionSpawnDataInfo (sub_101837F6C, +352)
//!   9. (tail) _spawnFactionNodeInfo      (sub_10165BB88, +354)
//!  10. (tail) _disableFactionSpawnPartyNameHashList (sub_100C52EA4, +360)
//!  11. (tail) _stageCategory, _closeFilter, _closeFilterByGroup,
//!      _globalFilterCharacterList, _questType, _stageDataType,
//!      _parentQuest, _parentStage, _ownerMissionInfo,
//!      _childStageList, _executorMissionList, _executorStageList,
//!      _executeTargetStageList, _playCondition (GameCondition),
//!      _closeCondition (GameCondition), _fieldInfo, _startPlayerList,
//!      _forbiddenCharacterList, _platformCharacter — and ~30 more.
//!
//! Stop at field 6 because field 7 is the SequencerDesc polymorphic
//! family. Body has 40+ more wire reads.
//!
//! ## SequencerStageChartDesc wire layout (sub_141D8C6D0, Win-IDA)
//!
//! Per-element reader for `_sequencerDesc` (stage_info field 7) and
//! `_loadingTargetInfo` / `_sequencerDescList` in
//! global_stage_sequencer_info. 232 mem bytes per element, 26 wire
//! fields. The first 13 fields (offsets +8..+55) are trivially
//! typeable; field 15 onward depends on the GameCondition stream-mode
//! anti-disassembly fix.
//!
//!  1. CString name                                    (mem +8)
//!  2. u32 raw                                         (mem +16)
//!  3. CString prefab_path                             (mem +24)
//!  4. Vec3 position                                   (mem +32, 12 b)
//!  5. u32 raw                                         (mem +44)
//!  6. u8 flag                                         (mem +48)
//!  7. u8 flag                                         (mem +49)
//!  8. u8 flag                                         (mem +50)
//!  9. u8 flag                                         (mem +51)
//! 10. u8 flag                                         (mem +52)
//! 11. u8 flag                                         (mem +53)
//! 12. u8 flag                                         (mem +54)
//! 13. u8 flag                                         (mem +55)
//! 14. sub_141106210 — 8 mem bytes                     (mem +56)
//! 15. sub_141103B30 — OptionalGameCondition           (mem +64)
//!     ← stream-mode GameCondition blocker starts here
//! 16. CString cstring_a                               (mem +72)
//! 17. CString cstring_b                               (mem +80)
//! 18. CArray<(CString, CString)>                      (mem +88, 16 b)
//! 19. CArray<sub_1410F2F90> — 56-byte items           (mem +104, 16 b)
//!     (per element: GameCondition + sub_14110C270 +
//!     sub_14110C110 + sub_14110BFB0 — sub_14110C270 is
//!     CArray<SequencerStageTrackChangeData_*> with its own
//!     polymorphic hierarchy)
//! 20. sub_14110E010 — 16 mem bytes                    (mem +120)
//! 21. sub_1410FFAC0 — CArray-shaped                   (mem +136)
//! 22. sub_1410FFAC0 — CArray-shaped                   (mem +152)
//! 23. sub_1410FEF40 — CArray<u32>                     (mem +168)
//! 24. sub_1410FEF40 — CArray<u32>                     (mem +184)
//! 25. sub_141102FF0 — 16 mem bytes                    (mem +200)
//! 26. sub_141102FF0 — 16 mem bytes                    (mem +216)
//!
//! Layout extracted from Win-IDA decompile of sub_141D8C6D0 (this
//! session). Promotion path: once GameCondition stream-mode lands,
//! fields 15-26 unblock; field 19 brings in a second polymorphic
//! family (SequencerStageTrackChangeData) that needs its own
//! family-decoder pass.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct StageInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub name: LocalizableString<'a>,
        pub stage_desc: LocalizableString<'a>,
        pub complete_log: LocalizableString<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\stageinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\stageinfo.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                StageInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "stageinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP: missing fixture {}", PABGB);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = StageInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            StageInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
