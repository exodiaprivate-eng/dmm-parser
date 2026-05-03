//! Probe special_mode_info tail to determine if vanilla data has small
//! enough variety to type.

#[cfg(test)]
mod tests {
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/specialmode.pabgb";
    const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/specialmode.pabgh";

    #[test]
    fn dump_records() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        eprintln!("{} records", ranges.len());
        for (i, (k, s, e)) in ranges.iter().take(2).enumerate() {
            let size = e - s;
            eprintln!("\n=== record [{}] k=0x{:x} size={} ===", i, k, size);
            // Print all bytes
            for j in (0..size).step_by(16) {
                eprint!("  +{:04x}: ", j);
                for kk in 0..16 {
                    if j + kk < size { eprint!("{:02x} ", data[s + j + kk]); }
                }
                eprintln!();
            }
        }
    }
}
