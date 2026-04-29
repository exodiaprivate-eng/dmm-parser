//! Hand-corrected: typed prefix + GlobalGameEventExecuteData wrapper.
//!
//! Reader: `sub_1410E5840` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u16 key                              (_key, pabgh format 2)
//!   2. CString string_key                   (_stringKey)
//!   3. u8 is_blocked                        (_isBlocked)
//!   4. u16 global_game_event_group_info     (_globalGameEventGroupInfo,
//!      inline u16 hash-key at qword_145F0E9C8)
//!   5. _executeData (sub_141156680 → POLYMORPHIC family
//!      `GlobalGameEventExecuteData` with sub_tag dispatch:
//!      0=VaryTradeItemPrice [88B], 1=OpenRoyalSupply [32B], 2=in-place,
//!      0=absent presence byte → no-op)
//!
//! ## Tier 1 — typed wrapper, byte-perfect round-trip
//!
//! `execute_data` uses the `Decoded | Raw` enum from
//! `crate::binary::variants::global_game_event_execute_data`. Decoded
//! captures presence + sub_tag; Raw is the verbatim-bytes fallback. The
//! family decoder is intentionally body-shape-agnostic (body stays
//! `Vec<u8>` inside Present) — full per-sub_tag typed payloads are a
//! follow-up enhancement; the current shape is enough for v3 clone-
//! between-entries and sub_tag-aware filtering.
//!
//! DO NOT REGENERATE. Hand-written; bulk_process.py guards via the
//! "Hand-corrected" header marker on line 1.

use crate::binary::variants::global_game_event_execute_data::GlobalGameEventExecuteData;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct GlobalGameEventInfo<'a> {
    pub key: u16,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub global_game_event_group_info: u16,
    /// Polymorphic execute_data wrapper. Decoded captures presence +
    /// sub_tag + body bytes; Raw passes through verbatim. Either way,
    /// round-trip is byte-perfect.
    pub execute_data: GlobalGameEventExecuteData,
}

impl<'a> GlobalGameEventInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start + entry_size;

        let key = u16::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let global_game_event_group_info = u16::read_from(data, offset)?;

        // GlobalGameEventExecuteData::read_from expects `data` sized to
        // exactly the wrapper. Pass a sub-slice of just the tail bytes.
        let wrapper_bytes = &data[*offset..entry_end];
        let mut wrapper_cur = 0usize;
        let execute_data = GlobalGameEventExecuteData::read_from(wrapper_bytes, &mut wrapper_cur)?;
        *offset = entry_end;

        Ok(Self {
            key,
            string_key,
            is_blocked,
            global_game_event_group_info,
            execute_data,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.global_game_event_group_info.write_to(w)?;
        self.execute_data.write_to(w)?;
        Ok(())
    }

    /// JSON shape:
    /// - Scalars (`key`, `string_key`, `is_blocked`,
    ///   `global_game_event_group_info`): individually editable.
    /// - `_execute_data_b64`: the wrapper bytes as base64 (clone between
    ///   entries; round-trips byte-perfect).
    /// - `_execute_data_kind`: read-only metadata, "absent" / "present" /
    ///   "raw" — tells consumers the structural state.
    /// - `_execute_data_sub_tag`: read-only metadata, the sub_tag byte
    ///   when kind == "present" (omitted otherwise).
    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert(
            "global_game_event_group_info".to_string(),
            self.global_game_event_group_info.to_json_value(),
        );
        let mut wrapper_bytes = Vec::new();
        self.execute_data.write_to(&mut wrapper_bytes).expect("write_to Vec");
        m.insert(
            "_execute_data_b64".to_string(),
            Value::String(B64.encode(&wrapper_bytes)),
        );
        let (kind, sub_tag) = match &self.execute_data {
            GlobalGameEventExecuteData::Absent => ("absent", None),
            GlobalGameEventExecuteData::Present { sub_tag, .. } => ("present", Some(*sub_tag)),
            GlobalGameEventExecuteData::Raw(_) => ("raw", None),
        };
        m.insert("_execute_data_kind".to_string(), Value::String(kind.to_string()));
        if let Some(t) = sub_tag {
            m.insert("_execute_data_sub_tag".to_string(), Value::Number(t.into()));
        }
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u16 as WriteJsonValue>::write_from_json(
            w,
            json_get_field(obj, "global_game_event_group_info")?,
        )?;
        // The kind/sub_tag are read-only metadata; on write we re-decode
        // from the b64 wrapper bytes. This keeps writes simple and
        // ensures wrapper integrity.
        let b64 = json_get_field(obj, "_execute_data_b64")?
            .as_str()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "GlobalGameEventInfo: _execute_data_b64 must be a base64 string",
                )
            })?;
        let bytes = B64.decode(b64).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("GlobalGameEventInfo: _execute_data_b64 invalid base64: {}", e),
            )
        })?;
        w.extend_from_slice(&bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\globalgameevent.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\globalgameevent.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP");
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH) else {
            eprintln!("SKIP");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GlobalGameEventInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items {
            it.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "globalgameevent roundtrip mismatch");
    }

    /// Sanity check: most entries should be Decoded (Absent/Present), not Raw.
    /// Raw is the byte-perfect fallback but we expect the dispatcher
    /// recipe to handle the common case cleanly.
    #[test]
    fn execute_data_kinds() {
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP");
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH) else {
            eprintln!("SKIP");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        let mut absent = 0;
        let mut present = 0;
        let mut raw = 0;
        for (_k, s, e) in ranges.iter() {
            let mut c = *s;
            let it = GlobalGameEventInfo::read_with_size(&data, &mut c, e - s).unwrap();
            match it.execute_data {
                GlobalGameEventExecuteData::Absent => absent += 1,
                GlobalGameEventExecuteData::Present { .. } => present += 1,
                GlobalGameEventExecuteData::Raw(_) => raw += 1,
            }
        }
        eprintln!(
            "globalgameevent execute_data: {} absent, {} present, {} raw",
            absent, present, raw
        );
        // We don't assert specific counts (data may evolve); the kinds
        // log just helps verify Raw stays low.
    }
}
