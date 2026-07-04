//! Parser for `QuickSlotInfo.pabgb` (new in 1.12). Game deser sub_101FB249C;
//! slot element (QuickSlotItem) sub_101FB2078 (22 fields, key-readers resolve
//! 4B StringInfoKey/2B ItemGroupKey to u16 at runtime; wire = raw widths below).
use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct QuickSlotItem<'a> {
        pub quick_slot_item_type: u8,
        pub template_name: u32,
        pub component_name: u32,
        pub ui_item_owner_component: CString<'a>,
        pub reserve_slot_key: CString<'a>,
        pub has_disable_background: u8,
        pub has_reserve_background: u8,
        pub equip_type: u32,
        pub equip_slot_no: u16,
        pub is_reserve_quick_slot: u8,
        pub is_change_item_immediately: u8,
        pub item_info_quick_slot_index: u8,
        pub use_special_reserve_slot: u8,
        pub use_on_select: u8,
        // 1.13.00: narrowed u16 -> u8 (−1 B per QuickSlotItem). Byte-diff
        // decisive: exactly one byte removed at element-offset +40 (this field's
        // high byte) in every slot element; all following fields shift −1.
        pub show_item_group_key: u8,
        pub show_item_tag: u32,
        pub fixed_status_key: u32,
        pub select_special_name: CString<'a>,
        pub is_self_player: u8,
        pub mercenary_type: CString<'a>,
        pub mercenary_index: u32,
        pub auto_select_reserve_slot: u8,
    }
}

py_binary_struct! {
    pub struct QuickSlotInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub slot_count: u32,
        pub is_default: u8,
        pub active_key: CString<'a>,
        pub slot_list: CArray<QuickSlotItem<'a>>,
    }
}
