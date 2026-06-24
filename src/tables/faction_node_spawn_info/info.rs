//! Tier 1 — fully typed parser for `FactionNodeSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410DED80: 6 fields. _patrolAISplineDataList is a
//! `CArray<COptional<{u8 + CArray<PatrolSplineElement>}>>` (sub_1413F8A20
//! outer, sub_1413F9BD0 inner). Each PatrolSplineElement is 65 wire bytes
//! / 68 mem bytes (Vec3 + 4× u32 + f32 + u8 + 2× Vec3 + f32 + u32).
//!
//! ─── v3.1 closure analysis (iter 74) ────────────────────────────────────
//! Cross-check via `sub_1410AB470` (typeinfo→record-reader path per the
//! iter 53 typeinfo registry). 7 wire reads:
//!
//!   word offset 0   4 bytes        → _key (u32)
//!   word offset 2   CString        → _stringKey  (sub_141076050)
//!   word offset 4   1 byte         → _isBlocked
//!   byte offset 18  sub_1410CEB40  → _factionNodeInfo (u16 hash lookup)
//!   word offset 5   12 bytes raw   → boundary_box_min  ([f32; 3])
//!   word offset 8   12 bytes raw   → boundary_box_max  ([f32; 3])
//!   word offset 12  sub_1410E2B00  → patrol_ai_spline_data_list
//!
//! NattKh schema lists `_boundaryBox` as a single 24-byte canonical at
//! this position; the rust struct already splits it into Vec3 min + Vec3
//! max. Two consecutive identical 12-byte raw reads at adjacent offsets
//! prove **`_boundaryBox` is a pure 1-to-2 wrapper** around
//! `boundary_box_min` + `boundary_box_max`. Closure path: 1-to-N alias
//! entry. No new decoder work needed.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// `sub_1413F9BD0` per-element. 65 wire bytes, 68 mem bytes.
    pub struct PatrolSplineElement {
        pub vec_a: [f32; 3],   // 12 bytes raw at mem +0
        pub block_a: [u32; 4], // 16 bytes via sub_1410AA0D0 (4× u32) at mem +12
        pub raw_a: f32,        // 4 bytes raw at mem +28
        pub flag_a: u8,        // 1 byte raw at mem +32
        pub vec_b: [f32; 3],   // 12 bytes raw at mem +36
        pub vec_c: [f32; 3],   // 12 bytes raw at mem +48
        pub raw_b: f32,        // 4 bytes raw at mem +60
        pub raw_c: u32,        // 4 bytes raw at mem +64
    }
}

/// `sub_141115890` per-element. Wire: 16 raw header bytes (single
/// 16-byte block per IDA — read as one __int128 memcpy, no internal
/// substructure visible to the game-side reader) followed by a
/// `COptional<PatrolSplineGroup>`. Header bytes split into 4× u32
/// for JSON field addressability (semantics not yet recovered; lane-c
/// 2026-04-30 promotion from [u8;16] for field-level access).
#[derive(Debug)]
pub struct PatrolSplineEntry {
    pub header_dword_0: u32,
    pub header_dword_1: u32,
    pub header_dword_2: u32,
    pub header_dword_3: u32,
    pub group: OptionalPatrolSplineGroup,
}

impl<'a> BinaryRead<'a> for PatrolSplineEntry {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let header_dword_0 = u32::read_from(data, offset)?;
        let header_dword_1 = u32::read_from(data, offset)?;
        let header_dword_2 = u32::read_from(data, offset)?;
        let header_dword_3 = u32::read_from(data, offset)?;
        let group = OptionalPatrolSplineGroup::read_from(data, offset)?;
        Ok(Self { header_dword_0, header_dword_1, header_dword_2, header_dword_3, group })
    }
}

impl BinaryWrite for PatrolSplineEntry {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.header_dword_0.write_to(w)?;
        self.header_dword_1.write_to(w)?;
        self.header_dword_2.write_to(w)?;
        self.header_dword_3.write_to(w)?;
        self.group.write_to(w)
    }
}

impl ToJsonValue for PatrolSplineEntry {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("header_dword_0".to_string(), self.header_dword_0.to_json_value());
        m.insert("header_dword_1".to_string(), self.header_dword_1.to_json_value());
        m.insert("header_dword_2".to_string(), self.header_dword_2.to_json_value());
        m.insert("header_dword_3".to_string(), self.header_dword_3.to_json_value());
        m.insert("group".to_string(), self.group.to_json_value());
        Value::Object(m)
    }
}

impl WriteJsonValue for PatrolSplineEntry {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "PatrolSplineEntry: expected object",
        ))?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "header_dword_0")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "header_dword_1")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "header_dword_2")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "header_dword_3")?)?;
        OptionalPatrolSplineGroup::write_from_json(w, json_get_field(obj, "group")?)
    }
}

/// `sub_1413F8A20` — `u8 presence + (if present: u8 + CArray<PatrolSplineElement>)`.
#[derive(Debug)]
pub struct OptionalPatrolSplineGroup {
    pub inner: Option<PatrolSplineGroup>,
}

