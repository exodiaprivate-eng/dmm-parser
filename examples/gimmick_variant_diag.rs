//! Diagnostic: characterize gimmick entries where GimmickPostBody fails.
//!
//! These are entries where alt_trigger_list.is_some() but GimmickPostBody::read_from fails.
//! Run against the 4-24 dump to understand the variant discriminators before IDA work.

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::tables::gimmick_info::{GimmickInfo, GimmickTail};

const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgb";
const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgh";

fn hex16(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

fn main() {
    let data = match std::fs::read(PABGB) {
        Ok(d) => d,
        Err(e) => { eprintln!("SKIP: {}", e); return; }
    };
    let entries = match load_pabgh_offsets(PABGH) {
        Some(e) => e,
        None => { eprintln!("SKIP pabgh"); return; }
    };
    let ranges = entry_ranges(&entries, data.len());
    println!("Total entries: {}", ranges.len());

    let mut total_decoded = 0usize;
    let mut post_body_ok = 0usize;
    let mut post_body_fail = 0usize; // alt_trigger_list.is_some() but post_body.is_none() and post_blob non-empty
    let mut no_atl = 0usize;

    // alt_trigger name → (count_of_entries, first_post_blob_bytes)
    let mut by_atl_count: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    // post_blob first-4-bytes histogram for failing entries
    let mut fail_first4: std::collections::BTreeMap<[u8;4], usize> = std::collections::BTreeMap::new();
    // sample failing entries for display
    let mut fail_samples: Vec<(u32, u32, Vec<u8>)> = Vec::new(); // (key, atl_count, post_blob)

    for (key, start, end) in &ranges {
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it,
            Err(_) => continue,
        };
        if let GimmickTail::Decoded { alt_trigger_list, post_body, post_blob, .. } = &item.tail {
            total_decoded += 1;
            match alt_trigger_list {
                None => { no_atl += 1; }
                Some(atl) => {
                    let cnt = atl.items.len() as u32;
                    *by_atl_count.entry(cnt).or_insert(0) += 1;
                    if post_body.is_some() {
                        post_body_ok += 1;
                    } else if !post_blob.is_empty() {
                        post_body_fail += 1;
                        if post_blob.len() >= 4 {
                            let fb: [u8;4] = post_blob[..4].try_into().unwrap();
                            *fail_first4.entry(fb).or_insert(0) += 1;
                        }
                        if fail_samples.len() < 20 {
                            fail_samples.push((*key, cnt, post_blob.clone()));
                        }
                    }
                }
            }
        }
    }

    println!("Decoded (GimmickTail::Decoded): {}", total_decoded);
    println!("  alt_trigger_list: None  : {}", no_atl);
    println!("  post_body decoded OK    : {}", post_body_ok);
    println!("  post_body FAILED (blob) : {}", post_body_fail);

    println!("\nalt_trigger_list count distribution:");
    for (cnt, n) in &by_atl_count {
        println!("  count={}: {} entries", cnt, n);
    }

    println!("\nFirst-4-bytes of post_blob for failing entries (top 20):");
    let mut sorted: Vec<_> = fail_first4.iter().collect();
    sorted.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (bytes, count) in sorted.iter().take(20) {
        println!("  {:02x} {:02x} {:02x} {:02x}  ({} entries)", bytes[0], bytes[1], bytes[2], bytes[3], count);
    }

    println!("\nSample failing entries (post_blob first 64 bytes):");
    for (key, atl_cnt, blob) in fail_samples.iter().take(10) {
        println!("  key=0x{:08x}  atl_count={}  blob_len={}", key, atl_cnt, blob.len());
        for chunk in (0..blob.len().min(64)).step_by(16) {
            let end = (chunk + 16).min(blob.len());
            println!("    [{:3}..{:3}]: {}", chunk, end, hex16(&blob[chunk..end]));
        }
    }
}
