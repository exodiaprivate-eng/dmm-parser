//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E1E60` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u16 key                       (_key, pabgh format 2)
//!   2. CString string_key            (_stringKey)
//!   3. u8 is_blocked                 (_isBlocked)
//!   4. u8 game_event_type            (_gameEventType)
//!   5. u32 player_condition          (_playerCondition, sub_1410FF430
//!      → qword_145F0E9C0)
//!   6. u32 event_condition           (_eventCondition, sub_1410FF430)
//!   7. u32 target_condition          (_targetCondition, sub_1410FF430)
//!   8. _gameEventHandlerData (sub_1415BE5E0 → struct +24, POLYMORPHIC
//!      variant allocator with vtable-dispatched destructor)
//!      ← TAIL STARTS HERE
//!   9. (tail) _isPendOnBattleState (u8 at struct +32)
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
        pub game_event_type: u8,
        pub player_condition: u32,
        pub event_condition: u32,
        pub target_condition: u32,
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