#[derive(Debug)]
pub struct PatrolSplineGroup {
    pub flag: u8,
    pub elements: CArray<PatrolSplineElement>,
}

impl<'a> BinaryRead<'a> for OptionalPatrolSplineGroup {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let presence = u8::read_from(data, offset)?;
        let inner = if presence != 0 {
            let flag = u8::read_from(data, offset)?;
            let elements = <CArray<PatrolSplineElement>>::read_from(data, offset)?;
            Some(PatrolSplineGroup { flag, elements })
        } else {
            None
        };
        Ok(Self { inner })
    }
}

impl BinaryWrite for OptionalPatrolSplineGroup {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(g) => { 1u8.write_to(w)?; g.flag.write_to(w)?; g.elements.write_to(w) }
            None => 0u8.write_to(w),
        }
    }
}

impl ToJsonValue for OptionalPatrolSplineGroup {
    fn to_json_value(&self) -> Value {
        match &self.inner {
            Some(g) => {
                let mut m = Map::new();
                m.insert("flag".to_string(), g.flag.to_json_value());
                m.insert("elements".to_string(), g.elements.to_json_value());
                Value::Object(m)
            }
            None => Value::Null,
        }
    }
}

impl WriteJsonValue for OptionalPatrolSplineGroup {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            0u8.write_to(w)
        } else {
            let obj = v.as_object().ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "OptionalPatrolSplineGroup: expected object or null",
            ))?;
            1u8.write_to(w)?;
            <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag")?)?;
            <CArray<PatrolSplineElement> as WriteJsonValue>::write_from_json(
                w, json_get_field(obj, "elements")?,
            )
        }
    }
}

#[derive(Debug)]
pub struct FactionNodeSpawnInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub faction_node_info: u32,
    /// 3D bounding box (Vec3 min, Vec3 max) — 24 wire bytes total.
    pub boundary_box_min: [f32; 3],
    pub boundary_box_max: [f32; 3],
    pub patrol_ai_spline_data_list: CArray<PatrolSplineEntry>,
}

impl<'a> FactionNodeSpawnInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start + entry_size;

        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let faction_node_info = u32::read_from(data, offset)?;
        let boundary_box_min = <[f32; 3]>::read_from(data, offset)?;
        let boundary_box_max = <[f32; 3]>::read_from(data, offset)?;
        let patrol_ai_spline_data_list = <CArray<PatrolSplineEntry>>::read_from(data, offset)?;
        if *offset != entry_end {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("FactionNodeSpawnInfo: under/over-read (cursor {} expected {})",
                    *offset, entry_end)));
        }

        Ok(Self {
            key, string_key, is_blocked, faction_node_info,
            boundary_box_min, boundary_box_max, patrol_ai_spline_data_list,
        })
    }

    pub fn read_tracked_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let entry_end = *offset + entry_size;
        let key = track_read_field::<u32>(data, offset, path, ranges, "key", "u32")?;
        let string_key = track_read_field::<CString<'a>>(data, offset, path, ranges, "string_key", "CString")?;
        let is_blocked = track_read_field::<u8>(data, offset, path, ranges, "is_blocked", "u8")?;
        let faction_node_info = track_read_field::<u32>(data, offset, path, ranges, "faction_node_info", "u32")?;
        let boundary_box_min = track_read_field::<[f32; 3]>(data, offset, path, ranges, "boundary_box_min", "[f32;3]")?;
        let boundary_box_max = track_read_field::<[f32; 3]>(data, offset, path, ranges, "boundary_box_max", "[f32;3]")?;
        let patrol_ai_spline_data_list = track_read_field::<CArray<PatrolSplineEntry>>(data, offset, path, ranges, "patrol_ai_spline_data_list", "CArray<PatrolSplineEntry>")?;
        if *offset != entry_end {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("FactionNodeSpawnInfo: under/over-read (cursor {} expected {})", *offset, entry_end)));
        }
        Ok(Self {
            key, string_key, is_blocked, faction_node_info,
            boundary_box_min, boundary_box_max, patrol_ai_spline_data_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.faction_node_info.write_to(w)?;
        self.boundary_box_min.write_to(w)?;
        self.boundary_box_max.write_to(w)?;
        self.patrol_ai_spline_data_list.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("faction_node_info".to_string(), self.faction_node_info.to_json_value());
        m.insert("boundary_box_min".to_string(), self.boundary_box_min.to_json_value());
        m.insert("boundary_box_max".to_string(), self.boundary_box_max.to_json_value());
        m.insert("patrol_ai_spline_data_list".to_string(),
            self.patrol_ai_spline_data_list.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "faction_node_info")?)?;
        <[f32; 3] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "boundary_box_min")?)?;
        <[f32; 3] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "boundary_box_max")?)?;
        <CArray<PatrolSplineEntry> as WriteJsonValue>::write_from_json(
            w, json_get_field(obj, "patrol_ai_spline_data_list")?,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("factionnodespawninfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = FactionNodeSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionnodespawninfo roundtrip bytes mismatch");
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
            let item = FactionNodeSpawnInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FactionNodeSpawnInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
