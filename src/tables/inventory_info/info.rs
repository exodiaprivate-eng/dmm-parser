//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader (Tier IDA verified 2026-05-19 vs CrimsonDesert.exe md5
//! 3d614280…): `sub_1410C20A0` — the InventoryInfo deserializer (found
//! via xref to the "InventoryInfo" reflection class-name block at
//! 0x144AF2DE0+). `a2` is `_WORD*`, so IDA's `a2+N` = byte offset 2N.
//! All 18 top-level fields confirmed in order/width. Per-element reader
//! for inventory_move_data_list is `sub_1410C1F20` (160-byte composite,
//! 10 fields). (Previously-cited `sub_1410E05E0` / `sub_1410E0460` were
//! stale — the former lands in an unrelated u32-key table reader in
//! this build.)
//!
//! Element-reader type confirmations (the kind byte-roundtrip can't prove):
//!   - pushable/excluded element (sub_1410E6560): u16 item_group
//!     (sub_1410E2CA0 reads 2 wire bytes + hash remap) + u8 item_type.
//!   - collection element (sub_1410E58C0): u32 item_info
//!     (sub_1410E1B70: 4 wire bytes → u16 RAM) + 8 RAW wire bytes copied
//!     opaquely (no float ops at read time). Rust models the 8 bytes as
//!     `u64` — bit-preserving and correct for the wire; the "f32/f32"
//!     reading is a usage-site guess the reader neither confirms nor
//!     denies.
//!   - key_guide_local_string_info (sub_1410E1600): 4 wire bytes → u16
//!     RAM. Rust `u32` correct for wire.
//!   - InventoryMoveData.from/to_inventory_info (sub_1410E64B0): 2 wire
//!     bytes (u16) + hash remap. convert_money_item_info (sub_1410E1B70):
//!     4 wire bytes (u32) → u16 RAM.
//! As with StoreInfo, all ID-reference fields read a wider value off the
//! wire than they keep in RAM; the Rust structs model the *wire* width,
//! which is what read/write roundtrip and modding require.
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1.  u16 key                                   (_key, pabgh format 2)
//!   2.  CString string_key                        (_stringKey)
//!   3.  u8 is_blocked                             (_isBlocked)
//!   4.  CArray<InventoryPushableData> pushable_item_type_list
//!       (_pushableItemTypeList, sub_141103FB0; element wire = u16
//!       _itemGroup + u8 _itemType = 3 bytes per element)
//!   5.  CArray<InventoryPushableData> excluded_item_type_list
//!       (_excludedItemTypeList, same shape)
//!   6.  CArray<InventoryMoveData> inventory_move_data_list
//!       (_inventoryMoveDataList, sub_141114720 → sub_1410E0460
//!       per-element reader; see InventoryMoveData)
//!   7.  u16 default_slot_count                    (_defaultSlotCount)
//!   8.  u16 max_slot_count                        (_maxSlotCount)
//!   9.  LocalizableString push_item_alert_ui_text (_pushItemAlertUIText)
//!  10.  LocalizableString inventory_name_ui_text  (_InventoryNameUIText)
//!  11.  u32 key_guide_local_string_info           (_keyGuideLocalStringInfo,
//!       sub_1410FF050 → qword_145F0DA60)
//!  12.  u8 pushable_check_type                    (_pushableCheckType)
//!  13.  u32 npc_usable_cooltime_min               (_npcUsableData
//!       _cooltimeMin_inGame, 8-byte struct first half)
//!  14.  u32 npc_usable_cooltime_max               (_npcUsableData
//!       _cooltimeMax_inGame, 8-byte struct second half)
//!  15.  u8 is_moveable_inventory                  (_isMoveableInventory)
//!  16.  u8 need_save_slot_count                   (_needSaveSlotCount)
//!  17.  u8 is_pushable_item_only_one              (_isPushableItemOnlyOne)
//!  18.  CArray<InventoryCollectionItemData> collection_item_list
//!       (_collectionItemList, sub_141103310 — element wire = u32
//!       _itemInfo + 8 raw bytes = 12 bytes)
//!
//! `InventoryMoveData` (sub_1410E0460) embeds an `OptionalGameCondition`
//! (sub_141103B30 → sub_141CEA810). Stream-mode reading uses lane A's
//! public `GameConditionNode::read_from`; the 0.2% Raw fallback in
//! `GameCondition::read_from` is unreachable here because we don't
//! have a slice-bounded context. If any vanilla entry hit an
//! anti-disassembly variant (tags 54/286), parsing would fail; the
//! roundtrip test below would catch it.

