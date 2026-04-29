//! Walk dmmsa's 0.pamt and confirm sublevelinfo.pabgb is registered with the
//! virtual directory the game looks up — must be `gamedata/binary__/client/bin`.
//! Anything else and the game won't find it in dmmsa and will fall back to
//! the vanilla group (which has the unmodified body).

use dmm_parser::binary::pamt::PackMeta;
use std::path::Path;

fn main() {
    let pamt_path = Path::new(
        r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\dmmsa\0.pamt",
    );
    let data = std::fs::read(pamt_path).unwrap();
    let meta = PackMeta::parse(&data, None).unwrap();

    println!("dmmsa/0.pamt has {} directories", meta.directories.len());
    for dir in &meta.directories {
        println!("\nDIR: {}", dir.path);
        for f in &dir.files {
            let marker = if f.name == "sublevelinfo.pabgb" { " ← TARGET" } else { "" };
            println!(
                "  {} | flags=0x{:02x} comp={:?} crypto={:?} | {} bytes ({} compressed){}",
                f.name, f.file.flags, f.file.compression, f.file.crypto,
                f.file.uncompressed_size, f.file.compressed_size, marker
            );
        }
    }
}
