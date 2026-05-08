//! Dump item 1's field offsets for two iteminfo files.
//! Usage: cargo run --release --example diag_item1 -- <file1> <file2>

use dmm_parser::binary::{BinaryRead, BinaryReadTracked, FieldRange};
use dmm_parser::item_info::ItemInfo;
use std::env;
use std::fs;

fn dump_one(path: &str) {
    let data = fs::read(path).expect("read");
    println!("\n=== {} ({} bytes) ===", path, data.len());
    let mut offset = 0usize;
    let mut path_buf = String::new();
    let mut ranges: Vec<FieldRange> = Vec::new();
    match ItemInfo::read_tracked(&data, &mut offset, &mut path_buf, &mut ranges) {
        Ok(_) => println!("item 1 parsed OK, ended at offset {}", offset),
        Err(e) => println!("item 1 PARSE FAILED at offset {}: {}", offset, e),
    }
    for r in ranges.iter().take(2000) {
        println!("  {:5}-{:5} ({:3}B)  {}", r.start, r.end, r.end - r.start, r.path);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    for a in &args[1..] {
        dump_one(a);
    }
}
