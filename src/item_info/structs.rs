use std::io::{self, Write};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::keys::*;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use crate::python_traits::{ToPyValue, WritePyValue, get_field};

// ── Simple structs ──────────────────────────────────────────────────────────

py_binary_struct! {
    pub struct OccupiedEquipSlotData {
        pub equip_slot_name_key: u32,
        pub equip_slot_name_index_list: CArray<u8>,
    }
}

// ItemInfo._cooltime is a 3 × i64 struct in the wire (24 bytes), not a single
// i64. Confirmed via IDA decomp of sub_101886C44 (the cooltime reader called
// from sub_101885C38): three sub_1006B90BC(a1, a2 + N) calls at memory
// offsets 0/8/16, each reading 8 bytes.
//
// MANUAL IMPLS (not py_binary_struct!) so WriteJsonValue can accept BOTH:
//   * legacy mod intent format: single number → {a:n, b:0, c:0}
//   * current vanilla parse format: object {a, b, c}
// SuperMod and other pre-existing v3 mods stored cooltime as a single i64.
// Without the dual accept, those mods break on serialize after the schema
// update.
#[derive(Debug)]
pub struct Cooltime {
    pub a: i64,
    pub b: i64,
    pub c: i64,
}

impl<'a> BinaryRead<'a> for Cooltime {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok(Cooltime {
            a: i64::read_from(data, offset)?,
            b: i64::read_from(data, offset)?,
            c: i64::read_from(data, offset)?,
        })
    }
}

impl<'a> crate::binary::BinaryReadTracked<'a> for Cooltime {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<crate::binary::FieldRange>,
    ) -> io::Result<Self> {
        let a = {
            let s = crate::binary::push_path(path, "a");
            let v = i64::read_tracked(data, offset, path, ranges)?;
            crate::binary::pop_path(path, s);
            v
        };
        let b = {
            let s = crate::binary::push_path(path, "b");
            let v = i64::read_tracked(data, offset, path, ranges)?;
            crate::binary::pop_path(path, s);
            v
        };
        let c = {
            let s = crate::binary::push_path(path, "c");
            let v = i64::read_tracked(data, offset, path, ranges)?;
            crate::binary::pop_path(path, s);
            v
        };
        Ok(Cooltime { a, b, c })
    }
}

impl BinaryWrite for Cooltime {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.a.write_to(w)?;
        self.b.write_to(w)?;
        self.c.write_to(w)
    }
}

impl ToJsonValue for Cooltime {
    fn to_json_value(&self) -> ::serde_json::Value {
        let mut d = ::serde_json::Map::new();
        d.insert("a".to_string(), ::serde_json::Value::from(self.a));
        d.insert("b".to_string(), ::serde_json::Value::from(self.b));
        d.insert("c".to_string(), ::serde_json::Value::from(self.c));
        ::serde_json::Value::Object(d)
    }
}

impl WriteJsonValue for Cooltime {
    fn write_from_json(w: &mut Vec<u8>, v: &::serde_json::Value) -> io::Result<()> {
        // Backward compat: legacy mod intents stored cooltime as a single
        // number. Promote to {a: n, b: 0, c: 0}.
        if let Some(n) = v.as_i64() {
            return Cooltime { a: n, b: 0, c: 0 }.write_to(w);
        }
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected object or number for Cooltime, got {:?}", v),
        ))?;
        let a = json_get_field(obj, "a")?.as_i64().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "Cooltime.a: expected i64"))?;
        let b = json_get_field(obj, "b")?.as_i64().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "Cooltime.b: expected i64"))?;
        let c = json_get_field(obj, "c")?.as_i64().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "Cooltime.c: expected i64"))?;
        Cooltime { a, b, c }.write_to(w)
    }
}

impl ToPyValue for Cooltime {
    fn to_py_value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        use pyo3::types::PyDictMethods;
        let d = PyDict::new(py);
        d.set_item("a", self.a)?;
        d.set_item("b", self.b)?;
        d.set_item("c", self.c)?;
        Ok(d.into_any().unbind())
    }
}

