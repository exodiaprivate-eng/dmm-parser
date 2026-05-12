//! Roundtrip verification against 1.06 fixtures extracted from live game install.

use dmm_parser::binary::variant::entry_ranges;
use dmm_parser::binary::{BinaryRead, BinaryWrite};
use dmm_parser::tables::dialog_voice_info::DialogVoiceInfo;
use dmm_parser::tables::mercenary_info::MercenaryInfo;
use dmm_parser::tables::reserve_slot_info::ReserveSlotInfo;

const DIR: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_1_06";

fn load_pabgh_offsets(pabgh: &[u8]) -> Option<Vec<(u32, usize)>> {
    if pabgh.len() < 2 { return None; }
    let count = u16::from_le_bytes([pabgh[0], pabgh[1]]) as usize;
    if pabgh.len() < 2 + count * 8 { return None; }
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let base = 2 + i * 8;
        let key = u32::from_le_bytes([pabgh[base], pabgh[base+1], pabgh[base+2], pabgh[base+3]]);
        let off = u32::from_le_bytes([pabgh[base+4], pabgh[base+5], pabgh[base+6], pabgh[base+7]]) as usize;
        entries.push((key, off));
    }
    Some(entries)
}

fn test_mercenary() {
    let path = format!(r"{}\mercenaryinfo.pabgb", DIR);
    let data = match std::fs::read(&path) { Ok(d) => d, Err(_) => { println!("[mercenary] SKIP: {} missing", path); return; } };
    let mut offset = 0;
    let mut items: Vec<MercenaryInfo> = Vec::new();
    let mut n = 0;
    while offset < data.len() {
        let start = offset;
        match MercenaryInfo::read_from(&data, &mut offset) {
            Ok(it) => items.push(it),
            Err(e) => { println!("[mercenary] FAIL record {} (offset {}/{}): {}", n, start, data.len(), e); return; }
        }
        n += 1;
    }
    if offset != data.len() { println!("[mercenary] FAIL: stopped at {}/{}", offset, data.len()); return; }
    let mut out = Vec::with_capacity(data.len());
    for it in &items {
        if let Err(e) = it.write_to(&mut out) { println!("[mercenary] FAIL write: {}", e); return; }
    }
    if out != data {
        let div = out.iter().zip(data.iter()).position(|(a, b)| a != b).unwrap_or(out.len().min(data.len()));
        println!("[mercenary] FAIL byte mismatch at offset {} (records: {}, in: {} bytes, out: {} bytes)", div, items.len(), data.len(), out.len());
        return;
    }
    println!("[mercenary] OK: {} records, {} bytes, byte-identical roundtrip", items.len(), data.len());
}

fn test_dialog_voice() {
    let path = format!(r"{}\dialogvoiceinfo.pabgb", DIR);
    let data = match std::fs::read(&path) { Ok(d) => d, Err(_) => { println!("[dialog_voice] SKIP"); return; } };
    let mut offset = 0;
    let mut items: Vec<DialogVoiceInfo> = Vec::new();
    let mut n = 0;
    while offset < data.len() {
        let start = offset;
        match DialogVoiceInfo::read_from(&data, &mut offset) {
            Ok(it) => items.push(it),
            Err(e) => { println!("[dialog_voice] FAIL record {} (offset {}/{}): {}", n, start, data.len(), e); return; }
        }
        n += 1;
    }
    if offset != data.len() { println!("[dialog_voice] FAIL: stopped at {}/{}", offset, data.len()); return; }
    let mut out = Vec::with_capacity(data.len());
    for it in &items {
        if let Err(e) = it.write_to(&mut out) { println!("[dialog_voice] FAIL write: {}", e); return; }
    }
    if out != data {
        let div = out.iter().zip(data.iter()).position(|(a, b)| a != b).unwrap_or(out.len().min(data.len()));
        println!("[dialog_voice] FAIL byte mismatch at offset {} (records: {}, in: {} bytes, out: {} bytes)", div, items.len(), data.len(), out.len());
        return;
    }
    println!("[dialog_voice] OK: {} records, {} bytes, byte-identical roundtrip", items.len(), data.len());
}

fn test_reserve_slot() {
    let pabgb_path = format!(r"{}\reserveslot.pabgb", DIR);
    let pabgh_path = format!(r"{}\reserveslot.pabgh", DIR);
    let pabgb = match std::fs::read(&pabgb_path) { Ok(d) => d, Err(_) => { println!("[reserve_slot] SKIP pabgb"); return; } };
    let pabgh = match std::fs::read(&pabgh_path) { Ok(d) => d, Err(_) => { println!("[reserve_slot] SKIP pabgh"); return; } };
    let entries = match load_pabgh_offsets(&pabgh) {
        Some(e) => e,
        None => { println!("[reserve_slot] FAIL: bad pabgh"); return; }
    };
    let ranges = entry_ranges(&entries, pabgb.len());
    let mut items: Vec<ReserveSlotInfo> = Vec::new();
    for (i, (k, s, e)) in ranges.iter().enumerate() {
        let mut c = *s;
        match ReserveSlotInfo::read_from(&pabgb, &mut c) {
            Ok(it) => items.push(it),
            Err(er) => { println!("[reserve_slot] FAIL entry {} key=0x{:x} at offset {}: {}", i, k, c, er); return; }
        }
        if c != *e {
            println!("[reserve_slot] FAIL entry {} key=0x{:x}: cursor at {} expected {} (delta {})", i, k, c, e, (c as i64) - (*e as i64));
            return;
        }
    }
    let mut out = Vec::with_capacity(pabgb.len());
    for it in &items {
        if let Err(e) = it.write_to(&mut out) { println!("[reserve_slot] FAIL write: {}", e); return; }
    }
    if out != pabgb {
        let div = out.iter().zip(pabgb.iter()).position(|(a, b)| a != b).unwrap_or(out.len().min(pabgb.len()));
        println!("[reserve_slot] FAIL byte mismatch at offset {} (entries: {}, in: {} bytes, out: {} bytes)", div, items.len(), pabgb.len(), out.len());
        return;
    }
    println!("[reserve_slot] OK: {} entries, {} bytes, byte-identical roundtrip", items.len(), pabgb.len());
}

fn main() {
    println!("=== 1.06 ROUNDTRIP VERIFICATION ===\n");
    test_mercenary();
    test_dialog_voice();
    test_reserve_slot();
}
