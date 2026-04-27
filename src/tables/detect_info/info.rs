//! Hand-corrected: IDA-derived parser for `DetectInfo.pabgb`.
//!
//! Per IDA sub_1410D9E36 (parser) + sub_1410D9B70 (DetectSenseData reader).
//! DetectSenseData is a RECURSIVE tree: it contains a list of children
//! that are themselves DetectSenseData. Use Box<CArray<...>> indirection
//! since CArray<T> already heap-allocates the element backing.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct DetectSenseData {
        pub field_a: u32,
        pub field_b: u32,
        pub field_c: u32,
        pub field_d: u32,
        pub field_e: u32,
        pub field_f: u32,
        pub field_g: u32,
        pub field_h: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub children: CArray<DetectSenseData>,
    }
}

py_binary_struct! {
    pub struct DetectInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub decrease_value_per_sec: u32,
        pub is_detectable_attached_docking: u8,
        pub target_lost_distance: u32,
        pub detect_sense_data: DetectSenseData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\detectinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DetectInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "detectinfo roundtrip bytes mismatch");
    }
}