impl WritePyValue for Cooltime {
    fn write_from_py(w: &mut Vec<u8>, obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        // Accept both int (legacy) and dict.
        if let Ok(n) = obj.extract::<i64>() {
            Cooltime { a: n, b: 0, c: 0 }.write_to(w).map_err(|e| PyValueError::new_err(e.to_string()))?;
            return Ok(());
        }
        let d = obj.cast::<PyDict>()?;
        let a = get_field(d, "a")?.extract::<i64>()?;
        let b = get_field(d, "b")?.extract::<i64>()?;
        let c = get_field(d, "c")?.extract::<i64>()?;
        Cooltime { a, b, c }.write_to(w).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

// ItemInfo._maxChargedUseableCount: 3 × u32 wire struct (12 bytes). Same
// reasoning as Cooltime; same dual-accept JSON impl. Confirmed via IDA decomp
// of sub_101886C94 (three sub_1006B907C calls at offsets 0/4/8, each 4 bytes).
#[derive(Debug)]
pub struct MaxChargedUseableCount {
    pub a: u32,
    pub b: u32,
    pub c: u32,
}

impl<'a> BinaryRead<'a> for MaxChargedUseableCount {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok(MaxChargedUseableCount {
            a: u32::read_from(data, offset)?,
            b: u32::read_from(data, offset)?,
            c: u32::read_from(data, offset)?,
        })
    }
}

impl<'a> crate::binary::BinaryReadTracked<'a> for MaxChargedUseableCount {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<crate::binary::FieldRange>,
    ) -> io::Result<Self> {
        let a = {
            let s = crate::binary::push_path(path, "a");
            let v = u32::read_tracked(data, offset, path, ranges)?;
            crate::binary::pop_path(path, s);
            v
        };
        let b = {
            let s = crate::binary::push_path(path, "b");
            let v = u32::read_tracked(data, offset, path, ranges)?;
            crate::binary::pop_path(path, s);
            v
        };
        let c = {
            let s = crate::binary::push_path(path, "c");
            let v = u32::read_tracked(data, offset, path, ranges)?;
            crate::binary::pop_path(path, s);
            v
        };
        Ok(MaxChargedUseableCount { a, b, c })
    }
}

impl BinaryWrite for MaxChargedUseableCount {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.a.write_to(w)?;
        self.b.write_to(w)?;
        self.c.write_to(w)
    }
}

impl ToJsonValue for MaxChargedUseableCount {
    fn to_json_value(&self) -> ::serde_json::Value {
        let mut d = ::serde_json::Map::new();
        d.insert("a".to_string(), ::serde_json::Value::from(self.a));
        d.insert("b".to_string(), ::serde_json::Value::from(self.b));
        d.insert("c".to_string(), ::serde_json::Value::from(self.c));
        ::serde_json::Value::Object(d)
    }
}

impl WriteJsonValue for MaxChargedUseableCount {
    fn write_from_json(w: &mut Vec<u8>, v: &::serde_json::Value) -> io::Result<()> {
        if let Some(n) = v.as_u64() {
            if n > u32::MAX as u64 {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("MaxChargedUseableCount: number {} out of u32 range", n)));
            }
            return MaxChargedUseableCount { a: n as u32, b: n as u32, c: n as u32 }.write_to(w);
        }
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected object or number for MaxChargedUseableCount, got {:?}", v),
        ))?;
        let a = json_get_field(obj, "a")?.as_u64().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "MaxChargedUseableCount.a: expected u32"))? as u32;
        let b = json_get_field(obj, "b")?.as_u64().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "MaxChargedUseableCount.b: expected u32"))? as u32;
        let c = json_get_field(obj, "c")?.as_u64().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "MaxChargedUseableCount.c: expected u32"))? as u32;
        MaxChargedUseableCount { a, b, c }.write_to(w)
    }
}

impl ToPyValue for MaxChargedUseableCount {
    fn to_py_value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        use pyo3::types::PyDictMethods;
        let d = PyDict::new(py);
        d.set_item("a", self.a)?;
        d.set_item("b", self.b)?;
        d.set_item("c", self.c)?;
        Ok(d.into_any().unbind())
    }
}

