//! Probe game_level_info tail data to reverse-engineer the wire structure
//! when the IDA per-record reader is buried.

#[cfg(test)]
mod tests {
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("levelinfo.pabgb") }
#[test]
    fn dump_records() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        eprintln!("{} records", ranges.len());
        for (i, (k, s, e)) in ranges.iter().take(3).enumerate() {
            let size = e - s;
            eprintln!("\n=== record [{}] k=0x{:x} size={} ===", i, k, size);
            // Dump bytes
            for j in (0..size.min(96)).step_by(16) {
                eprint!("  +{:03x}: ", j);
                for k in 0..16 {
                    if j + k < size { eprint!("{:02x} ", data[s + j + k]); }
                }
                eprintln!();
            }
        }
    }
}
