//! Hand-corrected: IDA-derived parser for `FactionSpawnDataInfo.pabgb`.
//!
//! Per IDA sub_1410DF1D0: 7 fields fully field-level typed.
//!
//! Wire layout:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. patrol_spawn_data:    COptional<PatrolSpawnData>          (sub_141115560)
//!   5. gimmick_spawn_data_list: CArray<GimmickElement>           (sub_141115390)
//!   6. schedule_spawn_info:   COptional<CArray<FactionScheduleSpawnData>> (sub_101935820)
//!      element {condition: u32, character_group_info: u16}  (1.10 fix; was u16)
//!   7. sequencer_spawn_info:  COptional<CArray<u32>>             (sub_141115190)
//!
//! ## Inner structs (recovered from nested IDA decompilation):
//!
//! `PatrolSpawnData` (sub_141115560 — 32-byte runtime when present):
//!   - patrol_named_list: CArray<PatrolNamedElement>     (sub_1411038D0)
//!   - patrol_element_list: CArray<PatrolElement>        (sub_1411156C0)
//!
//! `PatrolNamedElement` (sub_1411038D0 per element):
//!   - name: CString
//!   - key_hash: u32                                      (sub_1410FF430, qword_E9C0)
//!
//! `PatrolElement` (sub_1410DF020 per element, 33 fixed + 12*N nested):
//!   - field_a: u32
//!   - field_b: u32
//!   - field_c_hash: u32                                  (qword_D9F8 inline lookup)
//!   - nested: CArray<PatrolNestedElement>                (sub_1411037E0)
//!   - field_d_hash: u32                                  (sub_1410FF430, qword_E9C0)
//!   - field_e: u32
//!   - field_f: u32
//!   - flag: u8
//!
//! `PatrolNestedElement` (sub_1410DEF10 per element, 12 wire bytes):
//!   - field_a: u32                                       (sub_1410FF340, qword_DA08)
//!   - field_b: u16                                       (sub_1411003E0, qword_12668)
//!   - field_c: u32                                       (sub_1410FF340)
//!   - field_d: u16                                       (sub_1411003E0)
//!   - flag_a: u8
//!   - flag_b: u8
//!
//! `GimmickElement` (sub_141115390 per element):
//!   - name: CString
//!   - field_a: u16                                       (sub_1411003E0)
//!   - field_b: u32                                       (sub_1410FF430)


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::FactionSpawnDataInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  7
// Decoded by dmm-parser:   7
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _gimmickSpawnDataList
// ✅ _patrolSpawnData (reader_1B, stream=1)
// ✅ _sequencerSpawnInfo (reader_15B, stream=15)
// ✅ _scheduleSpawnInfo
// ✅ _key (reader_15B, stream=15)
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// Inner element of `PatrolElement.nested` (sub_1411037E0 →
    /// sub_1410DEF10). 12 wire bytes; runtime stride 10.
    /// ── 1.18.00: this is the exe's `FactionPatrolCharacterData`, which went
    /// **6 → 11 fields**; the element grew 14 → 28 bytes.
    ///
    /// The width is proven, not guessed: walking all 131 records with the
    /// element size as an unknown reconciles every one at exactly +14 (and at
    /// +1 for the enclosing PatrolPartyData) and at no other value.
    ///
    /// The two insert points are proven too. 93 element pairs carry a non-zero
    /// `field_d`, and they pin the split as **9 bytes at offset 10** then
    /// **5 bytes at offset 12** — the 536 all-zero pairs merely let difflib
    /// merge those into one 14-byte run, which is why the zero cases must not
    /// be trusted on their own.
    ///
    /// ⚠ The 9/5 split into names is the one inferred step: 9 = u32+u32+u8 and
    /// 5 = u32+u8, matched to the oracle's five new names by the symmetry
    /// (main gets reason + action-point tag + disable, sub gets reason +
    /// disable). Existing fields keep their placeholder names deliberately —
    /// renaming them would break the mod contract for anything already keyed
    /// on `field_a`..`flag_b`.
    pub struct PatrolNestedElement {
        pub field_a: u32,
        pub field_b: u16,
        pub field_c: u32,
        pub spawn_reason: u32,
        pub spawn_action_point_tag_hash: u32,
        pub disable_patrol: u8,
        pub field_d: u16,
        pub sub_spawn_reason: u32,
        pub sub_disable_patrol: u8,
        pub flag_a: u8,
        pub flag_b: u8,
    }
}

py_binary_struct! {
    /// Per-element of `PatrolSpawnData.patrol_element_list` (sub_1410DF020,
    /// 33 fixed bytes + 12*N nested).
    pub struct PatrolElement {
        pub field_a: u32,
        pub field_b: u32,
        pub field_c_hash: u32,
        pub nested: CArray<PatrolNestedElement>,
        pub field_d_hash: u32,
        pub field_e: u32,
        pub field_f: u32,
        // ── 1.18.00: `_isPlatformParty` (the exe's `PatrolPartyData`, 8 → 9
        // fields). Position is pinned, not assumed: 369 elements carry
        // `flag = 01`, and the new byte lands before it in every one.
        pub is_platform_party: u8,
        pub flag: u8,
    }
}

py_binary_struct! {
    /// Per-element of `PatrolSpawnData.patrol_named_list` (sub_1411038D0).
    pub struct PatrolNamedElement<'a> {
        pub name: CString<'a>,
        pub key_hash: u32,
    }
}

py_binary_struct! {
    /// Patrol spawn data inner struct (sub_141115560 inner, 32B runtime).
    pub struct PatrolSpawnData<'a> {
        pub patrol_named_list: CArray<PatrolNamedElement<'a>>,
        pub patrol_element_list: CArray<PatrolElement>,
    }
}