use crate::binary::optional_game_condition::OptionalGameCondition;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

// InventoryPushableData per canonical Mac names: 2 fields
// (_itemGroup u16, _itemType u8). Wire 3 bytes.
py_binary_struct! {
    pub struct InventoryPushableData {
        pub item_group: u16,
        pub item_type: u8,
    }
}

// InventoryItemMoveData (_itemMoveDataList element). 13 wire bytes
// per IDA sub_1411148E0:
//   u32 lookup_a (sub_1410FF5C0 → ItemInfoKey)
//   u32 lookup_b (sub_1410FF5C0 → ItemInfoKey)
//   u32 lookup_c (sub_141100370 → ?)
//   u8 flag
py_binary_struct! {
    pub struct InventoryItemMoveData {
        pub from_item_info: u32,
        pub to_item_info: u32,
        pub lookup_extra: u32,
        pub flag: u8,
        // ── 1.18.00: the exe's `ItemMoveData` went 4 → 5 fields, gaining
        // `_playerCondition`. Missing it made every element 4 bytes short, which
        // desynced the `move_condition` GameCondition read that follows the list
        // and surfaced as a bogus "unknown GameCondition case_tag: 72/75" — the
        // node tag is only ever 0..8, so those were misalignment garbage, NOT a
        // new tag. That silently blob-fell-back inventory key 0x2 "Character"
        // and cost the "I Like Space" mod 3 of its 13 intents.
        pub player_condition: u32,
    }
}

// InventoryCollectionItemData (_collectionItemList element). 12 wire
// bytes per IDA sub_141103310:
//   u32 lookup (sub_1410FF5C0 → ItemInfoKey)
//   8 raw wire bytes (per Mac error string ordering, paired f32/f32
//   most likely; promoted to u64 for field-level scalar access while
//   preserving any potential NaN bit patterns).
py_binary_struct! {
    pub struct InventoryCollectionItemData {
        pub item_info: u32,
        pub raw_8: u64,
    }
}

/// InventoryMoveData composite per IDA sub_1410E0460.
/// 10 fields matching canonical Mac names (`InventoryMoveData` in
/// docs/449_TABLE_CATALOG.md).
///
/// `move_condition` is an `OptionalGameCondition` — the
/// `GameConditionNode` is self-delimiting (tag dispatch, all leaf variants
/// include their own payload). There is no separate tail after the node;
/// `condition_fail_text` immediately follows.
///
/// Earlier revisions of this struct had a `move_condition_tail: Option<[u8;5]>`
/// field. That was a misidentification: bytes [2422..2427) in the Character
/// entry (k=2) are the last 5 bytes of the `GameConditionNode` payload, not a
/// separate field. Inserting an extra 5-byte read caused "not enough data"
/// when the real `condition_fail_text` was read from the wrong offset.
#[derive(Debug)]
pub struct InventoryMoveData<'a> {
    pub type_: u8,
    pub from_inventory_info: u16,
    pub to_inventory_info: u16,
    pub convert_money_item_info: u32,
    pub key_guide_text: LocalizableString<'a>,
    pub move_all_key_guide_text: LocalizableString<'a>,
    pub modal_text: LocalizableString<'a>,
    pub item_move_data_list: CArray<InventoryItemMoveData>,
    pub move_condition: OptionalGameCondition<'a>,
    pub condition_fail_text: LocalizableString<'a>,
}

