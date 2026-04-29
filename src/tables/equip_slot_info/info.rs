//! `equipslotinfo.pabgb` (EquipSlotInfo) — per-class equip-slot rules.
//!
//! Reverse-engineered from IDA decompilation of `sub_141048F10` (record reader)
//! and `sub_141048B40` (EquipInfoData reader). Cross-validated against the
//! Python parser at
//! `CRIMSON-DESERT-SAVE-EDITOR-AND-GAME-MODS/CrimsonGameMods/equipslotinfo_parser.py`
//! which round-trips byte-perfect on vanilla 1.04.
//!
//! ## Why this exists
//! Each pabgh key is a character/class identifier. Per record, an array of
//! `EquipInfoData` entries declares which `equip_type_info` hashes the class
//! may equip in each slot. Adding hashes to `etl_hashes` lets a class wield
//! more weapon/armor categories — this is the data path that "Universal
//! Proficiency" / "Super Mega Mod" / "any-character-holds-any-weapon" mods
//! modify. Field-level v3 mod intents address `entries[i].etl_hashes` to
//! grant new equip permissions without rebuilding the full overlay.
//!
//! ## Wire format
//!
//! Record (per pabgh key, length = pabgh-derived `entry_size`):
//! ```text
//!   key          u32
//!   header_blob  u32 size + bytes      (sub_1410830B0 prefix opaque blob)
//!   flag_u8      u8
//!   flag_u16     u16
//!   list_count   u32
//!   entries      EquipInfoData[list_count]
//!   footer       remaining bytes (entry_size - bytes consumed above)
//! ```
//!
//! `EquipInfoData` (sub_141048B40), 56 bytes plus variable etl_hashes + complex_blob:
//! ```text
//!   etl_hashes   CArray<u32>           ← UNLOCK FIELD: equip_type_info hashes accepted
//!   category_a   u32
//!   category_b   u32
//!   name_hash    u32
//!   slot_index   u16
//!   field_u64    u64
//!   name_hash_2  u32
//!   fields_u32   [u32; 4]
//!   complex_u8   u8
//!   complex_u64  u64
//!   complex_blob CArray<u8>            (u32 size + bytes)
//!   tail_bytes   [u8; 11]
//! ```
//!
//! ## Self-delimitation
//! Records are NOT self-delimiting — the trailing `footer` length is
//! determined by the per-record entry boundary in the pabgh index. Always
//! call `parse_equip_slot_info_to_json_with_pabgh` (or `read_with_size`
//! directly with a known `entry_size`). The non-pabgh `parse_equip_slot_info_to_json`
//! returns an explanatory error rather than guessing.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{json, Value};
use std::io::{self, Write};

// ── EquipInfoData ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct EquipInfoData {
    /// equip_type_info hashes the slot accepts. Add hashes to grant new equip permissions.
    pub etl_hashes: CArray<u32>,
    pub category_a: u32,
    pub category_b: u32,
    pub name_hash: u32,
    pub slot_index: u16,
    pub field_u64: u64,
    pub name_hash_2: u32,
    pub fields_u32: [u32; 4],
    pub complex_u8: u8,
    pub complex_u64: u64,
    pub complex_blob: CArray<u8>,
    pub tail_bytes: [u8; 11],
}

impl<'a> BinaryRead<'a> for EquipInfoData {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let etl_hashes = CArray::<u32>::read_from(data, offset)?;
        let category_a = u32::read_from(data, offset)?;
        let category_b = u32::read_from(data, offset)?;
        let name_hash = u32::read_from(data, offset)?;
        let slot_index = u16::read_from(data, offset)?;
        let field_u64 = u64::read_from(data, offset)?;
        let name_hash_2 = u32::read_from(data, offset)?;
        let fields_u32 = <[u32; 4]>::read_from(data, offset)?;
        let complex_u8 = u8::read_from(data, offset)?;
        let complex_u64 = u64::read_from(data, offset)?;
        let complex_blob = CArray::<u8>::read_from(data, offset)?;
        let tail_bytes = <[u8; 11]>::read_from(data, offset)?;
        Ok(Self {
            etl_hashes, category_a, category_b, name_hash, slot_index,
            field_u64, name_hash_2, fields_u32, complex_u8, complex_u64,
            complex_blob, tail_bytes,
        })
    }
}

