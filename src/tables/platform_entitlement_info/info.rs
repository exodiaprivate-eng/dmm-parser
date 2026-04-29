//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_10184B44C` at 0x10184B44C
//! (size 0x220). No on-disk pabgb dump (`entitlementinfo.pabgb`
//! doesn't exist) — table is runtime-allocated, test SKIPs.
//!
//! Wire reads, in order:
//!   1. u16 key                      (sub_100F23704, width 2)
//!   2. CString string_key           (sub_1006B3F50, struct +8)
//!   3. u8 is_blocked                (sub_1006B3CC0, struct +16)
//!   4. LocalizableString entitlement_name (sub_1006D8484, struct +24,
//!      stride 32)
//!   5. LocalizableString entitlement_desc (sub_1006D8484, struct +56)
//!   6. u32 icon_path                (inline u32 hash → StringInfoKey
//!      lookup → u16 index at struct +88, wire 4)
//!   7. u8 type_                     (direct vtable[2] call width=1
//!      at struct +90, wire 1)
//!      ← TAIL STARTS HERE
//!   8. (tail) _resultDropInfoList   (sub_101151B5C, struct +96,
//!      unknown CArray-like helper)
//!   9. (tail) _platformIdList       (sub_10187D538, struct +112,
//!      unknown helper)

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct PlatformEntitlementInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub entitlement_name: LocalizableString<'a>,
        pub entitlement_desc: LocalizableString<'a>,
        pub icon_path: u32,
        pub type_: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    // No on-disk pabgb dump for this table; test SKIPs.
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\entitlementinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\entitlementinfo.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                PlatformEntitlementInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "platformentitlementinfo roundtrip mismatch");
    }
}
