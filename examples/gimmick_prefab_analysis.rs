//! Show prefab_path and gimmick_group_info for failing vs succeeding entries.
//! The prefab_path is likely the variant discriminator.

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::tables::gimmick_info::{GimmickInfo, GimmickTail};

const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgb";
const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgh";

fn main() {
    let data = match std::fs::read(PABGB) {
        Ok(d) => d, Err(e) => { eprintln!("SKIP: {}", e); return; }
    };
    let entries = match load_pabgh_offsets(PABGH) {
        Some(e) => e, None => { eprintln!("SKIP pabgh"); return; }
    };
    let ranges = entry_ranges(&entries, data.len());

    // Histogram: prefab_path → (ok_count, fail_count)
    let mut prefab_hist: std::collections::BTreeMap<String, (usize, usize)> = std::collections::BTreeMap::new();
    // Histogram: gimmick_group_info → (ok_count, fail_count)
    let mut group_hist: std::collections::BTreeMap<u32, (usize, usize)> = std::collections::BTreeMap::new();
    // Fail breakdown by (atl_count, prefab_path_prefix)
    let mut fail_by_atl: std::collections::BTreeMap<(usize, String), usize> = std::collections::BTreeMap::new();

    for (_, start, end) in &ranges {
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it, Err(_) => continue,
        };

        let prefab = item.prefab_path.data.to_string();
        let group = item.gimmick_group_info;

        let (ok, fail, atl_count) = match &item.tail {
            GimmickTail::Decoded { alt_trigger_list, post_body, post_blob, .. } => {
                let atl_cnt = alt_trigger_list.as_ref().map(|a| a.items.len()).unwrap_or(0);
                let ok = post_body.is_some();
                let fail = !ok && !post_blob.is_empty();
                (ok, fail, atl_cnt)
            }
            GimmickTail::Raw(_) => (false, false, 0),
        };

        let e = prefab_hist.entry(prefab.clone()).or_insert((0, 0));
        if ok { e.0 += 1; } else if fail { e.1 += 1; }

        let ge = group_hist.entry(group).or_insert((0, 0));
        if ok { ge.0 += 1; } else if fail { ge.1 += 1; }

        if fail {
            // Truncate prefab to first path component or last 30 chars
            let short_prefab = if prefab.len() > 50 {
                format!("...{}", &prefab[prefab.len()-40..])
            } else { prefab.clone() };
            *fail_by_atl.entry((atl_count, short_prefab)).or_insert(0) += 1;
        }
    }

    // Show top failing prefab paths
    println!("=== Prefab paths: (ok_count, fail_count) sorted by fail_count ===");
    let mut sorted: Vec<_> = prefab_hist.iter().collect();
    sorted.sort_by_key(|(_, (ok, fail))| (*fail, *ok));
    sorted.reverse();
    for (prefab, (ok, fail)) in sorted.iter().take(30) {
        if *fail > 0 {
            println!("  ok={:5}  fail={:5}  {:?}", ok, fail, prefab);
        }
    }

    println!("\n=== Prefab paths that ONLY fail (ok=0) ===");
    for (prefab, (ok, fail)) in &prefab_hist {
        if *ok == 0 && *fail > 0 {
            println!("  fail={:5}  {:?}", fail, prefab);
        }
    }

    println!("\n=== Prefab paths that NEVER fail (fail=0, ok>0) ===");
    let mut only_ok: Vec<_> = prefab_hist.iter().filter(|(_, (_, f))| *f == 0).collect();
    only_ok.sort_by_key(|(_, (ok, _))| *ok);
    only_ok.reverse();
    for (prefab, (ok, _)) in only_ok.iter().take(10) {
        println!("  ok={:5}  {:?}", ok, prefab);
    }

    println!("\n=== Fail breakdown by (atl_count, prefab_suffix) ===");
    let mut fail_sorted: Vec<_> = fail_by_atl.iter().collect();
    fail_sorted.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for ((atl, prefab), cnt) in fail_sorted.iter().take(30) {
        println!("  {:4}x  atl={} prefab={:?}", cnt, atl, prefab);
    }

    println!("\n=== gimmick_group_info: top 10 with failures ===");
    let mut grp_sorted: Vec<_> = group_hist.iter().filter(|(_, (_, f))| *f > 0).collect();
    grp_sorted.sort_by_key(|(_, (_, f))| std::cmp::Reverse(*f));
    for (grp, (ok, fail)) in grp_sorted.iter().take(10) {
        println!("  group=0x{:08x}  ok={:5}  fail={:5}", grp, ok, fail);
    }
}
