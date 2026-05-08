use dmm_parser::binary::BinaryRead;
use dmm_parser::item_info::ItemInfo;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let data = fs::read(&args[1]).expect("read");
    println!("file: {} ({} bytes)", args[1], data.len());
    let mut offset = 0usize;
    let mut count = 0usize;
    let mut last_key: u32 = 0;
    while offset < data.len() {
        let start = offset;
        match ItemInfo::read_from(&data, &mut offset) {
            Ok(item) => {
                count += 1;
                last_key = item.key.0;
            }
            Err(e) => {
                println!("FAIL after {} items at offset {} (item started {}): {}", count, offset, start, e);
                println!("last successful key={}", last_key);
                println!("item-start hex: 0x{:x}", start);
                println!("fail-offset hex: 0x{:x}", offset);
                return;
            }
        }
    }
    println!("OK: {} items parsed", count);
}
