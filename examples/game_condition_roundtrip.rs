//! Round-trip validator for the typed GameCondition decoder.
//!
//! For each ConditionInfo entry's GameCondition blob:
//!   1. Read with GameConditionNode::read_from
//!   2. Write back with GameConditionNode::write_to
//!   3. Byte-compare against original
//!
//! Reports per-tag pass/fail counts so we know which ConditionData variants
//! still have wrong byte recipes.

use dmm_parser::binary::variants::game_condition::GameConditionNode;
use dmm_parser::binary::variant::{entry_ranges, find_cstring_u8_trailer, load_pabgh_offsets};
use dmm_parser::binary::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\conditioninfo.pabgb";
const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\conditioninfo.pabgh";

fn main() {
    let data = std::fs::read(Path::new(PABGB)).unwrap();
    let entries = load_pabgh_offsets(PABGH).unwrap();
    let ranges = entry_ranges(&entries, data.len());
    println!("ConditionInfo: {} entries", ranges.len());

    // Each ConditionInfo entry layout:
    //   u32 key, CString string_key, u8 is_blocked, GameCondition blob,
    //   CString original_string, u8 parser_type
    // The GameCondition blob runs from end-of-(key/string_key/is_blocked)
    // to start-of-(original_string/parser_type). We use find_cstring_u8_trailer
    // to find where the trailing CString starts.

    let mut total = 0usize;
    let mut decode_ok = 0usize;
    let mut decode_err = 0usize;
    let mut roundtrip_ok = 0usize;
    let mut roundtrip_mismatch = 0usize;
    let mut mismatch_examples: Vec<(u32, usize, usize)> = Vec::new();

    // Track per-root-case tag: total / pass / fail
    let mut case_stats: BTreeMap<u8, (usize, usize, usize)> = BTreeMap::new();
    // For case=3 (ConditionData) entries: per-u16-tag (rt_pass, rt_fail, decode_err)
    let mut cdata_tag_stats: BTreeMap<u16, (usize, usize, usize)> = BTreeMap::new();

    for (k, s, e) in &ranges {
        total += 1;
        // Skip prefix: u32 key, CString string_key, u8 is_blocked (matches diagnose_conditiondata)
        let mut cursor = *s + 4;
        let cs_len = u32::from_le_bytes([data[cursor], data[cursor+1], data[cursor+2], data[cursor+3]]) as usize;
        cursor += 4 + cs_len + 1;
        let blob_start = cursor;
        let blob_size = match find_cstring_u8_trailer(&data, blob_start, *e) {
            Ok(sz) => sz,
            Err(_) => { decode_err += 1; continue; }
        };
        let blob_end = blob_start + blob_size;
        if blob_end <= blob_start {
            decode_err += 1;
            continue;
        }
        let blob = &data[blob_start..blob_end];

        // Capture root case_tag for stats
        let root_case = blob.first().copied().unwrap_or(255);
        let entry = case_stats.entry(root_case).or_insert((0, 0, 0));
        entry.0 += 1;
        // For case=3 (ConditionData root), peek the u16 tag at bytes [1..3]
        let cdata_tag = if root_case == 3 && blob.len() >= 3 {
            Some(u16::from_le_bytes([blob[1], blob[2]]))
        } else {
            None
        };

        // Try to decode
        let mut parse_cur = 0usize;
        let node = match GameConditionNode::read_from(blob, &mut parse_cur) {
            Ok(n) => n,
            Err(_) => {
                decode_err += 1;
                entry.2 += 1;
                if let Some(t) = cdata_tag {
                    cdata_tag_stats.entry(t).or_insert((0,0,0)).2 += 1;
                }
                continue;
            }
        };
        decode_ok += 1;

        if parse_cur != blob.len() {
            // Decoder under/over-consumed
            roundtrip_mismatch += 1;
            entry.2 += 1;
            if let Some(t) = cdata_tag {
                cdata_tag_stats.entry(t).or_insert((0,0,0)).1 += 1;
            }
            if mismatch_examples.len() < 10 {
                mismatch_examples.push((*k, parse_cur, blob.len()));
            }
            continue;
        }

        // Round-trip: re-encode and compare
        let mut buf: Vec<u8> = Vec::with_capacity(blob.len());
        if let Err(_) = node.write_to(&mut buf) {
            roundtrip_mismatch += 1;
            entry.2 += 1;
            continue;
        }
        if buf == blob {
            roundtrip_ok += 1;
            entry.1 += 1;
            if let Some(t) = cdata_tag {
                cdata_tag_stats.entry(t).or_insert((0,0,0)).0 += 1;
            }
        } else {
            roundtrip_mismatch += 1;
            entry.2 += 1;
            if let Some(t) = cdata_tag {
                cdata_tag_stats.entry(t).or_insert((0,0,0)).1 += 1;
            }
            if mismatch_examples.len() < 10 {
                let diff_at = buf.iter().zip(blob.iter()).position(|(a, b)| a != b).unwrap_or(buf.len().min(blob.len()));
                mismatch_examples.push((*k, diff_at, blob.len()));
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total entries:        {}", total);
    println!("Decode OK:            {}", decode_ok);
    println!("Decode err:           {}", decode_err);
    println!("Round-trip OK:        {} ({:.1}%)", roundtrip_ok, roundtrip_ok as f64 * 100.0 / total as f64);
    println!("Round-trip mismatch:  {}", roundtrip_mismatch);

    println!("\n=== Per-root-case tag stats ===");
    println!("case | total | pass | fail | pass%");
    for (case, (tot, pass, fail)) in &case_stats {
        let pct = if *tot > 0 { *pass as f64 * 100.0 / *tot as f64 } else { 0.0 };
        println!("  {:3}  | {:5} | {:4} | {:4} | {:5.1}%", case, tot, pass, fail, pct);
    }

    if !mismatch_examples.is_empty() {
        println!("\n=== First {} mismatch examples (key, parsed_or_diff_byte, blob_len) ===", mismatch_examples.len());
        for (k, p, l) in &mismatch_examples {
            println!("  key=0x{:08X}: cursor/diff={}, blob_len={}", k, p, l);
        }
    }

    // Per-ConditionData-tag breakdown (case 3 only)
    println!("\n=== ConditionData (case 3) per-u16-tag round-trip stats ===");
    println!("tag  | total | pass | fail | pass%");
    let mut tags: Vec<(u16, (usize, usize))> = cdata_tag_stats.into_iter().collect();
    tags.sort_by_key(|(_, (p, f))| -(*f as isize));  // sort by failures desc
    let mut shown = 0usize;
    for (tag, (pass, fail)) in &tags {
        let total = pass + fail;
        if *fail == 0 { continue; }  // skip clean tags
        let pct = if total > 0 { *pass as f64 * 100.0 / total as f64 } else { 0.0 };
        println!("  {:4} | {:5} | {:4} | {:4} | {:5.1}%", tag, total, pass, fail, pct);
        shown += 1;
        if shown >= 50 { break; }
    }
    if shown < tags.len() {
        let remaining_failures: usize = tags.iter().skip(shown).filter(|(_, (_, f))| *f > 0).count();
        if remaining_failures > 0 {
            println!("  ... {} more failing tags suppressed", remaining_failures);
        }
    }
    let clean_tags: usize = tags.iter().filter(|(_, (_, f))| *f == 0).count();
    println!("\nClean tags (always round-trip):  {}", clean_tags);
}
