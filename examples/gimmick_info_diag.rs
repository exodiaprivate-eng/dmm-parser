//! Diagnostic: identify field boundaries in gimmick_info post_blob (fields 19+).
//!
//! Strategy: compare all entries against the minimum-size post_blob to find the
//! first byte offset where each entry diverges. This reveals field boundaries.

use dmm_parser::binary::*;
use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::tables::gimmick_info::{GimmickInfo, GimmickTail};

const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-03-29/gimmickinfo.pabgb";
const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-03-29/gimmickinfo.pabgh";

fn hexline(bytes: &[u8]) -> String {
    let hex: Vec<_> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let asc: String = bytes.iter().map(|&b| if b.is_ascii_graphic() { b as char } else { '.' }).collect();
    format!("{} |{}|", hex.join(" "), asc)
}

fn main() {
    let data = match std::fs::read(PABGB) { Ok(d) => d, Err(e) => { eprintln!("SKIP: {}", e); return; } };
    let entries = match load_pabgh_offsets(PABGH) { Some(e) => e, None => { eprintln!("SKIP pabgh"); return; } };
    let ranges = entry_ranges(&entries, data.len());
    println!("gimmickinfo: {} entries", ranges.len());

    // Collect all post_blobs from Decoded entries
    let mut blobs: Vec<(u32, Vec<u8>)> = Vec::new();
    for (_i, (key, start, end)) in ranges.iter().enumerate() {
        let mut c = *start;
        if let Ok(item) = GimmickInfo::read_with_size(&data, &mut c, end - start) {
            if let GimmickTail::Decoded { post_blob, .. } = item.tail {
                blobs.push((*key, post_blob));
            }
        }
    }
    println!("Decoded entries with post_blob: {}", blobs.len());

    let min_sz = blobs.iter().map(|(_, b)| b.len()).min().unwrap_or(0);
    println!("Minimum post_blob size: {}", min_sz);

    // Find the reference (minimum-size) blob
    let ref_blob = blobs.iter().find(|(_, b)| b.len() == min_sz).map(|(_, b)| b.clone()).unwrap();

    // For each byte position in [0..min_sz], count how many entries differ from reference
    println!("\nByte-position divergence scan (positions where any entry differs from reference):");
    let mut first_diffs: Vec<usize> = Vec::new(); // first-divergence position for each entry
    let mut pos_diverge_count = vec![0usize; min_sz];
    for (_, blob) in &blobs {
        let mut first_diff = min_sz;
        for i in 0..min_sz.min(blob.len()) {
            if blob[i] != ref_blob[i] {
                pos_diverge_count[i] += 1;
                if first_diff == min_sz { first_diff = i; }
            }
        }
        first_diffs.push(first_diff);
    }

    // Show positions with divergences (and their counts)
    let mut shown = 0;
    let mut last_pos = 0;
    for pos in 0..min_sz {
        if pos_diverge_count[pos] > 0 {
            if pos != last_pos + 1 || shown == 0 {
                print!("\n  pos {:3}: ", pos);
                shown += 1;
            }
            print!("  [{pos}]: {}/{}", pos_diverge_count[pos], blobs.len());
            last_pos = pos;
            if shown > 50 { println!("\n  ... (truncated)"); break; }
        }
    }
    println!();

    // Histogram of first-divergence positions (where do entries first differ from ref?)
    let mut first_div_hist: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    for &pos in &first_diffs {
        *first_div_hist.entry(pos).or_insert(0) += 1;
    }
    println!("\nFirst-divergence histogram (position → count of entries that first differ here):");
    for (pos, count) in first_div_hist.iter().take(30) {
        println!("  pos {:4}: {} entries", pos, count);
    }

    // Deep-dive: show all entries that diverge at the earliest non-reference position
    let earliest_div = first_diffs.iter().filter(|&&p| p < min_sz).min().cloned().unwrap_or(min_sz);
    if earliest_div < min_sz {
        println!("\nEarliest divergence at pos {earliest_div}. Ref bytes around there:");
        let lo = earliest_div.saturating_sub(4);
        let hi = (earliest_div + 32).min(min_sz);
        println!("  ref[{}..{}]: {}", lo, hi, hexline(&ref_blob[lo..hi]));

        println!("  Entries diverging at pos {earliest_div} (first 5):");
        let mut shown_early = 0;
        for (key, blob) in &blobs {
            if blob.len() > earliest_div && blob[earliest_div] != ref_blob[earliest_div] {
                if shown_early >= 5 { break; }
                shown_early += 1;
                let lo2 = earliest_div.saturating_sub(4);
                let hi2 = (earliest_div + 32).min(blob.len());
                println!("  key=0x{:08x}: {}", key, hexline(&blob[lo2..hi2]));
            }
        }
    }

    // Show reference blob in chunks for manual analysis
    println!("\nReference blob ({min_sz} bytes) in 16-byte lines:");
    for chunk_start in (0..min_sz.min(256)).step_by(16) {
        let end = (chunk_start + 16).min(min_sz);
        println!("  [{:4}..{:4}]: {}", chunk_start, end, hexline(&ref_blob[chunk_start..end]));
    }

    // ── Tag-16 body analysis ─────────────────────────────────────────────────
    // Identify entries whose post_blob starts with the TGPEHD tag=16 prefix:
    //   01 00 00 00  = CArray count=1
    //   01           = presence=1
    //   10           = dispatch_tag=16
    let tag16_prefix: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x01, 0x10];
    let tag16_bodies: Vec<(u32, &[u8])> = blobs.iter()
        .filter(|(_, b)| b.len() >= 6 && b[..6] == *tag16_prefix)
        .map(|(k, b)| (*k, &b[6..]))
        .collect();
    println!("\n── Tag-16 body analysis ({} entries) ──", tag16_bodies.len());

    // Find minimum body size
    let min16 = tag16_bodies.iter().map(|(_, b)| b.len()).min().unwrap_or(0);
    let max16 = tag16_bodies.iter().map(|(_, b)| b.len()).max().unwrap_or(0);
    println!("  body len range: {}..{}", min16, max16);

    // Show first 10 entries' bodies (80 bytes each)
    println!("\n  First 10 tag-16 bodies (96 bytes each):");
    for (i, (key, body)) in tag16_bodies.iter().take(10).enumerate() {
        println!("  [{i}] key=0x{key:08x} ({} bytes):", body.len());
        for chunk_start in (0..body.len().min(96)).step_by(16) {
            let end = (chunk_start + 16).min(body.len());
            println!("       [{:3}..{:3}]: {}", chunk_start, end, hexline(&body[chunk_start..end]));
        }
    }

    // Count-of-zero histogram: for each byte position [0..min16], how many
    // tag-16 bodies have that byte == 0x00?
    if min16 >= 32 {
        println!("\n  Zero-byte histogram (positions 0..32, tag-16 bodies only):");
        for pos in 0..32usize.min(min16) {
            let zero_cnt = tag16_bodies.iter().filter(|(_, b)| b[pos] == 0x00).count();
            let nonzero_cnt = tag16_bodies.len() - zero_cnt;
            // Show only if the byte varies (both zeros and non-zeros exist)
            if zero_cnt > 0 && nonzero_cnt > 0 {
                let sample_nonzero: Vec<_> = tag16_bodies.iter()
                    .filter(|(_, b)| b[pos] != 0x00)
                    .take(5)
                    .map(|(_, b)| format!("{:02x}", b[pos]))
                    .collect();
                println!("    pos {:3}: {} zeros, {} non-zero (samples: {})",
                    pos, zero_cnt, nonzero_cnt, sample_nonzero.join(" "));
            } else if nonzero_cnt > 0 {
                // All non-zero
                let uniq: std::collections::BTreeSet<u8> = tag16_bodies.iter().map(|(_, b)| b[pos]).collect();
                let uniq_s: Vec<_> = uniq.iter().take(8).map(|b| format!("{:02x}", b)).collect();
                println!("    pos {:3}: ALL non-zero (unique vals: {}{})",
                    pos, uniq_s.join(" "), if uniq.len() > 8 { "..." } else { "" });
            }
        }
    }

    // Try interpreting first field as CString: CString = u32 len (LE) + [u8; len] (no null).
    println!("\n  CString-probe (try reading CString at body offsets 0-8, NO null required):");
    for start_off in 0usize..=8 {
        let mut ok = 0usize;
        let mut total = 0usize;
        let mut sample_strings: Vec<String> = Vec::new();
        for (_, body) in tag16_bodies.iter().take(200) {
            if body.len() < start_off + 5 { continue; }
            total += 1;
            let len_bytes = [body[start_off], body[start_off+1], body[start_off+2], body[start_off+3]];
            let slen = u32::from_le_bytes(len_bytes) as usize;
            let needed = start_off + 4 + slen;
            if slen >= 1 && slen <= 128 && needed <= body.len() {
                let s = &body[start_off+4..start_off+4+slen];
                if s.iter().all(|&c| c.is_ascii() && (c.is_ascii_graphic() || c == b' ')) {
                    ok += 1;
                    if sample_strings.len() < 3 {
                        sample_strings.push(String::from_utf8_lossy(s).into_owned());
                    }
                }
            }
        }
        if total > 0 {
            println!("    offset {:2}: {}/{} plausible CStrings  samples: {:?}", start_off, ok, total, sample_strings);
        }
    }

    // Show diverse tag-16 entries (deduplicated by first-16-body-bytes)
    println!("\n  Diverse tag-16 entries (skip duplicates by first 16 body bytes):");
    let mut seen_prefixes: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
    let mut diverse_count = 0;
    for (key, body) in &tag16_bodies {
        if diverse_count >= 15 { break; }
        let prefix: Vec<u8> = body.iter().take(16).cloned().collect();
        if seen_prefixes.insert(prefix) {
            diverse_count += 1;
            println!("  [D{diverse_count}] key=0x{key:08x} ({} bytes):", body.len());
            for chunk_start in (0..body.len().min(80)).step_by(16) {
                let end = (chunk_start + 16).min(body.len());
                println!("       [{:3}..{:3}]: {}", chunk_start, end, hexline(&body[chunk_start..end]));
            }
        }
    }

    // Show minimum-size tag-16 entries completely (to find boundary of body vs post-TGPEHD fields)
    let actual_min16 = tag16_bodies.iter().map(|(_, b)| b.len()).min().unwrap_or(0);
    println!("\n  Minimum-size tag-16 bodies (len={}), showing first 3 completely:", actual_min16);
    let mut shown_min = 0;
    for (key, body) in &tag16_bodies {
        if body.len() != actual_min16 { continue; }
        if shown_min >= 3 { break; }
        shown_min += 1;
        println!("  [M{shown_min}] key=0x{key:08x} ({} bytes):", body.len());
        for chunk_start in (0..body.len()).step_by(16) {
            let end = (chunk_start + 16).min(body.len());
            println!("       [{:4}..{:4}]: {}", chunk_start, end, hexline(&body[chunk_start..end]));
        }
    }

    // Size distribution of tag-16 bodies
    println!("\n  Tag-16 body size distribution (bottom 10):");
    let mut sizes: Vec<usize> = tag16_bodies.iter().map(|(_, b)| b.len()).collect();
    sizes.sort();
    sizes.dedup();
    for &sz in sizes.iter().take(10) {
        let cnt = tag16_bodies.iter().filter(|(_, b)| b.len() == sz).count();
        println!("    len {:6}: {} entries", sz, cnt);
    }
}
