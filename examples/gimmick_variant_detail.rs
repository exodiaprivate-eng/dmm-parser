//! Detailed variant analysis:
//!  1. For count=0 failures: try parsing the post_blob through GimmickPostBody and show where it fails
//!  2. For count=1 entries: show the alt_trigger name and first 64 bytes of post_blob

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::binary::BinaryRead;
use dmm_parser::tables::gimmick_info::info::{GimmickInfo, GimmickPostBody, GimmickTail};

const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgb";
const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgh";

fn hex16(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

fn main() {
    let data = match std::fs::read(PABGB) {
        Ok(d) => d, Err(e) => { eprintln!("SKIP: {}", e); return; }
    };
    let entries = match load_pabgh_offsets(PABGH) {
        Some(e) => e, None => { eprintln!("SKIP pabgh"); return; }
    };
    let ranges = entry_ranges(&entries, data.len());

    // ── Part 1: count=0 failures ──────────────────────────────────────────────
    println!("=== Part 1: count=0 entries where GimmickPostBody fails ===");
    let mut fail0_errors: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut fail0_overshot: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new(); // delta → count
    let mut fail0_samples: Vec<(u32, Vec<u8>)> = Vec::new();
    let mut ok0 = 0usize;
    let mut fail0 = 0usize;

    // ── Part 2: count=1 entries ───────────────────────────────────────────────
    let mut atl1_name_hist: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut atl1_samples: Vec<(u32, String, Vec<u8>)> = Vec::new(); // (key, name, post_blob)

    for (key, start, end) in &ranges {
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it, Err(_) => continue,
        };
        let GimmickTail::Decoded { alt_trigger_list, post_body, post_blob, .. } = &item.tail else { continue };
        let Some(atl) = alt_trigger_list else { continue };

        if atl.items.is_empty() {
            // count=0
            if post_body.is_some() {
                ok0 += 1;
            } else if !post_blob.is_empty() {
                fail0 += 1;
                // Try parsing the blob to find the exact failure point
                let blob = post_blob;
                let mut probe = 0usize;
                match GimmickPostBody::read_from(blob, &mut probe) {
                    Ok(_) => {
                        // It parsed OK but must have overshot the entry_end
                        let overshoot = probe.saturating_sub(blob.len());
                        *fail0_overshot.entry(overshoot).or_insert(0) += 1;
                    }
                    Err(e) => {
                        let msg = format!("{}", e);
                        // Show truncated error key
                        let short: String = msg.chars().take(80).collect();
                        *fail0_errors.entry(format!("at probe={} err: {}", probe, short)).or_insert(0) += 1;
                    }
                }
                if fail0_samples.len() < 5 {
                    fail0_samples.push((*key, blob.clone()));
                }
            }
        } else if atl.items.len() == 1 {
            // count=1: read the trigger name
            let name = atl.items[0].value.as_ref().map(|cs| {
                cs.data.to_string()
            }).unwrap_or_else(|| "<None>".to_string());
            *atl1_name_hist.entry(name.clone()).or_insert(0) += 1;
            if atl1_samples.len() < 30 && !post_blob.is_empty() {
                atl1_samples.push((*key, name, post_blob.clone()));
            }
        }
    }

    println!("count=0: {} OK, {} fail", ok0, fail0);
    println!("\nFail reasons (errors):");
    for (msg, cnt) in fail0_errors.iter().take(20) {
        println!("  [{}x] {}", cnt, msg);
    }
    println!("\nFail reasons (overshoot delta → count):");
    for (delta, cnt) in fail0_overshot.iter().take(20) {
        println!("  overshoot +{}: {} entries", delta, cnt);
    }

    // ── Part 2 output ─────────────────────────────────────────────────────────
    println!("\n=== Part 2: count=1 entries — alt_trigger names ===");
    println!("Unique names ({} entries total):", atl1_name_hist.values().sum::<usize>());
    let mut name_sorted: Vec<_> = atl1_name_hist.iter().collect();
    name_sorted.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (name, cnt) in &name_sorted {
        println!("  {:4} × {:?}", cnt, name);
    }

    println!("\nSample count=1 entries (first 64 bytes of post_blob):");
    let mut shown_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (key, name, blob) in &atl1_samples {
        if shown_names.len() >= 15 { break; }
        if shown_names.insert(name.clone()) {
            println!("  key=0x{:08x}  name={:?}  blob_len={}", key, name, blob.len());
            for chunk in (0..blob.len().min(64)).step_by(16) {
                let end = (chunk + 16).min(blob.len());
                println!("    [{:3}..{:3}]: {}", chunk, end, hex16(&blob[chunk..end]));
            }
        }
    }

    // ── Part 3: size distribution of count=1 post_blobs by name ──────────────
    println!("\n=== Part 3: post_blob size range by alt_trigger name ===");
    let mut by_name: std::collections::BTreeMap<String, (usize, usize)> = std::collections::BTreeMap::new(); // name → (min, max)
    for (_, start, end) in &ranges {
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it, Err(_) => continue,
        };
        let GimmickTail::Decoded { alt_trigger_list, post_blob, .. } = &item.tail else { continue };
        let Some(atl) = alt_trigger_list else { continue };
        if atl.items.len() != 1 { continue; }
        let name = atl.items[0].value.as_ref().map(|cs| {
            cs.data.to_string()
        }).unwrap_or_default();
        let sz = post_blob.len();
        let e = by_name.entry(name).or_insert((sz, sz));
        e.0 = e.0.min(sz);
        e.1 = e.1.max(sz);
    }
    for (name, (min, max)) in &by_name {
        let cnt = atl1_name_hist.get(name).copied().unwrap_or(0);
        println!("  {:4}x  {:?}  blob_len={}..{}", cnt, name, min, max);
    }
}
