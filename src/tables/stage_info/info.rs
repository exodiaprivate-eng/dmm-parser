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

// Field 31 `_rematchStageDesc` (reader sub_101F78F38): a binarystring + a list of
// binarystrings + a list of StageKeys. (sub_100D395EC / sub_1013700D0 read CString
// wire even though mem stores hashes.)
py_binary_struct! {
    pub struct RematchStageDesc<'a> {
        pub start_sub_timeline_name: CString<'a>,
        pub end_sub_timeline_name_list: CArray<CString<'a>>,
        pub stage_info_list: CArray<u32>,
    }
}

// Field 43 element `SubTimelineBreakDesc` (reader sub_101F7A198): u8 + 3× StringInfoKey (u32 wire).
py_binary_struct! {
    pub struct SubTimelineBreakDesc {
        pub event_type: u8,
        pub npc_reaction_tag_non_battle: u32,
        pub npc_reaction_tag_battle: u32,
        pub sub_timeline_name: u32,
    }
}

// Field 45 element (reader sub_101FB70E0): u32 hash + CString.
py_binary_struct! {
    pub struct ScheduleAiEvent<'a> {
        pub ai_event_name_hash: u32,
        pub target_folder_name: CString<'a>,
    }
}

// Field 49 inner `StageInfo_GlobalEffect` (reader sub_101F7A000): optional trigger
// volume + GameGlobalEffectKey(u16) + priority(u32) + blendingDistance(u32). The
// trigger volume is absent on the abyss-weather records (the only ones that populate
// field 49), so COptional<u32> placeholder suffices for its presence byte.
// 40-byte transform (reader sub_100D39CD4): Vec3 + [u32;4] + Vec3 (same as level_gimmick).
py_binary_struct! {
    pub struct StageWorldTransform {
        pub a: [f32; 3],
        pub raw: [u32; 4],
        pub b: [f32; 3],
    }
}
// Trigger volume (reader sub_101AF7F08): u8 + transform + 2×CString + u8 + 2×Vec3 + 2×u8.
py_binary_struct! {
    pub struct StageTriggerVolume<'a> {
        pub flag_a: u8,
        pub transform: StageWorldTransform,
        pub cstring_a: CString<'a>,
        pub cstring_b: CString<'a>,
        pub flag_b: u8,
        pub vec_a: [f32; 3],
        pub vec_b: [f32; 3],
        pub flag_c: u8,
        pub flag_d: u8,
    }
}
py_binary_struct! {
    pub struct StageGlobalEffect<'a> {
        pub trigger_volume_data: COptional<StageTriggerVolume<'a>>,
        pub global_effect_info: u16,
        pub priority: u32,
        pub blending_distance: u32,
    }
}
// Field 63 element (reader sub_101D75F78): u32 + COptional<GameEventExecuteData
// {u8 type + 3× ConditionKey u32}>.
py_binary_struct! {
    pub struct GameEventExecuteData {
        pub game_event_type: u8,
        pub player_condition: u32,
        pub target_condition: u32,
        pub event_condition: u32,
    }
}
py_binary_struct! {
    pub struct StageGameEventData {
        pub hash: u32,
        pub event_execute: COptional<GameEventExecuteData>,
    }
}

