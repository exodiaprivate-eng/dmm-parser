//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410F5140` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u16 key (pabgh format 2)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. LocalizableString name
//!   5. u32 lookup_28 (sub_1411006D0)
//!   6. CArray<RegionSubA> sub_a_list (sub_141104230: each item =
//!      u32 lookup via sub_1411006D0 + u32 = 8 wire bytes)
//!   7. u16 lookup_40 (sub_1410FF220 → qword_145F0DA80)
//!   8. CArray<u16> list_44 (sub_1410FFAC0 → qword_145F0DA80)
//!   9. u8 u8_52
//!  10. u8 u8_105
//!  11. u32 u32_54
//!  12. u8 u8_56
//!  13. u32 lookup_57 (sub_1410FF430)
//!  14-21. 8× u8 (u8_58, u8_117, u8_59, u8_119, u8_60, u8_121, u8_61,
//!         u8_123 — interleaved struct slots)
//!  22. sub_1411043B0 → struct +128 (unknown helper) ← TAIL STARTS HERE
//!  23. sub_14110A900 → struct +144 (unknown helper)
//!  24. sub_1410FEF40 → struct +160 (unknown helper)
//!
//! Steps 1-21 are typed (21 typed fields). Tail captures the 3 unknown
//! body helpers.
//!
//! New helpers: `sub_1410FF220` = single u16 lookup at qword_145F0DA80
//! (wire 2); `sub_1410FFAC0` = CArray<u16> hash-keyed at same dict;
//! `sub_141104230` = CArray<{u32 lookup + u32}=8B>.

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct RegionSubA {
        pub lookup_a: u32,
        pub raw_b: u32,
    }
}

pabgh_typed_blob_table! {
    pub struct RegionInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub name: LocalizableString<'a>,
        pub lookup_28: u32,
        pub sub_a_list: CArray<RegionSubA>,
        pub lookup_40: u16,
        pub list_44: CArray<u16>,
        pub u8_52: u8,
        pub u8_105: u8,
        pub u32_54: u32,
        pub u8_56: u8,
        pub lookup_57: u32,
        pub u8_58: u8,
        pub u8_117: u8,
        pub u8_59: u8,
        pub u8_119: u8,
        pub u8_60: u8,
        pub u8_121: u8,
        pub u8_61: u8,
        pub u8_123: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\regioninfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\regioninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                RegionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "regioninfo roundtrip mismatch");
    }
}