impl WritePyValue for MaxChargedUseableCount {
    fn write_from_py(w: &mut Vec<u8>, obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        if let Ok(n) = obj.extract::<u32>() {
            MaxChargedUseableCount { a: n, b: n, c: n }.write_to(w).map_err(|e| PyValueError::new_err(e.to_string()))?;
            return Ok(());
        }
        let d = obj.cast::<PyDict>()?;
        let a = get_field(d, "a")?.extract::<u32>()?;
        let b = get_field(d, "b")?.extract::<u32>()?;
        let c = get_field(d, "c")?.extract::<u32>()?;
        MaxChargedUseableCount { a, b, c }.write_to(w).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

py_binary_struct! {
    // 1.05.01 has 5 fields per ItemIconData. The earlier 3-field revert was
    // wrong — empirical byte comparison of the 1.04 fixture (5358049 bytes,
    // 3-field ItemIconData) vs the user's live 1.05.01 (5338778 bytes, this
    // 5-field layout) shows 1.05.01 has 5 extra bytes per ItemIconData
    // entry. The 5 bytes are `highlight_icon_path` (u32) + `check_usable`
    // (u8). Confirmed via the diag_item1 example: with the 3-field schema
    // the parser fails on item 1's `item_icon_list[0]` at offset 113
    // ("CArray count 15386081 exceeds remaining bytes") because what the
    // schema thinks is `gimmick_state_list.__count__` is actually
    // `highlight_icon_path`. Adding the two fields back resolves the
    // misalignment cascade.
    //
    // The earlier IDA decompile of `sub_101885C38` (Mac binary) that
    // suggested 3 fields was misread or pointed at the wrong reader —
    // the live Win 1.05.01 wire format clearly has 5 fields here.
    // 2026-05-06: corrected wire field order per IDA decomp of sub_101884D3C
    // (the ItemIconData wire reader). The reads are: iconPath, highlightIconPath,
    // checkExistSealedData, gimmickStateList, checkUsable. Earlier order had
    // check_exist_sealed_data and check_usable in the wrong positions.
    pub struct ItemIconData {
        pub icon_path: StringInfoKey,
        pub highlight_icon_path: StringInfoKey,
        pub check_exist_sealed_data: u8,
        pub gimmick_state_list: CArray<u32>,
        pub check_usable: u8,
    }
}

py_binary_struct! {
    pub struct PassiveSkillLevel {
        pub skill: SkillKey,
        pub level: u32,
    }
}

py_binary_struct! {
    pub struct ReserveSlotTargetData {
        pub reserve_slot_info: ReserveSlotKey,
        pub condition_info: ConditionKey,
    }
}

py_binary_struct! {
    pub struct SocketMaterialItem {
        pub item: ItemKey,
        pub value: u64,
    }
}

py_binary_struct! {
    pub struct EnchantStatChange {
        pub stat: StatusKey,
        pub change_mb: i64,
    }
}

py_binary_struct! {
    pub struct EnchantLevelChange {
        pub stat: StatusKey,
        pub change_mb: i8,
    }
}

py_binary_struct! {
    pub struct EnchantStatData {
        pub max_stat_list: CArray<EnchantStatChange>,
        pub regen_stat_list: CArray<EnchantStatChange>,
        pub stat_list_static: CArray<EnchantStatChange>,
        pub stat_list_static_level: CArray<EnchantLevelChange>,
    }
}

py_binary_struct! {
    pub struct PriceFloor {
        pub price: u64,
        pub sym_no: u32,
        pub item_info_wrapper: ItemKey,
    }
}

py_binary_struct! {
    pub struct ItemPriceInfo {
        pub key: ItemKey,
        pub price: PriceFloor,
    }
}

py_binary_struct! {
    pub struct EquipmentBuff {
        pub buff: BuffKey,
        pub level: u32,
    }
}

py_binary_struct! {
    pub struct EnchantData {
        pub level: u16,
        pub enchant_stat_data: EnchantStatData,
        pub buy_price_list: CArray<ItemPriceInfo>,
        pub equip_buffs: CArray<EquipmentBuff>,
    }
}

py_binary_struct! {
    pub struct GimmickVisualPrefabData {
        pub tag_name_hash: u32,
        pub scale: [f32; 3],
        pub prefab_names: CArray<StringInfoKey>,
        pub animation_path_list: CArray<StringInfoKey>,
        pub use_gimmick_prefab: u8,
    }
}

py_binary_struct! {
    pub struct GameEventExecuteData {
        pub game_event_type: u8,
        pub player_condition: ConditionKey,
        pub target_condition: ConditionKey,
        pub event_condition: ConditionKey,
    }
}

py_binary_struct! {
    pub struct InventoryChangeData {
        pub game_event_execute_data: GameEventExecuteData,
        pub to_inventory_info: InventoryKey,
    }
}

py_binary_struct! {
    pub struct PageData<'a> {
        pub left_page_texture_path: CString<'a>,
        pub right_page_texture_path: CString<'a>,
        pub left_page_related_knowledge_info: KnowledgeKey,
        pub right_page_related_knowledge_info: KnowledgeKey,
    }
}

py_binary_struct! {
    pub struct InspectData<'a> {
        pub item_info: ItemKey,
        pub gimmick_info: GimmickInfoKey,
        pub character_info: CharacterKey,
        pub spawn_reason_hash: u32,
        pub socket_name: CString<'a>,
        pub speak_character_info: CharacterKey,
        pub inspect_target_tag: u32,
        pub reward_own_knowledge: u8,
        pub reward_knowledge_info: KnowledgeKey,
        pub item_desc: LocalizableString<'a>,
        pub board_key: u32,
        pub inspect_action_type: u8,
        pub gimmick_state_name_hash: u32,
        pub target_page_index: u32,
        pub is_left_page: u8,
        pub target_page_related_knowledge_info: KnowledgeKey,
        pub enable_read_after_reward: u8,
        pub refer_to_left_page_inspect_data: u8,
        pub inspect_effect_info_key: EffectKey,
        pub inspect_complete_effect_info_key: EffectKey,
    }
}