impl BinaryWrite for EquipInfoData {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.etl_hashes.write_to(w)?;
        self.category_a.write_to(w)?;
        self.category_b.write_to(w)?;
        self.name_hash.write_to(w)?;
        self.slot_index.write_to(w)?;
        self.field_u64.write_to(w)?;
        self.name_hash_2.write_to(w)?;
        self.fields_u32.write_to(w)?;
        self.complex_u8.write_to(w)?;
        self.complex_u64.write_to(w)?;
        self.complex_blob.write_to(w)?;
        self.tail_bytes.write_to(w)?;
        Ok(())
    }
}

impl ToJsonValue for EquipInfoData {
    fn to_json_value(&self) -> Value {
        json!({
            "etl_hashes": self.etl_hashes.to_json_value(),
            "category_a": self.category_a,
            "category_b": self.category_b,
            "name_hash": self.name_hash,
            "slot_index": self.slot_index,
            "field_u64": self.field_u64,
            "name_hash_2": self.name_hash_2,
            "fields_u32": self.fields_u32.to_json_value(),
            "complex_u8": self.complex_u8,
            "complex_u64": self.complex_u64,
            "complex_blob": self.complex_blob.to_json_value(),
            "tail_bytes": self.tail_bytes.to_json_value(),
        })
    }
}

impl WriteJsonValue for EquipInfoData {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "EquipInfoData: expected object"))?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "etl_hashes")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "category_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "category_b")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "name_hash")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "slot_index")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "field_u64")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "name_hash_2")?)?;
        <[u32; 4] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "fields_u32")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "complex_u8")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "complex_u64")?)?;
        <CArray<u8> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "complex_blob")?)?;
        <[u8; 11] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "tail_bytes")?)?;
        Ok(())
    }
}

// ── EquipSlotInfo (top-level record) ────────────────────────────────────────

#[derive(Debug)]
pub struct EquipSlotInfo {
    pub key: u32,
    /// Opaque prefix blob (sub_1410830B0). Preserved byte-perfect.
    pub header_blob: Vec<u8>,
    pub flag_u8: u8,
    pub flag_u16: u16,
    pub entries: Vec<EquipInfoData>,
    /// Trailing bytes from the end of `entries` to the pabgh-declared record
    /// boundary. Universally ends with `00 00 00 00 7c d8 54 b9` (8 bytes);
    /// some records carry additional opaque per-class data before the
    /// terminator. Preserved byte-perfect — never decoded by this module.
    pub footer: Vec<u8>,
}

impl EquipSlotInfo {
    /// Parse a single record. `entry_size` is the pabgh-declared record length;
    /// without it we cannot determine where the trailing `footer` ends.
    pub fn read_with_size(
        data: &[u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start
            .checked_add(entry_size)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "EquipSlotInfo: entry_size overflow"))?;
        if entry_end > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                format!("EquipSlotInfo: record extends past body ({} > {})", entry_end, data.len())));
        }

        let key = u32::read_from(data, offset)?;
        let blob_size = u32::read_from(data, offset)? as usize;
        if *offset + blob_size > entry_end {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("EquipSlotInfo k={}: header_blob size {} extends past record end", key, blob_size)));
        }
        let header_blob = data[*offset..*offset + blob_size].to_vec();
        *offset += blob_size;

        let flag_u8 = u8::read_from(data, offset)?;
        let flag_u16 = u16::read_from(data, offset)?;
        let list_count = u32::read_from(data, offset)? as usize;

        let mut entries = Vec::with_capacity(list_count);
        for i in 0..list_count {
            let e = EquipInfoData::read_from(data, offset).map_err(|e| io::Error::new(
                e.kind(),
                format!("EquipSlotInfo k={}: entries[{}]: {}", key, i, e),
            ))?;
            entries.push(e);
        }

        if *offset > entry_end {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("EquipSlotInfo k={}: entries over-consumed ({} > {})", key, *offset, entry_end)));
        }
        let footer = data[*offset..entry_end].to_vec();
        *offset = entry_end;

        Ok(Self { key, header_blob, flag_u8, flag_u16, entries, footer })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        (self.header_blob.len() as u32).write_to(w)?;
        w.write_all(&self.header_blob)?;
        self.flag_u8.write_to(w)?;
        self.flag_u16.write_to(w)?;
        (self.entries.len() as u32).write_to(w)?;
        for e in &self.entries {
            e.write_to(w)?;
        }
        w.write_all(&self.footer)?;
        Ok(())
    }
}

