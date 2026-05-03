//! Computes the byte offset of each GimmickPostBody field for a known-good entry,
//! then shows the bytes at probe ~1185 for a failing entry.

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

    // Find a good entry (count=0, post_body decoded OK, post_blob empty)
    // and a failing entry (count=0, post_body None, post_blob ~1451 bytes)
    let mut good_blob: Option<Vec<u8>> = None;
    let mut fail_blobs: Vec<Vec<u8>> = Vec::new();

    for (_, start, end) in &ranges {
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it, Err(_) => continue,
        };
        let GimmickTail::Decoded { alt_trigger_list, post_body, post_blob, .. } = &item.tail else { continue };
        let Some(atl) = alt_trigger_list else { continue };
        if !atl.items.is_empty() { continue; }

        if post_body.is_some() && post_blob.is_empty() && good_blob.is_none() {
            // Reconstruct what GimmickPostBody consumed by re-reading from entry data
            // We need the raw bytes at the post_body position. Since post_blob is empty,
            // all bytes after alt_trigger_list were in post_body. We can't easily get
            // those raw bytes without modifying the library.
            //
            // Alternative: just show that success happened.
            println!("Found good entry, post_body decoded, post_blob empty");
        }

        if post_body.is_none() && !post_blob.is_empty() {
            let sz = post_blob.len();
            if (1400..=1500).contains(&sz) && fail_blobs.len() < 3 {
                fail_blobs.push(post_blob.clone());
            }
            if good_blob.is_none() {
                // Can't easily get the raw bytes for a good entry from here
            }
        }
    }

    // For a failing blob of ~1451 bytes, show bytes at 1050-1220 range
    println!("\nFailing blobs ~1451 bytes — bytes at probe 1050-1220:");
    for (idx, blob) in fail_blobs.iter().enumerate() {
        println!("  blob #{idx} len={}", blob.len());
        let lo = 1040usize;
        let hi = blob.len().min(1220);
        let mut cur = lo;
        while cur < hi {
            let end = (cur + 16).min(hi);
            println!("    [{cur}..{end}]: {}", hex16(&blob[cur..end]));
            cur += 16;
        }
        println!();
    }

    // Also try to get a good blob to compare at the same offsets.
    // We need to capture the raw post_body bytes from a good entry.
    // Strategy: find a good entry and manually re-read from its raw data.
    // We need to track the offset where alt_trigger_list ends for a good entry.
    println!("=== Raw bytes from good entries at offsets 1050-1220 ===");
    let mut good_count = 0;
    for (_, start, end) in &ranges {
        if good_count >= 3 { break; }
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it, Err(_) => continue,
        };
        let GimmickTail::Decoded { alt_trigger_list, post_body, post_blob, .. } = &item.tail else { continue };
        let Some(atl) = alt_trigger_list else { continue };
        if !atl.items.is_empty() || post_body.is_none() || !post_blob.is_empty() { continue; }

        // Good entry: try to get the bytes that were decoded into post_body
        // We can re-read to find where alt_trigger_list ends, then read the
        // remaining bytes directly from the raw `data` slice.
        // The entry ends at *end, and cur = *end after GimmickInfo::read_with_size.
        // But we need to find where in `data` the alt_trigger_list ends.
        // Since post_blob is empty and post_body is Some, all bytes after atl were decoded.
        // The entry's raw data slice is data[*start..*end].
        let entry_data = &data[*start..*end];
        let total_entry = entry_data.len();
        // re-parse to find the offset where post_body bytes start within the entry
        // (we know that the entry ends at *end and post_blob is empty,
        //  meaning GimmickPostBody consumed all remaining bytes in the entry)
        // The entry includes the fixed GimmickInfo prefix (fields 1-6) before the tail.
        // Fields 1-6 consume: u32(key) + CString(string_key) + u8(is_blocked) + CString(prefab_path) + u32(gimmick_group_info) + u16(breakable_object_info)
        // These are variable-size. We can get the raw entry bytes at offset ~1050 from
        // *start into the data, but the "1050" would be relative to the post_body start,
        // not the entry start.

        // For now, just show that we found good entries and their total size
        let _ = (atl, post_body);
        println!("  Good entry: total_entry={} bytes", total_entry);
        good_count += 1;
    }

    // The most direct approach: manually build a minimal GimmickPostBody blob
    // and measure which field offsets correspond to byte positions.
    // We do this by parsing an all-zeros blob and tracking offsets.
    println!("\n=== Field offset estimation from all-zeros 2000-byte blob ===");
    let zeros = vec![0u8; 2000];
    let mut off = 0;
    match GimmickPostBody::read_from(&zeros, &mut off) {
        Ok(_) => println!("Parsed OK! Consumed {} bytes from zeros", off),
        Err(e) => println!("Error at offset {}: {}", off, e),
    }
}
