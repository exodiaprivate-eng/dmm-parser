//! Per-record consumed-vs-range check for MiniGameDataInfo, plus which fields
//! are still opaque.
//!
//! ⚠ A table-level "byte-roundtrip=true" does NOT mean the layout is right.
//! `blob_runtime` accepts an UNDER-READ record as a success and stores the
//! unread tail, so a wrong struct can still roundtrip byte-for-byte while
//! decoding nothing useful. The number that means something is: every record
//! consumed EXACTLY its pabgh range.
//!
//! Usage: DMM_PARSER_PABGB_DIR=<dir> cargo run --example minigame_consume_check
use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::tables::mini_game_data_info::info::MiniGameDataInfo;
use std::path::PathBuf;

fn main() {
    let dir = std::env::var("DMM_PARSER_PABGB_DIR")
        .unwrap_or_else(|_| r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\live_full".into());
    let data = std::fs::read(PathBuf::from(&dir).join("minigamedatainfo.pabgb")).unwrap();
    let entries = load_pabgh_offsets(PathBuf::from(&dir).join("minigamedatainfo.pabgh").to_str().unwrap()).unwrap();
    let ranges = entry_ranges(&entries, data.len());

    let (mut exact, mut under, mut err) = (0, 0, 0);
    for (key, start, end) in &ranges {
        let mut cur = *start;
        match MiniGameDataInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(_) => {
                let d = cur as i64 - *end as i64;
                if d == 0 {
                    exact += 1;
                } else {
                    under += 1;
                    println!("  k=0x{:x} size={:<5} consumed={:<5} SHORT BY {}", key, end - start, cur - start, -d);
                }
            }
            Err(e) => { err += 1; println!("  k=0x{:x} size={:<5} ERR {}", key, end - start, e); }
        }
    }
    println!("{} records: {} exact, {} under-read, {} error", ranges.len(), exact, under, err);
}
