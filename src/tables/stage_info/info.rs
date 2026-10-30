//! StageInfo — pabgh_typed_blob_table.
//!
//! 2.04.00 reader: Mac `sub_10204C2C8` (45 fields, names from its Korean asserts). 2.04
//! regrouped the 2.03 record (91 fields) into optional sub-descs: sequencerStage,
//! factionSpawnStage, hierarchyDesc, playDesc, characterDesc, uiDesc, weatherData,
//! globalEffectData. Wire order:
//!
//!   stringKey, key, isBlocked, name (COptional<LocalizableString>), sequencerDesc,
//!   sequencerStage, factionSpawnStage, hierarchyDesc, playDesc, characterDesc, uiDesc,
//!   weatherData, globalEffectData (optional sub-descs), stageCategory, closeFilter,
//!   closeFilterByGroup (i64), scheduleStageCompleteAIEventList, rewardDropSetInfoList,
//!   parentQuest, fieldInfo, factionSequencerSpawnTagHash, scheduleCompleteCondition,
//!   guideEffectName, questType, stageDataType, updatePriority, then 22 u8 flags
//!   (isIgnoreDistance .. keepActorAfterStageClosed).
//!
//! Typed here: the header and `sequencer_desc` (the funcnpc scene name and world position).
//! Everything after rides in `tail_blob`, byte-exact. DMM's Companions Everywhere preset
//! sets `is_blocked` on records matched by `string_key`, so those two must stay typed.
//! The 2.03 field structs (STAGEINFO_112_RE.md) no longer describe the record and were
//! removed; extend from the 2.04 reader when a sub-desc needs field access.

use crate::binary::*;
use crate::binary::variants::sequencer_stage_chart_desc::SequencerStageChartDescPartial;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct StageInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub name: COptional<LocalizableString<'a>>,
        pub sequencer_desc: SequencerStageChartDescPartial<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("stageinfo.pabgb") }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut out = Vec::with_capacity(data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = StageInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: {}", i, key, e));
            assert_eq!(item.key, *key, "entry {}: key does not match the pabgh index", i);
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "stageinfo roundtrip mismatch");
    }

    /// Companions Everywhere matches these by string_key and flips `is_blocked`.
    #[test]
    fn companion_forbid_records_are_addressable() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let mut found = 0;
        for (_, start, end) in entry_ranges(&entries, data.len()) {
            let mut cursor = start;
            let item = StageInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            if item.string_key.data.starts_with("Func_Damiane_Forbidden_")
                || item.string_key.data.starts_with("Func_Oongka_Forbidden_")
            {
                found += 1;
            }
        }
        eprintln!("stageinfo: {} companion forbid records", found);
        assert!(found > 0, "no Func_*_Forbidden_* records parsed");
    }
}