impl ToJsonValue for EquipSlotInfo {
    fn to_json_value(&self) -> Value {
        // string_key: empty placeholder so DMM's apply pipeline (which builds
        // by_name and by_key indexes) finds an entry under both maps. Vanilla
        // equipslotinfo records have no native string_key, so by_name lookup
        // resolves to "" for all records — intent authors must address by
        // numeric `key` (the class id), which falls through the by_name miss
        // into the by_key fallback in apply_one_intent.
        json!({
            "key": self.key,
            "string_key": "",
            "header_blob": self.header_blob.iter().copied().collect::<Vec<u8>>(),
            "flag_u8": self.flag_u8,
            "flag_u16": self.flag_u16,
            "entries": Value::Array(self.entries.iter().map(|e| e.to_json_value()).collect()),
            "footer": self.footer.iter().copied().collect::<Vec<u8>>(),
        })
    }
}

impl WriteJsonValue for EquipSlotInfo {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "EquipSlotInfo: expected object"))?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        // header_blob: array of u8 with explicit u32 length prefix.
        let header = json_get_field(obj, "header_blob")?
            .as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EquipSlotInfo.header_blob: expected array"))?;
        w.extend_from_slice(&(header.len() as u32).to_le_bytes());
        for elem in header {
            <u8 as WriteJsonValue>::write_from_json(w, elem)?;
        }
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_u8")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_u16")?)?;
        // entries: explicit u32 count + EquipInfoData each. Note that `entries`
        // does NOT use CArray's WriteJsonValue impl directly because we need
        // to emit `list_count` ourselves (CArray would too, but at this layer
        // the field is conceptually `Vec<EquipInfoData>` not `CArray<...>`).
        let entries = json_get_field(obj, "entries")?
            .as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EquipSlotInfo.entries: expected array"))?;
        w.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        for (i, e) in entries.iter().enumerate() {
            <EquipInfoData as WriteJsonValue>::write_from_json(w, e).map_err(|err| io::Error::new(
                err.kind(), format!("entries[{}]: {}", i, err)))?;
        }
        // footer: raw bytes, no length prefix (length implicit in record boundary).
        let footer = json_get_field(obj, "footer")?
            .as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EquipSlotInfo.footer: expected array"))?;
        for elem in footer {
            <u8 as WriteJsonValue>::write_from_json(w, elem)?;
        }
        Ok(())
    }
}

// ── Public parse/serialize API ──────────────────────────────────────────────

/// Parse equipslotinfo.pabgb — REQUIRES pabgh because records are not
/// self-delimiting (the trailing `footer` has no length prefix). Calling
/// without pabgh returns a clear error. Use
/// `parse_equip_slot_info_to_json_with_pabgh` directly for the normal path.
pub fn parse_equip_slot_info_to_json(_data: &[u8]) -> io::Result<Vec<Value>> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "equip_slot_info records are not self-delimiting; call \
         parse_equip_slot_info_to_json_with_pabgh(body, pabgh) instead",
    ))
}

