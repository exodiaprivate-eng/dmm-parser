//! IDA-derived parser for `CharacterGroupInfo.pabgb`.
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
    pub struct CharacterGroupInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub group_name: CString<'a>,
        pub character_info_list: CArray<u32>,
        pub group_gender_list: CArray<u8>,
        pub group_tribe_list: CArray<u32>,
        pub group_region_info_list: CArray<u16>,
        pub group_age_list: CArray<u8>,
        pub group_weapon_type_list: CArray<CString<'a>>,
        pub group_tier_list: CArray<u8>,
        pub group_ally_group_list: CArray<u32>,
        pub group_faction_list: CArray<u32>,
        pub group_job_info_list: CArray<u16>,
        pub stop_anim_constraint_dead: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\charactergroupinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(CharacterGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "charactergroupinfo roundtrip bytes mismatch");
    }
}
