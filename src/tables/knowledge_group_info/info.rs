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
        pub is_show_ui: u8,
        pub is_show_uialert: u8,
        pub is_meditation_learnable: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\knowledgegroupinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
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
}