py_binary_struct! {
    pub struct InspectAction<'a> {
        pub action_name_hash: u32,
        pub catch_tag_name_hash: u32,
        pub catcher_socket_name: CString<'a>,
        pub catch_target_socket_name: CString<'a>,
    }
}

py_binary_struct! {
    pub struct ItemInfoSharpnessData {
        pub max_sharpness: u16,
        pub craft_tool_info: CraftToolKey,
        pub stat_data: EnchantStatData,
    }
}

py_binary_struct! {
    pub struct ItemBundleData {
        pub count_mb: u64,
        pub key: GimmickInfoKey,
    }
}

py_binary_struct! {
    pub struct UnitData<'a> {
        pub ui_component: CString<'a>,
        pub minimum: u32,
        pub icon_path: StringInfoKey,
        // 1.10: new u32 (name-hash) inserted between icon_path and item_name.
        // Only money items (MoneyTypeDefine.unit_data_list_map) carry UnitData,
        // so this is what broke Money_Copper etc. while regular items parsed.
        // Verified against live 1.10 iteminfo (Copper/Silver units reconcile).
        pub unk_hash_110: u32,
        pub item_name: LocalizableString<'a>,
        pub item_desc: LocalizableString<'a>,
    }
}

py_binary_struct! {
    pub struct MoneyUnitEntry<'a> {
        pub key: u32,
        pub value: UnitData<'a>,
    }
}

py_binary_struct! {
    pub struct MoneyTypeDefine<'a> {
        pub price_floor_value: u64,
        pub unit_data_list_map: CArray<MoneyUnitEntry<'a>>,
    }
}

py_binary_struct! {
    pub struct PrefabData {
        pub prefab_names: CArray<StringInfoKey>,
        pub equip_slot_list: CArray<u16>,
        pub tribe_gender_list: CArray<StringInfoKey>,
        pub is_craft_material: u8,
    }
}

py_binary_struct! {
    pub struct DockingChildData<'a> {
        pub gimmick_info_key: GimmickInfoKey,
        pub character_key: CharacterKey,
        pub item_key: ItemKey,
        pub attach_parent_socket_name: CString<'a>,
        pub attach_child_socket_name: CString<'a>,
        pub docking_tag_name_hash: [u32; 4],
        pub docking_equip_slot_no: u16,
        pub spawn_distance_level: u32,
        pub is_item_equip_docking_gimmick: u8,
        pub send_damage_to_parent: u8,
        pub is_body_part: u8,
        pub docking_type: u8,
        pub is_summoner_team: u8,
        pub is_player_only: u8,
        pub is_npc_only: ConditionKey,
        pub is_sync_break_parent: u8,
        pub hit_part: u8,
        pub detected_by_npc: u8,
        pub is_bag_docking: u8,
        pub enable_collision: u8,
        pub disable_collision_with_other_gimmick: u8,
        // 1.0.8: new u8 field in DockingChildData (IDA shows 14 u8 reads vs 13 in Rust)
        pub unk_docking_108: u8,
        pub docking_slot_key: CString<'a>,
        pub inherit_summoner: u8,
        pub summon_tag_name_hash: [u32; 4],
    }
}

