//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E8BF0` in CrimsonDesert.exe (Win build).
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. CString chart_name (a second CString — game uses it as the
//!      sequencer's chart asset path)
//!   5. u32 lookup_a (sub_141104340 → qword_145F0E9B8 dict, stored as u32)
//!   6. COptional<SequencerStageChartDesc> (sub_141110D30 → uses
//!      sub_141D8C6D0 polymorphic dispatcher) ← TAIL STARTS HERE
//!   7. COptional<...> (sub_1411057F0)
//!   8. u8, u8, u32, u32, u32 (cooldown / interval scalars)
//!   9. u16 lookup (sub_1410FF430 → qword_145F0E9C0)
//!  10. CArray<SequencerStageChartDesc> (sub_141110B70 → polymorphic)
//!
//! Steps 1-5 are typed; everything from step 6 lives in `tail_blob`. The
//! polymorphic descriptors and cooldown scalars stay opaque until task
//! #66 finishes the variant family decoders, after which we can reopen
//! and split the tail into typed fields.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GlobalStageSequencerInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub chart_name: CString<'a>,
        pub lookup_a: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\globalstagesequencerinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\globalstagesequencerinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GlobalStageSequencerInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "globalstagesequencerinfo roundtrip mismatch");
    }
}
