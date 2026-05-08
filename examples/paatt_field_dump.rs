// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

use dmm_parser::binary::paatt::PaattFile;
use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use std::collections::BTreeMap;
use std::path::Path;

const GAME_DIR: &str = "/mnt/d/SteamLibrary/steamapps/common/Crimson Desert";

fn r32(d: &[u8], o: usize) -> f32 { f32::from_le_bytes(d[o..o+4].try_into().unwrap()) }
fn r16(d: &[u8], o: usize) -> u16 { u16::from_le_bytes(d[o..o+2].try_into().unwrap()) }
fn r32u(d: &[u8], o: usize) -> u32 { u32::from_le_bytes(d[o..o+4].try_into().unwrap()) }

fn show_u8(label: &str, data: &[Vec<u8>], off: usize) {
    let mut d: BTreeMap<u8,u32> = BTreeMap::new();
    for r in data { *d.entry(r[off]).or_insert(0) += 1; }
    let tot = data.len() as f64;
    let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect();
    v.sort_by(|a,b| b.1.cmp(&a.1));
    print!("{} ({} distinct):", label, d.len());
    for (val,cnt) in v.iter().take(6) { print!("  0x{:02x}@{:.0}%", val, *cnt as f64/tot*100.0); }
    println!();
}
fn show_u16(label: &str, data: &[Vec<u8>], off: usize) {
    let mut d: BTreeMap<u16,u32> = BTreeMap::new();
    for r in data { *d.entry(r16(r, off)).or_insert(0) += 1; }
    let tot = data.len() as f64;
    let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect();
    v.sort_by(|a,b| b.1.cmp(&a.1));
    print!("{} ({} distinct):", label, d.len());
    for (val,cnt) in v.iter().take(6) { print!("  0x{:04x}@{:.0}%", val, *cnt as f64/tot*100.0); }
    println!();
}
fn show_u32(label: &str, data: &[Vec<u8>], off: usize) {
    let mut d: BTreeMap<u32,u32> = BTreeMap::new();
    for r in data { *d.entry(r32u(r, off)).or_insert(0) += 1; }
    let tot = data.len() as f64;
    let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect();
    v.sort_by(|a,b| b.1.cmp(&a.1));
    print!("{} ({} distinct):", label, d.len());
    for (val,cnt) in v.iter().take(6) { print!("  0x{:08x}@{:.0}%", val, *cnt as f64/tot*100.0); }
    println!();
}
fn show_f32(label: &str, data: &[Vec<u8>], off: usize) {
    let mut d: BTreeMap<u32,u32> = BTreeMap::new();
    for r in data { *d.entry(r32(r,off).to_bits()).or_insert(0) += 1; }
    let tot = data.len() as f64;
    let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect();
    v.sort_by(|a,b| b.1.cmp(&a.1));
    print!("{} ({} distinct):", label, d.len());
    for (bits,cnt) in v.iter().take(6) {
        let f = f32::from_bits(*bits);
        print!("  {:.4}@{:.0}%", f, *cnt as f64/tot*100.0);
    }
    println!();
}

