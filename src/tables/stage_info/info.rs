//! StageInfo — pabgh_typed_blob_table, 96 wire fields.
//!
//! Wire format derived from Windows CrimsonDesert.exe v1.06 IDA decompilation
//! of sub_1410DC390 (field reader) and sub_141D8AC80 (constructor).
//! All field types verified against IDA sub-function signatures.

use crate::binary::*;
use crate::binary::variants::sequencer_stage_chart_desc::SequencerStageChartDescPartial;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;
use crate::tables::global_stage_sequencer_info::info::PlayerBehaviorOptional;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
use std::io::{self, Write};

// Wire: u32 + u32 + u32 + u32 + u8 + u8 + u8 = 19B per element
// IDA sub_1410D4D00: sub_1410E47D0(u32) + sub_1410E0EE0(u32) + 4B + 4B + 1B + 1B + 1B
py_binary_struct! {
    pub struct StageFilterEntry {
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub raw_a: u32,
        pub raw_b: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
    }
}

// Wire: u8 + u32 + u32 + u32 = 13B per element
// IDA sub_1410EAD70 inner: u8 + 3× sub_1410E0850(u32)
py_binary_struct! {
    pub struct StageMobMapEntry {
        pub flag: u8,
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub lookup_c: u32,
    }
}

// Wire: u32 + CString = variable per element
// IDA sub_1410EABE0 inner: u32 + sub_14108A800(CString)
py_binary_struct! {
    pub struct StageU32StringEntry<'a> {
        pub raw: u32,
        pub label: CString<'a>,
    }
}

// Wire: u32 + u64 + u64 + u64 = 28B per element
// IDA sub_1410E2A10 inner: read(4) + read(8) + read(8) + read(8)
py_binary_struct! {
    pub struct StageCompound28Entry {
        pub key: u32,
        pub val_a: u64,
        pub val_b: u64,
        pub val_c: u64,
    }
}

#[derive(Debug)]
pub struct StageBehaviorEntry {
    pub raw: u32,
    pub behavior: PlayerBehaviorOptional,
}

impl<'a> BinaryRead<'a> for StageBehaviorEntry {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok(Self {
            raw: u32::read_from(data, offset)?,
            behavior: PlayerBehaviorOptional::read_from(data, offset)?,
        })
    }
}
impl BinaryWrite for StageBehaviorEntry {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.raw.write_to(w)?;
        self.behavior.write_to(w)?;
        Ok(())
    }
}
impl ToJsonValue for StageBehaviorEntry {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("raw".into(), self.raw.to_json_value());
        m.insert("behavior".into(), self.behavior.to_json_value());
        Value::Object(m)
    }
}
impl WriteJsonValue for StageBehaviorEntry {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "StageBehaviorEntry: expected object"))?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw")?)?;
        PlayerBehaviorOptional::write_from_json(w, json_get_field(obj, "behavior")?)?;
        Ok(())
    }
}
impl<'a> BinaryReadTracked<'a> for StageBehaviorEntry {
    fn read_tracked(data: &'a [u8], offset: &mut usize, path: &mut String, ranges: &mut Vec<FieldRange>) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        ranges.push(FieldRange { path: path.clone(), start, end: *offset, ty: "StageBehaviorEntry" });
        Ok(item)
    }
}

// Wire order from IDA sub_14108AE40: read(12)→+28, sub_14108AD60(16)→+12, read(12)→+0
// So wire reads: pos_b FIRST, then block, then pos_a
py_binary_struct! {
    pub struct StagePosBlock {
        pub pos_b: [f32; 3],
        pub block: [u32; 4],
        pub pos_a: [f32; 3],
    }
}

// Wire from IDA sub_141D778B0:
// u8 + StagePosBlock(40B) + CString + CString + u8 + [f32;3] + [f32;3] + u8 + u8
py_binary_struct! {
    pub struct StagePlatformEntry<'a> {
        pub flag_a: u8,
        pub pos_block: StagePosBlock,
        pub key_hash: CString<'a>,
        pub label: CString<'a>,
        pub flag_b: u8,
        pub vec_a: [f32; 3],
        pub vec_b: [f32; 3],
        pub flag_c: u8,
        pub flag_d: u8,
    }
}

