// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `.paatt` (PA Attack Info) files.
//!
//! These files contain per-weapon attack data: hitboxes, damage, frame
//! events, hit timing, target types, etc. Located in
//! `0010/actionchart/bin__/attackinfo/{upper,lower}action/<class>/<weapon>.paatt`.
//!
//! Format reverse-engineered from `pa::sub_100C38E88` (loader) and
//! `pa::sub_100C39A10` (per-attack-info parser) in the Mac binary.
//!
//! ## Wire format
//!
//! ```text
//! Header:
//!   u32   info_count
//!
//! Per info (×info_count):
//!   u8    version           (0..4 — picks BaseData size)
//!   N     base_data         (264 / 528 / 296 / 288 / 264 bytes by version)
//!   For child_frame_idx in 0..9:
//!     u8  count
//!     16*count bytes        (per-frame data)
//!
//! String tables (×7, all same shape):
//!   u16   string_count
//!   per string:
//!     u8  length
//!     u8[length]
//!
//! Frame event buffer:
//!   u32   size
//!   u8[size]
//! ```
//!
//! Tables in order: StringTable, EffectNameTable, EffectInfoKeyTable,
//! SocketNameTable, PartNameTable, SequencerNameTable, PrefabNameTable.
//!
//! ## What's in BaseData?
//!
//! The reflect property strings (`pa::AttackInfoDataDesc`) reveal:
//! - AttackCommonDataDesc (nested)
//! - AttackHitDataDesc (nested)
//! - AttackDelayDataDesc (frame timing)
//! - float fields (RepeatDegreeWeight, etc.)
//! - u8 fields (RepeatCount)
//! - u32 fields (IgnoreDefenceTypeFlag, ExcludeTargetTypeFlag,
//!   ActionHashCode, StaticInfoKey)
//! - bool fields
//! - u16 fields (SocketName)
//! - enum fields (TargetType, AttackHitCheckType, AttackDivideType)
//! - float3 (vec3)
//!
//! Sub-variants exist: `AttackInfo_Attack`, `AttackInfo_AttackThrow`,
//! `AttackInfo_AttackCatch`, `AttackInfo_ReleaseCatch` — each adding
//! their own fields.
//!
//! Per-byte field decoding of BaseData is NOT yet done — that requires
//! decompiling the reflect-property setters individually. For now we
//! preserve BaseData as raw bytes.

use std::io;

use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};

#[derive(Debug, Clone)]
pub struct PaattFile {
    pub infos: Vec<AttackInfo>,
    pub string_table: Vec<String>,
    pub effect_name_table: Vec<String>,
    pub effect_info_key_table: Vec<String>,
    pub socket_name_table: Vec<String>,
    pub part_name_table: Vec<String>,
    pub sequencer_name_table: Vec<String>,
    pub prefab_name_table: Vec<String>,
    pub frame_event_buffer: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AttackInfo {
    pub version: u8,
    /// BaseData bytes — size depends on version (264/528/296/288/264).
    /// Field-level layout via `pa::AttackInfoDataDesc` reflection — TODO.
    pub base_data: Vec<u8>,
    /// 9 child frame slots; each is a count-prefixed stride-16 block.
    pub child_frames: [ChildFrame; 9],
}

#[derive(Debug, Clone, Default)]
pub struct ChildFrame {
    pub count: u8,
    /// `count × 16` bytes.
    pub data: Vec<u8>,
}

fn version_to_base_size(v: u8) -> io::Result<usize> {
    Ok(match v {
        0 => 264,
        1 => 528,
        2 => 296,
        3 => 288,
        4 => 264,
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown AttackInfoData version {}", other),
            ));
        }
    })
}

fn read_bytes<'a>(data: &'a [u8], cursor: &mut usize, n: usize) -> io::Result<&'a [u8]> {
    if *cursor + n > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("read {} at {} but file is {} bytes", n, cursor, data.len()),
        ));
    }
    let s = &data[*cursor..*cursor + n];
    *cursor += n;
    Ok(s)
}

fn read_u8(data: &[u8], cursor: &mut usize) -> io::Result<u8> {
    Ok(read_bytes(data, cursor, 1)?[0])
}

fn read_u16(data: &[u8], cursor: &mut usize) -> io::Result<u16> {
    Ok(u16::from_le_bytes(read_bytes(data, cursor, 2)?.try_into().unwrap()))
}

