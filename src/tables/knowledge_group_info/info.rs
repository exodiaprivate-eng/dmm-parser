//! IDA-derived parser for `KnowledgeGroupInfo.pabgb`.
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
    pub struct KnowledgeGroupInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub knowledge_group_name: LocalizableString<'a>,
        pub knowledge_group_unknown_name: LocalizableString<'a>,
        pub knowledge_group_desc: LocalizableString<'a>,
        pub ui_texture_name: u32,
        pub knowledge_group_icon_path: u32,
        pub ui_component_name: u32,
        pub knowledge_info_list: CArray<u32>,
        pub child_knowledge_group_info_list: CArray<u32>,
        pub parent_knowledge_group_info: u32,
        // game 2.00.00 — Korean field oracle put \_skillPointOwnerType here, right
        // after \_parentKnowledgeGroupInfo. Width is u8, established against the live
        // fixture: u16 and u32 both fail the roundtrip.
        //
        // ⚠ The Mac binary's `EnumReflectPropertyBind<..., SkillPointOwnerType, j>`
        // symbol says u32 — it is SAVE-DATA binding and does NOT describe the table
        // wire width. See dmm_enum_reflect_bind_is_savedata_not_tables.
        pub skill_point_owner_type: u8,
        pub is_show_ui: u8,
        pub is_show_uialert: u8,
        pub is_meditation_learnable: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("knowledgegroupinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(KnowledgeGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "knowledgegroupinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(KnowledgeGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            KnowledgeGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
