//! Hand-corrected: IDA-derived parser for `PartPrefabDyeSlotInfo.pabgb`.
//!
//! Per IDA sub_1410EF0B0 (outer): u32 key, CString string_key, u8 is_blocked,
//! CArray<DyeSlotEntry> sub_mesh_list, CString mesh_file_name.
//!
//! Per IDA sub_14110C970 (CArray reader) + sub_1410EEE40 (element reader):
//! DyeSlotEntry = CString + 3 u8 + 3 sub_1410A9D40-CString + 3 u8.
//! sub_1410A9D40 reads a length-prefixed CString from disk and resolves
//! it to a u32 hash in memory, but on disk it remains a CString — we
//! preserve the CString form for round-trip.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct DyeSlotEntry<'a> {
        pub name: CString<'a>,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub texture_a: CString<'a>,
        pub texture_b: CString<'a>,
        pub texture_c: CString<'a>,
        pub flag_d: u8,
        pub flag_e: u8,
        pub flag_f: u8,
    }
}

py_binary_struct! {
    pub struct PartPrefabDyeSlotInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub sub_mesh_list: CArray<DyeSlotEntry<'a>>,
        pub mesh_file_name: CString<'a>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\partprefabdyeslotinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(PartPrefabDyeSlotInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "partprefabdyeslotinfo roundtrip bytes mismatch");
    }
}