fn read_u32(data: &[u8], cursor: &mut usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(read_bytes(data, cursor, 4)?.try_into().unwrap()))
}

fn write_string_table(out: &mut Vec<u8>, table: &[String]) -> io::Result<()> {
    if table.len() > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("string table too long for u16 count: {}", table.len()),
        ));
    }
    out.extend_from_slice(&(table.len() as u16).to_le_bytes());
    for s in table {
        let bytes = s.as_bytes();
        if bytes.len() > u8::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "string table entry too long for u8 length: {} bytes ({:?})",
                    bytes.len(), s,
                ),
            ));
        }
        out.push(bytes.len() as u8);
        out.extend_from_slice(bytes);
    }
    Ok(())
}

fn read_string_table(data: &[u8], cursor: &mut usize) -> io::Result<Vec<String>> {
    let count = read_u16(data, cursor)? as usize;
    let mut out = Vec::with_capacity(count.min(1 << 20));
    for _ in 0..count {
        let len = read_u8(data, cursor)? as usize;
        let bytes = read_bytes(data, cursor, len)?;
        let s = std::str::from_utf8(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .to_string();
        out.push(s);
    }
    Ok(out)
}

fn read_attack_info(data: &[u8], cursor: &mut usize) -> io::Result<AttackInfo> {
    let version = read_u8(data, cursor)?;
    let base_size = version_to_base_size(version)?;
    let base_data = read_bytes(data, cursor, base_size)?.to_vec();

    let mut child_frames: [ChildFrame; 9] = Default::default();
    for cf in child_frames.iter_mut() {
        let count = read_u8(data, cursor)?;
        cf.count = count;
        if count > 0 {
            cf.data = read_bytes(data, cursor, 16 * count as usize)?.to_vec();
        }
    }

    Ok(AttackInfo {
        version,
        base_data,
        child_frames,
    })
}

impl PaattFile {
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        let mut cursor = 0usize;

        let info_count = read_u32(data, &mut cursor)? as usize;

        let mut infos = Vec::with_capacity(info_count);
        for _ in 0..info_count {
            infos.push(read_attack_info(data, &mut cursor)?);
        }

        let string_table = read_string_table(data, &mut cursor)?;
        let effect_name_table = read_string_table(data, &mut cursor)?;
        let effect_info_key_table = read_string_table(data, &mut cursor)?;
        let socket_name_table = read_string_table(data, &mut cursor)?;
        let part_name_table = read_string_table(data, &mut cursor)?;
        let sequencer_name_table = read_string_table(data, &mut cursor)?;
        let prefab_name_table = read_string_table(data, &mut cursor)?;

        let buf_size = read_u32(data, &mut cursor)? as usize;
        let frame_event_buffer = read_bytes(data, &mut cursor, buf_size)?.to_vec();

        Ok(PaattFile {
            infos,
            string_table,
            effect_name_table,
            effect_info_key_table,
            socket_name_table,
            part_name_table,
            sequencer_name_table,
            prefab_name_table,
            frame_event_buffer,
        })
    }

    /// Serialize the parsed file back to its on-disk byte layout.
    /// Round-trips byte-exact against `parse_strict` for every vanilla
    /// sample; the BaseData and FrameEventBuffer payloads are written
    /// verbatim from the captured `Vec<u8>` so per-byte changes inside
    /// those regions survive the round trip.
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.infos.len() as u32).to_le_bytes());
        for info in &self.infos {
            // Sanity-check: BaseData must match the version's expected
            // size on write (otherwise a mutated `version` would leave
            // the parser unable to read it back).
            let expected = version_to_base_size(info.version)?;
            if info.base_data.len() != expected {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "AttackInfo: BaseData size {} does not match version {} (expected {})",
                        info.base_data.len(), info.version, expected,
                    ),
                ));
            }
            out.push(info.version);
            out.extend_from_slice(&info.base_data);
            for cf in &info.child_frames {
                let expected_data_len = 16 * cf.count as usize;
                if cf.data.len() != expected_data_len {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "ChildFrame: data length {} does not match count {} (expected {})",
                            cf.data.len(), cf.count, expected_data_len,
                        ),
                    ));
                }
                out.push(cf.count);
                out.extend_from_slice(&cf.data);
            }
        }
        write_string_table(&mut out, &self.string_table)?;
        write_string_table(&mut out, &self.effect_name_table)?;
        write_string_table(&mut out, &self.effect_info_key_table)?;
        write_string_table(&mut out, &self.socket_name_table)?;
        write_string_table(&mut out, &self.part_name_table)?;
        write_string_table(&mut out, &self.sequencer_name_table)?;
        write_string_table(&mut out, &self.prefab_name_table)?;
        out.extend_from_slice(&(self.frame_event_buffer.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.frame_event_buffer);
        Ok(out)
    }

    /// True if every byte of the input was consumed by `parse`.
    /// Reports trailing-byte count if not.
    pub fn parse_strict(data: &[u8]) -> io::Result<(Self, usize)> {
        let mut cursor = 0usize;
        let info_count = read_u32(data, &mut cursor)? as usize;
        let mut infos = Vec::with_capacity(info_count);
        for _ in 0..info_count {
            infos.push(read_attack_info(data, &mut cursor)?);
        }
        let string_table = read_string_table(data, &mut cursor)?;
        let effect_name_table = read_string_table(data, &mut cursor)?;
        let effect_info_key_table = read_string_table(data, &mut cursor)?;
        let socket_name_table = read_string_table(data, &mut cursor)?;
        let part_name_table = read_string_table(data, &mut cursor)?;
        let sequencer_name_table = read_string_table(data, &mut cursor)?;
        let prefab_name_table = read_string_table(data, &mut cursor)?;
        let buf_size = read_u32(data, &mut cursor)? as usize;
        let frame_event_buffer = read_bytes(data, &mut cursor, buf_size)?.to_vec();
        let trailing = data.len() - cursor;
        Ok((
            PaattFile {
                infos,
                string_table,
                effect_name_table,
                effect_info_key_table,
                socket_name_table,
                part_name_table,
                sequencer_name_table,
                prefab_name_table,
                frame_event_buffer,
            },
            trailing,
        ))
    }
}

