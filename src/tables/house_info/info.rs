//! IDA-derived parser for `HouseInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.

use crate::binary::*;
use crate::py_binary_struct;

// Hand-corrected: sub_141112CE0 is CArray<{u16 + u32 + CString}>, not CArray<u16>
// as the auto-classifier guessed. Verified by decoding 4 vanilla entries against
// the pabgh-given entry sizes.
py_binary_struct! {
    pub struct HouseRegionPhase<'a> {
        pub phase_id: u16,
        pub region_hash: u32,
        pub texture_path: CString<'a>,
    }
}

py_binary_struct! {
    pub struct HouseInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub house_name: LocalizableString<'a>,
        pub unlock_condition_info: u32,
        pub house_region_data_list: CArray<HouseRegionPhase<'a>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\houseinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(HouseInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "houseinfo roundtrip bytes mismatch");
    }
}
