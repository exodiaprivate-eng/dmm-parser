//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E0100` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                              (_key)
//!   2. CString string_key                   (_stringKey)
//!   3. u8 is_blocked                        (_isBlocked)
//!   4. u8 trigger_type                      (_triggerType)
//!   5. u8 is_enable                         (_isEnable)
//!   6. u8 safe_zone_type                    (_safeZoneType)
//!   7. u32 player_condition_info            (_playerConditionInfo,
//!      sub_1410FF430 → qword_145F0E9C0)
//!   8. u32 ui_map_texture_info              (_uiMapTextureInfo,
//!      inline → qword_145F113B0)
//!   9. [u8; 12] position                    (_position, vec3)
//!  10. u32 rotation_y                       (_rotationY, f32-as-u32)
//!  11. u8 world_map_color_r                 (_worldMapColorR; G/B
//!      channels are body fields in the tail)
//!  12. u32 field_revive_info                (_fieldReviveInfo,
//!      inline → qword_145F1A890)
//!  13. _targetDataList (sub_141103D50 → struct +48, CArray of TAGGED
//!      VARIANT items: u8 tag + 4-byte lookup via case-dispatched
//!      helper sub_141104AE0/sub_1410FF5C0/sub_1410FF340/sub_141100740)
//!      ← TAIL STARTS HERE
//!
//! Steps 1-12 are typed; the tag-dispatched variant CArray lives in
//! tail. Reopens cleanly when each tag's helper is decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GamePlayTriggerInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub trigger_type: u8,
        pub is_enable: u8,
        pub safe_zone_type: u8,
        pub player_condition_info: u32,
        pub ui_map_texture_info: u32,
        pub position: [u8; 12],
        pub rotation_y: u32,
        pub world_map_color_r: u8,
        pub field_revive_info: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameplaytrigger.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameplaytrigger.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GamePlayTriggerInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gameplaytrigger roundtrip mismatch");
    }
}
