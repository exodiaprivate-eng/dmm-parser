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
    let mut field_19_typed = 0usize;
    let mut field_20_typed = 0usize;
    let mut field_21_typed = 0usize;
    let mut field_22_typed = 0usize;
    let mut field_23_typed = 0usize;
    let mut field_24_typed = 0usize;
    let mut field_25_typed = 0usize;
    let mut field_26_typed = 0usize;
    let mut field_27_typed = 0usize;
    let mut field_28_typed = 0usize;
    let mut field_29_typed = 0usize;
    let mut field_30_typed = 0usize;
    let mut field_31_typed = 0usize;
    let mut field_32_typed = 0usize;
    let mut field_33_typed = 0usize;
    let mut field_34_typed = 0usize;
    let mut field_35_typed = 0usize;
    let mut field_36_typed = 0usize;
    let mut field_37_typed = 0usize;
    let mut field_38_typed = 0usize;
    let mut field_39_typed = 0usize;
    let mut field_40_typed = 0usize;
    let mut field_41_typed = 0usize;
    let mut field_42_typed = 0usize;
    let mut field_43_typed = 0usize;
    let mut field_44_typed = 0usize;
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
                field_19_u32_list,
                field_20_u32_list,
                field_21_u32_list,
                field_22_u32_list,
                field_23_u32_list,
                field_24_u32_list,
                field_25_u32_list,
                field_26_u32,
                field_27_u32_list,
                field_28_u32,
                field_29_u32_list,
                field_30_u32_list,
                field_31_u32_list,
                field_32_u32_list,
                field_33_u32,
                field_34_u32,
                field_35_u32_list,
                field_36_u32,
                field_37_u32,
                field_38_u32,
                field_39_u32_list,
                field_40_u32_list,
                field_41_u32,
                field_42_u32,
                field_43_u32,
                field_44_u32,
                post_blob, ..
            } => {
                decoded += 1;
                if trigger_event_handler_list.is_some() { tgpehd_typed += 1; }
                if gimmick_chart_parameter_list.is_some() { chart_param_typed += 1; }
                if field_19_u32_list.is_some() { field_19_typed += 1; }
                if field_20_u32_list.is_some() { field_20_typed += 1; }
                if field_21_u32_list.is_some() { field_21_typed += 1; }
                if field_22_u32_list.is_some() { field_22_typed += 1; }
                if field_23_u32_list.is_some() { field_23_typed += 1; }
                if field_24_u32_list.is_some() { field_24_typed += 1; }
                if field_25_u32_list.is_some() { field_25_typed += 1; }
                if field_26_u32.is_some() { field_26_typed += 1; }
                if field_27_u32_list.is_some() { field_27_typed += 1; }
                if field_28_u32.is_some() { field_28_typed += 1; }
                if field_29_u32_list.is_some() { field_29_typed += 1; }
                if field_30_u32_list.is_some() { field_30_typed += 1; }
                if field_31_u32_list.is_some() { field_31_typed += 1; }
                if field_32_u32_list.is_some() { field_32_typed += 1; }
                if field_33_u32.is_some() { field_33_typed += 1; }
                if field_34_u32.is_some() { field_34_typed += 1; }
                if field_35_u32_list.is_some() { field_35_typed += 1; }
                if field_36_u32.is_some() { field_36_typed += 1; }
                if field_37_u32.is_some() { field_37_typed += 1; }
                if field_38_u32.is_some() { field_38_typed += 1; }
                if field_39_u32_list.is_some() { field_39_typed += 1; }
                if field_40_u32_list.is_some() { field_40_typed += 1; }
                if field_41_u32.is_some() { field_41_typed += 1; }
                if field_42_u32.is_some() { field_42_typed += 1; }
                if field_43_u32.is_some() { field_43_typed += 1; }
                if field_44_u32.is_some() { field_44_typed += 1; }
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
    println!("Field 19 (field_19_u32_list) typed:            {} / {}", field_19_typed, decoded);
    println!("Field 20 (field_20_u32_list) typed:            {} / {}", field_20_typed, decoded);
    println!("Field 21 (field_21_u32_list) typed:            {} / {}", field_21_typed, decoded);
    println!("Field 22 (field_22_u32_list) typed:            {} / {}", field_22_typed, decoded);
    println!("Field 23 (field_23_u32_list) typed:            {} / {}", field_23_typed, decoded);
    println!("Field 24 (field_24_u32_list) typed:            {} / {}", field_24_typed, decoded);
    println!("Field 25 (field_25_u32_list) typed:            {} / {}", field_25_typed, decoded);
    println!("Field 26 (field_26_u32 single u32)  typed:     {} / {}", field_26_typed, decoded);
    println!("Field 27 (field_27_u32_list)         typed:     {} / {}", field_27_typed, decoded);
    println!("Field 28 (field_28_u32 single u32)   typed:     {} / {}", field_28_typed, decoded);
    println!("Field 29 (field_29_u32_list)         typed:     {} / {}", field_29_typed, decoded);
    println!("Field 30 (field_30_u32_list)         typed:     {} / {}", field_30_typed, decoded);
    println!("Field 31 (field_31_u32_list)         typed:     {} / {}", field_31_typed, decoded);
    println!("Field 32 (field_32_u32_list)         typed:     {} / {}", field_32_typed, decoded);
    println!("Field 33 (field_33_u32 single u32)   typed:     {} / {}", field_33_typed, decoded);
    println!("Field 34 (field_34_u32 single u32)   typed:     {} / {}", field_34_typed, decoded);
    println!("Field 35 (field_35_u32_list)         typed:     {} / {}", field_35_typed, decoded);
    println!("Field 36 (field_36_u32 single u32)   typed:     {} / {}", field_36_typed, decoded);
    println!("Field 37 (field_37_u32 single u32)   typed:     {} / {}", field_37_typed, decoded);
    println!("Field 38 (field_38_u32 single u32)   typed:     {} / {}", field_38_typed, decoded);
    println!("Field 39 (field_39_u32_list)         typed:     {} / {}", field_39_typed, decoded);
    println!("Field 40 (field_40_u32_list)         typed:     {} / {}", field_40_typed, decoded);
    println!("Field 41 (field_41_u32 single u32)   typed:     {} / {}", field_41_typed, decoded);
    println!("Field 42 (field_42_u32 single u32)   typed:     {} / {}", field_42_typed, decoded);
    println!("Field 43 (field_43_u32 single u32)   typed:     {} / {}", field_43_typed, decoded);
    println!("Field 44 (field_44_u32 single u32)   typed:     {} / {}", field_44_typed, decoded);

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