// Fields 46/47: a struct of 4 CArrays (sub_101F79D50 / sub_101F79DB0 = 4× CArray builder).
// 47 = DropSetKey lists (u32 wire). 46's elements are larger (32B mem) but empty in
// vanilla → placeholder u32; fix if a record populates it.
py_binary_struct! {
    pub struct StageQuadList {
        pub a: CArray<u32>,
        pub b: CArray<u32>,
        pub c: CArray<u32>,
        pub d: CArray<u32>,
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
        // === Fields 15-23: byte-identical 1.06↔1.12 (same-record byte diff: HerStore
        //     matched to offset 491; the 1.08 insertion is at field 24). ===
        pub quest_type: u8,
        pub stage_data_type: u8,
        pub parent_quest: u32,
        pub parent_stage: u32,
        pub owner_mission_info: u32,
        pub child_stage_list: CArray<u32>,
        pub executor_mission_list: CArray<u32>,
        pub executor_stage_list: CArray<u32>,
        pub execute_target_stage_list: CArray<StageFilterEntry>,
        // === Field 24: NEW in the 1.08 update — `_logoutMercenaryGroupInfoList`
        //     (reader sub_101FB6CCC), inserted before _hideMercenaryGroupInfoList. ===
        pub logout_mercenary_group_info_list: CArray<u8>,
        // === Fields 25-30 (1.12, = old 24-29 shifted past the field-24 insertion) ===
        pub hide_mercenary_group_info_list: CArray<u8>,
        pub play_condition: u32,
        pub close_condition: u32,
        pub field_info: u32,
        pub start_player_list: CArray<u32>,
        pub forbidden_character_list: CArray<u32>,
        // === Fields 31-42 (authoritative 89-field map, nesting-depth paired) ===
        pub rematch_stage_desc: RematchStageDesc<'a>,        // 31
        pub platform_character: u32,                         // 32 CharacterKey (NOT a list!)
        pub platform_docking_tag_hash: u32,                  // 33
        pub platform_socket_name: CString<'a>,               // 34 binarystring
        pub is_ignore_distance: u8,                          // 35
        pub is_faction_sequencer: u8,                        // 36
        pub faction_sequencer_spawn_tag_hash: u32,           // 37
        pub reset_second: u32,                               // 38
        pub random_spawn_count: [u32; 2],                    // 39 sub_100D399C0 (2×u32)
        pub random_percent: u64,                             // 40
        pub random_repeat_time: u32,                         // 41
        pub complete_count: u16,                             // 42
        pub sub_timeline_break_desc_list: CArray<SubTimelineBreakDesc>, // 43
        pub schedule_complete_condition: u32,                // 44 ConditionKey
        pub schedule_stage_complete_ai_event_list: CArray<ScheduleAiEvent<'a>>, // 45
        pub item_condition_and_remove_array: StageQuadList,  // 46 (placeholder elems)
        pub reward_drop_set_info_list: StageQuadList,        // 47 DropSetKey quad
        pub level_name_hash: u32,                            // 48 StringInfoKey
        pub global_effect_data: COptional<StageGlobalEffect<'a>>,              // 49 opt (placeholder inner)
        pub guide_effect_name: u32,                          // 50 StringInfoKey
        pub field_revive_info: u32,                          // 51 FieldReviveKey
        pub stage_icon_path: u32,                            // 52 StringInfoKey
        pub stage_text_icon_path: u32,                       // 53
        pub stage_image_path: u32,                           // 54
        pub complete_image_path: u32,                        // 55
        pub npc_shop_character_info: u32,                    // 56 CharacterKey
        pub close_dialog_speaker_character: u32,             // 57 CharacterKey
        pub close_dialog_string: LocalizableString<'a>,      // 58
        pub close_dialog_sound_event_name: u32,              // 59 StringInfoKey
        pub update_priority: u8,                             // 60
        pub complete_alert_type: u8,                         // 61
        pub stage_knowledge: u32,                            // 62 KnowledgeKey
        pub stage_game_event_data_list: CArray<StageGameEventData>,         // 63 (placeholder 16B elem; empty in vanilla)
        pub spawn_block_type_flag: u32,                      // 64
        pub weather_info: u16,                               // 65 GameGlobalEffectKey
        pub game_level_info_for_validation: u32,             // 66 GameLevelKey
        pub game_level_data_name_for_validation: u32,        // 67 StringInfoKey
        pub weather_start_blend_time: u32,                   // 68
        pub weather_end_blend_time: u32,                     // 69
        pub weather_ing_time: u32,                           // 70
        pub begin_time: u32,                                 // 71
        pub end_time: u32,                                   // 72
        pub change_time: u32,                                // 73
        // Fields 74-89: 16 u8 flags
        pub use_commute: u8,                                 // 74
        pub show_stage_icon: u8,                             // 75
        pub is_save: u8,                                     // 76
        pub save_schedule: u8,                               // 77
        pub has_dynamic_actor: u8,                           // 78
        pub is_force_spawn_after_retreat: u8,                // 79
        pub is_force_spawn_near_distance: u8,                // 80
        pub is_force_spawn_all_actor: u8,                    // 81
        pub disable_give_up: u8,                             // 82
        pub revive_in_place_hard_difficulty: u8,             // 83
        pub evade_projectile: u8,                            // 84
        pub follow_parent_reaction: u8,                      // 85
        pub is_playable_on_wanted: u8,                       // 86
        pub allow_accompany: u8,                             // 87
        pub use_revive_point_for_dead: u8,                   // 88
        pub ignore_faction_close: u8,                        // 89
        // ── 1.12 TRUNCATION (verified against CrimsonDesert_Steam 1.12.02,
        //    reader sub_101F78FF8) ──────────────────────────────────────────
        // StageInfo grew from the 1.06 layout this struct was written for to
        // **89 wire fields** by the 1.08 update — a new field
        // `_logoutMercenaryGroupInfoList` was inserted (between
        // `_executeTargetStageList` and `_hideMercenaryGroupInfoList`) and a
        // large run of new fields (`_randomSpawnCount`, `_rewardDropSetInfoList`,
        // `_scheduleStageCompleteAIEventList`, `_fieldReviveInfo`,
        // `_globalEffectData`, …) was added in the middle. The old fields 15-96
        // below no longer matched 1.12 (every record blob-fell-back → 0 typed).
        //
        // Fields 1-14 ARE byte-identical between 1.06 and 1.12 (IDA-verified +
        // confirmed by same-record byte diff). Everything from field 15 on is
        // captured verbatim by `tail_blob`, so the table round-trips byte-exact
        // on all 51441 records while exposing the placement-relevant prefix —
        // crucially `sequencer_desc` (field 7) which carries the funcnpc scene
        // name + world position. Full 89-field map: STAGEINFO_112_RE.md.
        // To type more 1.12 fields later, extend from here per that map.
    }
    tail: tail_blob;
}