/// Parse equipslotinfo.pabgb using the sister pabgh index for record
/// boundaries. Returns a JSON dict per record (one per pabgh key), in pabgh
/// order. Output shape mirrors `parse_skill_to_json` so the v3 apply pipeline
/// in DMM treats it uniformly.
pub fn parse_equip_slot_info_to_json_with_pabgh(
    data: &[u8],
    pabgh: &[u8],
) -> io::Result<Vec<Value>> {
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets_from_bytes};
    let entries = load_pabgh_offsets_from_bytes(pabgh).ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, "equip_slot_info: pabgh parse failed"))?;
    let ranges = entry_ranges(&entries, data.len());
    let mut out = Vec::with_capacity(ranges.len());
    for (k, s, e) in ranges {
        let mut c = s;
        let item = EquipSlotInfo::read_with_size(data, &mut c, e - s).map_err(|err| io::Error::new(
            err.kind(),
            format!("equip_slot_info k=0x{:x}: {}", k, err),
        ))?;
        if c != e {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("equip_slot_info k=0x{:x}: under/over-consumed {}/{}", k, c - s, e - s)));
        }
        out.push(item.to_json_value());
    }
    Ok(out)
}

/// Inverse of `parse_equip_slot_info_to_json_with_pabgh`. Writes records back
/// to pabgb bytes in the order given. The pabgh sister file must be rebuilt
/// separately from the resulting offsets — this function does not produce one.
pub fn serialize_equip_slot_info_from_json(items: &[Value]) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(items.len() * 1024);
    for (i, v) in items.iter().enumerate() {
        EquipSlotInfo::write_from_json(&mut out, v).map_err(|e| io::Error::new(
            e.kind(), format!("equip_slot_info[{}]: {}", i, e)))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB: &str = r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\equipslotinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\equipslotinfo.pabgh";

    #[test]
    fn roundtrip_bytes() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP: pabgb missing"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP: pabgh missing"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = EquipSlotInfo::read_with_size(&data, &mut c, *e - *s)
                .unwrap_or_else(|err| panic!("e{} k=0x{:x}: {}", i, k, err));
            assert_eq!(c, *e, "e{} k=0x{:x}: under/over-consumed {}/{}", i, k, c - *s, *e - *s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "EquipSlotInfo binary round-trip mismatch");
    }

    #[test]
    fn roundtrip_json() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP: pabgb missing"); return; };
        let Ok(pabgh) = std::fs::read(PABGH) else { eprintln!("SKIP: pabgh missing"); return; };
        let json = parse_equip_slot_info_to_json_with_pabgh(&data, &pabgh)
            .expect("parse_equip_slot_info_to_json_with_pabgh");
        let out = serialize_equip_slot_info_from_json(&json)
            .expect("serialize_equip_slot_info_from_json");
        assert_eq!(out, data, "EquipSlotInfo JSON round-trip mismatch");
    }

    #[test]
    fn etl_hashes_addressable() {
        // Smoke test: the field that mod intents will target must be reachable
        // by name from JSON output. Without this, "set entries[0].etl_hashes"
        // intents would silently drop on apply.
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP: pabgb missing"); return; };
        let Ok(pabgh) = std::fs::read(PABGH) else { eprintln!("SKIP: pabgh missing"); return; };
        let json = parse_equip_slot_info_to_json_with_pabgh(&data, &pabgh).unwrap();
        let first = json.first().expect("at least one record");
        let entries = first.get("entries").and_then(|v| v.as_array()).expect("entries array");
        let first_entry = entries.first().expect("at least one entry");
        let etl = first_entry.get("etl_hashes").and_then(|v| v.as_array()).expect("etl_hashes array");
        // vanilla key=1 record's first entry has 3 etl_hashes
        assert!(!etl.is_empty(), "etl_hashes must not be empty for class 1 entry 0");
    }
}
