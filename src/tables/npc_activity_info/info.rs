//! IDA-derived parser for `npcactivityinfo.pabgb`.
//!
//! Reader: `sub_101915238` (CrimsonDesert_Steam 1.11, Mac IDB base 0x1_0000_0000).
//! Sequential table. Field order + wire sizes recovered via the Korean
//! per-field error strings (`NPCActivityInfo의 _<field>를 읽어들이는데 실패했다`)
//! and decompilation of each field reader.
//!
//! Wire layout (sub_101915238):
//!   1. _key                 u32         (sub_100FAAE34, 4B)
//!   2. _stringKey           CString     (sub_1006E817C)
//!   3. _isBlocked           u8          (sub_1006E7EEC, 1B)
//!   4. _activityTagList     CArray<u32> (sub_100CC2174)
//!   5. _sequenceList        CArray<NpcActivitySequence>        (sub_101949454)
//!   6. _interactionTaskList CArray<NpcActivityInteractionTask> (sub_1019495FC)
//!   7. _groupTaskList       CArray<NpcActivityGroupTask>       (sub_10194990C)
//!   8. _catchActionTaskList CArray<NpcActivityCatchActionTask> (sub_101949BFC)
//!   9. _flowControlList     CArray<NpcActivityFlowControl>     (sub_101949EE8)
//!
//! Inner element readers:
//!   NPCActivitySequence (sub_101949204): CArray of
//!     NPCActivityInstructionLocator { u8 instruction_type, u8 npc_activity_type,
//!                                     u32 index }.
//!   NPCActivityTask base (sub_101914960): u8 instruction_type, u8 activity_type,
//!     f32 duration_seconds (sub_1006E800C, 4B), u32 end_condition_info
//!     (sub_100F840EC ConditionKey, 4B wire → u16 hash at runtime).
//!   NPCActivityInteractionTask (sub_101914B78): base + u32 self_condition_info +
//!     u32 target_condition_info + u8 interaction_type (sub_101409844) +
//!     u32 interaction_key (sub_100F9C5C4) + CString interaction_tag
//!     (sub_1006E8320 — reads u32 len then advances the stream by len).
//!   NPCActivityGroupTask (sub_101914DD0): base + u32 self_condition_info +
//!     u32 target_condition_info + u8 group_activity_type + u32 group_limit.
//!   NPCActivityCatchActionTask (sub_101915018): base + u32 self_condition_info +
//!     u32 catch_target_condition + u32 catch_action_hash.
//!   NPCActivityFlowControl (sub_101914838): u8 instruction_type, u8 control_type,
//!     CArray<u32> sequence_index_list (sub_100CC2174).
//!
//! Note: end/self/target condition fields and interaction_key are stored wire as
//! u32 (a ConditionKey / hash). The engine resolves them to a u16 runtime index;
//! the parser keeps the raw u32 so the table round-trips byte-exact.

use crate::binary::*;
use crate::json_traits::{get_field as json_get_field, ToJsonValue, WriteJsonValue};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

// NOTE: The 1.11 inner task structs (NPCActivityInstructionLocator,
// NPCActivitySequence, NPCActivityInteractionTask, NPCActivityGroupTask,
// NPCActivityCatchActionTask, NPCActivityFlowControl) were removed when the
// table became Tier-1.5 on 1.12 (the body is now an opaque tail — see the
// struct below). Their full 1.11 wire layout is preserved in the file header
// doc comment above (and in git history) as the head-start for the eventual
// 1.12 full decode once an un-stripped reader is available.

// ── NpcActivityInfo (1.12: Tier-1.5 typed prefix + opaque tail) ──────────────
//
// 1.12 reworked the NPC-activity system: the per-record body after `is_blocked`
// gained a group reference (u32 = an npc_activity_group key, e.g. 1000001) plus
// a float, and the polymorphic NPCActivityTask base class grew new
// condition/range fields (FLT_MAX = 0x7f7fffff sentinels) across all 4 task
// variants. The full field-level decode requires the un-stripped 1.12 reader
// (the live Win/Mac exe is symbol-stripped, so the IDA Korean-error-string →
// xref method that recovered the 1.11 layout below is unavailable on 1.12).
//
// Until that decode lands, the record is parsed as a typed prefix (key,
// string_key, is_blocked — the fields DMM's resolver and linter need) plus an
// opaque `_tail_b64` blob bounded by the pabgh entry size. This ROUND-TRIPS
// BYTE-EXACT (the tail bytes are preserved verbatim), so the table is safe to
// mount/unmount on 1.12; only field-level modding of the task internals is
// deferred (see the file header doc + the NOTE above for the prior layout).
pub struct NpcActivityInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    /// Opaque 1.12 body (sequence/interaction/group/catch/flow task lists,
    /// reworked). Preserved verbatim for byte-exact round-trip.
    pub tail_blob: Vec<u8>,
}

impl<'a> NpcActivityInfo<'a> {
    /// Parse one record bounded by its pabgh entry size. Decodes the stable
    /// prefix and captures the remainder as an opaque tail.
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let start = *offset;
        let end = start
            .checked_add(entry_size)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "NpcActivityInfo: size overflow"))?;
        if end > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                format!("NpcActivityInfo: record extends past body ({} > {})", end, data.len())));
        }
        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        if *offset > end {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "NpcActivityInfo: prefix over-read past entry bound"));
        }
        let tail_blob = data[*offset..end].to_vec();
        *offset = end;
        Ok(Self { key, string_key, is_blocked, tail_blob })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        w.write_all(&self.tail_blob)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut d = Map::new();
        d.insert("key".to_string(), self.key.to_json_value());
        d.insert("string_key".to_string(), self.string_key.to_json_value());
        d.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        d.insert("_tail_b64".to_string(), Value::from(B64.encode(&self.tail_blob)));
        d
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, map: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(map, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(map, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(map, "is_blocked")?)?;
        let b64 = json_get_field(map, "_tail_b64")?.as_str().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "NpcActivityInfo._tail_b64: expected base64 string"))?;
        let tail = B64.decode(b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData, format!("NpcActivityInfo._tail_b64: {}", e)))?;
        w.extend_from_slice(&tail);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf {
        crate::testenv::resolve("npcactivityinfo.pabgb")
    }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) =
            load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy())
        else {
            eprintln!("SKIP: pabgh not found");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        // Tier-1.5: parse each pabgh-bounded record (typed prefix + opaque tail),
        // then write it back and assert byte-exact (covered region).
        let mut out = Vec::with_capacity(data.len());
        let mut covered = 0usize;
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = NpcActivityInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: {}", i, key, e));
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: read_with_size boundary", i, key);
            item.write_to(&mut out).unwrap();
            covered += end - start;
            // Also assert JSON round-trips to the same bytes as the typed write.
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            NpcActivityInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(from_json, from_typed, "entry {} key=0x{:x}: JSON round-trip diverges", i, key);
        }
        // The pabgh-covered region must round-trip byte-identically.
        assert_eq!(out.as_slice(), &data[..covered], "npcactivityinfo byte round-trip mismatch");
    }
}