#[derive(Debug)]
pub struct OptStagePlatformEntry<'a> {
    pub inner: Option<StagePlatformEntry<'a>>,
}
impl<'a> BinaryRead<'a> for OptStagePlatformEntry<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let p = u8::read_from(data, offset)?;
        Ok(Self { inner: if p != 0 { Some(StagePlatformEntry::read_from(data, offset)?) } else { None } })
    }
}
impl<'a> BinaryWrite for OptStagePlatformEntry<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner { Some(v) => { 1u8.write_to(w)?; v.write_to(w) } None => 0u8.write_to(w) }
    }
}
impl<'a> ToJsonValue for OptStagePlatformEntry<'a> {
    fn to_json_value(&self) -> Value { match &self.inner { Some(v) => v.to_json_value(), None => Value::Null } }
}
impl<'a> WriteJsonValue for OptStagePlatformEntry<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { 0u8.write_to(w) } else { 1u8.write_to(w)?; <StagePlatformEntry as WriteJsonValue>::write_from_json(w, v) }
    }
}
impl<'a> BinaryReadTracked<'a> for OptStagePlatformEntry<'a> {
    fn read_tracked(data: &'a [u8], offset: &mut usize, path: &mut String, ranges: &mut Vec<FieldRange>) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        ranges.push(FieldRange { path: path.clone(), start, end: *offset, ty: "OptStagePlatformEntry" });
        Ok(item)
    }
}

// IDA sub_1410EAA30: u8 presence + [sub_1410E88E0(platform) + sub_1410E9A70(u16) + u32 + u32]
#[derive(Debug)]
pub struct OptStageOpt52<'a> {
    pub inner: Option<StageOpt52Inner<'a>>,
}
#[derive(Debug)]
pub struct StageOpt52Inner<'a> {
    pub platform: OptStagePlatformEntry<'a>,
    pub lookup: u16,
    pub raw_a: u32,
    pub raw_b: u32,
}
impl<'a> BinaryRead<'a> for OptStageOpt52<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let p = u8::read_from(data, offset)?;
        Ok(Self { inner: if p != 0 { Some(StageOpt52Inner {
            platform: OptStagePlatformEntry::read_from(data, offset)?,
            lookup: u16::read_from(data, offset)?,
            raw_a: u32::read_from(data, offset)?,
            raw_b: u32::read_from(data, offset)?,
        }) } else { None } })
    }
}
impl<'a> BinaryWrite for OptStageOpt52<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(v) => { 1u8.write_to(w)?; v.platform.write_to(w)?; v.lookup.write_to(w)?; v.raw_a.write_to(w)?; v.raw_b.write_to(w)?; Ok(()) }
            None => 0u8.write_to(w),
        }
    }
}
impl<'a> ToJsonValue for OptStageOpt52<'a> {
    fn to_json_value(&self) -> Value {
        match &self.inner {
            Some(v) => { let mut m = Map::new(); m.insert("platform".into(), v.platform.to_json_value()); m.insert("lookup".into(), v.lookup.to_json_value()); m.insert("raw_a".into(), v.raw_a.to_json_value()); m.insert("raw_b".into(), v.raw_b.to_json_value()); Value::Object(m) }
            None => Value::Null,
        }
    }
}
impl<'a> WriteJsonValue for OptStageOpt52<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { 0u8.write_to(w)?; return Ok(()); }
        let obj = v.as_object().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "OptStageOpt52: expected object or null"))?;
        1u8.write_to(w)?;
        OptStagePlatformEntry::write_from_json(w, json_get_field(obj, "platform")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_b")?)?;
        Ok(())
    }
}
impl<'a> BinaryReadTracked<'a> for OptStageOpt52<'a> {
    fn read_tracked(data: &'a [u8], offset: &mut usize, path: &mut String, ranges: &mut Vec<FieldRange>) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        ranges.push(FieldRange { path: path.clone(), start, end: *offset, ty: "OptStageOpt52" });
        Ok(item)
    }
}

