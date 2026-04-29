//! Hand-corrected: IDA-derived parser for `FailMessageInfo.pabgb`.
//!
//! Per IDA sub_141114A80: fail_message_info_list element is
//! { u32 lookup_key + LocalizableString label }.
//! Verified by byte-level decode of entry #6 (key=1000006).

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct FailMessageEntry<'a> {
        pub lookup_key: u32,
        pub label: LocalizableString<'a>,
    }
}

py_binary_struct! {
    pub struct FailMessageInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub fail_message_info_list: CArray<FailMessageEntry<'a>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\failmessageinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(FailMessageInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "failmessageinfo roundtrip bytes mismatch");
    }
}
