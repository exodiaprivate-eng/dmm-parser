//! IDA-derived parser for `DialogVoiceInfo.pabgb`.
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
    pub struct DialogVoiceInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub sound_event: CString<'a>,
        pub foot_step_sound_event: CString<'a>,
        pub foot_step_crouch_sound_event: CString<'a>,
        pub foot_step_land_sound_event: CString<'a>,
        pub foot_step_ground_sound_event: CString<'a>,
        pub foot_step_sound_offset: u8,
        pub foot_step_crouch_sound_offset: u8,
        pub foot_step_land_sound_offset: u8,
        pub foot_step_ground_sound_offset: u8,
        pub gender: u8,
        pub character_age: u8,
        pub job_info_list: CArray<u16>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\dialogvoiceinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DialogVoiceInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "dialogvoiceinfo roundtrip bytes mismatch");
    }
}
