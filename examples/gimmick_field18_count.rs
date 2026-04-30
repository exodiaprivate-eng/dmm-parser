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
    let mut field_45_typed = 0usize;
    let mut field_46_typed = 0usize;
    let mut field_47_typed = 0usize;
    let mut field_48_typed = 0usize;
    let mut field_49_typed = 0usize;
    let mut field_50_typed = 0usize;
    let mut field_51_typed = 0usize;
    let mut field_52_typed = 0usize;
    let mut field_53_typed = 0usize;
    let mut field_54_typed = 0usize;
    let mut field_55_typed = 0usize;
    let mut field_56_typed = 0usize;
    let mut field_57_typed = 0usize;
    let mut field_58_typed = 0usize;
    let mut field_59_typed = 0usize;
    let mut field_60_typed = 0usize;
    let mut field_61_typed = 0usize;
    let mut field_62_typed = 0usize;
    let mut field_63_typed = 0usize;
    let mut field_64_typed = 0usize;
    let mut field_65_typed = 0usize;
    let mut field_66_typed = 0usize;
    let mut field_67_typed = 0usize;
    let mut field_68_typed = 0usize;
    let mut field_69_typed = 0usize;
    let mut field_70_typed = 0usize;
    let mut field_71_typed = 0usize;
    let mut field_72_typed = 0usize;
    let mut field_73_typed = 0usize;
    let mut field_74_typed = 0usize;
    let mut field_75_typed = 0usize;
    let mut field_76_typed = 0usize;
    let mut field_77_typed = 0usize;
    let mut field_78_typed = 0usize;
    let mut field_79_typed = 0usize;
    let mut field_80_typed = 0usize;
    let mut field_81_typed = 0usize;
    let mut field_82_typed = 0usize;
    let mut field_83_typed = 0usize;
    let mut field_84_typed = 0usize;
    let mut field_85_typed = 0usize;
    let mut field_86_typed = 0usize;
    let mut field_87_typed = 0usize;
    let mut field_88_typed = 0usize;
    let mut field_89_typed = 0usize;
    let mut field_90_typed = 0usize;
    let mut field_91_typed = 0usize;
    let mut field_92_typed = 0usize;
    let mut field_93_typed = 0usize;
    let mut field_94_typed = 0usize;
    let mut field_95_typed = 0usize;
    let mut field_96_typed = 0usize;
    let mut field_97_typed = 0usize;
    let mut field_98_typed = 0usize;
    let mut field_99_typed = 0usize;
    let mut field_100_typed = 0usize;
    let mut field_101_typed = 0usize;
    let mut field_102_typed = 0usize;
    let mut field_103_typed = 0usize;
    let mut field_104_typed = 0usize;
    let mut field_105_typed = 0usize;
    let mut field_106_typed = 0usize;
    let mut field_107_typed = 0usize;
    let mut field_108_typed = 0usize;
    let mut field_109_typed = 0usize;
    let mut field_110_typed = 0usize;
    let mut field_111_typed = 0usize;
    let mut field_112_typed = 0usize;
    let mut field_113_typed = 0usize;
    let mut field_114_typed = 0usize;
    let mut field_115_typed = 0usize;
    let mut field_116_typed = 0usize;
    let mut field_117_typed = 0usize;
    let mut field_118_typed = 0usize;
    let mut field_119_typed = 0usize;
    let mut field_120_typed = 0usize;
    let mut field_121_typed = 0usize;
    let mut field_122_typed = 0usize;
    let mut field_123_typed = 0usize;
    let mut field_124_typed = 0usize;
    let mut field_125_typed = 0usize;
    let mut field_126_typed = 0usize;
    let mut field_127_typed = 0usize;
    let mut field_128_typed = 0usize;
    let mut field_129_typed = 0usize;
    let mut field_130_typed = 0usize;
    let mut field_131_typed = 0usize;
    let mut field_132_typed = 0usize;
    let mut field_133_typed = 0usize;
    let mut field_134_typed = 0usize;
    let mut field_135_typed = 0usize;
    let mut field_136_typed = 0usize;
    let mut field_137_typed = 0usize;
    let mut field_138_typed = 0usize;
    let mut field_139_typed = 0usize;
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
                field_45_u32,
                field_46_u32,
                field_47_u32,
                field_48_u32,
                field_49_u32_list,
                field_50_u32_list,
                field_51_u32_list,
                field_52_u32_list,
                field_53_u32_list,
                field_54_u32_list,
                field_55_u32_list,
                field_56_u32_list,
                field_57_u32_list,
                field_58_u32_list,
                field_59_u32,
                field_60_u32,
                field_61_u32,
                field_62_u32,
                field_63_u32,
                field_64_u32,
                field_65_u32,
                field_66_u32,
                field_67_u32,
                field_68_u32,
                field_69_u32,
                field_70_u32,
                field_71_u32,
                field_72_u32,
                field_73_u32,
                field_74_u32,
                field_75_u32,
                field_76_u32,
                field_77_u32,
                field_78_u32,
                field_79_u32,
                field_80_u32,
                field_81_u32,
                field_82_u32,
                field_83_u32,
                field_84_u32,
                field_85_u32,
                field_86_u32,
                field_87_u32,
                field_88_u32,
                field_89_u32,
                field_90_u32,
                field_91_u32,
                field_92_u32,
                field_93_u32,
                field_94_u32,
                field_95_u32,
                field_96_u32,
                field_97_u32,
                field_98_u32,
                field_99_u32,
                field_100_u32,
                field_101_u32,
                field_102_u32,
                field_103_u32,
                field_104_u32,
                field_105_u32,
                field_106_u32,
                field_107_u32,
                field_108_u32,
                field_109_u32,
                field_110_u32,
                field_111_u32,
                field_112_u32,
                field_113_u32,
                field_114_u32,
                field_115_u32,
                field_116_u32,
                field_117_u32,
                field_118_u32,
                field_119_u32,
                field_120_u32,
                field_121_u32,
                field_122_u32,
                field_123_u32,
                field_124_u32,
                field_125_u32,
                field_126_u32,
                field_127_u32,
                field_128_u32,
                field_129_u32,
                field_130_u32,
                field_131_u32,
                field_132_u32,
                field_133_u32,
                field_134_u32,
                field_135_u32,
                field_136_u32,
                field_137_u32,
                field_138_u32,
                field_139_u32,
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
                if field_45_u32.is_some() { field_45_typed += 1; }
                if field_46_u32.is_some() { field_46_typed += 1; }
                if field_47_u32.is_some() { field_47_typed += 1; }
                if field_48_u32.is_some() { field_48_typed += 1; }
                if field_49_u32_list.is_some() { field_49_typed += 1; }
                if field_50_u32_list.is_some() { field_50_typed += 1; }
                if field_51_u32_list.is_some() { field_51_typed += 1; }
                if field_52_u32_list.is_some() { field_52_typed += 1; }
                if field_53_u32_list.is_some() { field_53_typed += 1; }
                if field_54_u32_list.is_some() { field_54_typed += 1; }
                if field_55_u32_list.is_some() { field_55_typed += 1; }
                if field_56_u32_list.is_some() { field_56_typed += 1; }
                if field_57_u32_list.is_some() { field_57_typed += 1; }
                if field_58_u32_list.is_some() { field_58_typed += 1; }
                if field_59_u32.is_some() { field_59_typed += 1; }
                if field_60_u32.is_some() { field_60_typed += 1; }
                if field_61_u32.is_some() { field_61_typed += 1; }
                if field_62_u32.is_some() { field_62_typed += 1; }
                if field_63_u32.is_some() { field_63_typed += 1; }
                if field_64_u32.is_some() { field_64_typed += 1; }
                if field_65_u32.is_some() { field_65_typed += 1; }
                if field_66_u32.is_some() { field_66_typed += 1; }
                if field_67_u32.is_some() { field_67_typed += 1; }
                if field_68_u32.is_some() { field_68_typed += 1; }
                if field_69_u32.is_some() { field_69_typed += 1; }
                if field_70_u32.is_some() { field_70_typed += 1; }
                if field_71_u32.is_some() { field_71_typed += 1; }
                if field_72_u32.is_some() { field_72_typed += 1; }
                if field_73_u32.is_some() { field_73_typed += 1; }
                if field_74_u32.is_some() { field_74_typed += 1; }
                if field_75_u32.is_some() { field_75_typed += 1; }
                if field_76_u32.is_some() { field_76_typed += 1; }
                if field_77_u32.is_some() { field_77_typed += 1; }
                if field_78_u32.is_some() { field_78_typed += 1; }
                if field_79_u32.is_some() { field_79_typed += 1; }
                if field_80_u32.is_some() { field_80_typed += 1; }
                if field_81_u32.is_some() { field_81_typed += 1; }
                if field_82_u32.is_some() { field_82_typed += 1; }
                if field_83_u32.is_some() { field_83_typed += 1; }
                if field_84_u32.is_some() { field_84_typed += 1; }
                if field_85_u32.is_some() { field_85_typed += 1; }
                if field_86_u32.is_some() { field_86_typed += 1; }
                if field_87_u32.is_some() { field_87_typed += 1; }
                if field_88_u32.is_some() { field_88_typed += 1; }
                if field_89_u32.is_some() { field_89_typed += 1; }
                if field_90_u32.is_some() { field_90_typed += 1; }
                if field_91_u32.is_some() { field_91_typed += 1; }
                if field_92_u32.is_some() { field_92_typed += 1; }
                if field_93_u32.is_some() { field_93_typed += 1; }
                if field_94_u32.is_some() { field_94_typed += 1; }
                if field_95_u32.is_some() { field_95_typed += 1; }
                if field_96_u32.is_some() { field_96_typed += 1; }
                if field_97_u32.is_some() { field_97_typed += 1; }
                if field_98_u32.is_some() { field_98_typed += 1; }
                if field_99_u32.is_some() { field_99_typed += 1; }
                if field_100_u32.is_some() { field_100_typed += 1; }
                if field_101_u32.is_some() { field_101_typed += 1; }
                if field_102_u32.is_some() { field_102_typed += 1; }
                if field_103_u32.is_some() { field_103_typed += 1; }
                if field_104_u32.is_some() { field_104_typed += 1; }
                if field_105_u32.is_some() { field_105_typed += 1; }
                if field_106_u32.is_some() { field_106_typed += 1; }
                if field_107_u32.is_some() { field_107_typed += 1; }
                if field_108_u32.is_some() { field_108_typed += 1; }
                if field_109_u32.is_some() { field_109_typed += 1; }
                if field_110_u32.is_some() { field_110_typed += 1; }
                if field_111_u32.is_some() { field_111_typed += 1; }
                if field_112_u32.is_some() { field_112_typed += 1; }
                if field_113_u32.is_some() { field_113_typed += 1; }
                if field_114_u32.is_some() { field_114_typed += 1; }
                if field_115_u32.is_some() { field_115_typed += 1; }
                if field_116_u32.is_some() { field_116_typed += 1; }
                if field_117_u32.is_some() { field_117_typed += 1; }
                if field_118_u32.is_some() { field_118_typed += 1; }
                if field_119_u32.is_some() { field_119_typed += 1; }
                if field_120_u32.is_some() { field_120_typed += 1; }
                if field_121_u32.is_some() { field_121_typed += 1; }
                if field_122_u32.is_some() { field_122_typed += 1; }
                if field_123_u32.is_some() { field_123_typed += 1; }
                if field_124_u32.is_some() { field_124_typed += 1; }
                if field_125_u32.is_some() { field_125_typed += 1; }
                if field_126_u32.is_some() { field_126_typed += 1; }
                if field_127_u32.is_some() { field_127_typed += 1; }
                if field_128_u32.is_some() { field_128_typed += 1; }
                if field_129_u32.is_some() { field_129_typed += 1; }
                if field_130_u32.is_some() { field_130_typed += 1; }
                if field_131_u32.is_some() { field_131_typed += 1; }
                if field_132_u32.is_some() { field_132_typed += 1; }
                if field_133_u32.is_some() { field_133_typed += 1; }
                if field_134_u32.is_some() { field_134_typed += 1; }
                if field_135_u32.is_some() { field_135_typed += 1; }
                if field_136_u32.is_some() { field_136_typed += 1; }
                if field_137_u32.is_some() { field_137_typed += 1; }
                if field_138_u32.is_some() { field_138_typed += 1; }
                if field_139_u32.is_some() { field_139_typed += 1; }
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
    println!("Field 45 (field_45_u32 single u32)   typed:     {} / {}", field_45_typed, decoded);
    println!("Field 46 (field_46_u32 single u32)   typed:     {} / {}", field_46_typed, decoded);
    println!("Field 47 (field_47_u32 single u32)   typed:     {} / {}", field_47_typed, decoded);
    println!("Field 48 (field_48_u32 single u32)   typed:     {} / {}", field_48_typed, decoded);
    println!("Field 49 (field_49_u32_list)          typed:     {} / {}", field_49_typed, decoded);
    println!("Field 50 (field_50_u32_list)          typed:     {} / {}", field_50_typed, decoded);
    println!("Field 51 (field_51_u32_list)          typed:     {} / {}", field_51_typed, decoded);
    println!("Field 52 (field_52_u32_list)          typed:     {} / {}", field_52_typed, decoded);
    println!("Field 53 (field_53_u32_list)          typed:     {} / {}", field_53_typed, decoded);
    println!("Field 54 (field_54_u32_list)          typed:     {} / {}", field_54_typed, decoded);
    println!("Field 55 (field_55_u32_list)          typed:     {} / {}", field_55_typed, decoded);
    println!("Field 56 (field_56_u32_list)          typed:     {} / {}", field_56_typed, decoded);
    println!("Field 57 (field_57_u32_list)          typed:     {} / {}", field_57_typed, decoded);
    println!("Field 58 (field_58_u32_list)          typed:     {} / {}", field_58_typed, decoded);
    println!("Field 59 (field_59_u32 = f32 0.5)     typed:     {} / {}", field_59_typed, decoded);
    println!("Field 60 (field_60_u32 = f32 0.1)     typed:     {} / {}", field_60_typed, decoded);
    println!("Field 61 (field_61_u32 = f32 0.1)     typed:     {} / {}", field_61_typed, decoded);
    println!("Field 62 (field_62_u32 = 257 flag)    typed:     {} / {}", field_62_typed, decoded);
    println!("Field 63 (field_63_u32 = 0)           typed:     {} / {}", field_63_typed, decoded);
    println!("Field 64 (field_64_u32 = 0)           typed:     {} / {}", field_64_typed, decoded);
    println!("Field 65 (field_65_u32 = f32 5.0)     typed:     {} / {}", field_65_typed, decoded);
    println!("Field 66 (field_66_u32 = f32 1.0)     typed:     {} / {}", field_66_typed, decoded);
    for (i, count) in [(67usize, field_67_typed), (68, field_68_typed), (69, field_69_typed),
                       (70, field_70_typed), (71, field_71_typed), (72, field_72_typed),
                       (73, field_73_typed), (74, field_74_typed),
                       (75, field_75_typed), (76, field_76_typed), (77, field_77_typed),
                       (78, field_78_typed), (79, field_79_typed), (80, field_80_typed),
                       (81, field_81_typed), (82, field_82_typed),
                       (83, field_83_typed), (84, field_84_typed), (85, field_85_typed),
                       (86, field_86_typed), (87, field_87_typed), (88, field_88_typed),
                       (89, field_89_typed), (90, field_90_typed),
                       (91, field_91_typed), (92, field_92_typed),
                       (93, field_93_typed), (94, field_94_typed),
                       (95, field_95_typed), (96, field_96_typed),
                       (97, field_97_typed), (98, field_98_typed),
                       (99, field_99_typed), (100, field_100_typed),
                       (101, field_101_typed), (102, field_102_typed),
                       (103, field_103_typed), (104, field_104_typed),
                       (105, field_105_typed), (106, field_106_typed),
                       (107, field_107_typed), (108, field_108_typed),
                       (109, field_109_typed), (110, field_110_typed),
                       (111, field_111_typed), (112, field_112_typed),
                       (113, field_113_typed), (114, field_114_typed),
                       (115, field_115_typed), (116, field_116_typed),
                       (117, field_117_typed), (118, field_118_typed),
                       (119, field_119_typed), (120, field_120_typed),
                       (121, field_121_typed), (122, field_122_typed),
                       (123, field_123_typed), (124, field_124_typed),
                       (125, field_125_typed), (126, field_126_typed),
                       (127, field_127_typed), (128, field_128_typed),
                       (129, field_129_typed), (130, field_130_typed),
                       (131, field_131_typed), (132, field_132_typed),
                       (133, field_133_typed), (134, field_134_typed),
                       (135, field_135_typed), (136, field_136_typed),
                       (137, field_137_typed), (138, field_138_typed),
                       (139, field_139_typed)] {
        println!("Field {} (field_{}_u32)                  typed:     {} / {}", i, i, count, decoded);
    }

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
