//! IDA-derived parser for `npcactivityinfo.pabgb`.
//! Reader: `sub_1410E4760` (CrimsonDesert.exe 1.0.8).
//! Sequential table, u32 key, 11 fields.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct NpcActivityInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub flag_a: u8,
        pub key_a: u32,
        pub key_b: u32,
        pub flag_b: u8,
        pub value_a: u32,
        pub hash_key: CString<'a>,
        pub flag_c: u8,
        pub list_a: CArray<u32>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str =
        r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/npcactivityinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { return; };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(NpcActivityInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len());
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data);
    }
}
