//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_1018373E8` at 0x1018373E8.
//! Pabgb dump path is `reviepointinfo.pabgb` (typo in filename — game
//! ships it that way).
//!
//! Wire reads, in order:
//!   1. u32 key                 (sub_100F1A450, width 4)
//!   2. CString string_key      (sub_1006B3F50, struct +8)
//!   3. u8 is_blocked           (sub_1006B3CC0, struct +16)
//!   4. [u8; 12] position       (sub_1006B48A8, struct +20, vec3 of
//!      3× f32 — same helper as `_projectileShotSpread` in
//!      game_global_effect_info)
//!   5. u32 rotation_y          (sub_1006B3DE0, struct +32, f32-as-u32)
//!      ← TAIL STARTS HERE
//!   6. (tail) _sequencerStageChartDesc (sub_10109D1F4, +40,
//!      stride 232) — POLYMORPHIC SequencerStageChartDesc family
//!      (same helper as in stage_info)
//!   7. (tail) _fieldInfoKey (sub_100EF8B04, +272)
//!   8. (tail) _knowledgeInfo (sub_10074B280, +276)
//!   9. (tail) _knowledgeLevel (sub_1006B3D60, +280)
//!  10. (tail) _useDefaultRevive (sub_1006B3CC0, +284, u8)
//!
//! Stop at field 5 because field 6 is the SequencerStageChartDesc
//! polymorphic helper (sub_10109D1F4).

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct FieldReviveInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub position: [u8; 12],
        pub rotation_y: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\reviepointinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\reviepointinfo.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                FieldReviveInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "fieldreviveinfo (reviepointinfo.pabgb) roundtrip mismatch");
    }
}
