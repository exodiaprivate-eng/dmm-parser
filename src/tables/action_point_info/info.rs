//! Hand-corrected: IDA-derived parser for `ActionPointInfo.pabgb`.
//!
//! Per IDA sub_1410D5120 (outer): u32 key, CString string_key, u8 is_blocked,
//! ActionPoint action_point (sub_1410D4FE0), u32 level_action_point_info
//! (sub_1410FED30 lookup).
//!
//! Per IDA sub_1410D4FE0 + sub_1410D4DF0 (ActionPoint inner reader):
//!   sub_1410D4DF0 reads (in disk order): u32, [u8;24], u32, [u8;16], u32, u32 = 56 bytes
//!   sub_1410D4FE0 then reads: u32 (lookup), u32, u32, u32 (lookup),
//!   [u8;12], u32 = 32 bytes
//! Total ActionPoint disk size = 88 bytes.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::ActionPointInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  6
// Decoded by dmm-parser:   4
// Missing in this struct:  2
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _levelActionPointInfo
// ✅ _key
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey
// ⏳ _actionYaw (direct_u32, stream=4)
// ⏳ _actionPosition (direct_12B, stream=12)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct ActionPoint {
        pub field_a: u32,
        // block_a [u8;24] split via empirical per-slot probe across all
        // 25988 entries: slots 0-3 (bytes 0-15) are always non-NaN f32
        // (zero in vanilla); slots 4-5 (bytes 16-23) are always NaN bit
        // patterns. Tail exposed as 2× u32 raw bit fields so JSON
        // consumers can edit each lane while preserving exact bit
        // pattern (u32 doesn't normalize through serde_json like f32).
        pub block_a_floats: [f32; 4],
        pub block_a_nan_tail_lo: u32,
        pub block_a_nan_tail_hi: u32,
        pub field_b: u32,
        pub block_b: [f32; 4],
        pub field_c: u32,
        pub field_d: u32,
        pub level_action_lookup: u32,
        pub field_e: u32,
        pub field_f: u32,
        pub field_g: u32,
        // block_c[2] (byte offset 84 of ActionPoint) carries NaN bit
        // patterns in action_point_b entries; expose as u32 raw bits to
        // survive serde_json (which serializes f32 NaN as null).
        pub block_c_xy: [f32; 2],
        pub block_c_nan_z: u32,
        pub field_h: u32,
    }
}

py_binary_struct! {
    pub struct ActionPointInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub action_point: ActionPoint,
        pub level_action_point_info: u32,
        pub action_point_b: ActionPoint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/actionpointinfo.pabgb";




    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ActionPointInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "actionpointinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ActionPointInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            ActionPointInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
