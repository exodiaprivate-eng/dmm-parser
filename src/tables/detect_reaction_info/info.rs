//! IDA-derived parser for `DetectReactionInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.
//!
//! ─── v3.1 closure analysis (iter 68) ────────────────────────────────────
//! Re-decompile of `sub_1410A7170` (Win, per iter 54 typeinfo registry)
//! confirms the on-wire layout:
//!
//!   offset 0    4 bytes        → _key (u32)
//!   offset 8    CString        → _stringKey
//!   offset 16   1 byte         → _isBlocked
//!   offset 17   5 × 12 bytes   → _reactionTable (5 rows × 12 u8 cells)
//!   offset 77   1 byte         → _buffReactionType
//!   offset 78   1 byte         → _strongBuffReactionType
//!   offset 79   1 byte         → _playerSensibleReactionType
//!
//! NattKh schema lists 7 canonicals: _key, _stringKey, _isBlocked,
//! _buffReactionType, _strongBuffReactionType, _playerSensibleReactionType,
//! _reactionTable. The "missing" canonical `_reactionTable` is a pure
//! 1-to-5 wrapper around the unrolled rust fields `reaction_row_0..4`.
//! Closure path: 1-to-N alias entry in FIELD_ALIASES_V3_1 mapping
//! `_reactionTable` → `[reaction_row_0, reaction_row_1, reaction_row_2,
//! reaction_row_3, reaction_row_4]`. No new decoder work needed.

use crate::binary::*;
use crate::py_binary_struct;

// Hand-corrected: parser reads a 5x12=60 byte fixed block for _reactionTable
// (nested for-loop in the IDA decompile — 60 individual u8 reads).
// Promoted to typed 5-row struct with 12 named cells per row.
// Each cell is a small enum value (0x01, 0x15, 0x17, 0x2a, 0x2d, 0x3b
// observed in vanilla); JSON consumers can edit any individual cell.
py_binary_struct! {
    pub struct DetectReactionRow {
        pub c0: u8, pub c1: u8, pub c2: u8, pub c3: u8,
        pub c4: u8, pub c5: u8, pub c6: u8, pub c7: u8,
        pub c8: u8, pub c9: u8, pub c10: u8, pub c11: u8,
    }
}

py_binary_struct! {
    pub struct DetectReactionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub reaction_row_0: DetectReactionRow,
        pub reaction_row_1: DetectReactionRow,
        pub reaction_row_2: DetectReactionRow,
        pub reaction_row_3: DetectReactionRow,
        pub reaction_row_4: DetectReactionRow,
        pub buff_reaction_type: u8,
        pub strong_buff_reaction_type: u8,
        pub player_sensible_reaction_type: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/detectreactioninfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DetectReactionInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "detectreactioninfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DetectReactionInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            DetectReactionInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
