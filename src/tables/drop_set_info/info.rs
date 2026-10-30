//! Tier 1 — fully typed parser for `DropSetInfo.pabgb`.
//!
//! Per IDA sub_1410DB650: 12 fields. `_list` is
//! `CArray<OptionalDropTarget>` via sub_141102760 + sub_141D03AA0 +
//! sub_141600210. Per-element wire shape (when present) is 63 fixed
//! bytes + variant tail dispatched on `dispatch_tag` at mem +112; full
//! decoder lives in `crate::binary::variants::drop_target`. Same payload shape
//! powers `ItemUseInfo` RandomBox `inner_data`.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::DropSetInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  12
// Decoded by dmm-parser:   12
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _dropTagNameHash (direct_u32, stream=4)
// ✅ _dropConditionString
// ✅ _neeSlotCount (direct_u16, stream=2)
// ✅ _list
// ✅ _totalDropRate (direct_u64, stream=8)
// ✅ _needWeight (direct_u64, stream=8)
// ✅ _originalString
// ✅ _key
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _dropRollCount (direct_u32, stream=4)
// ✅ _dropRollType (direct_u8, stream=1)

use crate::binary::*;
use crate::binary::variants::drop_target::OptionalDropTarget;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct DropSetInfo<'a> {
    // 2.04.00: the fields are read in a new order (Mac reader sub_1020517C4): string_key
    // first, is_equal_percent fourth, the two strings last.
    pub string_key: CString<'a>,
    pub key: u32,
    pub is_blocked: u8,
    // ── 1.18.00: `_isEqualPercent` (appended after original_string then; fourth since 2.04).
    pub is_equal_percent: u8,
    pub drop_roll_type: u8,
    pub nee_slot_count: u16,
    pub drop_tag_name_hash: u32,
    pub drop_roll_count: u32,
    pub list: CArray<OptionalDropTarget>,
    /// Wire u64 — fixed-point weight value (vanilla data shows values
    /// always within u32 range with high u32 zero, suggesting fixed-point
    /// scaling rather than f64).
    pub need_weight: u64,
    /// Wire u64 — fixed-point drop rate value. Vanilla samples show
    /// values like 1000000 (= 100% × 10000 PPM) supporting fixed-point
    /// interpretation. Treat as opaque u64 for round-trip safety; the
    /// game side knows the scaling factor.
    pub total_drop_rate: u64,
    // ── 2.01.00 (2026-09-03): `_rollDiceSkipRate`, u64 fixed-point like its neighbours.
    pub roll_dice_skip_rate: u64,
    // ── 2.04.00: a StringInfoKey (4 bytes: Pearl Abyss's checksum of the text, 0xEAC5E173 =
    // none) where 2.00-2.03 stored the text itself. The JSON name is kept; JSON text from an
    // older mod is converted (see `json_traits::write_string_info_key`).
    pub drop_condition_string: u32,
    pub original_string: CString<'a>,
}