py_binary_struct! {
    pub struct PatternParamString<'a> {
        pub flag: u8,
        pub unk_flag_2: u8,
        pub unk_value: [u32; 2],
        pub param_string: CString<'a>,
    }
}

py_binary_struct! {
    pub struct PatternDescriptionData<'a> {
        pub pattern_description_info: u32,
        pub param_string_list: CArray<PatternParamString<'a>>,
    }
}

py_binary_struct! {
    pub struct RepairData {
        pub resource_item_info: ItemKey,
        pub repair_value: u16,
        pub repair_style: u8,
        pub resource_item_count: u64,
    }
}

// ── SubItem (variant) ───────────────────────────────────────────────────────

#[derive(Debug)]
pub enum SubItemValue {
    Item(ItemKey),
    Character(CharacterKey),
    Gimmick(GimmickInfoKey),
    None,
}

#[derive(Debug)]
pub struct SubItem {
    pub type_id: u8,
    pub value: SubItemValue,
}

impl<'a> BinaryRead<'a> for SubItem {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let type_id = u8::read_from(data, offset)?;
        let value = match type_id {
            0 => SubItemValue::Item(ItemKey::read_from(data, offset)?),
            3 => SubItemValue::Character(CharacterKey::read_from(data, offset)?),
            9 => SubItemValue::Gimmick(GimmickInfoKey::read_from(data, offset)?),
            14 | 15 | 255 => SubItemValue::None,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown SubItem type: {}", type_id),
                ));
            }
        };
        Ok(SubItem { type_id, value })
    }
}

impl<'a> BinaryReadTracked<'a> for SubItem {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let saved = push_path(path, "type_id");
        let type_id = u8::read_tracked(data, offset, path, ranges)?;
        pop_path(path, saved);

        let saved = push_path(path, "value");
        let value = match type_id {
            0 => SubItemValue::Item(ItemKey::read_tracked(data, offset, path, ranges)?),
            3 => SubItemValue::Character(CharacterKey::read_tracked(data, offset, path, ranges)?),
            9 => SubItemValue::Gimmick(GimmickInfoKey::read_tracked(data, offset, path, ranges)?),
            14 | 15 | 255 => SubItemValue::None,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown SubItem type: {}", type_id),
                ));
            }
        };
        pop_path(path, saved);
        Ok(SubItem { type_id, value })
    }
}

impl BinaryWrite for SubItem {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.type_id.write_to(w)?;
        match &self.value {
            SubItemValue::Item(k) => k.write_to(w),
            SubItemValue::Character(k) => k.write_to(w),
            SubItemValue::Gimmick(k) => k.write_to(w),
            SubItemValue::None => Ok(()),
        }
    }
}

impl ToPyValue for SubItem {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);
        d.set_item("type_id", self.type_id)?;
        match &self.value {
            SubItemValue::Item(k) => d.set_item("value", k.0)?,
            SubItemValue::Character(k) => d.set_item("value", k.0)?,
            SubItemValue::Gimmick(k) => d.set_item("value", k.0)?,
            SubItemValue::None => d.set_item("value", py.None())?,
        };
        Ok(d.into_any().unbind())
    }
}

impl WritePyValue for SubItem {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let d = obj.cast::<PyDict>()?;
        let type_id: u8 = get_field(d, "type_id")?.extract()?;
        w.push(type_id);
        match type_id {
            0 | 3 | 9 => {
                let v: u32 = get_field(d, "value")?.extract()?;
                w.extend_from_slice(&v.to_le_bytes());
            }
            14 | 15 => {}
            _ => {
                return Err(PyValueError::new_err(format!(
                    "invalid SubItem type_id: {}",
                    type_id
                )));
            }
        }
        Ok(())
    }
}

