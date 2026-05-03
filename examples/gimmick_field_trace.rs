//! Field-by-field tracer: parse one failing GimmickPostBody blob and show which field fails.
//! Also traces a successful blob for comparison at the same offsets.

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::binary::BinaryRead;
use dmm_parser::tables::gimmick_info::info::{GimmickInfo, GimmickPostBody, GimmickTail};

const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgb";
const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgh";

fn hex16(bytes: &[u8]) -> String {
    let s: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    s.join(" ")
}

fn main() {
    let data = match std::fs::read(PABGB) {
        Ok(d) => d, Err(e) => { eprintln!("SKIP: {}", e); return; }
    };
    let entries = match load_pabgh_offsets(PABGH) {
        Some(e) => e, None => { eprintln!("SKIP pabgh"); return; }
    };
    let ranges = entry_ranges(&entries, data.len());

    // Collect failing blobs (count=0, GimmickPostBody fails) — largest ones first
    let mut fail_blobs: Vec<(u32, Vec<u8>)> = Vec::new();
    // Also collect some successful blobs of similar size
    let mut ok_blobs: Vec<(u32, Vec<u8>)> = Vec::new();

    for (key, start, end) in &ranges {
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it, Err(_) => continue,
        };
        let GimmickTail::Decoded { alt_trigger_list, post_body, post_blob, .. } = &item.tail else { continue };
        let Some(atl) = alt_trigger_list else { continue };
        if !atl.items.is_empty() { continue; }

        if post_body.is_some() {
            if ok_blobs.len() < 5 && post_blob.is_empty() {
                // Successfully fully decoded — post_blob should be empty
                // Actually post_blob is empty because all decoded into post_body
                ok_blobs.push((*key, post_blob.clone()));
            }
        } else if !post_blob.is_empty() {
            fail_blobs.push((*key, post_blob.clone()));
        }
    }

    // Sort fail_blobs by size descending, take the largest
    fail_blobs.sort_by_key(|(_, b)| std::cmp::Reverse(b.len()));
    println!("Failing blob count: {}", fail_blobs.len());
    println!("Largest failing blob sizes: {:?}", fail_blobs.iter().take(5).map(|(k, b)| (*k, b.len())).collect::<Vec<_>>());

    // Parse the 3 largest failing blobs field-by-field using our known field sequence
    // We'll parse each individual field and report the offset after each
    for (idx, (key, blob)) in fail_blobs.iter().take(3).enumerate() {
        println!("\n=== Failing blob #{idx} key=0x{key:08x} len={} ===", blob.len());
        trace_parse(blob);
    }

    // Also trace a successful blob (we need to get one from a known-success entry)
    // Find the minimum-size successful post_blob — these are fully parsed so post_blob=empty
    // Instead, let's find entries where we know the parse succeeded by checking that post_blob is empty
    // Actually we need to trace parsing on raw data. Let's grab a successful entry's raw tail bytes.
    println!("\n=== Byte dumps around offset 1150-1210 for failing blobs ===");
    for (key, blob) in fail_blobs.iter().take(3) {
        let lo = 1140usize;
        let hi = (lo + 80).min(blob.len());
        if hi <= lo { println!("blob too short"); continue; }
        println!("key=0x{key:08x} bytes[{lo}..{hi}]:");
        for chunk in (lo..hi).step_by(16) {
            let end = (chunk + 16).min(hi);
            println!("  [{chunk}..{end}]: {}", hex16(&blob[chunk..end]));
        }
    }
}

fn trace_parse(blob: &[u8]) {
    // We can't easily step field-by-field without modifying the library.
    // Instead, do a binary search: try parsing from byte N and see when it fails.
    // If parsing fails from byte N but succeeds from byte N-1, we know the issue is at byte N.

    // Strategy: check if parsing the WHOLE blob succeeds
    let mut probe = 0;
    match GimmickPostBody::read_from(blob, &mut probe) {
        Ok(_) => println!("  Parsed OK! Consumed {} of {} bytes", probe, blob.len()),
        Err(e) => println!("  Error at probe={}: {}", probe, e),
    }

    // Show bytes around the failure point
    let fail_probe = probe;
    if fail_probe > 8 {
        let lo = fail_probe.saturating_sub(16);
        let hi = (fail_probe + 16).min(blob.len());
        println!("  Bytes [{lo}..{hi}] around failure (probe={fail_probe}):");
        for chunk in (lo..hi).step_by(16) {
            let end = (chunk + 16).min(hi);
            println!("    [{chunk}..{end}]: {}", hex16(&blob[chunk..end]));
        }
    }

    // Binary search: find latest prefix that still succeeds... Actually can't do this easily.
    // Instead: show what the 4 bytes at probe look like (the bad CArray count)
    if fail_probe + 4 <= blob.len() {
        let count_bytes = &blob[fail_probe..fail_probe+4];
        let bad_count = u32::from_le_bytes(count_bytes.try_into().unwrap());
        println!("  Bad CArray count: {} = 0x{:08x} at byte {}", bad_count, bad_count, fail_probe);
        println!("  As bytes: {}", hex16(count_bytes));
    }

    // Show which f-field offset ~1185 corresponds to by comparing with known minimum-size structure
    // Minimum GimmickPostBody with all-empty CArrays: estimate key field offsets
    // F87 starts around byte ~900 based on rough calculation
    // Let's show bytes 900-1300 in 16-byte lines
    let lo = 900usize;
    let hi = (lo + 400).min(blob.len());
    if hi > lo {
        println!("  Bytes [{lo}..{hi}] (field area around probe):");
        let mut cur = lo;
        while cur < hi {
            let end = (cur + 16).min(hi);
            let marker = if cur <= fail_probe && fail_probe < end { " ← FAIL HERE" } else { "" };
            println!("    [{cur}..{end}]: {}{marker}", hex16(&blob[cur..end]));
            cur += 16;
        }
    }
}
