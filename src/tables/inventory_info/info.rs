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
//! `InventoryMoveData` (sub_1410E0460) embeds an `OptionalGameConditionNoTail`
//! (sub_141103B30 → sub_141CEA810). Stream-mode reading uses lane A's
//! public `GameConditionNode::read_from`; the 0.2% Raw fallback in
//! `GameCondition::read_from` is unreachable here because we don't
//! have a slice-bounded context. If any vanilla entry hit an
//! anti-disassembly variant (tags 54/286), parsing would fail; the
//! roundtrip test below would catch it.

use crate::binary::optional_game_condition::OptionalGameConditionNoTail;
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
/// `move_condition` is an `OptionalGameConditionNoTail` — the
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
    pub move_condition: OptionalGameConditionNoTail<'a>,
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
        let move_condition = OptionalGameConditionNoTail::read_from(data, offset)?;
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
        OptionalGameConditionNoTail::write_from_json(w, json_get_field(obj, "move_condition")?)?;
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
    pub key: u16,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub pushable_item_type_list: CArray<InventoryPushableData>,
    pub excluded_item_type_list: CArray<InventoryPushableData>,
    pub inventory_move_data_list: CArray<InventoryMoveData<'a>>,
    pub default_slot_count: u16,
    pub max_slot_count: u16,
    pub push_item_alert_ui_text: LocalizableString<'a>,
    pub inventory_name_ui_text: LocalizableString<'a>,
    pub key_guide_local_string_info: u32,
    pub pushable_check_type: u8,
    /// `_npcUsableData` (8-byte struct: 2× u32 cooltime min/max).
    /// First half (`cooltime_min_in_game`).
    pub npc_usable_cooltime_min: u32,
    /// Second half (`cooltime_max_in_game`).
    pub npc_usable_cooltime_max: u32,
    pub is_moveable_inventory: u8,
    pub need_save_slot_count: u8,
    pub is_pushable_item_only_one: u8,
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
        let key = u16::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let pushable_item_type_list = CArray::<InventoryPushableData>::read_from(data, offset)?;
        let excluded_item_type_list = CArray::<InventoryPushableData>::read_from(data, offset)?;
        let inventory_move_data_list = CArray::<InventoryMoveData>::read_from(data, offset)?;
        let default_slot_count = u16::read_from(data, offset)?;
        let max_slot_count = u16::read_from(data, offset)?;
        let push_item_alert_ui_text = LocalizableString::read_from(data, offset)?;
        let inventory_name_ui_text = LocalizableString::read_from(data, offset)?;
        let key_guide_local_string_info = u32::read_from(data, offset)?;
        let pushable_check_type = u8::read_from(data, offset)?;
        let npc_usable_cooltime_min = u32::read_from(data, offset)?;
        let npc_usable_cooltime_max = u32::read_from(data, offset)?;
        let is_moveable_inventory = u8::read_from(data, offset)?;
        let need_save_slot_count = u8::read_from(data, offset)?;
        let is_pushable_item_only_one = u8::read_from(data, offset)?;
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
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.pushable_item_type_list.write_to(w)?;
        self.excluded_item_type_list.write_to(w)?;
        self.inventory_move_data_list.write_to(w)?;
        self.default_slot_count.write_to(w)?;
        self.max_slot_count.write_to(w)?;
        self.push_item_alert_ui_text.write_to(w)?;
        self.inventory_name_ui_text.write_to(w)?;
        self.key_guide_local_string_info.write_to(w)?;
        self.pushable_check_type.write_to(w)?;
        self.npc_usable_cooltime_min.write_to(w)?;
        self.npc_usable_cooltime_max.write_to(w)?;
        self.is_moveable_inventory.write_to(w)?;
        self.need_save_slot_count.write_to(w)?;
        self.is_pushable_item_only_one.write_to(w)?;
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
        let key = track_read_field::<u16>(data, offset, path, ranges, "key", "u16")?;
        let string_key = track_read_field::<CString<'a>>(data, offset, path, ranges, "string_key", "CString")?;
        let is_blocked = track_read_field::<u8>(data, offset, path, ranges, "is_blocked", "u8")?;
        let pushable_item_type_list = track_read_field::<CArray<InventoryPushableData>>(data, offset, path, ranges, "pushable_item_type_list", "CArray<InventoryPushableData>")?;
        let excluded_item_type_list = track_read_field::<CArray<InventoryPushableData>>(data, offset, path, ranges, "excluded_item_type_list", "CArray<InventoryPushableData>")?;
        let inventory_move_data_list = track_read_field::<CArray<InventoryMoveData<'a>>>(data, offset, path, ranges, "inventory_move_data_list", "CArray<InventoryMoveData>")?;
        let default_slot_count = track_read_field::<u16>(data, offset, path, ranges, "default_slot_count", "u16")?;
        let max_slot_count = track_read_field::<u16>(data, offset, path, ranges, "max_slot_count", "u16")?;
        let push_item_alert_ui_text = track_read_field::<LocalizableString<'a>>(data, offset, path, ranges, "push_item_alert_ui_text", "LocalizableString")?;
        let inventory_name_ui_text = track_read_field::<LocalizableString<'a>>(data, offset, path, ranges, "inventory_name_ui_text", "LocalizableString")?;
        let key_guide_local_string_info = track_read_field::<u32>(data, offset, path, ranges, "key_guide_local_string_info", "u32")?;
        let pushable_check_type = track_read_field::<u8>(data, offset, path, ranges, "pushable_check_type", "u8")?;
        let npc_usable_cooltime_min = track_read_field::<u32>(data, offset, path, ranges, "npc_usable_cooltime_min", "u32")?;
        let npc_usable_cooltime_max = track_read_field::<u32>(data, offset, path, ranges, "npc_usable_cooltime_max", "u32")?;
        let is_moveable_inventory = track_read_field::<u8>(data, offset, path, ranges, "is_moveable_inventory", "u8")?;
        let need_save_slot_count = track_read_field::<u8>(data, offset, path, ranges, "need_save_slot_count", "u8")?;
        let is_pushable_item_only_one = track_read_field::<u8>(data, offset, path, ranges, "is_pushable_item_only_one", "u8")?;
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
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("pushable_item_type_list".to_string(), self.pushable_item_type_list.to_json_value());
        m.insert("excluded_item_type_list".to_string(), self.excluded_item_type_list.to_json_value());
        m.insert("inventory_move_data_list".to_string(), self.inventory_move_data_list.to_json_value());
        m.insert("default_slot_count".to_string(), self.default_slot_count.to_json_value());
        m.insert("max_slot_count".to_string(), self.max_slot_count.to_json_value());
        m.insert("push_item_alert_ui_text".to_string(), self.push_item_alert_ui_text.to_json_value());
        m.insert("inventory_name_ui_text".to_string(), self.inventory_name_ui_text.to_json_value());
        m.insert("key_guide_local_string_info".to_string(), self.key_guide_local_string_info.to_json_value());
        m.insert("pushable_check_type".to_string(), self.pushable_check_type.to_json_value());
        m.insert("npc_usable_cooltime_min".to_string(), self.npc_usable_cooltime_min.to_json_value());
        m.insert("npc_usable_cooltime_max".to_string(), self.npc_usable_cooltime_max.to_json_value());
        m.insert("is_moveable_inventory".to_string(), self.is_moveable_inventory.to_json_value());
        m.insert("need_save_slot_count".to_string(), self.need_save_slot_count.to_json_value());
        m.insert("is_pushable_item_only_one".to_string(), self.is_pushable_item_only_one.to_json_value());
        m.insert("collection_item_list".to_string(), self.collection_item_list.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <CArray<InventoryPushableData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pushable_item_type_list")?)?;
        <CArray<InventoryPushableData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "excluded_item_type_list")?)?;
        <CArray<InventoryMoveData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "inventory_move_data_list")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "default_slot_count")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "max_slot_count")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "push_item_alert_ui_text")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "inventory_name_ui_text")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key_guide_local_string_info")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pushable_check_type")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "npc_usable_cooltime_min")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "npc_usable_cooltime_max")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_moveable_inventory")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "need_save_slot_count")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_pushable_item_only_one")?)?;
        <CArray<InventoryCollectionItemData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "collection_item_list")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("inventoryinfo.pabgb") }

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

    fn read_u8_at(data: &[u8], offset: &mut usize) -> u8 {
        let v = data[*offset];
        *offset += 1;
        v
    }
    fn read_u16_at(data: &[u8], offset: &mut usize) -> u16 {
        let v = u16::from_le_bytes(data[*offset..*offset+2].try_into().unwrap());
        *offset += 2;
        v
    }
    fn read_u32_at(data: &[u8], offset: &mut usize) -> u32 {
        let v = u32::from_le_bytes(data[*offset..*offset+4].try_into().unwrap());
        *offset += 4;
        v
    }
    fn read_u64_at(data: &[u8], offset: &mut usize) -> u64 {
        let v = u64::from_le_bytes(data[*offset..*offset+8].try_into().unwrap());
        *offset += 8;
        v
    }
    fn read_cstring_len(data: &[u8], offset: &mut usize) -> u32 {
        // reads len and skips content, returns len
        if *offset + 4 > data.len() {
            eprintln!("  read_cstring_len: TRUNCATED at {}", *offset);
            return 0;
        }
        let len = read_u32_at(data, offset);
        let advance = (len as usize).min(data.len() - *offset);
        if advance < len as usize {
            eprintln!("  read_cstring_len: len={} but only {} bytes remain — clamping", len, data.len() - *offset);
        }
        *offset += advance;
        len
    }
    fn read_localizable_string(data: &[u8], offset: &mut usize, label: &str) {
        let start = *offset;
        if *offset + 13 > data.len() {
            eprintln!("  {} at {}: TRUNCATED (only {} remain)", label, start, data.len() - *offset);
            return;
        }
        let cat = read_u8_at(data, offset);
        let idx = read_u64_at(data, offset);
        // peek clen before consuming
        let clen = u32::from_le_bytes(data[*offset..*offset+4].try_into().unwrap());
        *offset += 4;
        let content_end = (*offset + clen as usize).min(data.len());
        let valid_utf8 = std::str::from_utf8(&data[*offset..content_end]).is_ok();
        if *offset + clen as usize > data.len() {
            eprintln!("  {} at {}: cat={} idx={} clen={} UTF8={} OVERFLOW (only {} remain)",
                      label, start, cat, idx, clen, valid_utf8, data.len() - *offset);
        } else {
            eprintln!("  {} at {}: cat={} idx={} clen={} UTF8={} end={}",
                      label, start, cat, idx, clen, valid_utf8, *offset + clen as usize);
            *offset += clen as usize;
        }
    }

    #[test]
    fn diag_character_entry() {
        let p = pabgb_path();
        let Ok(data) = std::fs::read(&p) else { eprintln!("SKIP: {}", p.display()); return; };
        let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP pabgh"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let Some(&(_, s, e)) = ranges.iter().find(|(k,_,_)| *k == 2) else {
            panic!("Character entry (k=2) not found");
        };
        eprintln!("Character: s={} e={} size={}", s, e, e - s);

        // ── Manual header walk ───────────────────────────────────────────────
        let mut c = s;
        let key        = read_u16_at(&data, &mut c);
        let sk_len     = read_cstring_len(&data, &mut c);
        let is_blocked = read_u8_at(&data, &mut c);
        let push_count = read_u32_at(&data, &mut c);
        c += push_count as usize * 3;      // InventoryPushableData = u16 + u8 = 3 bytes
        let excl_count = read_u32_at(&data, &mut c);
        c += excl_count as usize * 3;
        eprintln!("header: key={} sk_len={} is_blocked={} push={} excl={} → cursor={}",
                  key, sk_len, is_blocked, push_count, excl_count, c);

        // ── inventory_move_data_list ─────────────────────────────────────────
        let move_count = read_u32_at(&data, &mut c);
        eprintln!("inventory_move_data_list count={} cursor={}", move_count, c);

        for i in 0..move_count as usize {
            let elem_start = c;
            let saved_c = c;
            match InventoryMoveData::read_from(&data, &mut c) {
                Ok(elem) => {
                    let cft_len = elem.condition_fail_text.default.length;
                    let kgt_len = elem.key_guide_text.default.length;
                    let mak_len = elem.move_all_key_guide_text.default.length;
                    let mod_len = elem.modal_text.default.length;
                    let imd_ct  = elem.item_move_data_list.items.len();
                    let cond_p  = elem.move_condition.inner.is_some();
                    eprintln!("[{}] OK  start={} end={} size={}  type_={} from_={} to_={} conv={}",
                              i, elem_start, c, c - elem_start,
                              elem.type_, elem.from_inventory_info, elem.to_inventory_info,
                              elem.convert_money_item_info);
                    eprintln!("     kgt_len={} mak_len={} mod_len={} imd_ct={} cond={} cft_len={}",
                              kgt_len, mak_len, mod_len, imd_ct, cond_p, cft_len);
                }
                Err(er) => {
                    eprintln!("[{}] FAIL start={} cursor_at_fail={} err={}", i, elem_start, c, er);
                    // ── Per-field manual trace ───────────────────────────────
                    let mut mc = saved_c;
                    let type_  = read_u8_at(&data, &mut mc);
                    let from_  = read_u16_at(&data, &mut mc);
                    let to_    = read_u16_at(&data, &mut mc);
                    let conv   = read_u32_at(&data, &mut mc);
                    eprintln!("  type_={} from_={} to_={} conv={} mc={}", type_, from_, to_, conv, mc);
                    read_localizable_string(&data, &mut mc, "key_guide_text");
                    read_localizable_string(&data, &mut mc, "move_all_key_guide_text");
                    read_localizable_string(&data, &mut mc, "modal_text");
                    if mc + 4 <= data.len() {
                        let imd_count = read_u32_at(&data, &mut mc);
                        let imd_advance = (imd_count as usize * 13).min(data.len() - mc);
                        mc += imd_advance;
                        eprintln!("  item_move_data_list count={} mc={}", imd_count, mc);
                    }
                    if mc < data.len() {
                        let presence = read_u8_at(&data, &mut mc);
                        eprintln!("  move_condition presence={} mc={}", presence, mc);
                        if presence != 0 {
                            let mut cond_c = mc;
                            match crate::binary::variants::game_condition::GameConditionNode::read_from(&data, &mut cond_c) {
                                Ok(_) => {
                                    eprintln!("  GameConditionNode OK  start={} end={} size={}", mc, cond_c, cond_c - mc);
                                    mc = cond_c;
                                }
                                Err(er2) => {
                                    eprintln!("  GameConditionNode FAIL at={}: {}", cond_c, er2);
                                    let lo = mc;
                                    let hi = (cond_c + 16).min(data.len());
                                    eprintln!("  bytes[{}..{}]: {:02x?}", lo, hi, &data[lo..hi]);
                                    break;
                                }
                            }
                        }
                    }
                    read_localizable_string(&data, &mut mc, "condition_fail_text");
                    eprintln!("  mc after condition_fail_text={} (expected end={})", mc, e);
                    break;
                }
            }
        }

        // ── Raw hex dump of key areas ────────────────────────────────────────
        // Element 6 starts at 2345. Its condition node should be near 2418.
        // Dump the 60 bytes around cft (condition_fail_text) to verify structure.
        {
            let lo = 2410usize;
            let hi = (lo + 80).min(data.len());
            eprintln!("--- hex[{}..{}]:", lo, hi);
            for (i, chunk) in data[lo..hi].chunks(16).enumerate() {
                let addr = lo + i * 16;
                let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
                let ascii: String = chunk.iter().map(|&b| if b.is_ascii_graphic() { b as char } else { '.' }).collect();
                eprintln!("  {:04x}: {}  {}", addr, hex.join(" "), ascii);
            }
        }
        // Dump last 128 bytes of the Character entry (for post-list analysis)
        {
            let lo = e.saturating_sub(128);
            let hi = e.min(data.len());
            eprintln!("--- hex[{}..{}] (end of entry):", lo, hi);
            for (i, chunk) in data[lo..hi].chunks(16).enumerate() {
                let addr = lo + i * 16;
                let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
                let ascii: String = chunk.iter().map(|&b| if b.is_ascii_graphic() { b as char } else { '.' }).collect();
                eprintln!("  {:04x}: {}  {}", addr, hex.join(" "), ascii);
            }
        }

        // ── Also try the full typed parse so we have a reference error line ──
        eprintln!("---");
        let mut c2 = s;
        match InventoryInfo::read_from(&data, &mut c2) {
            Ok(_)  => eprintln!("read_from: OK consumed={} expected={}", c2 - s, e - s),
            Err(er) => eprintln!("read_from: ERR {} (cursor={})", er, c2),
        }
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