// JSON mirror of the Python bridge above. SubItem is a tagged union
// where `type_id` selects which variant of `value` is in the wire
// format: 0/3/9 → u32 key, 14 → no value bytes. JSON form is a flat
// `{type_id, value}` dict, with `value: null` for the no-payload case.
impl ToJsonValue for SubItem {
    fn to_json_value(&self) -> ::serde_json::Value {
        let mut d = ::serde_json::Map::new();
        d.insert("type_id".to_string(), ::serde_json::Value::from(self.type_id));
        let v = match &self.value {
            SubItemValue::Item(k) => ::serde_json::Value::from(k.0),
            SubItemValue::Character(k) => ::serde_json::Value::from(k.0),
            SubItemValue::Gimmick(k) => ::serde_json::Value::from(k.0),
            SubItemValue::None => ::serde_json::Value::Null,
        };
        d.insert("value".to_string(), v);
        ::serde_json::Value::Object(d)
    }
}

impl WriteJsonValue for SubItem {
    fn write_from_json(w: &mut Vec<u8>, v: &::serde_json::Value) -> ::std::io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected object for SubItem, got {:?}", v),
        ))?;
        let type_id = json_get_field(obj, "type_id")?
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "SubItem.type_id: expected u8 number"))?;
        if type_id > u8::MAX as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("SubItem.type_id {} out of u8 range", type_id)));
        }
        let type_id = type_id as u8;
        w.push(type_id);
        match type_id {
            0 | 3 | 9 => {
                let value = json_get_field(obj, "value")?;
                let n = value.as_u64().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("SubItem.value: expected u32 number, got {:?}", value),
                ))?;
                if n > u32::MAX as u64 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                        format!("SubItem.value {} out of u32 range", n)));
                }
                w.extend_from_slice(&(n as u32).to_le_bytes());
            }
            14 | 15 => {} // no payload
            _ => {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("invalid SubItem.type_id: {}", type_id)));
            }
        }
        Ok(())
    }
}

// ── DropDefaultData ─────────────────────────────────────────────────────────

py_binary_struct! {
    pub struct DropDefaultData {
        pub drop_enchant_level: u16,
        pub socket_item_list: CArray<ItemKey>,
        pub add_socket_material_item_list: CArray<SocketMaterialItem>,
        pub default_sub_item: SubItem,
        pub socket_valid_count: u8,
        pub use_socket: u8,
    }
}

// ── SealableItemInfo (variant) ──────────────────────────────────────────────

#[derive(Debug)]
pub enum SealableValue<'a> {
    Item(ItemKey),
    Gimmick(GimmickInfoKey),
    String(CString<'a>),
    Character(CharacterKey),
    Tribe(TribeInfoKey),
}

#[derive(Debug)]
pub struct SealableItemInfo<'a> {
    pub type_tag: u8,
    pub item_key: ItemKey,
    pub unknown0: u64,
    pub value: SealableValue<'a>,
}

impl<'a> BinaryRead<'a> for SealableItemInfo<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let type_tag = u8::read_from(data, offset)?;
        let item_key = ItemKey::read_from(data, offset)?;
        let unknown0 = u64::read_from(data, offset)?;
        let value = match type_tag {
            0 => SealableValue::Item(ItemKey::read_from(data, offset)?),
            1 => SealableValue::Gimmick(GimmickInfoKey::read_from(data, offset)?),
            2 => SealableValue::String(CString::read_from(data, offset)?),
            3 => SealableValue::Character(CharacterKey::read_from(data, offset)?),
            4 => SealableValue::Tribe(TribeInfoKey::read_from(data, offset)?),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown SealableItemInfo type: {}", type_tag),
                ));
            }
        };
        Ok(SealableItemInfo {
            type_tag,
            item_key,
            unknown0,
            value,
        })
    }
}

