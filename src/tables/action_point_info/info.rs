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

// 1.0.8: ActionPointInfo was completely restructured (two ActionPoint blocks
// merged, records grew from ~78B to ~258B). Switched to pabgh_blob_table
// for safe roundtrip.
crate::pabgh_blob_table! {
    pub struct ActionPointInfo<'a> {
        key: u32,
        blob_field: body,
    }
}

impl<'a> ActionPointInfo<'a> {
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

// Tests removed: ActionPointInfo was switched to pabgh_blob_table! in 1.0.8
// (record layout changed significantly). Blob tables use read_with_size +
// pabgh-driven loops, not read_from. Coverage via dispatch integration tests.
