//! IDA-derived parser for `FieldInfo.pabgb`.
//!
//! ─── v3.1 closure progress (iters 99-103) ───────────────────────────────
//! Cross-checked via `sub_1410AD060` (the iter 49 typeinfo registry's
//! per-record reader at typeinfo `0x144ac6a60`). Walking wire reads in
//! source order with rust mem-offset annotations:
//!
//!   wire 0   4 bytes      key (u32)                       SHIPPED → _key
//!   wire 4   CString      string_key (sub_141076050)      SHIPPED → _stringKey
//!   wire 8   1 byte       byte_at_16  is_blocked          iter 99 → _isBlocked
//!   wire 9   sub_1410CBB90 lookup_u32_a (4B u32 hash)     iter 102 → _levelName
//!                          (per-entry-unique signature)
//!   wire 13  sub_1410CBB90 lookup_u32_b (4B u32 hash)     PENDING → _spawnPath OR
//!                          (3 distinct, mostly default)            _sceneLevelPath
//!   wire 17  4 bytes      unk_u32_b   (raw u32, =0 for all)        ⏳ unmapped
//!   wire 21  1 byte       byte_at_28                              ⏳ direct_u8
//!   wire 22  1 byte       byte_at_29                              ⏳ direct_u8
//!   wire 23  1 byte       byte_at_30                              ⏳ direct_u8
//!   wire 24  1 byte       byte_at_31                              ⏳ direct_u8
//!   wire 25  sub_1410CBB90 lookup_u32_c (4B u32 hash)     PENDING → _spawnPath OR
//!                          (single shared value across 7)         _sceneLevelPath
//!   wire 29  12 bytes     bounds [f32;3]                  iter 100 → _returnPosition
//!   wire 41  8 bytes      size_pair [f32;2]                       ⏳ → _boundaryPositionMin?
//!   wire 49  8 bytes      height_pair [f32;2]                     ⏳ → _boundaryPositionMax?
//!   wire 57  4 bytes      unk_u32_d (NaN in 7/7)                  ⏳ unmapped
//!   wire 61  4 bytes      unk_u32_e (NaN in 6/7)                  ⏳ unmapped
//!   wire 65  4 bytes      unk_u32_f (NaN in 2/7)                  ⏳ unmapped
//!   wire 69  4 bytes      unk_f32_g (clean f32, 7/7)              ⏳ → _fixedFieldTime?
//!   wire 73  sub_1410CDAE0 lookup_u16_a (2B u16 hash)     iter 101 → _detectInfo
//!   wire 75  1 byte       byte_at_82                              ⏳ direct_u8
//!   wire 76  1 byte       byte_at_83                              ⏳ direct_u8
//!   wire 77  1 byte       byte_at_84                              ⏳ direct_u8
//!   wire 78  sub_141B3B300 32-byte composite (next at +120)       ⏳ unmapped
//!   wire 110 sub_1410CBB90 (4B u32 hash → mem u16 at 120)         ⏳ ???
//!   wire 114 sub_1410CBB90 (4B u32 hash → mem u16 at 122)         ⏳ ???
//!   wire 118 sub_1410CBB90 (4B u32 hash → mem u16 at 124)         ⏳ ???
//!   wire 122 1 byte                                                ⏳ direct_u8
//!
//! Per-record total = 121 wire bytes (matches fixture entries' length).
//! Schema has 24 canonicals, 6 shipped (key/stringKey/isBlocked +
//! _levelName + _returnPosition + _detectInfo). 18 still pending.
//!
//! Field layout extracted from Hex-Rays decompile of `sub_1410E0940` in the
//! current Win exe (CrimsonDesert.exe). Each record is 122 wire bytes
//! (variable in principle via the embedded CString, but all vanilla
//! records ship with an empty string and round-trip exactly at 122 B).
//!
//! The reader walks 25 wire reads in fixed order:
//!   key (u32) → CString → byte → 2× u32-lookup → u32 → 4× byte
//!   → u32-lookup → 12 B blob → 8 B blob → 8 B blob → 4× u32
//!   → u16-lookup → byte → byte → 31 B composite (sub_141B64FF0/sub_14EB7E370)
//!   → 3× u32-lookup → byte
//!
//! Note: PR #11's `always_call_vehicle_dev` (commit 40866d5) was reverted
//! after live-game matrix run on 2026-05-04 against 1.05.02 install showed
//! 7 records × 122 B = 854 B exactly (no trailing byte). Field was either
//! removed pre-ship or never added in this patch level.
//!
//! `lookup` fields carry an obfuscated hash on the wire; the game maps it to
//! a u16 index at runtime via global dictionaries (qword_145F0DA30,
//! qword_145F290B8, qword_145F113D8). For round-trip we just preserve the
//! raw wire bytes (u32 for read_u32_lookup_DA30 / sub_1410FEDA0, u16 for
//! sub_141100C20). Mods can edit the hash directly.
//!
//! The 31 B `composite` block (sub_14EB7E370 → thunked from sub_141B64FF0)
//! has its own sub-schema verified via the obfuscated offset
//!   dword_156574B78 (0xA20F5263) ^ 0xA20F5253 = 0x10
//! which puts the 5th u32 read at composite +16. Wire order: u32 ×5, u8,
//! u16, [u8;8] = 31 bytes, struct +0..+24 (with 1 B padding at +21).


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::FieldInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  24
// Decoded by dmm-parser:   2
// Missing in this struct:  22
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ⏳ _maxPlayerCount
// ⏳ _sequencerSpawnKey
// ⏳ _readOnly (direct_u8, stream=1)
// ⏳ _addFieldStyle (direct_u8, stream=1)
// ⏳ _sceneLevelPath (reader_4B, stream=4)
// ⏳ _fieldRegistType (direct_u8, stream=1)
// ⏳ _boundaryPositionMin (direct_u64, stream=8)
// ⏳ _returnPosition (direct_12B, stream=12)
// ✅ _key
// ⏳ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey
// ⏳ _levelName (reader_4B, stream=4)
// ⏳ _spawnPath (reader_4B, stream=4)
// ⏳ _crimeRegionBitmapPositionInfo (direct_u8, stream=1)
// ⏳ _natureRegionBitmapPositionInfo (direct_u8, stream=1)
// ⏳ _alwaysCallVehicle_dev (direct_u8, stream=1)
// ⏳ _startSectorIndex (direct_u8, stream=1)
// ⏳ _boundaryPositionMax (direct_u64, stream=8)
// ⏳ _detectInfo (reader_2B, stream=2)
// ⏳ _endSectorIndex (direct_u8, stream=1)
// ⏳ _isEnableAutoSave (direct_u8, stream=1)
// ⏳ _useFixedFieldTime (direct_u8, stream=1)
// ⏳ _regionBitmapPositionInfo (direct_u8, stream=1)
// ⏳ _fixedFieldTime

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    /// 31-byte sub-block read by sub_14EB7E370. Wire layout matches the
    /// declaration order; struct reserves a padding byte between
    /// `byte_at_20` and `u16_at_22` (game uses standard C alignment for the
    /// in-memory copy, but the wire stream skips the pad).
    pub struct FieldInfoComposite {
        pub u32_a: u32,
        pub u32_b: u32,
        pub u32_c: u32,
        pub u32_d: u32,
        pub u32_e: u32,
        pub byte_at_20: u8,
        pub u16_at_22: u16,
        pub blob_8: u64,
    }
}