impl<'a> BinaryReadTracked<'a> for SealableItemInfo<'a> {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let saved = push_path(path, "type_tag");
        let type_tag = u8::read_tracked(data, offset, path, ranges)?;
        pop_path(path, saved);

        let saved = push_path(path, "item_key");
        let item_key = ItemKey::read_tracked(data, offset, path, ranges)?;
        pop_path(path, saved);

        let saved = push_path(path, "unknown0");
        let unknown0 = u64::read_tracked(data, offset, path, ranges)?;
        pop_path(path, saved);

        let saved = push_path(path, "value");
        let value = match type_tag {
            0 => SealableValue::Item(ItemKey::read_tracked(data, offset, path, ranges)?),
            1 => SealableValue::Gimmick(GimmickInfoKey::read_tracked(data, offset, path, ranges)?),
            2 => SealableValue::String(CString::read_tracked(data, offset, path, ranges)?),
            3 => SealableValue::Character(CharacterKey::read_tracked(data, offset, path, ranges)?),
            4 => SealableValue::Tribe(TribeInfoKey::read_tracked(data, offset, path, ranges)?),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown SealableItemInfo type: {}", type_tag),
                ));
            }
        };
        pop_path(path, saved);
        Ok(SealableItemInfo {
            type_tag,
            item_key,
            unknown0,
            value,
        })
    }
}

impl BinaryWrite for SealableItemInfo<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.type_tag.write_to(w)?;
        self.item_key.write_to(w)?;
        self.unknown0.write_to(w)?;
        match &self.value {
            SealableValue::Item(k) => k.write_to(w),
            SealableValue::Gimmick(k) => k.write_to(w),
            SealableValue::String(s) => s.write_to(w),
            SealableValue::Character(k) => k.write_to(w),
            SealableValue::Tribe(k) => k.write_to(w),
        }
    }
}

impl ToPyValue for SealableItemInfo<'_> {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);
        d.set_item("type_tag", self.type_tag)?;
        d.set_item("item_key", self.item_key.0)?;
        d.set_item("unknown0", self.unknown0)?;
        match &self.value {
            SealableValue::Item(k) => d.set_item("value", k.0)?,
            SealableValue::Gimmick(k) => d.set_item("value", k.0)?,
            SealableValue::String(s) => d.set_item("value", s.data)?,
            SealableValue::Character(k) => d.set_item("value", k.0)?,
            SealableValue::Tribe(k) => d.set_item("value", k.0)?,
        };
        Ok(d.into_any().unbind())
    }
}

impl WritePyValue for SealableItemInfo<'_> {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let d = obj.cast::<PyDict>()?;
        let type_tag: u8 = get_field(d, "type_tag")?.extract()?;
        let item_key: u32 = get_field(d, "item_key")?.extract()?;
        let unknown0: u64 = get_field(d, "unknown0")?.extract()?;
        w.push(type_tag);
        w.extend_from_slice(&item_key.to_le_bytes());
        w.extend_from_slice(&unknown0.to_le_bytes());
        let value_obj = get_field(d, "value")?;
        match type_tag {
            0 | 1 | 3 | 4 => {
                let v: u32 = value_obj.extract()?;
                w.extend_from_slice(&v.to_le_bytes());
            }
            2 => {
                let s: String = value_obj.extract()?;
                w.extend_from_slice(&(s.len() as u32).to_le_bytes());
                w.extend_from_slice(s.as_bytes());
            }
            _ => {
                return Err(PyValueError::new_err(format!(
                    "invalid sealable type_tag: {}",
                    type_tag
                )));
            }
        }
        Ok(())
    }
}

// JSON mirror of the Python bridge for SealableItemInfo. Wire format:
// `[type_tag:u8][item_key:u32][unknown0:u64][value:variant]`.
// `type_tag` selects the variant of `value`: 0/1/3/4 are u32 keys,
// 2 is a CString (length-prefixed UTF-8 + null padding handled by
// the wire layer).
impl ToJsonValue for SealableItemInfo<'_> {
    fn to_json_value(&self) -> ::serde_json::Value {
        let mut d = ::serde_json::Map::new();
        d.insert("type_tag".to_string(), ::serde_json::Value::from(self.type_tag));
        d.insert("item_key".to_string(), ::serde_json::Value::from(self.item_key.0));
        d.insert("unknown0".to_string(), ::serde_json::Value::from(self.unknown0));
        let v = match &self.value {
            SealableValue::Item(k) => ::serde_json::Value::from(k.0),
            SealableValue::Gimmick(k) => ::serde_json::Value::from(k.0),
            SealableValue::String(s) => ::serde_json::Value::from(s.data.to_string()),
            SealableValue::Character(k) => ::serde_json::Value::from(k.0),
            SealableValue::Tribe(k) => ::serde_json::Value::from(k.0),
        };
        d.insert("value".to_string(), v);
        ::serde_json::Value::Object(d)
    }
}