impl<'a> DropSetInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let _ = entry_size; // typed reader is byte-perfect; size is informational

        let string_key = CString::read_from(data, offset)?;
        let key = u32::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let is_equal_percent = u8::read_from(data, offset)?;
        let drop_roll_type = u8::read_from(data, offset)?;
        let nee_slot_count = u16::read_from(data, offset)?;
        let drop_tag_name_hash = u32::read_from(data, offset)?;
        let drop_roll_count = u32::read_from(data, offset)?;
        let list = <CArray<OptionalDropTarget>>::read_from(data, offset)?;
        let need_weight = u64::read_from(data, offset)?;
        let total_drop_rate = u64::read_from(data, offset)?;
        let roll_dice_skip_rate = u64::read_from(data, offset)?;
        let drop_condition_string = u32::read_from(data, offset)?;
        let original_string = CString::read_from(data, offset)?;

        Ok(Self {
            string_key, key, is_blocked, is_equal_percent, drop_roll_type, nee_slot_count,
            drop_tag_name_hash, drop_roll_count, list, need_weight, total_drop_rate,
            roll_dice_skip_rate, drop_condition_string, original_string,
        })
    }

    pub fn read_tracked_with_size(
        data: &'a [u8],
        offset: &mut usize,
        _entry_size: usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let string_key = track_read_field::<CString<'a>>(data, offset, path, ranges, "string_key", "CString")?;
        let key = track_read_field::<u32>(data, offset, path, ranges, "key", "u32")?;
        let is_blocked = track_read_field::<u8>(data, offset, path, ranges, "is_blocked", "u8")?;
        let is_equal_percent = track_read_field::<u8>(data, offset, path, ranges, "is_equal_percent", "u8")?;
        let drop_roll_type = track_read_field::<u8>(data, offset, path, ranges, "drop_roll_type", "u8")?;
        let nee_slot_count = track_read_field::<u16>(data, offset, path, ranges, "nee_slot_count", "u16")?;
        let drop_tag_name_hash = track_read_field::<u32>(data, offset, path, ranges, "drop_tag_name_hash", "u32")?;
        let drop_roll_count = track_read_field::<u32>(data, offset, path, ranges, "drop_roll_count", "u32")?;
        let list = track_read_field::<CArray<OptionalDropTarget>>(data, offset, path, ranges, "list", "CArray<OptionalDropTarget>")?;
        let need_weight = track_read_field::<u64>(data, offset, path, ranges, "need_weight", "u64")?;
        let total_drop_rate = track_read_field::<u64>(data, offset, path, ranges, "total_drop_rate", "u64")?;
        let roll_dice_skip_rate = track_read_field::<u64>(data, offset, path, ranges, "roll_dice_skip_rate", "u64")?;
        let drop_condition_string = track_read_field::<u32>(data, offset, path, ranges, "drop_condition_string", "u32")?;
        let original_string = track_read_field::<CString<'a>>(data, offset, path, ranges, "original_string", "CString")?;
        Ok(Self {
            string_key, key, is_blocked, is_equal_percent, drop_roll_type, nee_slot_count,
            drop_tag_name_hash, drop_roll_count, list, need_weight, total_drop_rate,
            roll_dice_skip_rate, drop_condition_string, original_string,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.string_key.write_to(w)?;
        self.key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.is_equal_percent.write_to(w)?;
        self.drop_roll_type.write_to(w)?;
        self.nee_slot_count.write_to(w)?;
        self.drop_tag_name_hash.write_to(w)?;
        self.drop_roll_count.write_to(w)?;
        self.list.write_to(w)?;
        self.need_weight.write_to(w)?;
        self.total_drop_rate.write_to(w)?;
        self.roll_dice_skip_rate.write_to(w)?;
        self.drop_condition_string.write_to(w)?;
        self.original_string.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("is_equal_percent".to_string(), self.is_equal_percent.to_json_value());
        m.insert("drop_roll_type".to_string(), self.drop_roll_type.to_json_value());
        m.insert("nee_slot_count".to_string(), self.nee_slot_count.to_json_value());
        m.insert("drop_tag_name_hash".to_string(), self.drop_tag_name_hash.to_json_value());
        m.insert("drop_roll_count".to_string(), self.drop_roll_count.to_json_value());
        m.insert("list".to_string(), self.list.to_json_value());
        m.insert("need_weight".to_string(), self.need_weight.to_json_value());
        m.insert("total_drop_rate".to_string(), self.total_drop_rate.to_json_value());
        m.insert("roll_dice_skip_rate".to_string(), self.roll_dice_skip_rate.to_json_value());
        m.insert("drop_condition_string".to_string(), self.drop_condition_string.to_json_value());
        m.insert("original_string".to_string(), self.original_string.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        // Null-tolerant: a V3 mod authored before 1.18 has no key for this.
        <u8 as WriteJsonValue>::write_from_json(
            w, obj.get("is_equal_percent").unwrap_or(&Value::Null))?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "drop_roll_type")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "nee_slot_count")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "drop_tag_name_hash")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "drop_roll_count")?)?;
        <CArray<OptionalDropTarget> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "list")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "need_weight")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "total_drop_rate")?)?;
        // Null-tolerant: a V3 mod authored before 2.01.00 has no key for this (Stormsteel
        // 1.0 ships 585 whole-record dropset intents built on 2.00.02).
        <u64 as WriteJsonValue>::write_from_json(
            w, obj.get("roll_dice_skip_rate").unwrap_or(&Value::Null))?;
        crate::json_traits::write_string_info_key(w, json_get_field(obj, "drop_condition_string")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "original_string")?)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("dropsetinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = DropSetInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "dropsetinfo roundtrip bytes mismatch");
    }

    /// DIAG_KEY=<hex> cargo test --lib drop_set_info::info::tests::diag_tracked_one -- --ignored --nocapture
    #[test]
    #[ignore = "diagnostic"]
    fn diag_tracked_one() {
        let want = u32::from_str_radix(&std::env::var("DIAG_KEY").unwrap_or_default(), 16).ok();
        let Ok(data) = std::fs::read(pabgb_path()) else { return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { return; };
        for (k, s, e) in entry_ranges(&entries, data.len()) {
            if want.map_or(false, |w| k as u32 != w) { continue; }
            let (mut c, mut path, mut ranges) = (s, String::new(), Vec::new());
            let r = DropSetInfo::read_tracked_with_size(&data, &mut c, e - s, &mut path, &mut ranges);
            eprintln!("k=0x{:x} [{}..{}) len={} result={:?} consumed {}", k, s, e, e - s, r.as_ref().map(|_| "ok").map_err(|x| x.to_string()), c as i64 - s as i64);
            for f in ranges.iter().take(120) { eprintln!("   rel {:>5}..{:<5} {:<14} {}", f.start - s, f.end - s, f.ty, f.path); }
            eprintln!("   bytes: {:02x?}", &data[s..e.min(s + 200)]);
            break;
        }
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: pabgh not found");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = DropSetInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            DropSetInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
