//! IDA-derived parser for `EquipTypeInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct EquipTypeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub destroyed_ai_event: [u8; 12],
        pub use_resource_item_type: u32,
        pub fake_equip_reserve_slot_data: u32,
        pub apply_status_group_info_on_activate: u32,
        pub apply_passive_skill_on_activate: u8,
        pub is_show_stamina: u8,
        pub decrease_endurance_percent: u64,
        pub on_guard_damage_reduction_percent: u64,
        pub is_critical_collidable: u8,
        pub enable_transfer: u8,
        pub enable_enchant: u8,
        pub use_action_on_quick_slot: u8,
        pub camera_preset_hash: u32,
        pub dye_rotation_value: u32,
        pub equip_able_hash_list: CArray<u32>,
        pub equip_type_name: LocalizableString<'a>,
        pub show_helm_on_battle_stance: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\equiptypeinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(EquipTypeInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "equiptypeinfo roundtrip bytes mismatch");
    }
}