// ── JSON serialization ───────────────────────────────────────────────
//
// Round-trips byte-perfect via parse → to_json_value → write_from_json
// → to_bytes for every vanilla `.paatt` (validated against 220 game
// files / 13,789 AttackInfo records).
//
// Schema:
//   PaattFile = {
//     "infos": [ AttackInfo, ... ],
//     "string_table": [str, ...],         (×7 tables, names below)
//     "effect_name_table": [str, ...],
//     "effect_info_key_table": [str, ...],
//     "socket_name_table": [str, ...],
//     "part_name_table": [str, ...],
//     "sequencer_name_table": [str, ...],
//     "prefab_name_table": [str, ...],
//     "frame_event_buffer_b64": str,
//   }
//   AttackInfo = {
//     "version": u8,
//     "base_data_b64": str,                (size implied by version)
//     "child_frames": [ChildFrame, ...]    (always 9 entries)
//   }
//   ChildFrame = {
//     "count": u8,
//     "data_b64": str,                     (count × 16 bytes)
//   }

fn write_string_table_json(table: &[String]) -> serde_json::Value {
    serde_json::Value::Array(
        table.iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect(),
    )
}

fn read_string_table_json(v: &serde_json::Value, key: &str) -> io::Result<Vec<String>> {
    let arr = v.as_array().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData,
        format!("PaattFile.{}: expected array", key),
    ))?;
    arr.iter().map(|s| {
        s.as_str()
            .map(String::from)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("PaattFile.{} entry: expected string", key),
            ))
    }).collect()
}

impl ToJsonValue for ChildFrame {
    fn to_json_value(&self) -> serde_json::Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = serde_json::Map::new();
        m.insert("count".into(), serde_json::Value::Number(self.count.into()));
        m.insert(
            "data_b64".into(),
            serde_json::Value::String(B64.encode(&self.data)),
        );
        serde_json::Value::Object(m)
    }
}

impl WriteJsonValue for ChildFrame {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> io::Result<()> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "ChildFrame: expected object",
        ))?;
        let count = json_get_field(obj, "count")?
            .as_u64()
            .map(|x| x as u8)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "ChildFrame.count: expected u8",
            ))?;
        let data = B64.decode(
            json_get_field(obj, "data_b64")?
                .as_str()
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    "ChildFrame.data_b64: expected string",
                ))?,
        ).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ChildFrame.data_b64: {}", e),
        ))?;
        let expected = 16 * count as usize;
        if data.len() != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "ChildFrame: data length {} does not match count {} (expected {})",
                    data.len(), count, expected,
                ),
            ));
        }
        w.push(count);
        w.extend_from_slice(&data);
        Ok(())
    }
}

