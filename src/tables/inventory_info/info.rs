//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader: `sub_1410E05E0` in CrimsonDesert.exe (Win build).
//! Per-element reader for inventory_move_data_list: `sub_1410E0460`
//! (160-byte composite). Inner readers fully decoded.
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1.  u16 key                                   (_key, pabgh format 2)
//!   2.  CString string_key                        (_stringKey)
//!   3.  u8 is_blocked                             (_isBlocked)
//!   4.  CArray<InventorySubA> pushable_item_type_list
//!       (_pushableItemTypeList, sub_141103FB0; element wire = u16
//!       lookup + u8 = 3 bytes per element)
//!   5.  CArray<InventorySubA> excluded_item_type_list
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
//!  13.  u32 npc_usable_data_a                     (_npcUsableData first u32)
//!  14.  u32 npc_usable_data_b                     (_npcUsableData second u32)
//!  15.  u8 is_moveable_inventory                  (_isMoveableInventory)
//!  16.  u8 need_save_slot_count                   (_needSaveSlotCount)
//!  17.  u8 flag_158                               (extra byte preserved)
//!  18.  CArray<NpcUsableExtraData> npc_usable_extra_data_list
//!       (sub_141103310 — element wire = u32 lookup + 8 raw bytes = 12 bytes)
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

py_binary_struct! {
    pub struct InventorySubA {
        pub lookup: u16,
        pub byte_2: u8,
    }
}

py_binary_struct! {
    pub struct InventoryMoveExtraData {
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub lookup_c: u32,
        pub flag: u8,
    }
}

py_binary_struct! {
    pub struct NpcUsableExtraData {
        pub lookup: u32,
        pub raw_8: [u8; 8],
    }
}

/// 160-byte InventoryMoveData composite per IDA sub_1410E0460.
#[derive(Debug)]
pub struct InventoryMoveData<'a> {
    pub flag_a: u8,
    pub lookup_a: u16,    // sub_141103F00 → qword_145F0DA18 (u16 wire)
    pub lookup_b: u16,    // sub_141103F00 (u16 wire)
    pub item_lookup: u32, // sub_1410FF5C0 → qword_145F0DA00 (u32 wire)
    pub locstr_a: LocalizableString<'a>,
    pub locstr_b: LocalizableString<'a>,
    pub locstr_c: LocalizableString<'a>,
    pub extra_data_list: CArray<InventoryMoveExtraData>,
    pub condition: OptionalGameCondition<'a>,
    pub locstr_d: LocalizableString<'a>,
}

