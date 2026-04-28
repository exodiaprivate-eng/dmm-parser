//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E1E60` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u16 key (pabgh format 2)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u8 byte_17
//!   5. u32 lookup_18 (sub_1410FF430 → qword_145F0E9C0)
//!   6. u32 lookup_20 (sub_1410FF430)
//!   7. u32 lookup_22 (sub_1410FF430)
//!   8. sub_1415BE5E0(&v11, a1) → struct +24 (POLYMORPHIC variant
//!      allocator with vtable-dispatched destructor) ← TAIL STARTS HERE
//!   9. (After tail) u8 at struct +32
//!
//! Steps 1-7 are typed; step 8 onward (the polymorphic
//! GameEventHandler variant + trailing u8) lives in `tail_blob`.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GameEventHandlerInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub byte_17: u8,
        pub lookup_18: u32,
        pub lookup_20: u32,
        pub lookup_22: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameeventhandler.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameeventhandler.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GameEventHandlerInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gameeventhandler roundtrip mismatch");
    }
}