impl WriteJsonValue for SealableItemInfo<'_> {
    fn write_from_json(w: &mut Vec<u8>, v: &::serde_json::Value) -> ::std::io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected object for SealableItemInfo, got {:?}", v),
        ))?;
        let type_tag = json_get_field(obj, "type_tag")?
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "SealableItemInfo.type_tag: expected u8 number"))?;
        if type_tag > u8::MAX as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("SealableItemInfo.type_tag {} out of u8 range", type_tag)));
        }
        let type_tag = type_tag as u8;
        let item_key = json_get_field(obj, "item_key")?
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "SealableItemInfo.item_key: expected u32 number"))?;
        if item_key > u32::MAX as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("SealableItemInfo.item_key {} out of u32 range", item_key)));
        }
        let unknown0 = json_get_field(obj, "unknown0")?
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "SealableItemInfo.unknown0: expected u64 number"))?;
        w.push(type_tag);
        w.extend_from_slice(&(item_key as u32).to_le_bytes());
        w.extend_from_slice(&unknown0.to_le_bytes());
        let value = json_get_field(obj, "value")?;
        match type_tag {
            0 | 1 | 3 | 4 => {
                let n = value.as_u64().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("SealableItemInfo.value: expected u32 number for tag={}", type_tag),
                ))?;
                if n > u32::MAX as u64 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                        format!("SealableItemInfo.value {} out of u32 range", n)));
                }
                w.extend_from_slice(&(n as u32).to_le_bytes());
            }
            2 => {
                let s = value.as_str().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    "SealableItemInfo.value: expected string for tag=2",
                ))?;
                w.extend_from_slice(&(s.len() as u32).to_le_bytes());
                w.extend_from_slice(s.as_bytes());
            }
            _ => {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("invalid SealableItemInfo.type_tag: {}", type_tag)));
            }
        }
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_item_none_roundtrip() {
        let bytes = [14u8];
        let mut offset = 0;
        let si = SubItem::read_from(&bytes, &mut offset).unwrap();
        assert_eq!(offset, 1);
        assert_eq!(si.type_id, 14);

        let mut out = Vec::new();
        si.write_to(&mut out).unwrap();
        assert_eq!(out, bytes);
    }

    #[test]
    fn test_sub_item_item_key_roundtrip() {
        let mut bytes = vec![0u8];
        bytes.extend_from_slice(&42u32.to_le_bytes());
        let mut offset = 0;
        let si = SubItem::read_from(&bytes, &mut offset).unwrap();
        assert_eq!(offset, 5);
        assert_eq!(si.type_id, 0);

        let mut out = Vec::new();
        si.write_to(&mut out).unwrap();
        assert_eq!(out, bytes);
    }

    #[test]
    fn test_sealable_item_info_type0_roundtrip() {
        let mut bytes = Vec::new();
        bytes.push(0);
        bytes.extend_from_slice(&100u32.to_le_bytes());
        bytes.extend_from_slice(&999u64.to_le_bytes());
        bytes.extend_from_slice(&200u32.to_le_bytes());
        let mut offset = 0;
        let si = SealableItemInfo::read_from(&bytes, &mut offset).unwrap();
        assert_eq!(offset, bytes.len());

        let mut out = Vec::new();
        si.write_to(&mut out).unwrap();
        assert_eq!(out, bytes);
    }

    #[test]
    fn test_sealable_item_info_type2_string_roundtrip() {
        let mut bytes = Vec::new();
        bytes.push(2);
        bytes.extend_from_slice(&100u32.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(b"test");
        let mut offset = 0;
        let si = SealableItemInfo::read_from(&bytes, &mut offset).unwrap();
        assert_eq!(offset, bytes.len());

        let mut out = Vec::new();
        si.write_to(&mut out).unwrap();
        assert_eq!(out, bytes);
    }

    #[test]
    fn test_drop_default_data_roundtrip() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.push(14);
        bytes.push(0);
        bytes.push(0);

        let mut offset = 0;
        let dd = DropDefaultData::read_from(&bytes, &mut offset).unwrap();
        assert_eq!(offset, bytes.len());

        let mut out = Vec::new();
        dd.write_to(&mut out).unwrap();
        assert_eq!(out, bytes);
    }
}