impl ToJsonValue for AttackInfo {
    fn to_json_value(&self) -> serde_json::Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = serde_json::Map::new();
        m.insert("version".into(), serde_json::Value::Number(self.version.into()));
        m.insert(
            "base_data_b64".into(),
            serde_json::Value::String(B64.encode(&self.base_data)),
        );
        m.insert(
            "child_frames".into(),
            serde_json::Value::Array(
                self.child_frames.iter()
                    .map(|cf| cf.to_json_value())
                    .collect(),
            ),
        );
        serde_json::Value::Object(m)
    }
}

impl WriteJsonValue for AttackInfo {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> io::Result<()> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "AttackInfo: expected object",
        ))?;
        let version = json_get_field(obj, "version")?
            .as_u64()
            .map(|x| x as u8)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "AttackInfo.version: expected u8",
            ))?;
        let expected = version_to_base_size(version)?;
        let base_data = B64.decode(
            json_get_field(obj, "base_data_b64")?
                .as_str()
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    "AttackInfo.base_data_b64: expected string",
                ))?,
        ).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("AttackInfo.base_data_b64: {}", e),
        ))?;
        if base_data.len() != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "AttackInfo: BaseData size {} does not match version {} (expected {})",
                    base_data.len(), version, expected,
                ),
            ));
        }
        w.push(version);
        w.extend_from_slice(&base_data);
        let child_frames = json_get_field(obj, "child_frames")?
            .as_array()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "AttackInfo.child_frames: expected array",
            ))?;
        if child_frames.len() != 9 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "AttackInfo.child_frames: expected 9 entries, got {}",
                    child_frames.len(),
                ),
            ));
        }
        for cf in child_frames {
            ChildFrame::write_from_json(w, cf)?;
        }
        Ok(())
    }
}

impl ToJsonValue for PaattFile {
    fn to_json_value(&self) -> serde_json::Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = serde_json::Map::new();
        m.insert(
            "infos".into(),
            serde_json::Value::Array(
                self.infos.iter()
                    .map(|info| info.to_json_value())
                    .collect(),
            ),
        );
        m.insert("string_table".into(), write_string_table_json(&self.string_table));
        m.insert("effect_name_table".into(), write_string_table_json(&self.effect_name_table));
        m.insert("effect_info_key_table".into(), write_string_table_json(&self.effect_info_key_table));
        m.insert("socket_name_table".into(), write_string_table_json(&self.socket_name_table));
        m.insert("part_name_table".into(), write_string_table_json(&self.part_name_table));
        m.insert("sequencer_name_table".into(), write_string_table_json(&self.sequencer_name_table));
        m.insert("prefab_name_table".into(), write_string_table_json(&self.prefab_name_table));
        m.insert(
            "frame_event_buffer_b64".into(),
            serde_json::Value::String(B64.encode(&self.frame_event_buffer)),
        );
        serde_json::Value::Object(m)
    }
}

