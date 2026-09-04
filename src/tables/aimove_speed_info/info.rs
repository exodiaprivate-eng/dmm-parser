//! IDA-derived parser for `AIMoveSpeedInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.

use crate::binary::*;
use crate::py_binary_struct;

// 2.01.00 (2026-09-03) — the six presence-flagged slots are UNCHANGED (the list reader
// sub_10207C31C still reads a u8 flag and allocates a 0x84-byte element, six times over).
// What changed is the element (sub_10201454C, 127 -> 131 wire bytes): 2.01 split
// `_accPreventDistanceAfterCurve` into `_min` / `_max`. `moveAcc` / `moveDcc` are read as
// four 8-byte values each, i.e. the same 32 bytes as the eight f32 exposed here; the f32
// names stay because they are the mod contract. Record count went 189 -> 346 (data).
py_binary_struct! {
    pub struct AIMoveSpeedData {
        pub target_move_speed: f32,
        pub min_move_speed: f32,
        pub move_acc_0: f32, pub move_acc_1: f32, pub move_acc_2: f32, pub move_acc_3: f32,
        pub move_acc_4: f32, pub move_acc_5: f32, pub move_acc_6: f32, pub move_acc_7: f32,
        pub move_dcc_0: f32, pub move_dcc_1: f32, pub move_dcc_2: f32, pub move_dcc_3: f32,
        pub move_dcc_4: f32, pub move_dcc_5: f32, pub move_dcc_6: f32, pub move_dcc_7: f32,
        pub look_forward_sec: f32,
        pub look_forward_turn_sec: f32,
        pub min_degree_diff: f32,
        pub max_degree_diff: f32,
        pub rotation_damping: f32,
        pub max_rotation_speed: f32,
        // 2.01.00: was one `acc_prevent_distance_after_curve`; now a min/max pair. The
        // old name is gone rather than aliased — a mod that set it must choose a bound.
        pub acc_prevent_distance_after_curve_min: f32,
        pub acc_prevent_distance_after_curve_max: f32,
        pub min_degree_diff_stride: f32,
        pub max_degree_diff_stride: f32,
        pub min_move_speed_stride: f32,
        pub min_distance_rotate_to_target: f32,
        pub max_distance_rotate_to_target: f32,
        pub speed_down_distance_before_curve_limit: f32,
        pub acc_count: u8,
        pub dcc_count: u8,
        pub rotate_to_target_sync_with_ik: u8,
    }
}
py_binary_struct! {
    pub struct AIMoveSpeedInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub slot_0: COptional<AIMoveSpeedData>,
        pub slot_1: COptional<AIMoveSpeedData>,
        pub slot_2: COptional<AIMoveSpeedData>,
        pub slot_3: COptional<AIMoveSpeedData>,
        pub slot_4: COptional<AIMoveSpeedData>,
        pub slot_5: COptional<AIMoveSpeedData>,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("aimovespeedinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(AIMoveSpeedInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "aimovespeedinfo roundtrip bytes mismatch");
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
            items.push(AIMoveSpeedInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            AIMoveSpeedInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