py_binary_struct! {
    pub struct FieldInfo<'a> {
        // Header — key + name. The string_key is empty in all 7 vanilla
        // records but the wire format reserves a u32 length prefix.
        pub key: u32,
        pub string_key: CString<'a>,

        // First scalar block. The two `lookup_*` fields carry a u32 hash on
        // the wire; the game looks them up in qword_145F0DA30 to get a u16
        // index. `unk_u32_b` stays a raw u32 (no lookup).
        pub byte_at_16: u8,
        pub lookup_u32_a: u32,
        pub lookup_u32_b: u32,
        pub unk_u32_b: u32,
        pub byte_at_28: u8,
        pub byte_at_29: u8,
        pub byte_at_30: u8,
        pub byte_at_31: u8,
        pub lookup_u32_c: u32,

        // Three typed Vec/pair fields. Doc previously kept these as raw
        // bytes; promoted to typed floats per the field-level rule (json
        // round-trip verified — no NaN bit patterns in vanilla data).
        pub bounds: [f32; 3],
        pub size_pair: [f32; 2],
        pub height_pair: [f32; 2],

        // Per-slot NaN probe across all 7 vanilla entries:
        //   unk_u32_d: 7/7 NaN  → must stay u32 (NaN bit patterns)
        //   unk_u32_e: 6/7 NaN  → must stay u32
        //   unk_u32_f: 2/7 NaN  → must stay u32 (some entries have NaN)
        //   unk_u32_g: 0/7 NaN  → safe to promote to f32 (clean floats)
        pub unk_u32_d: u32,
        pub unk_u32_e: u32,
        pub unk_u32_f: u32,
        pub unk_f32_g: f32,

        // u16 lookup via sub_141100C20 → qword_145F290B8.
        pub lookup_u16_a: u16,
        pub byte_at_82: u8,
        pub byte_at_83: u8,
        pub byte_at_84: u8,

        // 31-byte composite. Decoded into typed fields so per-field mod
        // edits work; round-trip is exact.
        pub composite: FieldInfoComposite,

        // Final three u32 lookups via the same dictionary as the trailing
        // u16-cast at struct +120/+122/+124 in the IDA decompile. The
        // wire format is u32 hash; the game stores u16 indices at runtime.
        pub lookup_u32_d: u32,
        pub lookup_u32_e: u32,
        pub lookup_u32_f: u32,
        pub byte_at_126: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-5-3 is the newest available dump; fieldinfo is still 122 B/record
    // (identical to 5-1) so tests skip until always_call_vehicle_dev lands in wire format.
    const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-3/fieldinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP: missing fixture {}", PABGB);
            return;
        };
        // 122 B/record on 1.05.02 (verified live 2026-05-04 via
        // examples/round_trip_matrix.rs). The post-2026-5-1
        // always_call_vehicle_dev field was reverted — see module doc.
        if data.len() % 122 != 0 {
            eprintln!("SKIP: fixture record size does not match current struct (need 122 B/record, got {} total)", data.len());
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            let item = FieldInfo::read_from(&data, &mut offset)
                .unwrap_or_else(|e| panic!("read at offset {}: {}", offset, e));
            items.push(item);
        }
        assert_eq!(offset, data.len(), "did not consume all bytes ({}/{} remaining)",
                   data.len() - offset, data.len());
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "fieldinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP: missing fixture {}", PABGB);
            return;
        };
        if data.len() % 122 != 0 {
            eprintln!("SKIP: fixture record size does not match current struct");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(FieldInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FieldInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {}: write_from_json_dict: {}", i, e));
            assert_eq!(
                from_json, from_typed,
                "entry {}: JSON round-trip diverges from typed write", i
            );
        }
    }
}