pabgh_typed_blob_table! {
    pub struct StageInfo<'a> {
        // === Fields 1-7: Header ===
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub name: LocalizableString<'a>,
        pub stage_desc: LocalizableString<'a>,
        pub complete_log: LocalizableString<'a>,
        pub sequencer_desc: SequencerStageChartDescPartial<'a>,
        // === Fields 8-9: Spawn faction (u32 wire → lookup) ===
        pub spawn_faction_spawn_data_info: u32,
        pub spawn_faction_node_info: u32,
        // === Field 10: Disable faction list ===
        pub disable_faction_spawn_party_name_hash_list: CArray<u32>,
        // === Fields 11-13: u64 category/filter ===
        pub stage_category: u64,
        pub close_filter: u64,
        pub close_filter_by_group: u64,
        // === Field 14: Global filter character list ===
        pub global_filter_character_list: CArray<u32>,
        // === Fields 15-16: Type flags ===
        pub quest_type: u8,
        pub stage_data_type: u8,
        // === Fields 17-19: Parent/owner lookups (u32 wire) ===
        pub parent_quest: u32,
        pub parent_stage: u32,
        pub owner_mission_info: u32,
        // === Fields 20-22: Child/executor lists ===
        pub child_stage_list: CArray<u32>,
        pub executor_mission_list: CArray<u32>,
        pub executor_stage_list: CArray<u32>,
        // === Field 23: Filter entry list (19B compound elements) ===
        pub execute_target_stage_list: CArray<StageFilterEntry>,
        // === Field 24: Hide mercenary group list (CArray<u8> wire!) ===
        pub hide_mercenary_group_info_list: CArray<u8>,
        // === Fields 25-27: Condition/field lookups ===
        pub play_condition: u32,
        pub close_condition: u32,
        pub field_info: u32,
        // === Fields 28-29: Character lists (KEY for char swap mod) ===
        pub start_player_list: CArray<u32>,
        pub forbidden_character_list: CArray<u32>,
        // === Fields 30-31: CStrings ===
        pub rematch_stage_desc: CString<'a>,
        pub platform_character: CString<'a>,
        // === Field 32: Stage condition list ===
        pub stage_condition_list: CArray<u32>,
        // === Fields 33-34: Lookups ===
        pub platform_socket_name: u32,
        pub raw_d: u32,
        // === Field 35: Guide effect name CString ===
        pub guide_effect_name: CString<'a>,
        // === Fields 36-37: Flags ===
        pub flag_c: u8,
        pub flag_d: u8,
        // === Fields 38-39: Raw u32s ===
        pub raw_e: u32,
        pub raw_f: u32,
        // === Field 40: Pair of u32s (sub_14108AD00 = 4B+4B) ===
        pub pair_a: u32,
        pub pair_b: u32,
        // === Fields 41-43: Individual fields (was raw_ghi_block) ===
        pub raw_g: u64,
        pub raw_h: u32,
        pub raw_i: u16,
        // === Field 44: Mob map list (u8+u32+u32+u32 per element) ===
        pub mob_map_list: CArray<StageMobMapEntry>,
        // === Field 45: Lookup ===
        pub lookup_j: u32,
        // === Field 46: String entry list (u32+CString per element) ===
        pub string_entry_list: CArray<StageU32StringEntry<'a>>,
        // === Fields 47-50: 4× compound 28B lists (u32+u64+u64+u64) ===
        pub compound_list_a: CArray<StageCompound28Entry>,
        pub compound_list_b: CArray<StageCompound28Entry>,
        pub compound_list_c: CArray<StageCompound28Entry>,
        pub compound_list_d: CArray<StageCompound28Entry>,
        // === Fields 51-54: 4× close filter d lists ===
        pub close_filter_d_a: CArray<u32>,
        pub close_filter_d_b: CArray<u32>,
        pub close_filter_d_c: CArray<u32>,
        pub close_filter_d_d: CArray<u32>,
        // === Field 55: list_d ===
        pub list_d: CArray<u32>,
        // === Field 56: Platform entry (nested optional) ===
        pub platform_entry: OptStageOpt52<'a>,
        // === Fields 57-64: 8× u32 lookups ===
        pub lookup_k: u32,
        pub lookup_l: u32,
        pub lookup_m: u32,
        pub lookup_n: u32,
        pub lookup_o: u32,
        pub lookup_p: u32,
        pub lookup_q: u32,
        pub lookup_r: u32,
        // === Field 65: LocalizableString ===
        pub label_b: LocalizableString<'a>,
        // === Field 66: Lookup ===
        pub lookup_s: u32,
        // === Fields 67-68: u8 flags ===
        pub flag_e: u8,
        pub flag_f: u8,
        // === Field 69: Lookup (sub_1410E2250 = u32 wire) ===
        pub lookup_s2: u32,
        // === Field 70: Behavior entry list ===
        pub behavior_entry_list: CArray<StageBehaviorEntry>,
        // === Field 71: Raw u32 ===
        pub raw_j: u32,
        // === Field 72: u16 lookup (sub_1410E9A70 reads u16 wire) ===
        pub lookup_u: u16,
        // === Fields 73-74: u32 lookups ===
        pub lookup_v: u32,
        pub lookup_w: u32,
        // === Fields 75-80: 6× u32 raw ===
        pub raw_k: u32,
        pub raw_l: u32,
        pub raw_m: u32,
        pub raw_n: u32,
        pub raw_o: u32,
        pub raw_p: u32,
        // === Fields 81-96: 16× u8 flags ===
        pub flag_g: u8,
        pub flag_h: u8,
        pub flag_i: u8,
        pub flag_j: u8,
        pub flag_k: u8,
        pub flag_l: u8,
        pub flag_m: u8,
        pub flag_n: u8,
        pub flag_o: u8,
        pub flag_p: u8,
        pub flag_q: u8,
        pub flag_r: u8,
        pub flag_s: u8,
        pub flag_t: u8,
        pub flag_u: u8,
        pub flag_v: u8,
    }
    tail: tail_blob;
}
