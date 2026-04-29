//! IDA-derived parser for `MercenaryInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.

use crate::binary::*;
use crate::py_binary_struct;

// Hand-corrected: parent_mercenary_group_info reads as u8 (file) but is stored
// as u16 (looked-up via dictionary). hired_skill_info_list element is
// {u32 lookup_key + u32 value} per IDA sub_141100A00.
py_binary_struct! {
    pub struct HiredSkillData {
        pub skill_lookup_key: u32,
        pub level: u32,
    }
}

py_binary_struct! {
    pub struct MercenaryInfo<'a> {
        pub key: u8,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub default_limit_summon_count: u32,
        pub default_limit_hire_count: u32,
        pub max_limit_hire_count: u32,
        pub far_from_leader_option: u8,
        pub combat_targeting_flags: [u8; 4],
        pub is_controllable: u8,
        pub is_playable: u8,
        pub set_new_mercenary_is_main: u8,
        pub main_mercenary_per_tribe: u8,
        pub is_force_stackable: u8,
        pub is_sellable: u8,
        pub use_camp_level: u8,
        pub apply_equip_item_stat: u8,
        pub spawn_position_type: u8,
        pub parent_mercenary_group_info: u8,
        pub hired_skill_info_list: CArray<HiredSkillData>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\mercenaryinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(MercenaryInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "mercenaryinfo roundtrip bytes mismatch");
    }
}
