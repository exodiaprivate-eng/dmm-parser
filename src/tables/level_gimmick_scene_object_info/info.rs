//! Tier 1 — fully typed parser for `LevelGimmickSceneObjectInfo.pabgb`.
//!
//! ─── v3.1 closure analysis (iter 80) ────────────────────────────────────
//! Decompiled `sub_1410B7EB0` (iter 60 typeinfo registry). The 7 sequential
//! u8 reads at offsets 72-78 pair with 7 rust u8 fields:
//!   72→show_icon_condition_type, 73→use_teleport, 74→use_guide_effect,
//!   75→is_sub_inner_gimmick, 76→check_game_level_load_state,
//!   77→unk_new_u8_a, 78→unk_new_u8_b
//!
//! Schema has only 24 canonicals — rust struct adds 3 extras. Of the 2
//! "missing" canonicals:
//!   `_levelGimmickSceneObjectDataList` → SHIPPED (data_list alias added
//!                                        in MANUAL_OVERRIDES).
//!   `_onDiscoverOnlyEnable` → maps to either `unk_new_u8_a` or
//!                             `unk_new_u8_b` (both u8). Resolving which
//!                             needs string-xref work (function-string-
//!                             associate plugin against the byte's
//!                             error-message anchor). Deferred to a
//!                             future iter; partial close (1 of 2) for
//!                             this iter.
//!
//! Per IDA sub_1410EB480: 25 fields. `_levelGimmickSceneObjectDataList`
//! is a `CArray<LevelGimmickSceneObjectData>` via sub_14110ECD0 +
//! sub_1410EB270. Despite the original "polymorphic" label, sub_1410EB270
//! is a fixed-shape reader: 4× u32 (raw + 3 hash lookups) + CString +
//! u32 + u32-hash + 16 raw + u32 + CString + 2× SceneObjectAA1B0Block
//! (each = Vec3 + [u32;4] + Vec3) + 12 raw bytes. Mem stride 160.

crate::pabgh_blob_table! {
    pub struct LevelGimmickSceneObjectInfo<'a> {
        key: u32,
        blob_field: body,
    }
}

impl<'a> LevelGimmickSceneObjectInfo<'a> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("levelgimmicksceneobjectinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = LevelGimmickSceneObjectInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "levelgimmicksceneobjectinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: pabgh not found");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = LevelGimmickSceneObjectInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            LevelGimmickSceneObjectInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
