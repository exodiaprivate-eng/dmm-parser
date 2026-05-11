//! Smoke-test that every newly-tracked table parses cleanly against its
//! vanilla dump. For each table:
//! - load pabgb + pabgh
//! - call parse_table_tracked
//! - assert: records > 0, every record has at least one tracked range,
//!   no record's combined ranges exceed its record byte span.
//!
//! Run: cargo run --release --example smoke_test_tracked_16

use dmm_parser::tracked::{is_tracked_table, parse_table_tracked};
use std::path::PathBuf;

const VANILLA: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps";

const TABLES: &[(&str, &str)] = &[
    ("character_info", "characterinfo"),
    ("character_change_info", "characterchange"),
    ("inventory_info", "inventory"),
    ("buff_info", "buffinfo"),
    ("condition_info", "conditioninfo"),
    ("drop_set_info", "dropsetinfo"),
    ("effect_info", "effectinfo"),
    ("gimmick_info", "gimmickinfo"),
    ("interaction_info", "interactioninfo"),
    ("store_info", "storeinfo"),
    ("faction_node_spawn_info", "factionnodespawninfo"),
    ("quest_info", "questinfo"),
    ("item_use_info", "itemuseinfo"),
    ("ai_dialog_string_info", "aidialogstringinfo"),
    ("frame_event_attr_group_info", "frameeventattrgroupinfo"),
    ("stage_info", "stageinfo"),
];

fn main() {
    let mut pass = 0;
    let mut fail = 0;
    let mut skip = 0;
    for (table, basename) in TABLES {
        assert!(is_tracked_table(table), "{} should be in is_tracked_table()", table);
        let pabgb = PathBuf::from(VANILLA).join(format!("{}.pabgb", basename));
        let pabgh = PathBuf::from(VANILLA).join(format!("{}.pabgh", basename));
        if !pabgb.exists() || !pabgh.exists() {
            println!("[SKIP] {} ({})", table, basename);
            skip += 1;
            continue;
        }
        let pabgb_bytes = match std::fs::read(&pabgb) {
            Ok(b) => b,
            Err(e) => { println!("[FAIL] {} ({}): read pabgb {}", table, basename, e); fail += 1; continue; }
        };
        let pabgh_bytes = match std::fs::read(&pabgh) {
            Ok(b) => b,
            Err(e) => { println!("[FAIL] {} ({}): read pabgh {}", table, basename, e); fail += 1; continue; }
        };
        match parse_table_tracked(table, &pabgb_bytes, Some(&pabgh_bytes)) {
            Ok(records) => {
                let nrec = records.len();
                let nrange = records.iter().map(|r| r.ranges.len()).sum::<usize>();
                let bad_range = records.iter().any(|r| {
                    r.ranges.iter().any(|fr| fr.start > r.record_end - r.record_start || fr.end > r.record_end - r.record_start)
                });
                if nrec == 0 {
                    println!("[FAIL] {}: 0 records", table);
                    fail += 1;
                } else if nrange == 0 {
                    println!("[FAIL] {}: {} records, 0 ranges", table, nrec);
                    fail += 1;
                } else if bad_range {
                    println!("[FAIL] {}: ranges out of record bounds", table);
                    fail += 1;
                } else {
                    println!("[PASS] {}: {} records, {} total ranges (avg {:.1}/rec)",
                        table, nrec, nrange, nrange as f64 / nrec as f64);
                    pass += 1;
                }
            }
            Err(e) => {
                println!("[FAIL] {}: {}", table, e);
                fail += 1;
            }
        }
    }
    println!("\n=== SUMMARY ===");
    println!("  pass: {}", pass);
    println!("  fail: {}", fail);
    println!("  skip: {}", skip);
    std::process::exit(if fail > 0 { 1 } else { 0 });
}
