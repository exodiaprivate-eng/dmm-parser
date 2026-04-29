//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410FD200` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                   (_key)
//!   2. CString string_key        (_stringKey)
//!   3. u8 is_blocked             (_isBlocked)
//!   4. u32 min_level             (_minLevel)
//!   5. u32 max_level             (_maxLevel)
//!   6. [u8; 28] exp              (_exp; sub_141E2BB80 reads
//!      u64+u64+u64+u32 — 3 large identifiers + a flag/count, kept
//!      as raw bytes for round-trip)
//!   7. _conditionInfo (sub_1410FF430 lookup) ← TAIL STARTS HERE
//!   8. _alertComponentName, _alertComponentNameForVaryExp,
//!      _knowledgeInfo, _buffInfo, _moneyInfo, _rewardDropSetInfo,
//!      _subLevelExpDataList, _additionalRewardList, … (more body
//!      fields, plus a CArray with sub_1410E2030 polymorphic dispatch)
//!
//! Step 6 is typed because sub_141E2BB80 is just 4 sequential primitive
//! reads. Tail starts at step 7 because the chain after it includes
//! sub_1410E2030 (polymorphic) embedded in a CArray.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct SubLevelInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub min_level: u32,
        pub max_level: u32,
        pub exp: [u8; 28],
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\sublevelinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\sublevelinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                SubLevelInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "sublevelinfo roundtrip mismatch");
    }
}
