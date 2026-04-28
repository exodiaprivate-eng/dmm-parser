//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410FB840` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                              (_key, sub_141BF6600 wrapper)
//!   2. CString string_key                   (_stringKey)
//!   3. u8 is_blocked                        (_isBlocked)
//!   4. CArray<u32> action_name_hash_list    (_actionNameHashList,
//!      sub_141101AB0: u32 count + N×u32)
//!   5. u8 type_                             (_type — `type` is reserved
//!      in Rust, suffixed with underscore)
//!   6. CArray<CString> keyword_lower_string_list (_keywordLowerStringList,
//!      sub_14106BAC0)
//!   7. _convertingData (sub_1410FB680 composite) ← TAIL STARTS HERE

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct ValidScheduleActionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub action_name_hash_list: CArray<u32>,
        pub type_: u8,
        pub keyword_lower_string_list: CArray<CString<'a>>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\validscheduleaction.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\validscheduleaction.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                ValidScheduleActionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "validscheduleaction roundtrip mismatch");
    }
}
