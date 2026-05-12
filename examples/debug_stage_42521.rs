//! Debug entry 42521 of stageinfo.pabgb — the one that breaks roundtrip.
//! Uses read_tracked_with_size to dump every field's (start, end, type)
//! up to the failure point.

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets_from_bytes};
use dmm_parser::binary::FieldRange;
use dmm_parser::tables::stage_info::StageInfo;

const DIR: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_1_06";

fn main() {
    let pabgb = std::fs::read(format!(r"{}\stageinfo.pabgb", DIR)).expect("pabgb");
    let pabgh = std::fs::read(format!(r"{}\stageinfo.pabgh", DIR)).expect("pabgh");
    let entries = load_pabgh_offsets_from_bytes(&pabgh).expect("pabgh parse");
    let ranges = entry_ranges(&entries, pabgb.len());

    let target: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(42521);
    let (k, s, e) = ranges[target];
    let size = e - s;
    println!("Entry {} key=0x{:x} start={} end={} size={}", target, k, s, e, size);
    println!();

    // Try the tracked read
    let mut cursor = s;
    let mut path = String::new();
    let mut field_ranges: Vec<FieldRange> = Vec::new();
    let result = StageInfo::read_tracked_with_size(&pabgb, &mut cursor, size, &mut path, &mut field_ranges);

    // Dump all fields read up to the failure
    println!("Successfully read {} fields before failure:", field_ranges.len());
    for fr in &field_ranges {
        let rel_start = fr.start - s;
        let rel_end = fr.end - s;
        let n = fr.end - fr.start;
        println!("  [{:>4}..{:>4}] ({:>3} bytes) {:<8} {}", rel_start, rel_end, n, fr.ty, fr.path);
    }

    println!();
    match result {
        Ok(_) => println!("Read SUCCESS — cursor at {} (expected {})", cursor, e),
        Err(er) => {
            println!("Read FAIL at absolute offset {} (relative offset {} of {}-byte entry):", cursor, cursor - s, size);
            println!("  Error: {}", er);
            // Dump 32 bytes around the failure point
            let dump_start = cursor.saturating_sub(8);
            let dump_end = (cursor + 24).min(pabgb.len());
            print!("  Bytes [{}-{}] (cursor at offset {}): ", dump_start, dump_end, cursor);
            for (i, b) in pabgb[dump_start..dump_end].iter().enumerate() {
                if dump_start + i == cursor { print!("|>"); }
                print!("{:02x} ", b);
            }
            println!();
        }
    }
}