impl<'a> InventoryMoveData<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let flag_a = u8::read_from(data, offset)?;
        let lookup_a = u16::read_from(data, offset)?;
        let lookup_b = u16::read_from(data, offset)?;
        let item_lookup = u32::read_from(data, offset)?;
        let locstr_a = LocalizableString::read_from(data, offset)?;
        let locstr_b = LocalizableString::read_from(data, offset)?;
        let locstr_c = LocalizableString::read_from(data, offset)?;
        let extra_data_list = CArray::<InventoryMoveExtraData>::read_from(data, offset)?;
        let condition = OptionalGameCondition::read_from(data, offset)?;
        let locstr_d = LocalizableString::read_from(data, offset)?;
        Ok(Self {
            flag_a, lookup_a, lookup_b, item_lookup, locstr_a, locstr_b, locstr_c,
            extra_data_list, condition, locstr_d,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.flag_a.write_to(w)?;
        self.lookup_a.write_to(w)?;
        self.lookup_b.write_to(w)?;
        self.item_lookup.write_to(w)?;
        self.locstr_a.write_to(w)?;
        self.locstr_b.write_to(w)?;
        self.locstr_c.write_to(w)?;
        self.extra_data_list.write_to(w)?;
        self.condition.write_to(w)?;
        self.locstr_d.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("flag_a".to_string(), self.flag_a.to_json_value());
        m.insert("lookup_a".to_string(), self.lookup_a.to_json_value());
        m.insert("lookup_b".to_string(), self.lookup_b.to_json_value());
        m.insert("item_lookup".to_string(), self.item_lookup.to_json_value());
        m.insert("locstr_a".to_string(), self.locstr_a.to_json_value());
        m.insert("locstr_b".to_string(), self.locstr_b.to_json_value());
        m.insert("locstr_c".to_string(), self.locstr_c.to_json_value());
        m.insert("extra_data_list".to_string(), self.extra_data_list.to_json_value());
        m.insert("condition".to_string(), self.condition.to_json_value());
        m.insert("locstr_d".to_string(), self.locstr_d.to_json_value());
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
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_a")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_a")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_b")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "item_lookup")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "locstr_a")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "locstr_b")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "locstr_c")?)?;
        <CArray<InventoryMoveExtraData> as WriteJsonValue>::write_from_json(
            w, json_get_field(obj, "extra_data_list")?,
        )?;
        OptionalGameCondition::write_from_json(w, json_get_field(obj, "condition")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "locstr_d")?)?;
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
    pub pushable_item_type_list: CArray<InventorySubA>,
    pub excluded_item_type_list: CArray<InventorySubA>,
    pub inventory_move_data_list: CArray<InventoryMoveData<'a>>,
    pub default_slot_count: u16,
    pub max_slot_count: u16,
    pub push_item_alert_ui_text: LocalizableString<'a>,
    pub inventory_name_ui_text: LocalizableString<'a>,
    pub key_guide_local_string_info: u32,
    pub pushable_check_type: u8,
    pub npc_usable_data_a: u32,
    pub npc_usable_data_b: u32,
    pub is_moveable_inventory: u8,
    pub need_save_slot_count: u8,
    pub flag_158: u8,
    pub npc_usable_extra_data_list: CArray<NpcUsableExtraData>,
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
        let pushable_item_type_list = CArray::<InventorySubA>::read_from(data, offset)?;
        let excluded_item_type_list = CArray::<InventorySubA>::read_from(data, offset)?;
        let inventory_move_data_list = CArray::<InventoryMoveData>::read_from(data, offset)?;
        let default_slot_count = u16::read_from(data, offset)?;
        let max_slot_count = u16::read_from(data, offset)?;
        let push_item_alert_ui_text = LocalizableString::read_from(data, offset)?;
        let inventory_name_ui_text = LocalizableString::read_from(data, offset)?;
        let key_guide_local_string_info = u32::read_from(data, offset)?;
        let pushable_check_type = u8::read_from(data, offset)?;
        let npc_usable_data_a = u32::read_from(data, offset)?;
        let npc_usable_data_b = u32::read_from(data, offset)?;
        let is_moveable_inventory = u8::read_from(data, offset)?;
        let need_save_slot_count = u8::read_from(data, offset)?;
        let flag_158 = u8::read_from(data, offset)?;
        let npc_usable_extra_data_list = CArray::<NpcUsableExtraData>::read_from(data, offset)?;
        Ok(Self {
            key, string_key, is_blocked, pushable_item_type_list,
            excluded_item_type_list, inventory_move_data_list, default_slot_count,
            max_slot_count, push_item_alert_ui_text, inventory_name_ui_text,
            key_guide_local_string_info, pushable_check_type, npc_usable_data_a,
            npc_usable_data_b, is_moveable_inventory, need_save_slot_count,
            flag_158, npc_usable_extra_data_list,
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
        self.npc_usable_data_a.write_to(w)?;
        self.npc_usable_data_b.write_to(w)?;
        self.is_moveable_inventory.write_to(w)?;
        self.need_save_slot_count.write_to(w)?;
        self.flag_158.write_to(w)?;
        self.npc_usable_extra_data_list.write_to(w)?;
        Ok(())
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
        m.insert("npc_usable_data_a".to_string(), self.npc_usable_data_a.to_json_value());
        m.insert("npc_usable_data_b".to_string(), self.npc_usable_data_b.to_json_value());
        m.insert("is_moveable_inventory".to_string(), self.is_moveable_inventory.to_json_value());
        m.insert("need_save_slot_count".to_string(), self.need_save_slot_count.to_json_value());
        m.insert("flag_158".to_string(), self.flag_158.to_json_value());
        m.insert("npc_usable_extra_data_list".to_string(), self.npc_usable_extra_data_list.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <CArray<InventorySubA> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pushable_item_type_list")?)?;
        <CArray<InventorySubA> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "excluded_item_type_list")?)?;
        <CArray<InventoryMoveData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "inventory_move_data_list")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "default_slot_count")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "max_slot_count")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "push_item_alert_ui_text")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "inventory_name_ui_text")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key_guide_local_string_info")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pushable_check_type")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "npc_usable_data_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "npc_usable_data_b")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_moveable_inventory")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "need_save_slot_count")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_158")?)?;
        <CArray<NpcUsableExtraData> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "npc_usable_extra_data_list")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\inventory.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\inventory.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
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
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
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

    #[test]
    fn fields_addressable() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
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
            "pushable_check_type", "npc_usable_data_a", "npc_usable_data_b",
            "is_moveable_inventory", "need_save_slot_count", "flag_158",
            "npc_usable_extra_data_list",
        ] {
            assert!(dict.contains_key(f), "missing field `{}` in JSON dict", f);
        }
        assert!(!dict.contains_key("_tail_b64"), "Tier 1.5 _tail_b64 leaked");
    }
}
