//! IDA-derived parser for `GameAdviceInfo.pabgb`.
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
    pub struct GameAdviceInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub title_local_string_info: LocalizableString<'a>,
        pub desc_local_string_info: LocalizableString<'a>,
        pub key_mouse_input_desc_local_string_info: LocalizableString<'a>,
        pub game_advice_unknown_name: LocalizableString<'a>,
        pub ui_texture_name_string_info: u32,
        pub ui_video_path_string_info: u32,
        pub widget_id_string_info: u32,
        pub is_once: u8,
        pub is_show_guide_list: u8,
        pub game_advice_group_info: u32,
        pub is_default: u8,
        pub is_use_loading_view: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\gameadviceinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(GameAdviceInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "gameadviceinfo roundtrip bytes mismatch");
    }
}
