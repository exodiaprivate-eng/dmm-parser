//! IDA-derived parser for `PartPrefabDyeTexturePalleteInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.

use crate::binary::*;
use crate::py_binary_struct;

// Hand-corrected: texture_set_array element is {4x CString + f32} per pabgb
// byte-by-byte decode; auto-classifier guessed CArray<u32>.
py_binary_struct! {
    pub struct DyeTextureSetElem<'a> {
        pub name: CString<'a>,
        pub texture_a: CString<'a>,
        pub texture_b: CString<'a>,
        pub material_type: CString<'a>,
        pub weight: f32,
    }
}

py_binary_struct! {
    pub struct PartPrefabDyeTexturePalleteInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub texture_set_index: u32,
        pub texture_set_array: CArray<DyeTextureSetElem<'a>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\partprefabdyetexturepalleteinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(PartPrefabDyeTexturePalleteInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "partprefabdyetexturepalleteinfo roundtrip bytes mismatch");
    }
}
