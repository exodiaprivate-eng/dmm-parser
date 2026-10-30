//! Hand-corrected: IDA-derived parser for `VehicleInfo.pabgb`.
//!
//! ─── v3.1 closure analysis (iter 79) ────────────────────────────────────
//! Both schema "missing" canonicals are pure structural wraps already
//! documented in the rust struct field comments below:
//!
//!   `_vehicleSeatDataList`       → 1-to-16 wrap around
//!                                  `vehicle_seat_data_00..15` (16 × u64)
//!   `_parentLinkAttachDataList`  → 1-to-2 wrap around
//!                                  `parent_link_attach_data_a` + `_b` (2 × u64)
//!
//! Closure path: 1-to-N alias entries. No new decoder work — the per-
//! record reader already consumes all bytes under the unrolled rust
//! field names. Total missing-canonical reduction when the v3.1 alias
//! mechanism is extended: 2 (vehicle_info goes from 19/21 to 21/21).
//!
//! Per IDA sub_1410FE440: 22 fields matching mac binary __cstring order.
//! Two fixed-loop "list" fields (vehicle_seat_data_list × 16, parent_link × 2)
//! and one CArray<u8> (cargo_seat_index_list).
//!
//! All "_*Action" / "_*Hash" / "_*VoxelType" reads are 4-byte u32s on disk
//! (some flow through u16 dictionary lookups in memory). For round-trip
//! preservation we keep the u32 file representation everywhere.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::VehicleInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  21
// Decoded by dmm-parser:   18
// Missing in this struct:  3
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _sendDamageTo (direct_15B, stream=15)
// ✅ _uiMapTextureInfo (direct_u32, stream=4)
// ✅ _maxAllowableHeight (direct_u32, stream=4)
// ✅ _characterSwitchable (direct_15B, stream=15)
// ✅ _riderSpawnLowerAction (direct_u32, stream=4)
// ✅ _riderSpawnUpperAction (direct_u32, stream=4)
// ✅ _escapeRoadGroupType (direct_15B, stream=15)
// ✅ _vehicleSpawnUpperAction (direct_u32, stream=4)
// ✅ _callVehicleVoxelType (direct_u32, stream=4)
// ✅ _cargoSeatIndexList (direct_u32, stream=4)
// ⏳ _showCountOnUI (direct_15B, stream=15)
// ✅ _isMainDischargeable
// ✅ _isBlocked (direct_15B, stream=15)
// ✅ _stringKey
// ✅ _iconPath (reader_4B, stream=4)
// ✅ _vehicleTypeNameHash (direct_u32, stream=4)
// ⏳ _vehicleSeatDataList (direct_u32, stream=4)
// ✅ _maxVehicleSeat (direct_15B, stream=15)
// ⏳ _parentLinkAttachDataList (direct_u32, stream=4)
// ✅ _maxParentLinkAttachCount (direct_15B, stream=15)
// ✅ _key

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    // 2.04.00 order (Mac reader sub_102066244). Each seat / link entry is two StringInfoKeys
    // (8 bytes, sub_101AD0694), kept as one u64 as before. New: ride-on/off crew audio
    // states, two platform-shake thresholds and removeFogRadius. `_attachToDockingGimmickTag`
    // is now `_seatGimmickDockingTag`; the two trailing u32s are callVehicleVoxelType and
    // checkDistanceToGround.
    pub struct VehicleInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        pub ride_on_crew_audio_state: u8,
        pub show_count_on_ui: u8,
        pub is_platform_vehicle: u8,
        pub send_damage_to: u8,
        pub platform_hit_event_min_attack_impulse_level: u8,
        pub contact_impulse_event: u8,
        pub character_switchable: u8,
        pub ride_off_crew_audio_state: u8,
        pub max_parent_link_attach_count: u8,
        pub max_vehicle_seat: u8,
        pub escape_road_group_type: u8,
        pub icon_path: u32,
        pub rider_detect_info: u16,
        pub ui_map_texture_info: u32,
        pub min_platform_shake_angular_acc_threshold: f32,
        pub min_platform_shake_angular_vel_threshold: f32,
        pub remove_fog_radius: f32,
        pub trailing_u32_110: u32,      // _callVehicleVoxelType
        pub vehicle_spawn_upper_action: u32,
        pub rider_spawn_upper_action: u32,
        pub trailing_u32_111: u32,      // _checkDistanceToGround
        pub attach_to_docking_gimmick_tag: CString<'a>,   // _seatGimmickDockingTag
        pub max_allowable_height: u32,
        pub vehicle_type_name_hash: u32,
        pub vehicle_seat_data_00: u64,
        pub vehicle_seat_data_01: u64,
        pub vehicle_seat_data_02: u64,
        pub vehicle_seat_data_03: u64,
        pub vehicle_seat_data_04: u64,
        pub vehicle_seat_data_05: u64,
        pub vehicle_seat_data_06: u64,
        pub vehicle_seat_data_07: u64,
        pub vehicle_seat_data_08: u64,
        pub vehicle_seat_data_09: u64,
        pub vehicle_seat_data_10: u64,
        pub vehicle_seat_data_11: u64,
        pub vehicle_seat_data_12: u64,
        pub vehicle_seat_data_13: u64,
        pub vehicle_seat_data_14: u64,
        pub vehicle_seat_data_15: u64,
        pub parent_link_attach_data_a: u64,
        pub parent_link_attach_data_b: u64,
        pub cargo_seat_index_list: CArray<u8>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("vehicleinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(VehicleInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "vehicleinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(VehicleInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            VehicleInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
