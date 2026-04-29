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
