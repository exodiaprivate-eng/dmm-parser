//! IDA-derived parser for `FieldLevelNameTableInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.

use crate::binary::*;
use crate::py_binary_struct;

// Hand-corrected: per IDA sub_14110EA20, each level_name_info_data_list
// element is { u32 hash_key + u32 group_key + CString + u8 + [f32;3] + [f32;3] }
// — auto-classifier guessed [u8; 33] (correct for fixed-byte tail but missed
// the variable-length CString in the middle).
py_binary_struct! {
    pub struct LevelNameInfoData<'a> {
        pub hash_key: u32,
        pub group_key: u32,
        pub name: CString<'a>,
        pub flag: u8,
        pub vec_a: [f32; 3],
        pub vec_b: [f32; 3],
    }
}

py_binary_struct! {
    pub struct FieldLevelNameTableInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub field_level_name: CString<'a>,
        pub level_name_info_data_list: CArray<LevelNameInfoData<'a>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\fieldlevelnametableinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(FieldLevelNameTableInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "fieldlevelnametableinfo roundtrip bytes mismatch");
    }
}