fn main() {
    let game_dir = Path::new(GAME_DIR);
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .expect("read game dir")
        .filter_map(|e| e.ok()).filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    let mut v0: Vec<Vec<u8>> = Vec::new();
    let mut v1: Vec<Vec<u8>> = Vec::new();
    let mut v2: Vec<Vec<u8>> = Vec::new();
    let mut v3: Vec<Vec<u8>> = Vec::new();

    for g in &groups {
        let gd = game_dir.join(g);
        let pd = match std::fs::read(gd.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let pm = match PackMeta::parse(&pd, None) { Ok(p) => p, Err(_) => continue };
        let ei = pm.header.encrypt_info.encrypt_info;
        for dir in &pm.directories {
            for f in &dir.files {
                if !f.name.to_ascii_lowercase().ends_with(".paatt") { continue; }
                let bytes = match paz::extract_file(&gd, f, &dir.path, &ei) { Ok(b)=>b, Err(_)=>continue };
                let paatt = match PaattFile::parse(&bytes) { Ok(p)=>p, Err(_)=>continue };
                for info in &paatt.infos {
                    match info.version {
                        0 => v0.push(info.base_data.clone()),
                        1 => v1.push(info.base_data.clone()),
                        2 => v2.push(info.base_data.clone()),
                        3 => v3.push(info.base_data.clone()),
                        _ => {}
                    }
                }
            }
        }
    }
    println!("V0:{} V1:{} V2:{} V3:{}", v0.len(), v1.len(), v2.len(), v3.len());

    println!("\n=== V0 ===");
    show_u16("0x0036 _unk0036",    &v0, 0x0036);
    show_u8 ("0x0072 _unk0072",    &v0, 0x0072);
    show_u8 ("0x0073 _unk0073",    &v0, 0x0073);
    {
        let mut pairs: BTreeMap<(u8,u8),u32> = BTreeMap::new();
        for d in &v0 { *pairs.entry((d[0x0072],d[0x0073])).or_insert(0) += 1; }
        let tot = v0.len() as f64;
        let mut pv: Vec<_> = pairs.iter().collect(); pv.sort_by(|a,b| b.1.cmp(a.1));
        println!("  0072+0073 pairs: {:?}", pv.iter().take(6).map(|((a,b),c)| format!("({},{:#x})@{:.0}%",a,b,**c as f64/tot*100.0)).collect::<Vec<_>>());
    }
    show_u8 ("0x00b8 _unk00b8",    &v0, 0x00b8);
    show_u8 ("0x00bb _unk00bb",    &v0, 0x00bb);
    show_u8 ("0x00cc _unk00cc",    &v0, 0x00cc);
    show_u8 ("0x00cf _unk00cf",    &v0, 0x00cf);
    show_u8 ("0x00d3 _unk00d3",    &v0, 0x00d3);
    show_f32("0x00d4 _unk_f32_00d4",&v0, 0x00d4);
    show_u8 ("0x00f4 _unk00f4",    &v0, 0x00f4);
    show_u16("0x00f8 _unk00f8",    &v0, 0x00f8);
    show_u8 ("0x00fa _unk00fa",    &v0, 0x00fa);
    show_u8 ("0x00fc _unk00fc",    &v0, 0x00fc);
    show_u16("0x0100 _unk0100",    &v0, 0x0100);
    show_u8 ("0x0104 _unk0104",    &v0, 0x0104);

    println!("\n=== V1 catch_desc (catch_desc starts at byte 264) ===");
    let cd = 264usize;
    show_u32("_cd_unk0000", &v1, cd+0x0000);
    show_u32("_cd_unk0004", &v1, cd+0x0004);
    show_u32("_cd_unk0008", &v1, cd+0x0008);
    show_u16("_cd_unk000c", &v1, cd+0x000c);
    show_u32("_cd_unk0010", &v1, cd+0x0010);
    show_f32("_cd_unk0024", &v1, cd+0x0024);
    show_f32("_cd_unk0028", &v1, cd+0x0028);
    show_f32("_cd_unk002c", &v1, cd+0x002c);
    show_f32("_cd_unk0030", &v1, cd+0x0030);
    // check if 002c == -0030
    {
        let sym = v1.iter().filter(|d| { let a=r32(d,cd+0x002c); let b=r32(d,cd+0x0030); (a+b).abs()<1e-5 }).count();
        println!("  _cd002c + _cd0030 == 0 (symmetric): {}/{} ({:.0}%)", sym, v1.len(), sym as f64/v1.len() as f64*100.0);
    }
    show_f32("_cd_unk0040", &v1, cd+0x0040);
    show_f32("_cd_unk0044", &v1, cd+0x0044);
    show_f32("_cd_unk0050", &v1, cd+0x0050);
    show_f32("_cd_unk0054", &v1, cd+0x0054);
    {
        let same = v1.iter().filter(|d| r32(d,cd+0x0050).to_bits()==r32(d,cd+0x0054).to_bits()).count();
        println!("  _cd0050 == _cd0054: {}/{} ({:.0}%)", same, v1.len(), same as f64/v1.len() as f64*100.0);
        let sym2 = v1.iter().filter(|d| { let a=r32(d,cd+0x002c); let b=r32(d,cd+0x0050); (a-b).abs()<1e-5 }).count();
        println!("  _cd002c == _cd0050: {}/{} ({:.0}%)", sym2, v1.len(), sym2 as f64/v1.len() as f64*100.0);
    }
    // degrees analysis of 002c/0050
    {
        println!("  _cd002c in degrees (top values):");
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in &v1 { *d.entry(r32(r,cd+0x002c).to_bits()).or_insert(0) += 1; }
        let tot = v1.len() as f64;
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        for (bits,cnt) in v.iter().take(8) {
            let f = f32::from_bits(*bits);
            println!("    {:.4} rad = {:.2}° @ {:.0}%", f, f.to_degrees(), *cnt as f64/tot*100.0);
        }
    }

    // _cd0040 == _cd0044 always?
    let same40_44 = v1.iter().filter(|d| r32(d,cd+0x0040).to_bits()==r32(d,cd+0x0044).to_bits()).count();
    println!("_cd0040 == _cd0044: {}/{} ({:.0}%)", same40_44, v1.len(), same40_44 as f64/v1.len() as f64*100.0);

    // _cd0028 exact bit pattern
    let exact_neg0 = v1.iter().filter(|d| r32(d,cd+0x0028).to_bits() == 0x80000000u32).count();
    let exact_pos0 = v1.iter().filter(|d| r32(d,cd+0x0028).to_bits() == 0x00000000u32).count();
    println!("_cd0028: exact -0.0(0x80000000) = {}, +0.0(0x00000000) = {} (total {})", exact_neg0, exact_pos0, v1.len());

    delay_analysis(&v0);
    hit_data_analysis(&v0);
    cross_version_analysis(&v0, &v2, &v3);
    v2_analysis(&v2);
    v3_analysis(&v3);
    v0_extra(&v0);
    v0_type_analysis(&v0, &v1);
    constcheck_analysis(&v0, &v2, &v3);
}

fn delay_analysis(v0: &[Vec<u8>]) {
    println!("\n=== Delay sub-struct analysis ===");
    // 5 floats at 0x0058..0x006c (ds1) and 0x0078..0x008c (ds2)
    for (label, base) in [("ds1", 0x0058usize), ("ds2", 0x0078usize)] {
        println!("\n--- {} (base=0x{:04x}) ---", label, base);
        for (i, off) in (0..5).map(|i| (i, base + i*4)) {
            show_f32(&format!("  f{} @ 0x{:04x}", i, off), v0, off);
        }
        // Is f0 always < f1 (start <= end)?
        let lt = v0.iter().filter(|d| r32(d, base) < r32(d, base+4)).count();
        let eq = v0.iter().filter(|d| r32(d, base).to_bits() == r32(d, base+4).to_bits()).count();
        let gt = v0.iter().filter(|d| r32(d, base) > r32(d, base+4)).count();
        println!("  f0 < f1: {}  f0 == f1: {}  f0 > f1: {} (of {})", lt, eq, gt, v0.len());
        // Is f2 always zero?
        let f2_zero = v0.iter().filter(|d| r32(d, base+8).to_bits() == 0).count();
        println!("  f2 == 0: {}/{} ({:.1}%)", f2_zero, v0.len(), f2_zero as f64/v0.len() as f64*100.0);
        // f0 all-values distribution
        println!("  f0 ALL values:");
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in v0 { *d.entry(r32(r, base).to_bits()).or_insert(0) += 1; }
        let tot = v0.len() as f64;
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        for (bits,cnt) in v.iter().take(12) {
            println!("    {:.6} @ {:.1}%", f32::from_bits(*bits), *cnt as f64/tot*100.0);
        }
        // f1 all-values
        println!("  f1 ALL values:");
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in v0 { *d.entry(r32(r, base+4).to_bits()).or_insert(0) += 1; }
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        for (bits,cnt) in v.iter().take(12) {
            println!("    {:.6} @ {:.1}%", f32::from_bits(*bits), *cnt as f64/tot*100.0);
        }
        // is ds1.f0 == ds2.f0?
    }
    // Are ds1 and ds2 mirrors?
    let mirror = v0.iter().filter(|d| {
        (0..5).all(|i| r32(d, 0x0058+i*4).to_bits() == r32(d, 0x0078+i*4).to_bits())
    }).count();
    println!("\nds1 == ds2 (all 5 floats): {}/{} ({:.1}%)", mirror, v0.len(), mirror as f64/v0.len() as f64*100.0);
    // f3 non-zero — what does f1 look like then?
    let f3_nz: Vec<_> = v0.iter().filter(|d| r32(d, 0x0058+12).abs() > 1e-6).collect();
    println!("ds1.f3 non-zero: {}/{}", f3_nz.len(), v0.len());
    if !f3_nz.is_empty() {
        println!("  (f0, f1, f2, f3, f4) samples when f3 != 0:");
        for d in f3_nz.iter().take(5) {
            println!("    ({:.4}, {:.4}, {:.4}, {:.4}, {:.4})", r32(d,0x0058), r32(d,0x005c), r32(d,0x0060), r32(d,0x0064), r32(d,0x0068));
        }
    }

    // ds1.f4 integer check
    let ds1_f4_int = v0.iter().filter(|d| { let f = r32(d,0x0068); (f - f.round()).abs() < 1e-5 }).count();
    let ds2_f4_int = v0.iter().filter(|d| { let f = r32(d,0x0088); (f - f.round()).abs() < 1e-5 }).count();
    println!("\nds1.f4 is integer-valued: {}/{} ({:.1}%)", ds1_f4_int, v0.len(), ds1_f4_int as f64/v0.len() as f64*100.0);
    println!("ds2.f4 is integer-valued: {}/{} ({:.1}%)", ds2_f4_int, v0.len(), ds2_f4_int as f64/v0.len() as f64*100.0);

    // ds1.f4 ALL values
    println!("ds1.f4 ALL values (0x0068):");
    {
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in v0 { *d.entry(r32(r, 0x0068).to_bits()).or_insert(0) += 1; }
        let tot = v0.len() as f64;
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        for (bits,cnt) in v.iter() {
            println!("  {:.4} @ {:.1}%", f32::from_bits(*bits), *cnt as f64/tot*100.0);
        }
    }
    println!("ds2.f4 ALL values (0x0088):");
    {
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in v0 { *d.entry(r32(r, 0x0088).to_bits()).or_insert(0) += 1; }
        let tot = v0.len() as f64;
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        for (bits,cnt) in v.iter() {
            println!("  {:.4} @ {:.1}%", f32::from_bits(*bits), *cnt as f64/tot*100.0);
        }
    }

    // Relationship: ds1.f0 * 30 ≈ ds1.f4 (frame time hypothesis)?
    let approx_fps: Vec<(f32,f32)> = v0.iter()
        .filter(|d| r32(d,0x0068) > 0.5)
        .map(|d| (r32(d,0x0058), r32(d,0x0068)))
        .take(10).collect();
    println!("\nSamples where ds1.f4 > 0 (f0, f4):");
    for (f0, f4) in &approx_fps {
        println!("  f0={:.4}  f4={:.4}  f4/f0={:.1}", f0, f4, if f0.abs() > 1e-6 { f4/f0 } else { 0.0 });
    }

    // ds2.f2 ALL values (to compare with ds1.f2)
    println!("\nds1.f2 ALL values (0x0060):");
    {
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in v0 { *d.entry(r32(r, 0x0060).to_bits()).or_insert(0) += 1; }
        let tot = v0.len() as f64;
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        for (bits,cnt) in v.iter() {
            println!("  {:.4} @ {:.1}%", f32::from_bits(*bits), *cnt as f64/tot*100.0);
        }
    }
    // ds1.f0 == ds2.f0?
    let f0_match = v0.iter().filter(|d| r32(d,0x0058).to_bits()==r32(d,0x0078).to_bits()).count();
    println!("ds1.f0 == ds2.f0: {}/{} ({:.1}%)", f0_match, v0.len(), f0_match as f64/v0.len() as f64*100.0);
    let f1_match = v0.iter().filter(|d| r32(d,0x005c).to_bits()==r32(d,0x007c).to_bits()).count();
    println!("ds1.f1 == ds2.f1: {}/{} ({:.1}%)", f1_match, v0.len(), f1_match as f64/v0.len() as f64*100.0);
    let f2_match = v0.iter().filter(|d| r32(d,0x0060).to_bits()==r32(d,0x0080).to_bits()).count();
    println!("ds1.f2 == ds2.f2: {}/{} ({:.1}%)", f2_match, v0.len(), f2_match as f64/v0.len() as f64*100.0);
    // Cases where ds1 != ds2 — show (ds1.f0,f1,f2,f4) vs (ds2.f0,f1,f2,f4)
    println!("Samples ds1 != ds2 (first 6):");
    let mut cnt = 0;
    for d in v0 {
        if (0..5).any(|i| r32(d, 0x0058+i*4).to_bits() != r32(d, 0x0078+i*4).to_bits()) {
            println!("  ds1=({:.3},{:.3},{:.3},{:.3},{:.3}) ds2=({:.3},{:.3},{:.3},{:.3},{:.3})",
                r32(d,0x0058),r32(d,0x005c),r32(d,0x0060),r32(d,0x0064),r32(d,0x0068),
                r32(d,0x0078),r32(d,0x007c),r32(d,0x0080),r32(d,0x0084),r32(d,0x0088));
            cnt += 1;
            if cnt >= 6 { break; }
        }
    }
}

fn hit_data_analysis(v0: &[Vec<u8>]) {
    println!("\n=== _unk00f4 / hit_rotation_type correlation ===");
    // _unk00f4: 11 values. Correlate with hit_degree (0x0090) and hit_check_type (0x0034)
    println!("_unk00f4 paired with hit_degree:");
    let mut pairs: BTreeMap<(u8,u32),u32> = BTreeMap::new();
    for d in v0 {
        let f4 = d[0x00f4];
        let deg_bits = r32(d, 0x0090).to_bits();
        *pairs.entry((f4, deg_bits)).or_insert(0) += 1;
    }
    let tot = v0.len() as f64;
    let mut pv: Vec<_> = pairs.iter().collect(); pv.sort_by(|a,b| b.1.cmp(a.1));
    for ((f4, deg_bits), c) in pv.iter().take(12) {
        println!("  _unk00f4={}  hit_degree={:.2}°  count={} ({:.1}%)", f4, f32::from_bits(*deg_bits), c, **c as f64/tot*100.0);
    }

    println!("\n_unk00fc ALL values:");
    let mut d: BTreeMap<u8,u32> = BTreeMap::new();
    for r in v0 { *d.entry(r[0x00fc]).or_insert(0) += 1; }
    let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
    for (val,cnt) in &v { println!("  {}={}", val, cnt); }

    // _unk00fc paired with _unk00f4
    println!("\n_unk00f4 vs _unk00fc pairs (top 8):");
    let mut pairs2: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0 { *pairs2.entry((d[0x00f4], d[0x00fc])).or_insert(0) += 1; }
    let mut pv2: Vec<_> = pairs2.iter().collect(); pv2.sort_by(|a,b| b.1.cmp(a.1));
    for ((f4, fc), c) in pv2.iter().take(8) {
        println!("  _unk00f4={}  _unk00fc={}  count={} ({:.1}%)", f4, fc, c, **c as f64/tot*100.0);
    }

    // _unk0100 non-zero values alongside _unk00f8
    println!("\n_unk0100 non-zero:");
    let mut d: BTreeMap<u16,u32> = BTreeMap::new();
    for r in v0 { let v = u16::from_le_bytes(r[0x0100..0x0102].try_into().unwrap()); if v != 0 { *d.entry(v).or_insert(0) += 1; } }
    let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
    for (val,cnt) in &v { println!("  0x{:04x} = {}", val, cnt); }

    println!("\n_unk0104 vs _unk00f4 pairs:");
    let mut pairs3: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0 { *pairs3.entry((d[0x0104], d[0x00f4])).or_insert(0) += 1; }
    let mut pv3: Vec<_> = pairs3.iter().collect(); pv3.sort_by(|a,b| b.1.cmp(a.1));
    for ((t4, f4), c) in pv3.iter().take(8) {
        println!("  _unk0104={}  _unk00f4={}  count={}", t4, f4, c);
    }

    // _unk0100 co-occurrence with hit_data_str_idx
    println!("\n_unk0100 non-zero: co-occurrence with hit_data_str_idx (0x00f8):");
    let mut both=0u32; let mut only0100=0u32; let mut only00f8=0u32;
    for r in v0 {
        let v100 = u16::from_le_bytes(r[0x0100..0x0102].try_into().unwrap());
        let vf8  = u16::from_le_bytes(r[0x00f8..0x00fa].try_into().unwrap());
        if v100 != 0 && vf8 != 0 { both += 1; }
        else if v100 != 0 { only0100 += 1; }
        else if vf8 != 0 { only00f8 += 1; }
    }
    println!("  both non-zero: {}  only _unk0100: {}  only hit_data_str_idx: {}", both, only0100, only00f8);

    println!("\n_unk0100 non-zero, paired with attack_hit_check_type:");
    let mut pairs4: BTreeMap<(u16,u16),u32> = BTreeMap::new();
    for d in v0 {
        let v100 = u16::from_le_bytes(d[0x0100..0x0102].try_into().unwrap());
        if v100 != 0 {
            let hct = u16::from_le_bytes(d[0x0034..0x0036].try_into().unwrap());
            *pairs4.entry((hct, v100)).or_insert(0) += 1;
        }
    }
    let mut pv4: Vec<_> = pairs4.iter().collect(); pv4.sort_by(|a,b| b.1.cmp(a.1));
    for ((hct, v100), c) in pv4.iter().take(10) {
        println!("  hit_check_type=0x{:04x}  _unk0100=0x{:04x}  count={}", hct, v100, c);
    }
}

fn cross_version_analysis(v0: &[Vec<u8>], v2: &[Vec<u8>], v3: &[Vec<u8>]) {
    println!("\n=== Cross-version byte comparison ===");
    for (off, label) in [(0x00b8usize, "_unk00b8"), (0x00b9, "_unk00b9"), (0x00ba, "_unk00ba"),
                         (0x00bb, "_unk00bb"), (0x00bc, "_pad00bc"), (0x00bd, "_unk00bd"),
                         (0x00be, "_pad00be"), (0x00bf, "_unk00bf"), (0x00c0, "_unk00c0")] {
        let v0_top = {
            let mut d: BTreeMap<u8,u32> = BTreeMap::new();
            for r in v0 { *d.entry(r[off]).or_insert(0) += 1; }
            let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
            v.iter().take(3).map(|(k,c)| format!("0x{:02x}@{:.0}%", k, *c as f64/v0.len() as f64*100.0)).collect::<Vec<_>>().join(" ")
        };
        let v2_top = {
            let mut d: BTreeMap<u8,u32> = BTreeMap::new();
            for r in v2 { *d.entry(r[off]).or_insert(0) += 1; }
            let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
            v.iter().take(3).map(|(k,c)| format!("0x{:02x}@{:.0}%", k, *c as f64/v2.len() as f64*100.0)).collect::<Vec<_>>().join(" ")
        };
        let v3_top = {
            let mut d: BTreeMap<u8,u32> = BTreeMap::new();
            for r in v3 { *d.entry(r[off]).or_insert(0) += 1; }
            let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
            v.iter().take(3).map(|(k,c)| format!("0x{:02x}@{:.0}%", k, *c as f64/v3.len() as f64*100.0)).collect::<Vec<_>>().join(" ")
        };
        println!("  0x{:04x} {:12} V0:[{}]  V2:[{}]  V3:[{}]", off, label, v0_top, v2_top, v3_top);
    }
    // Also check 0x00c8-0x00d3 region
    println!();
    for (off, label) in [(0x00c8usize, "_unk00c8"), (0x00c9, "_unk00c9"), (0x00ca, "_unk00ca"),
                         (0x00cb, "_unk00cb"), (0x00cc, "_unk00cc"), (0x00cd, "_unk00cd"),
                         (0x00ce, "_unk00ce"), (0x00cf, "_unk00cf")] {
        let v0_top = {
            let mut d: BTreeMap<u8,u32> = BTreeMap::new();
            for r in v0 { *d.entry(r[off]).or_insert(0) += 1; }
            let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
            v.iter().take(3).map(|(k,c)| format!("0x{:02x}@{:.0}%", k, *c as f64/v0.len() as f64*100.0)).collect::<Vec<_>>().join(" ")
        };
        let v2_top = {
            let mut d: BTreeMap<u8,u32> = BTreeMap::new();
            for r in v2 { *d.entry(r[off]).or_insert(0) += 1; }
            let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
            v.iter().take(3).map(|(k,c)| format!("0x{:02x}@{:.0}%", k, *c as f64/v2.len() as f64*100.0)).collect::<Vec<_>>().join(" ")
        };
        let v3_top = {
            let mut d: BTreeMap<u8,u32> = BTreeMap::new();
            for r in v3 { *d.entry(r[off]).or_insert(0) += 1; }
            let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
            v.iter().take(3).map(|(k,c)| format!("0x{:02x}@{:.0}%", k, *c as f64/v3.len() as f64*100.0)).collect::<Vec<_>>().join(" ")
        };
        println!("  0x{:04x} {:12} V0:[{}]  V2:[{}]  V3:[{}]", off, label, v0_top, v2_top, v3_top);
    }
}

fn v2_analysis(v2: &[Vec<u8>]) {
    if v2.is_empty() { println!("\n=== V2: no records ==="); return; }
    println!("\n=== V2 extra fields (0x0108..0x0127) ===");
    let tot = v2.len() as f64;
    // _unk0108 and _unk010a distributions
    show_u16("_unk0108 (0x0108)", v2, 0x0108);
    show_u16("_unk010a (0x010a)", v2, 0x010a);
    // check if reading as u32 makes more sense
    {
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in v2 { *d.entry(r32u(r, 0x0108)).or_insert(0) += 1; }
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        print!("  as u32 0x0108 ({} distinct):", d.len());
        for (val,cnt) in v.iter().take(8) { print!("  0x{:08x}@{:.0}%", val, *cnt as f64/tot*100.0); }
        println!();
    }
    // action_hash_code distribution (high cardinality expected)
    {
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in v2 { *d.entry(r32u(r, 0x010c)).or_insert(0) += 1; }
        println!("action_hash_code (0x010c): {} distinct values (should be high-cardinality)", d.len());
    }
    // frame_time distribution
    show_f32("frame_time (0x0110)", v2, 0x0110);
    // ai_event_key
    show_u32("ai_event_key (0x0114)", v2, 0x0114);
    // _unk011c byte-by-byte
    for i in 0..4 { show_u8(&format!("  _unk011c[{}] (0x{:04x})", i, 0x011c+i), v2, 0x011c+i); }
    // _unk0120 byte-by-byte
    for i in 0..4 { show_u8(&format!("  _unk0120[{}] (0x{:04x})", i, 0x0120+i), v2, 0x0120+i); }
    // _unk_f32_0124 — check raw bits
    println!("\n_unk_f32_0124 raw bits (0x0124):");
    {
        let mut d: BTreeMap<u32,u32> = BTreeMap::new();
        for r in v2 { *d.entry(r32u(r, 0x0124)).or_insert(0) += 1; }
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        for (bits,cnt) in v.iter() {
            println!("  0x{:08x} = f32({:.6}) @ {:.1}%", bits, f32::from_bits(*bits), *cnt as f64/tot*100.0);
        }
    }
    show_f32("_unk_f32_0124 (0x0124)", v2, 0x0124);
    // pairing of _unk0108 and _unk010a
    println!("\n_unk0108 + _unk010a pairs:");
    {
        let mut pairs: BTreeMap<(u16,u16),u32> = BTreeMap::new();
        for d in v2 { *pairs.entry((r16(d,0x0108), r16(d,0x010a))).or_insert(0) += 1; }
        let mut pv: Vec<_> = pairs.iter().collect(); pv.sort_by(|a,b| b.1.cmp(a.1));
        for ((a,b),c) in pv.iter().take(12) {
            println!("  ({}, {}) count={} ({:.1}%)", a, b, c, **c as f64/tot*100.0);
        }
    }
}

fn v3_analysis(v3: &[Vec<u8>]) {
    if v3.is_empty() { println!("\n=== V3: no records ==="); return; }
    println!("\n=== V3 extra fields (0x0108..0x011f) ===");
    let tot = v3.len() as f64;
    show_f32("_unk_f32_0108 (0x0108)", v3, 0x0108);
    show_f32("_unk_f32_010c (0x010c)", v3, 0x010c);
    show_u32("_unk0110 (0x0110)", v3, 0x0110);
    show_u32("_unk0114 (0x0114)", v3, 0x0114);
    for i in 0..4 { show_u8(&format!("  _unk0118[{}] (0x{:04x})", i, 0x0118+i), v3, 0x0118+i); }
    for i in 0..4 { show_u8(&format!("  _pad011c[{}] (0x{:04x})", i, 0x011c+i), v3, 0x011c+i); }
    let _ = tot;
}

fn v0_extra(v0: &[Vec<u8>]) {
    // _unk00f8 (u16) cluster 0x0450-0x0455
    let mut d: std::collections::BTreeMap<u16,u32> = std::collections::BTreeMap::new();
    for r in v0 { *d.entry(r16(r, 0x00f8)).or_insert(0) += 1; }
    let tot = v0.len() as f64;
    println!("\n_unk00f8 ALL values (u16):");
    let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
    for (val,cnt) in v.iter() {
        println!("  0x{:04x} = {:5} ({:.1}%)", val, cnt, *cnt as f64/tot*100.0);
    }

    // Check: does _unk00f8 == _unk0100 ever?
    let same = v0.iter().filter(|d| r16(d,0x00f8)==r16(d,0x0100)).count();
    println!("_unk00f8 == _unk0100: {}/{}", same, v0.len());

    // Distribution of _unk00f8 non-zero values alongside attack_hit_check_type
    println!("\n_unk00f8 non-zero, paired with attack_hit_check_type:");
    let mut pairs: std::collections::BTreeMap<(u16,u16),u32> = std::collections::BTreeMap::new();
    for d in v0.iter().filter(|d| r16(d,0x00f8) != 0) {
        *pairs.entry((r16(d,0x0034), r16(d,0x00f8))).or_insert(0) += 1;
    }
    let mut pv: Vec<_> = pairs.iter().collect(); pv.sort_by(|a,b| b.1.cmp(a.1));
    for ((hit_type, f8), c) in pv.iter().take(10) {
        println!("  hit_check_type=0x{:04x}  _unk00f8=0x{:04x}  count={}", hit_type, f8, c);
    }
}

fn v0_type_analysis(v0: &[Vec<u8>], v1: &[Vec<u8>]) {
    let tot = v0.len() as f64;
    println!("\n=== _unk00b8 / _unk_f32_00d4 analysis ===");

    // _unk_f32_00d4 — 0.0 vs 198.0; correlate with _unk009c and _unk00b8
    println!("\n_unk_f32_00d4 paired with _unk009c:");
    let mut pairs: BTreeMap<(u32, u8), u32> = BTreeMap::new();
    for d in v0 { *pairs.entry((r32u(d,0x00d4), d[0x009c])).or_insert(0) += 1; }
    let mut pv: Vec<_> = pairs.iter().collect(); pv.sort_by(|a,b| b.1.cmp(a.1));
    for ((d4_bits, c9), c) in pv.iter().take(8) {
        println!("  d4={}  _unk009c=0x{:02x}  count={} ({:.1}%)", f32::from_bits(*d4_bits), c9, c, **c as f64/tot*100.0);
    }

    println!("\n_unk_f32_00d4 paired with _unk00b8:");
    let mut pairs2: BTreeMap<(u32, u8), u32> = BTreeMap::new();
    for d in v0 { *pairs2.entry((r32u(d,0x00d4), d[0x00b8])).or_insert(0) += 1; }
    let mut pv2: Vec<_> = pairs2.iter().collect(); pv2.sort_by(|a,b| b.1.cmp(a.1));
    for ((d4_bits, b8), c) in pv2.iter().take(8) {
        println!("  d4={}  _unk00b8=0x{:02x}  count={} ({:.1}%)", f32::from_bits(*d4_bits), b8, c, **c as f64/tot*100.0);
    }

    // _unk00b8 in V0 vs V1 (first 264 bytes of V1 = V0 layout)
    println!("\n_unk00b8 in V1 records:");
    let mut d: BTreeMap<u8,u32> = BTreeMap::new();
    for r in v1 { *d.entry(r[0x00b8]).or_insert(0) += 1; }
    let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
    let t1 = v1.len() as f64;
    for (val,cnt) in v.iter().take(6) { print!("  0x{:02x}@{:.0}%", val, *cnt as f64/t1*100.0); }
    println!();

    // _unk00b9 — 14% 0x07 in V0; what pairs with it?
    println!("\n_unk00b9 non-zero paired with _unk00b8 in V0:");
    let mut pairs3: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0 { *pairs3.entry((d[0x00b9], d[0x00b8])).or_insert(0) += 1; }
    let mut pv3: Vec<_> = pairs3.iter().collect(); pv3.sort_by(|a,b| b.1.cmp(a.1));
    for ((b9, b8), c) in pv3.iter().take(8) {
        println!("  _unk00b9=0x{:02x}  _unk00b8=0x{:02x}  count={} ({:.1}%)", b9, b8, c, **c as f64/tot*100.0);
    }

    // _unk009c paired with _unk00b8
    println!("\n_unk009c paired with _unk00b8 in V0 (top 8):");
    let mut pairs4: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0 { *pairs4.entry((d[0x009c], d[0x00b8])).or_insert(0) += 1; }
    let mut pv4: Vec<_> = pairs4.iter().collect(); pv4.sort_by(|a,b| b.1.cmp(a.1));
    for ((c9, b8), c) in pv4.iter().take(10) {
        println!("  _unk009c=0x{:02x}  _unk00b8=0x{:02x}  count={} ({:.1}%)", c9, b8, c, **c as f64/tot*100.0);
    }

    // _unk00cf paired with _unk00b8 in V0
    println!("\n_unk00cf paired with _unk00b8 in V0 (top 6):");
    let mut pairs5: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0 { *pairs5.entry((d[0x00cf], d[0x00b8])).or_insert(0) += 1; }
    let mut pv5: Vec<_> = pairs5.iter().collect(); pv5.sort_by(|a,b| b.1.cmp(a.1));
    for ((cf, b8), c) in pv5.iter().take(6) {
        println!("  _unk00cf=0x{:02x}  _unk00b8=0x{:02x}  count={} ({:.1}%)", cf, b8, c, **c as f64/tot*100.0);
    }
    // Also show V2 _unk00cf
    println!("  (V2 _unk00cf: 0x02@64%, 0x05@34%)  (V3 CONST=0x04)");

    // _unk00bb across V0 and paired with _unk00b8
    println!("\n_unk00bb paired with _unk00b8 in V0 (top 6):");
    let mut pairs6: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0 { *pairs6.entry((d[0x00bb], d[0x00b8])).or_insert(0) += 1; }
    let mut pv6: Vec<_> = pairs6.iter().collect(); pv6.sort_by(|a,b| b.1.cmp(a.1));
    for ((bb, b8), c) in pv6.iter().take(8) {
        println!("  _unk00bb=0x{:02x}  _unk00b8=0x{:02x}  count={} ({:.1}%)", bb, b8, c, **c as f64/tot*100.0);
    }

    // d4=198 — what makes it 198 vs 0 within b8=0x03?
    println!("\n_unk_f32_00d4 non-zero: paired with _unk00bb and _unk00cf:");
    let mut pairs7: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0.iter().filter(|d| r32u(d,0x00d4) != 0) {
        *pairs7.entry((d[0x00bb], d[0x00cf])).or_insert(0) += 1;
    }
    let mut pv7: Vec<_> = pairs7.iter().collect(); pv7.sort_by(|a,b| b.1.cmp(a.1));
    for ((bb, cf), c) in pv7.iter().take(6) {
        println!("  _unk00bb=0x{:02x}  _unk00cf=0x{:02x}  count={}", bb, cf, c);
    }
    let d4_zero_b8_03 = v0.iter().filter(|d| r32u(d,0x00d4)==0 && d[0x00b8]==0x03).count();
    let d4_198_b8_03  = v0.iter().filter(|d| r32u(d,0x00d4)!=0 && d[0x00b8]==0x03).count();
    println!("  b8=0x03: d4=0 count={}, d4=198 count={}", d4_zero_b8_03, d4_198_b8_03);
    // check _unk00bb for these two groups
    let mut bb_d4_0: BTreeMap<u8,u32> = BTreeMap::new();
    let mut bb_d4_198: BTreeMap<u8,u32> = BTreeMap::new();
    for d in v0.iter().filter(|d| d[0x00b8]==0x03) {
        if r32u(d,0x00d4)==0 { *bb_d4_0.entry(d[0x00bb]).or_insert(0) += 1; }
        else { *bb_d4_198.entry(d[0x00bb]).or_insert(0) += 1; }
    }
    let mut bv0: Vec<_> = bb_d4_0.iter().map(|(&k,&c)|(k,c)).collect(); bv0.sort_by(|a,b| b.1.cmp(&a.1));
    let mut bv1: Vec<_> = bb_d4_198.iter().map(|(&k,&c)|(k,c)).collect(); bv1.sort_by(|a,b| b.1.cmp(&a.1));
    print!("  b8=0x03, d4=0: bb top=");
    for (bb,c) in bv0.iter().take(4) { print!("  0x{:02x}({})", bb, c); } println!();
    print!("  b8=0x03, d4=198: bb top=");
    for (bb,c) in bv1.iter().take(4) { print!("  0x{:02x}({})", bb, c); } println!();

    // _unk00c9/cb/cc/ce: when is value 2 vs 1?
    println!("\n_unk00c9 value=2 paired with _unk00b8 in V0:");
    let mut p_c9: BTreeMap<u8,u32> = BTreeMap::new();
    for d in v0.iter().filter(|d| d[0x00c9]==2) { *p_c9.entry(d[0x00b8]).or_insert(0) += 1; }
    let mut v_c9: Vec<_> = p_c9.iter().map(|(&k,&c)|(k,c)).collect(); v_c9.sort_by(|a,b| b.1.cmp(&a.1));
    for (b8,c) in v_c9.iter().take(6) { print!("  b8=0x{:02x}({})", b8, c); } println!();
    let c9_eq_cb = v0.iter().filter(|d| d[0x00c9]==d[0x00cb]).count();
    let c9_eq_cc = v0.iter().filter(|d| d[0x00c9]==d[0x00cc]).count();
    let c9_eq_ce = v0.iter().filter(|d| d[0x00c9]==d[0x00ce]).count();
    let all_same = v0.iter().filter(|d| d[0x00c9]==d[0x00cb] && d[0x00c9]==d[0x00cc] && d[0x00c9]==d[0x00ce]).count();
    println!("  c9==cb: {}/{}, c9==cc: {}/{}, c9==ce: {}/{}, all same: {}/{}", c9_eq_cb, v0.len(), c9_eq_cc, v0.len(), c9_eq_ce, v0.len(), all_same, v0.len());

    // _unk00bb distinct values and their bb/cf relationship
    println!("\n_unk00bb all distinct values in V0:");
    let mut d_bb: BTreeMap<u8,u32> = BTreeMap::new();
    for d in v0 { *d_bb.entry(d[0x00bb]).or_insert(0) += 1; }
    let mut v_bb: Vec<_> = d_bb.iter().map(|(&k,&c)|(k,c)).collect(); v_bb.sort_by(|a,b| b.1.cmp(&a.1));
    for (bb,c) in v_bb.iter() { print!("  0x{:02x}={}", bb, c); } println!();
    // check: is bb always == cf/5? or some other formula?
    let bb_eq_cf_div5 = v0.iter().filter(|d| d[0x00bb] == d[0x00cf]/5).count();
    println!("  bb == cf/5 (integer): {}/{}", bb_eq_cf_div5, v0.len());

    // V1 base region 00c9/cb/cf
    println!("\n_unk00c9 in V1 (first 264 bytes = V0 layout):");
    let mut d_v1: BTreeMap<u8,u32> = BTreeMap::new();
    for r in v1 { *d_v1.entry(r[0x00c9]).or_insert(0) += 1; }
    let mut vv1: Vec<_> = d_v1.iter().map(|(&k,&c)|(k,c)).collect(); vv1.sort_by(|a,b| b.1.cmp(&a.1));
    for (val,cnt) in vv1.iter().take(4) { print!("  {}={}", val, cnt); } println!();

    println!("_unk00cf in V1:");
    let mut d_cf1: BTreeMap<u8,u32> = BTreeMap::new();
    for r in v1 { *d_cf1.entry(r[0x00cf]).or_insert(0) += 1; }
    let mut vcf1: Vec<_> = d_cf1.iter().map(|(&k,&c)|(k,c)).collect(); vcf1.sort_by(|a,b| b.1.cmp(&a.1));
    for (val,cnt) in vcf1.iter().take(4) { print!("  0x{:02x}={}", val, cnt); } println!();

    println!("_unk00bb in V1:");
    let mut d_bb1: BTreeMap<u8,u32> = BTreeMap::new();
    for r in v1 { *d_bb1.entry(r[0x00bb]).or_insert(0) += 1; }
    let mut vbb1: Vec<_> = d_bb1.iter().map(|(&k,&c)|(k,c)).collect(); vbb1.sort_by(|a,b| b.1.cmp(&a.1));
    for (val,cnt) in vbb1.iter().take(4) { print!("  0x{:02x}={}", val, cnt); } println!();

    // Does _unk00bb == repeat_count (0x0045)?
    let bb_eq_rc = v0.iter().filter(|d| d[0x00bb] == d[0x0045]).count();
    println!("\n_unk00bb == repeat_count (0x0045): {}/{}", bb_eq_rc, v0.len());
    // Does _unk00bb == attack_group_index (0x0044)?
    let bb_eq_ag = v0.iter().filter(|d| d[0x00bb] == d[0x0044]).count();
    println!("_unk00bb == attack_group_index (0x0044): {}/{}", bb_eq_ag, v0.len());
    // _unk00bb paired with repeat_count
    println!("_unk00bb vs repeat_count (top 6 pairs):");
    let mut pairs_bb_rc: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0 { *pairs_bb_rc.entry((d[0x00bb], d[0x0045])).or_insert(0) += 1; }
    let mut pvbbrc: Vec<_> = pairs_bb_rc.iter().collect(); pvbbrc.sort_by(|a,b| b.1.cmp(a.1));
    for ((bb,rc),c) in pvbbrc.iter().take(6) {
        println!("  _unk00bb=0x{:02x}  repeat_count={}  count={}", bb, rc, c);
    }
}

fn constcheck_analysis(v0: &[Vec<u8>], v2: &[Vec<u8>], v3: &[Vec<u8>]) {
    println!("\n=== Potential CONST/hardcode candidates ===");
    // Check 0x00e8 across all versions
    for (label, data) in [("V0", v0), ("V2", v2), ("V3", v3)] {
        let mut d: BTreeMap<u8,u32> = BTreeMap::new();
        for r in data { *d.entry(r[0x00e8]).or_insert(0) += 1; }
        let tot = data.len() as f64;
        let mut v: Vec<_> = d.iter().map(|(&k,&c)|(k,c)).collect(); v.sort_by(|a,b| b.1.cmp(&a.1));
        print!("  0x00e8 {}: ", label);
        for (val,cnt) in v.iter().take(3) { print!("0x{:02x}@{:.0}% ", val, *cnt as f64/tot*100.0); }
        println!();
    }
    // Check 0x0072 and 0x0073 in V0 only (V0 records)
    // _unk0072 is u8: 0x00@85%, 0x01@15% — what pairs with 0x01?
    println!("\n_unk0072=1 paired with _unk00b8 in V0:");
    let mut p72: BTreeMap<u8,u32> = BTreeMap::new();
    for d in v0.iter().filter(|d| d[0x0072]==1) { *p72.entry(d[0x00b8]).or_insert(0) += 1; }
    let mut vp72: Vec<_> = p72.iter().map(|(&k,&c)|(k,c)).collect(); vp72.sort_by(|a,b| b.1.cmp(&a.1));
    for (b8,c) in vp72.iter().take(4) { print!("  b8=0x{:02x}({})", b8, c); } println!();
    // and with _unk0073
    println!("_unk0072 vs _unk0073 pairs:");
    let mut p7273: BTreeMap<(u8,u8),u32> = BTreeMap::new();
    for d in v0 { *p7273.entry((d[0x0072], d[0x0073])).or_insert(0) += 1; }
    let mut vp7273: Vec<_> = p7273.iter().collect(); vp7273.sort_by(|a,b| b.1.cmp(a.1));
    for ((a,b),c) in vp7273.iter().take(6) { print!("  ({},{})@{}", a, b, c); } println!();
}
