//! Field-by-field offset trace through GimmickPostBody for a specific failing entry.
//! Reads each field individually and prints offset before/after.

use dmm_parser::binary::variant::{entry_ranges, load_pabgh_offsets};
use dmm_parser::binary::{BinaryRead, CArray, COptional, CString};
use dmm_parser::tables::gimmick_info::info::{
    GimmickInfo, GimmickTail,
    GimmickF20Elem, GimmickF24Elem, GimmickF34Elem, GimmickF35Elem,
    GimmickF46Data, GimmickF75Elem, GimmickF76Elem, GimmickF78Elem,
    GimmickF79Elem, GimmickF81Elem, GimmickF87Elem, GimmickF88Elem,
    GimmickF89Elem, GimmickF90Elem, GimmickF92Elem, GimmickF97Elem,
    GimmickF117Data, GimmickF119Elem, GimmickF125Elem, GimmickF126Elem,
    GimmickF128Elem, GimmickF129Elem, GimmickF130Elem, GimmickF132,
    GimmickF168Inner, GimmickF170Elem,
};

const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgb";
const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-4-24/gimmickinfo.pabgh";

macro_rules! read_field {
    ($name:expr, $ty:ty, $data:expr, $offset:expr) => {{
        let before = *$offset;
        match <$ty>::read_from($data, $offset) {
            Ok(v) => {
                println!("  {:6}..{:6}  {}", before, *$offset, $name);
                v
            }
            Err(e) => {
                println!("  FAIL at {}..{}: {} → {}", before, *$offset, $name, e);
                return;
            }
        }
    }};
}

