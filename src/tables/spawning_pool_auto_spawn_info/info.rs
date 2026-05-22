//! Tier 1 — fully typed parser for `SpawningPoolAutoSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410F9B80: 16 fields. `_spawnList` is `CArray<AutoSpawnEntry>`
//! via sub_1411092E0 + sub_1410FA2A0 (shared with
//! TerrainRegionAutoSpawnInfo; element layout in
//! `crate::binary::variants::auto_spawn_entry`).

crate::pabgh_blob_table! {
    pub struct SpawningPoolAutoSpawnInfo<'a> {
        key: u32,
        blob_field: body,
    }
}

impl<'a> SpawningPoolAutoSpawnInfo<'a> {
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

    const PABGB_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/spawningpoolautospawninfo.pabgb";
    const PABGH_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/spawningpoolautospawninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = SpawningPoolAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "spawningpoolautospawninfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH_PATH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = SpawningPoolAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            SpawningPoolAutoSpawnInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
