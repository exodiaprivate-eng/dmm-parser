//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_10184B66C` at 0x10184B66C.
//! Korean error strings leak the full field-name sequence:
//!
//! Wire reads, in order:
//!   1. u16 key                  (sub_100F25384, format-2 pabgh)
//!   2. CString string_key       (sub_1006B3F50, struct +8)
//!   3. u8 is_blocked            (sub_1006B3CC0, struct +16)
//!      ← TAIL STARTS HERE
//!   4. (tail) _condition        (sub_101045678, struct +24) — POLYMORPHIC
//!      GameCondition tree (sub_141D8C6D0 family on Win)
//!   5. (tail) _projectileKey, _projectileShotKey,
//!      _projectileChasePhysicsMaterialHash (3× u32, sub_1006B3D80 family)
//!   6. (tail) _projectileShotSpread (sub_1006B48A8, +44, struct stride
//!      12 → likely vec3 or similar composite)
//!   7. (tail) _projectileShotInterval (sub_1006B4C60, +56, stride 8)
//!   8. (tail) _projectileHeightOffset, _projectileCreateDelayTime
//!      (2× sub_1006B3DE0 floats)
//!   9. (tail) _projectileHitRate, _projectileShotCount
//!      (2× sub_1006B3D00 = u8)
//!  10. (tail) _effectData (sub_10184B8B4, +80) — CArray of polymorphic
//!      GameGlobalEffectInfo_Effect descriptors
//!  11. (tail) _weatherData (sub_10187D7A8, +88), _postProcessData
//!      (sub_10187D8A8, +96), _isAdvanced (sub_1006B3CC0, +104, u8)
//!
//! Stop at field 3 (is_blocked) because field 4 is the GameCondition
//! polymorphic tree. Body has 14 more wire reads after the cutoff.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GameGlobalEffectInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameglobaleffectinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameglobaleffectinfo.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GameGlobalEffectInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gameglobaleffectinfo roundtrip mismatch");
    }
}
