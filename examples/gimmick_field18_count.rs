//! Count how many gimmick_info entries successfully decoded field 18
//! (gimmick_chart_parameter_list) and the size distribution of post_blob.

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::tables::gimmick_info::info::{GimmickInfo, GimmickTail};

const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gimmickinfo.pabgb";
const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gimmickinfo.pabgh";

fn main() {
    let data = std::fs::read(PABGB).expect("read");
    let entries = load_pabgh_offsets(PABGH).expect("pabgh");
    let ranges = entry_ranges(&entries, data.len());

    let mut decoded = 0usize;
    let mut raw = 0usize;
    let mut tgpehd_typed = 0usize;
    let mut chart_param_typed = 0usize;
    let mut post_blob_sizes: Vec<usize> = vec![];

    for (_key, start, end) in &ranges {
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it,
            Err(_) => continue,
        };
        match &item.tail {
            GimmickTail::Decoded {
                trigger_event_handler_list,
                gimmick_chart_parameter_list,
                post_blob, ..
            } => {
                decoded += 1;
                if trigger_event_handler_list.is_some() { tgpehd_typed += 1; }
                if gimmick_chart_parameter_list.is_some() { chart_param_typed += 1; }
                post_blob_sizes.push(post_blob.len());
            }
            GimmickTail::Raw(_) => { raw += 1; }
        }
    }

    println!("Total entries: {}", ranges.len());
    println!("Decoded:       {}", decoded);
    println!("Raw:           {}", raw);
    println!("TGPEHD typed:  {} / {}", tgpehd_typed, decoded);
    println!("Field 18 (gimmick_chart_parameter_list) typed: {} / {}", chart_param_typed, decoded);

    post_blob_sizes.sort();
    if !post_blob_sizes.is_empty() {
        let n = post_blob_sizes.len();
        println!("\npost_blob size distribution:");
        println!("  min={}", post_blob_sizes[0]);
        println!("  p25={}", post_blob_sizes[n/4]);
        println!("  p50={}", post_blob_sizes[n/2]);
        println!("  p75={}", post_blob_sizes[3*n/4]);
        println!("  max={}", post_blob_sizes[n-1]);
        println!("  avg={}", post_blob_sizes.iter().sum::<usize>() / n);
        println!("  total bytes: {}", post_blob_sizes.iter().sum::<usize>());
    }
}
