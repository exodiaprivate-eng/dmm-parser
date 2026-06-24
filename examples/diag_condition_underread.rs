//! Measure the GameCondition under-read magnitude on 1.07 condition_info:
//! region_size - node_consumed - 3 (tails). If consistent → one variant short.

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets, find_cstring_u8_trailer};
use dmm_parser::binary::variants::game_condition::GameConditionNode;
use dmm_parser::binary::{BinaryRead, CString};
use dmm_parser::binary::variants::condition_data::TAG_TRAIL;
use std::collections::BTreeMap;

const B: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\live_full\conditioninfo.pabgb";
const H: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\live_full\conditioninfo.pabgh";

fn main() {
    let data = match std::fs::read(B) { Ok(d) => d, Err(_) => { println!("no file"); return; } };
    let entries = load_pabgh_offsets(H).unwrap();
    let ranges = entry_ranges(&entries, data.len());
    let mut hist: BTreeMap<i64, usize> = BTreeMap::new();
    let mut errs: BTreeMap<String, usize> = BTreeMap::new();
    let mut ok = 0; let mut total = 0;
    let mut single_minus2: BTreeMap<u16, usize> = BTreeMap::new();
    let mut shown_plus2 = 0;
    for (_k, s, e) in &ranges {
        total += 1;
        let mut c = *s;
        let _key = u32::read_from(&data, &mut c).unwrap();
        let _sk = CString::read_from(&data, &mut c).unwrap();
        let _ib = u8::read_from(&data, &mut c).unwrap();
        let post_pre = c;
        let Ok(region_size) = find_cstring_u8_trailer(&data, post_pre, *e) else { continue };
        let region = &data[post_pre..post_pre + region_size];
        let mut nc = 0usize;
        match GameConditionNode::read_from(region, &mut nc) {
            Ok(_) => {
                let underread = region_size as i64 - nc as i64 - 3; // 3 tail u8s
                if underread == 0 { ok += 1; }
                else {
                    *hist.entry(underread).or_default() += 1;
                    // Capture the variant trail (disc, post_offset) for the
                    // first few -2 records — the last disc tends to be where
                    // the +2-short variant left off.
                    if underread == 2 {
                        // +2 trees under-read by 2. For SHORT trails (≤3 nodes)
                        // dump per-variant consumed (offset deltas) so the short
                        // variant is identifiable, with region hex.
                        TAG_TRAIL.with(|t| {
                            let tr = t.borrow();
                            for (d, _) in tr.iter() { *single_minus2.entry(*d).or_default() += 1; }
                            if tr.len() <= 3 && shown_plus2 < 6 {
                                shown_plus2 += 1;
                                let mut prev = post_pre;
                                let parts: Vec<String> = tr.iter().map(|(d, off)| {
                                    let cons = off.saturating_sub(prev); prev = *off;
                                    format!("disc{}={}b", d, cons)
                                }).collect();
                                let hx: String = region.iter().enumerate().map(|(i,b)| {
                                    let c = if b.is_ascii_graphic() { *b as char } else { '.' };
                                    format!("{:02x}{}", b, if i%4==3 {format!("|{}",c)} else {String::new()})
                                }).collect::<Vec<_>>().join(" ");
                                println!("  +2 trail({} nodes, region {}B): {} | hex: {}",
                                    tr.len(), region_size, parts.join(" "), hx);
                            }
                        });
                    }
                }
            }
            Err(e) => {
                *hist.entry(-9999).or_default() += 1;
                let msg = e.to_string();
                let key = msg.split(" at offset ").next().unwrap_or(&msg).to_string();
                *errs.entry(key).or_default() += 1;
            }
        }
    }
    println!("condition_info: {} records, node+3==region (clean): {}", total, ok);
    println!("under/over-read magnitude histogram (region - node_consumed - 3):");
    for (mag, n) in &hist { println!("  {:>6} bytes : {} records", mag, n); }
    println!("\nnode-decode error messages (the -9999 bucket):");
    let mut ev: Vec<_> = errs.iter().collect();
    ev.sort_by(|a, b| b.1.cmp(a.1));
    for (msg, n) in ev.iter().take(15) { println!("  {:>5}  {}", n, msg); }
    println!("\nsingle-variant -2 records by disc (the unambiguous over-readers / fix worklist):");
    let mut sv: Vec<_> = single_minus2.iter().collect();
    sv.sort_by(|a, b| b.1.cmp(a.1));
    for (disc, n) in &sv { println!("  disc {:>3} : {} records", disc, n); }
}
