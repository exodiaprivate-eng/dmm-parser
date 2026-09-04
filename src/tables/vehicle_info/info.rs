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
    pub struct VehicleInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub vehicle_type_name_hash: u32,
        pub icon_path: u32,
        pub max_vehicle_seat: u8,
        // 16 × u64 fixed-loop slots per docstring's "× 16" annotation.
        // Split into individual named fields for JSON-addressable access.
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
        pub max_parent_link_attach_count: u8,
        // Split 2 × 8-byte fixed-loop slots (per docstring "× 2") into u64 pair.
        pub parent_link_attach_data_a: u64,
        pub parent_link_attach_data_b: u64,
        pub rider_spawn_upper_action: u32,
        // GAME VERSION 14 (2026-07-16): `_riderSpawnLowerAction` was REMOVED — every
        // record shrank by exactly 4 bytes and the drift localizes to this spawn-
        // action trio (dropping any one of the three consecutive u32s reconciles
        // the record boundaries; dropping a tail u32 does not). CONFIRMED against
        // the 1.14 binary (IDA Professional 9.0/1.14/CrimsonDesert.exe md5 1454813b):
        // the `_riderSpawnLower` reflection name is present in 1.13 and ABSENT in
        // 1.14, while `_riderSpawnUpper` and `_vehicleSpawnUpper` remain in both.
        // So the two survivors are rider_spawn_upper_action then
        // vehicle_spawn_upper_action; byte-exact roundtrip is unchanged by the
        // rename (same 2×u32 layout).
        pub vehicle_spawn_upper_action: u32,
        pub escape_road_group_type: u8,
        pub cargo_seat_index_list: CArray<u8>,
        // 1.11: _callVehicleVoxelType widened from a single u32 to a CArray<u32>
        // (count + N×u32). Verified via wire-walker: count=1 in most records,
        // count=2 in the 0x424F record; all 34 reconcile.
        // ── 2.01.00 REMOVED `_callVehicleSpawnVoxelTypeList` (this was it: a CArray<u32>
        // that every record carried as count=1, value=4 — the 8 bytes bytediff saw
        // deleted from all 34 records). Kept out rather than commented in: a V3 mod that
        // names it gets an unresolved-field report, which is the truthful outcome.
        pub show_count_on_ui: u8,
        pub ui_map_texture_info: u32,
        pub rider_detect_info: u16,
        pub send_damage_to: u8,
        pub character_switchable: u8,
        pub max_allowable_height: u32,
        // 1.10: one new u32 added to the fixed tail (position length-equivalent
        // within the fixed-width tail). Verified via wire-walker: reconciles
        // all 34 records (byte-exact roundtrip).
        pub trailing_u32_110: u32,
        // ── 1.18.00: `_contactImpulseEvent`, one u8 immediately before the
        // FLT_MAX float below. Pinned by key 0x424A "Ship", whose tail is
        // `… 00 | ff ff 7f 7f | 0a 00 00 00 "controller" | 01` — the new byte
        // brackets the float on the left, and `isPlatformVehicle` on the right.
        pub contact_impulse_event: u8,
        // 1.11: one new 4-byte field appended to the fixed tail (a float —
        // 0x7f7fffff = FLT_MAX default, or real values e.g. 0x44a8c000 = 1350.0);
        // kept as u32 for bit-exact roundtrip. The other +4 bytes of the 1.11
        // growth came from call_vehicle_voxel_type widening to a CArray (above).
        // Verified via wire-walker: all 34 records byte-exact.
        pub trailing_u32_111: u32,
        // The binary's LAST field, `_attachToDockingGimmickTag` — a CString, not
        // a u32. The 1.13 pass could not tell: it is empty (len 0 → the four
        // bytes `00 00 00 00`) in every record that existed before 1.16, so a u32
        // read consumed exactly the same 4 bytes and round-tripped.
        //
        // 1.16.00 added the first record that fills it: key 0x424a "Ship", whose
        // tail is `0a 00 00 00` + "controller". Nothing else in this table drifted
        // — all 33 pre-existing records are byte-identical to the v14 fixture, so
        // this one type change is the whole 1.16 delta.
        //
        // ⚠ RENAME (deliberate, not silent): `trailing_u32_113` -> this. Changing
        // u32 -> CString already breaks the JSON contract for that field, so the
        // name had to move with it rather than keep "u32" in the name of a string
        // field. Vanilla value was empty everywhere, so nothing could have set it
        // meaningfully.
        pub attach_to_docking_gimmick_tag: CString<'a>,
        // ── 1.18.00: `_isPlatformVehicle`, one u8 appended after the docking
        // tag string. 0 in 33 records and **1 in key 0x424A "Ship"** — the one
        // vehicle you can walk around on. Semantic confirmation, not just a
        // width that happens to fit.
        pub is_platform_vehicle: u8,
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
