//! Check whether any item in the user's iteminfo has nonzero cooltime.b or cooltime.c.
//! If all are zero, exposing cooltime as a single number in JSON is safe.
use dmm_parser::item_info::parse_iteminfo_to_json;
use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).expect("usage: check_cooltime_bc <file>");
    let data = fs::read(&path).expect("read");
    let items = parse_iteminfo_to_json(&data).expect("parse");
    let mut nz_b = 0;
    let mut nz_c = 0;
    let mut nz_max_b = 0;
    let mut nz_max_c = 0;
    for item in &items {
        let obj = item.as_object().expect("item should be object");
        if let Some(ct) = obj.get("cooltime").and_then(|v| v.as_object()) {
            if ct.get("b").and_then(|x| x.as_i64()).unwrap_or(0) != 0 { nz_b += 1; }
            if ct.get("c").and_then(|x| x.as_i64()).unwrap_or(0) != 0 { nz_c += 1; }
        }
        if let Some(mc) = obj.get("max_charged_useable_count").and_then(|v| v.as_object()) {
            if mc.get("b").and_then(|x| x.as_u64()).unwrap_or(0) != 0 { nz_max_b += 1; }
            if mc.get("c").and_then(|x| x.as_u64()).unwrap_or(0) != 0 { nz_max_c += 1; }
        }
    }
    println!("items: {}", items.len());
    println!("cooltime.b nonzero: {}", nz_b);
    println!("cooltime.c nonzero: {}", nz_c);
    println!("max_charged_useable_count.b nonzero: {}", nz_max_b);
    println!("max_charged_useable_count.c nonzero: {}", nz_max_c);
}