impl WriteJsonValue for PaattFile {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> io::Result<()> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "PaattFile: expected object",
        ))?;
        let infos = json_get_field(obj, "infos")?
            .as_array()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "PaattFile.infos: expected array",
            ))?;
        if infos.len() > u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("PaattFile.infos: too many entries: {}", infos.len()),
            ));
        }
        w.extend_from_slice(&(infos.len() as u32).to_le_bytes());
        for info in infos {
            AttackInfo::write_from_json(w, info)?;
        }
        for (key, table) in [
            ("string_table", read_string_table_json(json_get_field(obj, "string_table")?, "string_table")?),
            ("effect_name_table", read_string_table_json(json_get_field(obj, "effect_name_table")?, "effect_name_table")?),
            ("effect_info_key_table", read_string_table_json(json_get_field(obj, "effect_info_key_table")?, "effect_info_key_table")?),
            ("socket_name_table", read_string_table_json(json_get_field(obj, "socket_name_table")?, "socket_name_table")?),
            ("part_name_table", read_string_table_json(json_get_field(obj, "part_name_table")?, "part_name_table")?),
            ("sequencer_name_table", read_string_table_json(json_get_field(obj, "sequencer_name_table")?, "sequencer_name_table")?),
            ("prefab_name_table", read_string_table_json(json_get_field(obj, "prefab_name_table")?, "prefab_name_table")?),
        ] {
            let _ = key;
            write_string_table(w, &table)?;
        }
        let buf = B64.decode(
            json_get_field(obj, "frame_event_buffer_b64")?
                .as_str()
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    "PaattFile.frame_event_buffer_b64: expected string",
                ))?,
        ).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("PaattFile.frame_event_buffer_b64: {}", e),
        ))?;
        if buf.len() > u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("PaattFile.frame_event_buffer too long: {}", buf.len()),
            ));
        }
        w.extend_from_slice(&(buf.len() as u32).to_le_bytes());
        w.extend_from_slice(&buf);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_to_base_size_v0_v4() {
        // Empirical sizes confirmed from the 220 vanilla `.paatt`
        // files / 13,789 AttackInfo records (Session 10 stats).
        assert_eq!(version_to_base_size(0).unwrap(), 264);
        assert_eq!(version_to_base_size(1).unwrap(), 528);
        assert_eq!(version_to_base_size(2).unwrap(), 296);
        assert_eq!(version_to_base_size(3).unwrap(), 288);
        assert_eq!(version_to_base_size(4).unwrap(), 264);
    }

    #[test]
    fn version_to_base_size_unknown_errors() {
        for v in [5u8, 7, 99, 255] {
            let err = version_to_base_size(v).expect_err(
                &format!("version {} should be unknown", v));
            let msg = format!("{}", err);
            assert!(msg.contains("unknown"),
                "error should mention unknown for version {}: {}", v, msg);
            assert!(msg.contains(&v.to_string()),
                "error should include the version number {}: {}", v, msg);
        }
    }

    #[test]
    fn paatt_minimal_round_trip() {
        // The smallest valid .paatt: 0 infos + 7 empty string tables +
        // 0-byte frame_event_buffer.
        let mut data = Vec::new();
        data.extend_from_slice(&0u32.to_le_bytes()); // info_count
        for _ in 0..7 {
            data.extend_from_slice(&0u16.to_le_bytes()); // empty table
        }
        data.extend_from_slice(&0u32.to_le_bytes()); // buffer size

        let (paatt, trailing) = PaattFile::parse_strict(&data).expect("parse ok");
        assert_eq!(paatt.infos.len(), 0);
        assert_eq!(paatt.string_table.len(), 0);
        assert_eq!(paatt.effect_name_table.len(), 0);
        assert_eq!(paatt.frame_event_buffer.len(), 0);
        assert_eq!(trailing, 0);

        let written = paatt.to_bytes().expect("write ok");
        assert_eq!(written, data);
    }

    #[test]
    fn paatt_to_bytes_validates_basedata_size() {
        // Construct a file with BaseData size that doesn't match
        // version. Writer should reject.
        let info = AttackInfo {
            version: 0,
            // version 0 expects 264 bytes; provide 100.
            base_data: vec![0u8; 100],
            child_frames: Default::default(),
        };
        let paatt = PaattFile {
            infos: vec![info],
            string_table: vec![],
            effect_name_table: vec![],
            effect_info_key_table: vec![],
            socket_name_table: vec![],
            part_name_table: vec![],
            sequencer_name_table: vec![],
            prefab_name_table: vec![],
            frame_event_buffer: vec![],
        };
        let err = paatt.to_bytes().expect_err("must reject mismatched size");
        let msg = format!("{}", err);
        assert!(msg.contains("BaseData") || msg.contains("size"),
            "error should mention BaseData or size: {}", msg);
    }

    #[test]
    fn paatt_to_bytes_validates_child_frame_data_length() {
        let info = AttackInfo {
            version: 0,
            base_data: vec![0u8; 264],
            child_frames: {
                let mut frames: [ChildFrame; 9] = Default::default();
                // Set count = 2 (expects 32 bytes data) but provide 16
                frames[0] = ChildFrame { count: 2, data: vec![0u8; 16] };
                frames
            },
        };
        let paatt = PaattFile {
            infos: vec![info],
            string_table: vec![],
            effect_name_table: vec![],
            effect_info_key_table: vec![],
            socket_name_table: vec![],
            part_name_table: vec![],
            sequencer_name_table: vec![],
            prefab_name_table: vec![],
            frame_event_buffer: vec![],
        };
        let err = paatt.to_bytes().expect_err("must reject mismatched data length");
        let msg = format!("{}", err);
        assert!(msg.contains("ChildFrame") || msg.contains("length"),
            "error should mention ChildFrame or length: {}", msg);
    }
}