impl<'a> InventoryMoveData<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let type_ = u8::read_from(data, offset)?;
        let from_inventory_info = u16::read_from(data, offset)?;
        let to_inventory_info = u16::read_from(data, offset)?;
        let convert_money_item_info = u32::read_from(data, offset)?;
        let key_guide_text = LocalizableString::read_from(data, offset)?;
        let move_all_key_guide_text = LocalizableString::read_from(data, offset)?;
        let modal_text = LocalizableString::read_from(data, offset)?;
        let item_move_data_list = CArray::<InventoryItemMoveData>::read_from(data, offset)?;
        let move_condition = OptionalGameCondition::read_from(data, offset)?;
        let condition_fail_text = LocalizableString::read_from(data, offset)?;
        Ok(Self {
            type_, from_inventory_info, to_inventory_info, convert_money_item_info,
            key_guide_text, move_all_key_guide_text, modal_text,
            item_move_data_list, move_condition, condition_fail_text,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.type_.write_to(w)?;
        self.from_inventory_info.write_to(w)?;
        self.to_inventory_info.write_to(w)?;
        self.convert_money_item_info.write_to(w)?;
        self.key_guide_text.write_to(w)?;
        self.move_all_key_guide_text.write_to(w)?;
        self.modal_text.write_to(w)?;
        self.item_move_data_list.write_to(w)?;
        self.move_condition.write_to(w)?;
        self.condition_fail_text.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("type_".to_string(), self.type_.to_json_value());
        m.insert("from_inventory_info".to_string(), self.from_inventory_info.to_json_value());
        m.insert("to_inventory_info".to_string(), self.to_inventory_info.to_json_value());
        m.insert("convert_money_item_info".to_string(), self.convert_money_item_info.to_json_value());
        m.insert("key_guide_text".to_string(), self.key_guide_text.to_json_value());
        m.insert("move_all_key_guide_text".to_string(), self.move_all_key_guide_text.to_json_value());
        m.insert("modal_text".to_string(), self.modal_text.to_json_value());
        m.insert("item_move_data_list".to_string(), self.item_move_data_list.to_json_value());
        m.insert("move_condition".to_string(), self.move_condition.to_json_value());
        m.insert("condition_fail_text".to_string(), self.condition_fail_text.to_json_value());
        m
    }
}

impl<'a> ToJsonValue for InventoryMoveData<'a> {
    fn to_json_value(&self) -> Value {
        Value::Object(self.to_json_dict())
    }
}

impl<'a> WriteJsonValue for InventoryMoveData<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "InventoryMoveData: expected object",
        ))?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "type_")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "from_inventory_info")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "to_inventory_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "convert_money_item_info")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key_guide_text")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "move_all_key_guide_text")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "modal_text")?)?;
        <CArray<InventoryItemMoveData> as WriteJsonValue>::write_from_json(
            w, json_get_field(obj, "item_move_data_list")?,
        )?;
        OptionalGameCondition::write_from_json(w, json_get_field(obj, "move_condition")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "condition_fail_text")?)?;
        Ok(())
    }
}

impl<'a> BinaryRead<'a> for InventoryMoveData<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Self::read_from(data, offset)
    }
}

impl<'a> BinaryReadTracked<'a> for InventoryMoveData<'a> {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        _path: &mut String,
        _ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        Self::read_from(data, offset)
    }
}

impl<'a> BinaryWrite for InventoryMoveData<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        Self::write_to(self, w)
    }
}

