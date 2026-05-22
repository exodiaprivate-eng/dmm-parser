//! Per-record item_use roundtrip + consume check: determine if the lossy
//! decoded records UNDER-CONSUME (read < payload) or WRITE-LOSSY (consume
//! full but write short). Bucket by disc + cause.

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::tables::item_use_info::ItemUseInfo;
use std::collections::BTreeMap;

const B: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\live_full\itemuseinfo.pabgb";
const H: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\live_full\itemuseinfo.pabgh";

fn main() {
    let data = std::fs::read(B).unwrap();
    let entries = load_pabgh_offsets(H).unwrap();
    let ranges = entry_ranges(&entries, data.len());
    let mut ok = 0;
    let mut read_err = 0;
    let mut under = 0;      // read succeeded but consumed < entry_size
    let mut write_lossy = 0; // consumed == size but serialize != original
    let mut by_cause: BTreeMap<String, usize> = BTreeMap::new();
    for (_k, s, e) in &ranges {
        let size = e - s;
        let mut c = *s;
        match ItemUseInfo::read_with_size(&data, &mut c, size) {
            Err(_) => { read_err += 1; }
            Ok(item) => {
                let consumed = c - s;
                if consumed != size {
                    under += 1;
                    *by_cause.entry(format!("UNDER consumed {}/{}", consumed, size)).or_default() += 1;
                } else {
                    let mut out = Vec::new();
                    item.write_to(&mut out).unwrap();
                    if out.as_slice() == &data[*s..*e] { ok += 1; }
                    else {
                        write_lossy += 1;
                        *by_cause.entry(format!("WRITE-LOSSY out={} in={}", out.len(), size)).or_default() += 1;
                    }
                }
            }
        }
    }
    println!("item_use: {} records | ok={} read_err(→blob)={} under_consume={} write_lossy={}",
        ranges.len(), ok, read_err, under, write_lossy);
    let mut v: Vec<_> = by_cause.iter().collect();
    v.sort_by(|a,b| b.1.cmp(a.1));
    for (c, n) in v.iter().take(12) { println!("  {:>4}  {}", n, c); }
}
