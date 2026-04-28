//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E5840` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u16 key (pabgh format 2)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u16 lookup_a (inline u16 hash-key at qword_145F0E9C8;
//!      raw u16 round-trips)
//!   5. sub_141156680 → struct +24 (POLYMORPHIC variant dispatcher
//!      with vftables `pa::GlobalGameEventExecuteData_OpenRoyalSupply`
//!      [32B], `_VaryTradeItemPrice` [88B]) ← TAIL STARTS HERE
//!
//! Steps 1-4 are typed. Step 5 is a polymorphic
//! GlobalGameEventExecuteData variant — reopens cleanly when the
//! family decoders are written.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GlobalGameEventInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub lookup_a: u16,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\globalgameevent.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\globalgameevent.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GlobalGameEventInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "globalgameevent roundtrip mismatch");
    }
}
