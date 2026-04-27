//! Hand-corrected: IDA-derived parser for `ConditionInfo.pabgb`.
//!
//! Per IDA sub_1410D9F60: u32 key, CString string_key, u8 is_blocked,
//! GameCondition (sub_141CEA810 → recursive variant tree via meta-dispatcher
//! sub_141E65330), CString original_string, u8 parser_type.
//!
//! ## Status: blob-tail (Tier 2) pending ConditionData variant recipe fix
//!
//! The full GameCondition tree decoder is implemented in
//! `crate::binary::variants::game_condition::GameConditionNode` — all 9
//! meta-dispatcher cases are mapped (BinaryOp_A/B, UnaryOp, ConditionData,
//! BranchConditionData, ScheduleComplete, ConditionGimmick, StageChart,
//! GlobalEffect). Ready to wire in once these blockers clear:
//!
//!   - 35-46 of 405 ConditionData variants have wrong `tail_bytes` in the
//!     auto-generated recipe (the obfuscated read functions in 0x14F0xxxxx
//!     range XOR-pack their stream-size constants, and the current recipe
//!     just records 0). Empirical verification shows e.g. tag 206 (Weather)
//!     reads 5 bytes per instance, not 0.
//!   - The exact conditions under which the per-instance optional_subcond
//!     tail (1-byte presence + (cstring + u64 + 3 u8s) if non-zero) is
//!     applied are not yet pinned down — the recipe says it's always after
//!     each ConditionData variant body, but empirically that overshoots.
//!
//! Until these are corrected per-variant, ConditionInfo stays as
//! blob-tail (round-trips byte-perfect via the original probe). Pre-fields
//! (key, string_key, is_blocked) and trailing fields (original_string,
//! parser_type) remain individually field-addressable for v3 mods. The
//! GameCondition payload is captured as raw bytes and can be cloned
//! between entries via v3's `clone_blob_from` op.
//!
//! See `dmm-pabgb-aio/mac_extract/game_condition_tree_recipe.json` for the
//! full GameCondition tree spec, and `RECIPE_NEXT.md` for the wiring
//! roadmap.
//!
//! DO NOT REGENERATE. Hand-written; bulk_process.py guards via the
//! "Hand-corrected" header marker on line 1.

use crate::binary::variant::find_cstring_u8_trailer;
use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct ConditionInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    /// Polymorphic recursive expression tree. Captured as raw bytes; round-trips
    /// byte-perfect. See module docs for typed-decoder status.
    pub game_condition: Vec<u8>,
    pub original_string: CString<'a>,
    pub parser_type: u8,
}

impl<'a> ConditionInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start + entry_size;

        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;

        let post_pre = *offset;
        let variant_size = find_cstring_u8_trailer(data, post_pre, entry_end)?;
        let game_condition = data[post_pre..post_pre + variant_size].to_vec();
        *offset = post_pre + variant_size;

        let original_string = CString::read_from(data, offset)?;
        let parser_type = u8::read_from(data, offset)?;

        Ok(Self {
            key,
            string_key,
            is_blocked,
            game_condition,
            original_string,
            parser_type,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        w.write_all(&self.game_condition)?;
        self.original_string.write_to(w)?;
        self.parser_type.write_to(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str =
        r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\conditioninfo.pabgb";
    const PABGH_PATH: &str =
        r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\conditioninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing pabgb fixture {}", PABGB_PATH);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else {
            eprintln!("SKIP: missing/unparseable pabgh fixture {}", PABGH_PATH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = ConditionInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| {
                    panic!(
                        "parse failed at entry {} (key=0x{:x}, offset 0x{:x}, size {}): {}",
                        i,
                        key,
                        start,
                        end - start,
                        e
                    )
                });
            assert_eq!(
                cursor, *end,
                "entry {} (key 0x{:x}) under/over-consumed: read {} bytes, expected {}",
                i,
                key,
                cursor - start,
                end - start
            );
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out.len(), data.len(), "conditioninfo roundtrip size mismatch");
        assert_eq!(out, data, "conditioninfo roundtrip bytes mismatch");
    }
}