py_binary_struct! {
    /// Per-element of `gimmick_spawn_data_list`. Mac reader sub_1018FFD5C
    /// (`FactionGimmickActorSpawnInfo`): _gimmickSpawnTag (CString),
    /// _characterGroupInfo (CharacterGroupKey, u16 wire), _condition
    /// (ConditionKey, u32 wire). field_a=character_group, field_b=condition.
    pub struct GimmickElement<'a> {
        pub name: CString<'a>,
        pub field_a: u16,
        pub field_b: u32,
    }
}

py_binary_struct! {
    /// Element of `schedule_spawn_info`'s inner list. Mac reader sub_1018FFF68
    /// (`FactionScheduleSpawnData`): _condition (ConditionKey, u32 wire → u16),
    /// _characterGroupInfo (CharacterGroupKey, u16 wire → u16). 6 wire bytes.
    ///
    /// 2026-06-05: the prior parser modeled schedule_spawn_info as
    /// `COptional<CArray<u16>>` — i.e. a 2-byte element. That was only ever
    /// exercised on entries where the optional is ABSENT (most of them), so it
    /// "passed" until an entry with a present schedule (key 0xf4255) under-read
    /// by 4 bytes per element AND then misread the following (present)
    /// sequencer list as absent — a 28-byte shortfall. Confirmed via the Mac
    /// FactionScheduleSpawnData reader.
    pub struct FactionScheduleSpawnData {
        pub condition: u32,
        pub character_group_info: u16,
    }
}

#[derive(Debug)]
pub struct FactionSpawnDataInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub patrol_spawn_data: COptional<PatrolSpawnData<'a>>,
    pub gimmick_spawn_data_list: CArray<GimmickElement<'a>>,
    pub schedule_spawn_info: COptional<CArray<FactionScheduleSpawnData>>,
    pub sequencer_spawn_info: COptional<CArray<u32>>,
    // ── 1.18.00: `_zoneSpawnDataList`, appended after the sequencer list.
    // All 131 records grew by exactly 4 bytes at the very end, and those 4
    // bytes are zero in every one — a CArray with count 0.
    // ⚠ The ELEMENT type is a placeholder. The 1.18 exe declares the real
    // element as `FactionZoneSpawnInfo` with two fields — `dailyRotuinePartData`
    // (sic, the game's own typo) and `routineZoneList` — so it is a struct, NOT
    // a u32. Vanilla never populates the list, so there is no wire evidence for
    // the widths and no way to model it correctly yet. u32 is chosen only
    // because count is 0 everywhere and any element type round-trips byte-exact
    // today. Model it properly the moment a build ships a populated record.
    pub zone_spawn_data_list: CArray<u32>,
}

impl<'a> FactionSpawnDataInfo<'a> {
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
        let patrol_spawn_data = COptional::<PatrolSpawnData>::read_from(data, offset)?;
        let gimmick_spawn_data_list = CArray::<GimmickElement>::read_from(data, offset)?;
        let schedule_spawn_info = COptional::<CArray<FactionScheduleSpawnData>>::read_from(data, offset)?;
        let sequencer_spawn_info = COptional::<CArray<u32>>::read_from(data, offset)?;
        let zone_spawn_data_list = CArray::<u32>::read_from(data, offset)?;

        if *offset != entry_end {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "FactionSpawnDataInfo k=0x{:x}: under/over-read (consumed {} of {} bytes)",
                    key, *offset - entry_start, entry_size,
                ),
            ));
        }

        Ok(Self {
            key, string_key, is_blocked,
            patrol_spawn_data, gimmick_spawn_data_list,
            schedule_spawn_info, sequencer_spawn_info, zone_spawn_data_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.patrol_spawn_data.write_to(w)?;
        self.gimmick_spawn_data_list.write_to(w)?;
        self.schedule_spawn_info.write_to(w)?;
        self.sequencer_spawn_info.write_to(w)?;
        self.zone_spawn_data_list.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("patrol_spawn_data".to_string(), self.patrol_spawn_data.to_json_value());
        m.insert("gimmick_spawn_data_list".to_string(), self.gimmick_spawn_data_list.to_json_value());
        m.insert("schedule_spawn_info".to_string(), self.schedule_spawn_info.to_json_value());
        m.insert("sequencer_spawn_info".to_string(), self.sequencer_spawn_info.to_json_value());
        m.insert("zone_spawn_data_list".to_string(), self.zone_spawn_data_list.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <COptional<PatrolSpawnData> as WriteJsonValue>::write_from_json(
            w, json_get_field(obj, "patrol_spawn_data")?,
        )?;
        <CArray<GimmickElement> as WriteJsonValue>::write_from_json(
            w, json_get_field(obj, "gimmick_spawn_data_list")?,
        )?;
        <COptional<CArray<FactionScheduleSpawnData>> as WriteJsonValue>::write_from_json(
            w, json_get_field(obj, "schedule_spawn_info")?,
        )?;
        <COptional<CArray<u32>> as WriteJsonValue>::write_from_json(
            w, json_get_field(obj, "sequencer_spawn_info")?,
        )?;
        // Null-tolerant on purpose (the store_info::low_price_threshold_count_116
        // idiom): a V3 mod authored before 1.18 carries no key for this field,
        // and must keep applying rather than aborting the whole overlay.
        // CArray's WriteJsonValue turns Null into count 0.
        <CArray<u32> as WriteJsonValue>::write_from_json(
            w, obj.get("zone_spawn_data_list").unwrap_or(&Value::Null),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("factionspawndatainfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = FactionSpawnDataInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionspawndatainfo roundtrip bytes mismatch");
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
            let item = FactionSpawnDataInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FactionSpawnDataInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