fn trace(blob: &[u8]) {
    let mut off = 0usize;
    macro_rules! f {
        ($name:literal, $ty:ty) => { read_field!($name, $ty, blob, &mut off) };
    }

    f!("f20 CArray<GimmickF20Elem>",     CArray<GimmickF20Elem>);
    f!("f21 u8",                          u8);
    f!("f22 CArray<u32>",                 CArray<u32>);
    f!("f23 CArray<u32>",                 CArray<u32>);
    f!("f24 CArray<GimmickF24Elem>",      CArray<GimmickF24Elem>);
    f!("f25 u64",                         u64);
    f!("f26_32 [u8;7]",                   [u8;7]);
    f!("f33_a u32",                       u32);
    f!("f33_b u8",                        u8);
    f!("f33_c u8",                        u8);
    f!("f34 CArray<GimmickF34Elem>",      CArray<GimmickF34Elem>);
    f!("f35 CArray<GimmickF35Elem>",      CArray<GimmickF35Elem>);
    f!("f36 u8",                          u8);
    f!("f37 u32",                         u32);
    f!("f38 u32",                         u32);
    f!("f39 u32",                         u32);
    f!("f40_41 [u8;2]",                   [u8;2]);
    f!("f42 u32",                         u32);
    f!("f43_flag u8",                     u8);
    f!("f43_list CArray<u64>",            CArray<u64>);
    f!("f44 u64",                         u64);
    f!("f45 u64",                         u64);
    f!("f46 COptional<GimmickF46Data>",   COptional<GimmickF46Data>);
    f!("f47 [u32;3]",                     [u32;3]);
    f!("f48 u32",                         u32);
    f!("f49 u32",                         u32);
    f!("f50 u32",                         u32);
    f!("f51 u8",                          u8);
    f!("f52 u32",                         u32);
    f!("f53 u32",                         u32);
    f!("f54 u32",                         u32);
    f!("f55 u32",                         u32);
    f!("f56 u32",                         u32);
    f!("f57 [u32;3]",                     [u32;3]);
    f!("f58 u32",                         u32);
    f!("f59 u32",                         u32);
    f!("f60 u32",                         u32);
    f!("f61 u32",                         u32);
    f!("f61b u8",                         u8);
    f!("f62 u8",                          u8);
    f!("f63 u32",                         u32);
    f!("f64 u32",                         u32);
    f!("f65 u32",                         u32);
    f!("f66 u32",                         u32);
    f!("f67 u32",                         u32);
    f!("f68_70 [u8;3]",                   [u8;3]);
    f!("f71 u32",                         u32);
    f!("f72 [u32;3]",                     [u32;3]);
    f!("f73 u32",                         u32);
    f!("f74 u32",                         u32);
    f!("f75 CArray<GimmickF75Elem>",      CArray<GimmickF75Elem>);
    f!("f76 CArray<GimmickF76Elem>",      CArray<GimmickF76Elem>);
    f!("f77 COptional<GimmickF76Elem>",   COptional<GimmickF76Elem>);
    f!("f78 CArray<GimmickF78Elem>",      CArray<GimmickF78Elem>);
    f!("f79 CArray<GimmickF79Elem>",      CArray<GimmickF79Elem>);
    f!("f80 CArray<u32>",                 CArray<u32>);
    f!("f81 CArray<GimmickF81Elem>",      CArray<GimmickF81Elem>);
    f!("f82 u32",                         u32);
    f!("f83 u32",                         u32);
    f!("f84_85 [u8;2]",                   [u8;2]);
    f!("f86_str_a CString",               CString);
    f!("f86_str_b CString",               CString);
    f!("f86_a u32",                       u32);
    f!("f86_b u32",                       u32);
    f!("f86_c u32",                       u32);
    f!("f87 CArray<GimmickF87Elem>",      CArray<GimmickF87Elem>);
    f!("f88 CArray<GimmickF88Elem>",      CArray<GimmickF88Elem>);
    f!("f89 CArray<GimmickF89Elem>",      CArray<GimmickF89Elem>);
    f!("f90 CArray<GimmickF90Elem>",      CArray<GimmickF90Elem>);
    f!("f91 u32",                         u32);
    f!("f92 CArray<GimmickF92Elem>",      CArray<GimmickF92Elem>);
    f!("f93 u32",                         u32);
    f!("f94 u32",                         u32);
    f!("f95 u32",                         u32);
    f!("f96 u32",                         u32);
    f!("f97 CArray<GimmickF97Elem>",      CArray<GimmickF97Elem>);
    f!("f98 u8",                          u8);
    f!("f99 u32",                         u32);
    f!("f100 CArray<u32>",                CArray<u32>);
    f!("f101 CArray<u32>",                CArray<u32>);
    f!("f102_103 [u8;2]",                 [u8;2]);
    f!("f104 u16",                        u16);
    f!("f105 u16",                        u16);
    f!("f106_115 [u8;10]",               [u8;10]);
    f!("f116 u32",                        u32);
    f!("f117 COptional<GimmickF117Data>", COptional<GimmickF117Data>);
    f!("f118 u8",                         u8);
    f!("f119 CArray<GimmickF119Elem>",    CArray<GimmickF119Elem>);
    f!("f120 u32",                        u32);
    f!("f121 u32",                        u32);
    f!("f122 u32",                        u32);
    f!("f123 u8",                         u8);
    f!("f124 CString",                    CString);
    f!("f125 CArray<GimmickF125Elem>",    CArray<GimmickF125Elem>);
    f!("f126 CArray<GimmickF126Elem>",    CArray<GimmickF126Elem>);
    f!("f127 CArray<GimmickF126Elem>",    CArray<GimmickF126Elem>);
    f!("f128 CArray<GimmickF128Elem>",    CArray<GimmickF128Elem>);
    f!("f129 CArray<GimmickF129Elem>",    CArray<GimmickF129Elem>);
    f!("f130 CArray<GimmickF130Elem>",    CArray<GimmickF130Elem>);
    f!("f131 u32",                        u32);
    f!("f132 GimmickF132",                GimmickF132);
    f!("f133 u32",                        u32);
    f!("f134 u8",                         u8);
    f!("f135 u32",                        u32);
    f!("f136_138 [u8;3]",                 [u8;3]);
    f!("f139 u32",                        u32);
    f!("f140 u32",                        u32);
    f!("f141 u32",                        u32);
    f!("f142_144 [u8;3]",                 [u8;3]);
    f!("f145 u32",                        u32);
    f!("f146_a u32",                      u32);
    f!("f146_b u32",                      u32);
    f!("f146_c u32",                      u32);
    f!("f146_d u8",                       u8);
    f!("f146_e u8",                       u8);
    f!("f147 u16",                        u16);
    f!("f148 CArray<u16>",                CArray<u16>);
    f!("f149 u8",                         u8);
    f!("f150 u16",                        u16);
    f!("f151 u16",                        u16);
    f!("f152_155 [u8;4]",                 [u8;4]);
    f!("f154 CString",                    CString);
    f!("f155_163 [u8;9]",                 [u8;9]);
    f!("f164 u32",                        u32);
    f!("f165 u64",                        u64);
    f!("f166 CArray<GimmickF75Elem>",     CArray<GimmickF75Elem>);
    f!("f167 CArray<GimmickF75Elem>",     CArray<GimmickF75Elem>);
    f!("f168 CArray<COptional<GimmickF168Inner>>", CArray<COptional<GimmickF168Inner>>);
    f!("f169 CArray<COptional<GimmickF168Inner>>", CArray<COptional<GimmickF168Inner>>);
    f!("f170_a u32",                      u32);
    f!("f170_b u32",                      u32);
    f!("f170_c u32",                      u32);
    f!("f170_list CArray<GimmickF170Elem>", CArray<GimmickF170Elem>);
    f!("f171 u32",                        u32);
    f!("f172_175 [u8;4]",                 [u8;4]);
    f!("f176 u32",                        u32);
    f!("f177 u8",                         u8);
    f!("f178 u32",                        u32);
    f!("f179 u32",                        u32);
    println!("  SUCCESS: consumed {} of {} bytes", off, blob.len());
}

fn main() {
    let data = match std::fs::read(PABGB) {
        Ok(d) => d, Err(e) => { eprintln!("SKIP: {}", e); return; }
    };
    let entries = match load_pabgh_offsets(PABGH) {
        Some(e) => e, None => { eprintln!("SKIP pabgh"); return; }
    };
    let ranges = entry_ranges(&entries, data.len());

    // Trace 2 failing count=0 entries (any blob size)
    let mut traces = 0;
    for (key, start, end) in &ranges {
        if traces >= 2 { break; }
        let mut cur = *start;
        let item = match GimmickInfo::read_with_size(&data, &mut cur, end - start) {
            Ok(it) => it, Err(_) => continue,
        };
        let GimmickTail::Decoded { alt_trigger_list, post_body, post_blob, .. } = &item.tail else { continue };
        let Some(atl) = alt_trigger_list else { continue };
        if !atl.items.is_empty() || post_body.is_some() { continue; }

        println!("\n=== Tracing key=0x{:08x} blob_len={} ===", key, post_blob.len());
        trace(post_blob);
        traces += 1;
    }
}
