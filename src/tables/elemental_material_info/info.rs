//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DC8F0` in CrimsonDesert.exe (Win build).
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                                   (_key, sub_141BF6840
//!      writer wrapper)
//!   2. CString string_key                        (_stringKey)
//!   3. u8 is_blocked                             (_isBlocked) — was
//!      mis-named `byte_16` in initial Tier 1.5 promotion
//!   4. u8 elemental_material_system_type         (_elementalMaterialSystemType)
//!   5. CString elemental_material_key            (_elementalMaterialKey)
//!   6. u32 total_fuel_amount                     (_totalFuelAmount)
//!   7. u32 fuel_standard_obb_size                (_fuelStandardObbSize)
//!   8. u32 fuel_end_passive_skill_key            (_fuelEndPassiveSkillKey)
//!   9. u32 fuel_end_passive_skill_level          (_fuelEndPassiveSkillLevel)
//!  10. u32 fuel_end_active_skill_key             (_fuelEndActiveSkillKey)
//!  11. u32 fuel_end_active_skill_level           (_fuelEndActiveSkillLevel)
//!  12. u8 use_temperature_transfer_margin        (_useTemperatureTransferMargin)
//!  13. sub_1411166F0 (CArray of 24-byte elements via sub_1411168A0
//!      sub-helper) ← TAIL STARTS HERE
//!  14. (body) _elementalMaterialStateDataList, _minStatList,
//!      _maxStatList, _parentMaterialKeyListDeprecatedXXX, _flag,
//!      _isSystemType, _elementalMaterialStatDataList,
//!      _sceneObjectSpawnableType, …
//!
//! Steps 1-12 are typed; everything from step 13 lives in `tail_blob`.
//! `sub_1411166F0` exceeds the 3-IDA-call budget (nested
//! `sub_1411168A0`); reopens cleanly when the array helper family is
//! decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct ElementalMaterialInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub elemental_material_system_type: u8,
        pub elemental_material_key: CString<'a>,
        pub total_fuel_amount: u32,
        pub fuel_standard_obb_size: u32,
        pub fuel_end_passive_skill_key: u32,
        pub fuel_end_passive_skill_level: u32,
        pub fuel_end_active_skill_key: u32,
        pub fuel_end_active_skill_level: u32,
        pub use_temperature_transfer_margin: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\elementalmaterialinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\elementalmaterialinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                ElementalMaterialInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "elementalmaterialinfo roundtrip mismatch");
    }
}