#[derive(Debug)]
pub struct InventoryInfo<'a> {
    pub string_key: CString<'a>,
    pub key: u16,
    pub is_blocked: u8,
    pub is_pushable_item_only_one: u8,
    pub is_moveable_inventory: u8,
    pub need_save_slot_count: u8,
    pub pushable_check_type: u8,
    pub default_slot_count: u16,
    pub key_guide_local_string_info: u32,
    pub max_slot_count: u16,
    pub pushable_item_type_list: CArray<InventoryPushableData>,
    pub excluded_item_type_list: CArray<InventoryPushableData>,
    pub inventory_move_data_list: CArray<InventoryMoveData<'a>>,
    pub push_item_alert_ui_text: LocalizableString<'a>,
    pub inventory_name_ui_text: LocalizableString<'a>,
    /// `_npcUsableData` (8-byte struct: 2× u32 cooltime min/max).
    /// First half (`cooltime_min_in_game`).
    pub npc_usable_cooltime_min: u32,
    /// Second half (`cooltime_max_in_game`).
    pub npc_usable_cooltime_max: u32,
    pub collection_item_list: CArray<InventoryCollectionItemData>,
}

impl<'a> InventoryInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("InventoryInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }

    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let string_key = CString::read_from(data, offset)?;
        let key = u16::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let is_pushable_item_only_one = u8::read_from(data, offset)?;
        let is_moveable_inventory = u8::read_from(data, offset)?;
        let need_save_slot_count = u8::read_from(data, offset)?;
        let pushable_check_type = u8::read_from(data, offset)?;
        let default_slot_count = u16::read_from(data, offset)?;
        let key_guide_local_string_info = u32::read_from(data, offset)?;
        let max_slot_count = u16::read_from(data, offset)?;
        let pushable_item_type_list = CArray::<InventoryPushableData>::read_from(data, offset)?;
        let excluded_item_type_list = CArray::<InventoryPushableData>::read_from(data, offset)?;
        let inventory_move_data_list = CArray::<InventoryMoveData>::read_from(data, offset)?;
        let push_item_alert_ui_text = LocalizableString::read_from(data, offset)?;
        let inventory_name_ui_text = LocalizableString::read_from(data, offset)?;
        let npc_usable_cooltime_min = u32::read_from(data, offset)?;
        let npc_usable_cooltime_max = u32::read_from(data, offset)?;
        let collection_item_list = CArray::<InventoryCollectionItemData>::read_from(data, offset)?;
        Ok(Self {
            key, string_key, is_blocked, pushable_item_type_list,
            excluded_item_type_list, inventory_move_data_list, default_slot_count,
            max_slot_count, push_item_alert_ui_text, inventory_name_ui_text,
            key_guide_local_string_info, pushable_check_type,
            npc_usable_cooltime_min, npc_usable_cooltime_max,
            is_moveable_inventory, need_save_slot_count,
            is_pushable_item_only_one, collection_item_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.string_key.write_to(w)?;
        self.key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.is_pushable_item_only_one.write_to(w)?;
        self.is_moveable_inventory.write_to(w)?;
        self.need_save_slot_count.write_to(w)?;
        self.pushable_check_type.write_to(w)?;
        self.default_slot_count.write_to(w)?;
        self.key_guide_local_string_info.write_to(w)?;
        self.max_slot_count.write_to(w)?;
        self.pushable_item_type_list.write_to(w)?;
        self.excluded_item_type_list.write_to(w)?;
        self.inventory_move_data_list.write_to(w)?;
        self.push_item_alert_ui_text.write_to(w)?;
        self.inventory_name_ui_text.write_to(w)?;
        self.npc_usable_cooltime_min.write_to(w)?;
        self.npc_usable_cooltime_max.write_to(w)?;
        self.collection_item_list.write_to(w)?;
        Ok(())
    }

    /// Tracked variant of `read_with_size`. Top-level FieldRange per field.
    pub fn read_tracked_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let start = *offset;
        let string_key = track_read_field::<CString<'a>>(data, offset, path, ranges, "string_key", "CString")?;
        let key = track_read_field::<u16>(data, offset, path, ranges, "key", "u16")?;
        let is_blocked = track_read_field::<u8>(data, offset, path, ranges, "is_blocked", "u8")?;
        let is_pushable_item_only_one = track_read_field::<u8>(data, offset, path, ranges, "is_pushable_item_only_one", "u8")?;
        let is_moveable_inventory = track_read_field::<u8>(data, offset, path, ranges, "is_moveable_inventory", "u8")?;
        let need_save_slot_count = track_read_field::<u8>(data, offset, path, ranges, "need_save_slot_count", "u8")?;
        let pushable_check_type = track_read_field::<u8>(data, offset, path, ranges, "pushable_check_type", "u8")?;
        let default_slot_count = track_read_field::<u16>(data, offset, path, ranges, "default_slot_count", "u16")?;
        let key_guide_local_string_info = track_read_field::<u32>(data, offset, path, ranges, "key_guide_local_string_info", "u32")?;
        let max_slot_count = track_read_field::<u16>(data, offset, path, ranges, "max_slot_count", "u16")?;
        let pushable_item_type_list = track_read_field::<CArray<InventoryPushableData>>(data, offset, path, ranges, "pushable_item_type_list", "CArray<InventoryPushableData>")?;
        let excluded_item_type_list = track_read_field::<CArray<InventoryPushableData>>(data, offset, path, ranges, "excluded_item_type_list", "CArray<InventoryPushableData>")?;
        let inventory_move_data_list = track_read_field::<CArray<InventoryMoveData<'a>>>(data, offset, path, ranges, "inventory_move_data_list", "CArray<InventoryMoveData>")?;
        let push_item_alert_ui_text = track_read_field::<LocalizableString<'a>>(data, offset, path, ranges, "push_item_alert_ui_text", "LocalizableString")?;
        let inventory_name_ui_text = track_read_field::<LocalizableString<'a>>(data, offset, path, ranges, "inventory_name_ui_text", "LocalizableString")?;
        let npc_usable_cooltime_min = track_read_field::<u32>(data, offset, path, ranges, "npc_usable_cooltime_min", "u32")?;
        let npc_usable_cooltime_max = track_read_field::<u32>(data, offset, path, ranges, "npc_usable_cooltime_max", "u32")?;
        let collection_item_list = track_read_field::<CArray<InventoryCollectionItemData>>(data, offset, path, ranges, "collection_item_list", "CArray<InventoryCollectionItemData>")?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("InventoryInfo: consumed {} bytes, expected {}", consumed, entry_size)));
        }
        Ok(Self {
            key, string_key, is_blocked, pushable_item_type_list,
            excluded_item_type_list, inventory_move_data_list, default_slot_count,
            max_slot_count, push_item_alert_ui_text, inventory_name_ui_text,
            key_guide_local_string_info, pushable_check_type,
            npc_usable_cooltime_min, npc_usable_cooltime_max,
            is_moveable_inventory, need_save_slot_count,
            is_pushable_item_only_one, collection_item_list,
        })
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("is_pushable_item_only_one".to_string(), self.is_pushable_item_only_one.to_json_value());
        m.insert("is_moveable_inventory".to_string(), self.is_moveable_inventory.to_json_value());
        m.insert("need_save_slot_count".to_string(), self.need_save_slot_count.to_json_value());
        m.insert("pushable_check_type".to_string(), self.pushable_check_type.to_json_value());
        m.insert("default_slot_count".to_string(), self.default_slot_count.to_json_value());
        m.insert("key_guide_local_string_info".to_string(), self.key_guide_local_string_info.to_json_value());
        m.insert("max_slot_count".to_string(), self.max_slot_count.to_json_value());
        m.insert("pushable_item_type_list".to_string(), self.pushable_item_type_list.to_json_value());
        m.insert("excluded_item_type_list".to_string(), self.excluded_item_type_list.to_json_value());
        m.insert("inventory_move_data_list".to_string(), self.inventory_move_data_list.to_json_value());
        m.insert("push_item_alert_ui_text".to_string(), self.push_item_alert_ui_text.to_json_value());
        m.insert("inventory_name_ui_text".to_string(), self.inventory_name_ui_text.to_json_value());
        m.insert("npc_usable_cooltime_min".to_string(), self.npc_usable_cooltime_min.to_json_value());
        m.insert("npc_usable_cooltime_max".to_string(), self.npc_usable_cooltime_max.to_json_value());
        m.insert("collection_item_list".to_string(), self.collection_item_list.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_pushable_item_only_one")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_moveable_inventory")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "need_save_slot_count")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pushable_check_type")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "default_slot_count")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key_guide_local_string_info")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "max_slot_count")?)?;
        <CArray<InventoryPushableData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pushable_item_type_list")?)?;
        <CArray<InventoryPushableData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "excluded_item_type_list")?)?;
        <CArray<InventoryMoveData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "inventory_move_data_list")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "push_item_alert_ui_text")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "inventory_name_ui_text")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "npc_usable_cooltime_min")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "npc_usable_cooltime_max")?)?;
        <CArray<InventoryCollectionItemData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "collection_item_list")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("inventory.pabgb") }

    #[test]
    fn roundtrip() {
        let p = pabgb_path();
        let Ok(data) = std::fs::read(&p) else { eprintln!("SKIP: {}", p.display()); return; };
        let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP pabgh"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = InventoryInfo::read_from(&data, &mut c)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e, "e{} k=0x{:x}: under/over-read {}/{}", i, k, c - s, e - s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "inventory roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let p = pabgb_path();
        let Ok(data) = std::fs::read(&p) else { eprintln!("SKIP: {}", p.display()); return; };
        let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP pabgh"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = InventoryInfo::read_from(&data, &mut cursor).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            InventoryInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }

    /// "I Like Space" edits the Character inventory's slot counts; it must decode typed.
    #[test]
    fn character_entry_slot_counts() {
        let p = pabgb_path();
        let Ok(data) = std::fs::read(&p) else { eprintln!("SKIP: {}", p.display()); return; };
        let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP pabgh"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let Some(&(_, s, e)) = ranges.iter().find(|(k, _, _)| *k == 2) else {
            panic!("Character entry (k=2) not found");
        };
        let mut c = s;
        let item = InventoryInfo::read_with_size(&data, &mut c, e - s).unwrap();
        assert_eq!(item.string_key.data, "Character");
        assert!(item.default_slot_count > 0 && item.default_slot_count <= item.max_slot_count,
            "implausible slot counts {} / {}", item.default_slot_count, item.max_slot_count);
    }

    #[test]
    fn fields_addressable() {
        let p = pabgb_path();
        let Ok(data) = std::fs::read(&p) else { eprintln!("SKIP: {}", p.display()); return; };
        let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP pabgh"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let Some((_, s, _)) = ranges.first() else { eprintln!("SKIP: no entries"); return; };
        let mut c = *s;
        let item = InventoryInfo::read_from(&data, &mut c).unwrap();
        let dict = item.to_json_dict();
        for f in [
            "key", "string_key", "is_blocked", "pushable_item_type_list",
            "excluded_item_type_list", "inventory_move_data_list",
            "default_slot_count", "max_slot_count", "push_item_alert_ui_text",
            "inventory_name_ui_text", "key_guide_local_string_info",
            "pushable_check_type", "npc_usable_cooltime_min",
            "npc_usable_cooltime_max", "is_moveable_inventory",
            "need_save_slot_count", "is_pushable_item_only_one",
            "collection_item_list",
        ] {
            assert!(dict.contains_key(f), "missing field `{}` in JSON dict", f);
        }
        assert!(!dict.contains_key("_tail_b64"), "Tier 1.5 _tail_b64 leaked");
    }
}
