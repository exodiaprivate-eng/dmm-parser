//! Tier 1 — fully typed parser for `SpawningPoolAutoSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410F9B80: 16 fields. `_spawnList` is `CArray<AutoSpawnEntry>`
//! via sub_1411092E0 + sub_1410FA2A0 (shared with
//! TerrainRegionAutoSpawnInfo; element layout in
//! `crate::binary::variants::auto_spawn_entry`).
//!
//! **spawn_list inter-element separator**: same N-1 pattern as
//! TerrainRegionAutoSpawnInfo — a single `0xff` byte between consecutive
//! elements. Additionally, a mandatory trailing `0xff` byte is written after the
//! last element regardless of count (F4 in the 16-field wire layout). This is
//! decoded as a separate `terminator: u8` field so the roundtrip is lossless and
//! the field remains visible/patchable in JSON.

use crate::binary::*;
use crate::binary::variants::auto_spawn_entry::AutoSpawnEntry;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct SpawningPoolAutoSpawnInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub spawn_list: CArray<AutoSpawnEntry>,
    /// F4 — always `0xff`; mandatory byte written immediately after the last
    /// AutoSpawnEntry (or in place of any entry when count is zero).
    pub terminator: u8,
    pub hash_list: CArray<u32>,
    pub socket_os_name: CString<'a>,
    pub raw_a: u32,
    pub raw_b: u32,
    pub raw_c: u32,
    pub raw_d: u32,
    pub flag_a: u8,
    pub flag_b: u8,
    pub flag_c: u8,
    pub flag_d: u8,
    pub final_u16: u16,
}

impl<'a> SpawningPoolAutoSpawnInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let _ = entry_size; // typed reader is byte-perfect; size is informational

        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;

        // spawn_list: CArray<AutoSpawnEntry> with N-1 inter-element separator bytes
        // (value 0xff) between consecutive elements. For N elements, N-1 separator
        // bytes are inserted between them; entries with a single ASE have no
        // separator; entries with two or more ASEs each contribute one separator byte.
        let spawn_list = {
            let count = u32::read_from(data, offset)? as usize;
            let remaining = data.len().saturating_sub(*offset);
            if count > remaining {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("spawn_list CArray count {} exceeds remaining bytes {} at offset {}",
                        count, remaining, *offset)));
            }
            let mut items = Vec::with_capacity(count.min(1 << 20));
            for i in 0..count {
                items.push(AutoSpawnEntry::read_from(data, offset)?);
                if i + 1 < count {
                    // skip the inter-element separator byte
                    u8::read_from(data, offset)?;
                }
            }
            CArray { items }
        };

        // F4: mandatory trailing 0xff after the last element (always present,
        // even when spawn_list is empty).
        let terminator = u8::read_from(data, offset)?;

        let hash_list = CArray::<u32>::read_from(data, offset)?;
        let socket_os_name = CString::read_from(data, offset)?;
        let raw_a = u32::read_from(data, offset)?;
        let raw_b = u32::read_from(data, offset)?;
        let raw_c = u32::read_from(data, offset)?;
        let raw_d = u32::read_from(data, offset)?;
        let flag_a = u8::read_from(data, offset)?;
        let flag_b = u8::read_from(data, offset)?;
        let flag_c = u8::read_from(data, offset)?;
        let flag_d = u8::read_from(data, offset)?;
        let final_u16 = u16::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked,
            spawn_list, terminator,
            hash_list, socket_os_name,
            raw_a, raw_b, raw_c, raw_d,
            flag_a, flag_b, flag_c, flag_d,
            final_u16,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        // spawn_list: write count then elements with N-1 separator bytes (0xff) between them
        (self.spawn_list.items.len() as u32).write_to(w)?;
        for (i, entry) in self.spawn_list.items.iter().enumerate() {
            entry.write_to(w)?;
            if i + 1 < self.spawn_list.items.len() {
                w.write_all(&[0xff])?;
            }
        }
        // F4: trailing terminator byte (always 0xff)
        self.terminator.write_to(w)?;
        self.hash_list.write_to(w)?;
        self.socket_os_name.write_to(w)?;
        self.raw_a.write_to(w)?;
        self.raw_b.write_to(w)?;
        self.raw_c.write_to(w)?;
        self.raw_d.write_to(w)?;
        self.flag_a.write_to(w)?;
        self.flag_b.write_to(w)?;
        self.flag_c.write_to(w)?;
        self.flag_d.write_to(w)?;
        self.final_u16.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("spawn_list".to_string(), self.spawn_list.to_json_value());
        m.insert("terminator".to_string(), self.terminator.to_json_value());
        m.insert("hash_list".to_string(), self.hash_list.to_json_value());
        m.insert("socket_os_name".to_string(), self.socket_os_name.to_json_value());
        m.insert("raw_a".to_string(), self.raw_a.to_json_value());
        m.insert("raw_b".to_string(), self.raw_b.to_json_value());
        m.insert("raw_c".to_string(), self.raw_c.to_json_value());
        m.insert("raw_d".to_string(), self.raw_d.to_json_value());
        m.insert("flag_a".to_string(), self.flag_a.to_json_value());
        m.insert("flag_b".to_string(), self.flag_b.to_json_value());
        m.insert("flag_c".to_string(), self.flag_c.to_json_value());
        m.insert("flag_d".to_string(), self.flag_d.to_json_value());
        m.insert("final_u16".to_string(), self.final_u16.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        // spawn_list: custom write — N-1 inter-element separator bytes (0xff) between elements
        {
            let v = json_get_field(obj, "spawn_list")?;
            let arr = v.as_array().ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, "spawn_list is not a JSON array"))?;
            w.extend_from_slice(&(arr.len() as u32).to_le_bytes());
            for (i, item) in arr.iter().enumerate() {
                <AutoSpawnEntry as WriteJsonValue>::write_from_json(w, item)?;
                if i + 1 < arr.len() {
                    w.push(0xff);
                }
            }
        }
        // F4: trailing terminator
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "terminator")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "hash_list")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "socket_os_name")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_b")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_c")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_d")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_a")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_b")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_c")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_d")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "final_u16")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("spawningpoolautospawninfo.pabgb") }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = SpawningPoolAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end - start, e));
            assert_eq!(cursor, *end,
                "entry {} key=0x{:x}: cursor {} != end {} (under/over-read by {} bytes)",
                i, key, cursor, end, cursor as i64 - *end as i64);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "spawningpoolautospawninfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
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
            let item = SpawningPoolAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end - start, e));
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
