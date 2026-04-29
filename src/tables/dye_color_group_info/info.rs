//! Hand-corrected: IDA-derived parser for `DyeColorGroupInfo.pabgb`.
//!
//! Per IDA sub_1410DA7C9: dye_color_data_list element is
//! { u32 raw + u32 sub_1410FF430-hash } = 8 bytes total.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct DyeColorEntry {
        pub raw_color: u32,
        pub texture_lookup: u32,
    }
}

py_binary_struct! {
    pub struct DyeColorGroupInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub dye_color_data_list: CArray<DyeColorEntry>,
        pub dye_color_group_name: LocalizableString<'a>,
        pub icon_path: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\dyecolorgroupinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DyeColorGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "dyecolorgroupinfo roundtrip bytes mismatch");
    }
}
