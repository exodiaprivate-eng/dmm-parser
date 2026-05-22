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

// 1.0.8: FieldInfo was completely restructured (CString replaced hash fields,
// several fields changed type). Switched to pabgh_blob_table for safe roundtrip.
crate::pabgh_blob_table! {
    pub struct FieldInfo<'a> {
        key: u32,
        blob_field: body,
    }
}

impl<'a> FieldInfo<'a> {
    pub fn to_json_dict(&self) -> serde_json::Map<String, serde_json::Value> {
        use base64::Engine;
        let mut m = serde_json::Map::new();
        m.insert("key".into(), serde_json::Value::from(self.key));
        m.insert("string_key".into(), serde_json::Value::from(
            std::str::from_utf8(self.string_key.data.as_bytes()).unwrap_or("")));
        m.insert("is_blocked".into(), serde_json::Value::from(self.is_blocked));
        m.insert("_body_b64".into(), serde_json::Value::from(
            base64::engine::general_purpose::STANDARD.encode(&self.body)));
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &serde_json::Map<String, serde_json::Value>) -> std::io::Result<()> {
        use crate::binary::BinaryWrite;
        use base64::Engine;
        let key = obj.get("key").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        key.write_to(w)?;
        let sk = obj.get("string_key").and_then(|v| v.as_str()).unwrap_or("");
        (sk.len() as u32).write_to(w)?;
        w.extend_from_slice(sk.as_bytes());
        let blocked = obj.get("is_blocked").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        blocked.write_to(w)?;
        if let Some(b64) = obj.get("_body_b64").and_then(|v| v.as_str()) {
            let body = base64::engine::general_purpose::STANDARD.decode(b64)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            w.extend_from_slice(&body);
        }
        Ok(())
    }
}
