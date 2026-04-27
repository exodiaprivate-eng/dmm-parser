//! IDA-derived parser for `AIEventTableInfo.pabgb`.
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
    pub struct AIEventTableInfo<'a> {
        pub key: [u8; 12],
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub show_name: CString<'a>,
        pub delegate_event_handler: u32,
        pub reaction_level: u32,
        pub allow_type_flag: u32,
        pub event_type: u32,
        pub event_delay_type: u64,
        pub is_sequencer_interrupt_event: u8,
        pub is_target_must_exist: u8,
        pub is_must_handled: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\aieventtableinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(AIEventTableInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "aieventtableinfo roundtrip bytes mismatch");
    }
}
