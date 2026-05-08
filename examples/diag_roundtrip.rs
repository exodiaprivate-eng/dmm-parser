//! Verify byte-perfect round-trip on user's 1.05.01 iteminfo.
//! Usage: cargo run --release --example diag_roundtrip -- <file>

use dmm_parser::item_info::{parse_iteminfo_to_json, serialize_iteminfo_from_json};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("usage: diag_roundtrip <file>");
    let original = fs::read(path).expect("read file");
    println!("file: {} ({} bytes)", path, original.len());

    let items = parse_iteminfo_to_json(&original).expect("parse");
    println!("parsed {} items", items.len());

    let serialized = serialize_iteminfo_from_json(&items).expect("serialize");
    println!("serialized {} bytes", serialized.len());

    if serialized.len() != original.len() {
        println!(
            "FAIL: length mismatch: original={} serialized={} (diff={})",
            original.len(),
            serialized.len(),
            serialized.len() as i64 - original.len() as i64
        );
        return;
    }
    if &serialized[..] == &original[..] {
        println!("OK: byte-perfect round-trip");
        return;
    }
    // Find first difference
    for i in 0..original.len() {
        if original[i] != serialized[i] {
            let s = i.saturating_sub(8);
            let e = (i + 16).min(original.len());
            println!("FAIL: bytes differ at offset {} (0x{:x})", i, i);
            print!("  original:   ");
            for b in &original[s..e] {
                print!("{:02x} ", b);
            }
            println!();
            print!("  serialized: ");
            for b in &serialized[s..e] {
                print!("{:02x} ", b);
            }
            println!();
            break;
        }
    }
}
