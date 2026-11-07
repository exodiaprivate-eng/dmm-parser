//! Hand-corrected: typed head + opaque tail for `GlobalGameEvent.pabgb`,
//! game 2.04.08.
//!
//! The 1.x reader decoded `string_key, key u16, is_blocked, group u16,
//! event_desc (8 raw bytes), execute_data_list, execute_data`. On game 2.04
//! `_eventDesc` is a LocalizableString (u8 category + u64 index + CString
//! default; 13 bytes when empty), not 8 raw bytes. Reading it as u64 left the
//! rest of the record one byte out of step:
//! - every 2.04.08 record (191 base + 215 dlc_01) came back with
//!   `execute_data` = raw and an empty `execute_data_list`;
//! - the two records with a non-empty description (dlc_01 16961
//!   Graymane_Party_Hernandian, 16968 Graymane_Party_Pailunese) failed outright
//!   and fell back to a whole-record `_blob_b64`.
//!
//! This struct reads the head as 2.04 writes it and keeps everything after
//! `event_desc` as one `_tail_b64` blob until the 2.04 shape of the
//! execute-data part is decoded. Every record now parses (0 whole-record
//! fallbacks); `event_desc` is editable as text. Every record round-trips byte
//! for byte, and through JSON (tests below).
//!
//! On 2.04.08 the description index is (record key << 32) | code, or a hash in
//! the high half, and `default` is that index in decimal (as other 2.04
//! LocalizableStrings).

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GlobalGameEventInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        pub global_game_event_group_info: u16,
        pub event_desc: LocalizableString<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;

    // A missing fixture is skipped.
    const FIXTURES: &[&str] = &["globalgameevent.pabgb", "globalgameevent.dlc_01.pabgb"];

    fn load(name: &str) -> Option<(Vec<u8>, Vec<u8>)> {
        let data = std::fs::read(crate::testenv::resolve(name)).ok()?;
        let pabgh = std::fs::read(crate::testenv::resolve(&name.replace(".pabgb", ".pabgh"))).ok()?;
        Some((data, pabgh))
    }

    /// pabgh = u16 count + count x (u16 key, u32 offset); records in body order.
    fn offsets(pabgh: &[u8], body_len: usize) -> Vec<(u16, usize, usize)> {
        let count = u16::from_le_bytes([pabgh[0], pabgh[1]]) as usize;
        assert_eq!(2 + count * 6, pabgh.len(), "pabgh = u16 count + count x (u16 key, u32 offset)");
        let mut e: Vec<(u16, usize)> = (0..count)
            .map(|i| {
                let p = 2 + i * 6;
                (
                    u16::from_le_bytes([pabgh[p], pabgh[p + 1]]),
                    u32::from_le_bytes(pabgh[p + 2..p + 6].try_into().unwrap()) as usize,
                )
            })
            .collect();
        e.sort_by_key(|x| x.1);
        let mut out = Vec::new();
        for i in 0..e.len() {
            let end = if i + 1 < e.len() { e[i + 1].1 } else { body_len };
            out.push((e[i].0, e[i].1, end - e[i].1));
        }
        out
    }

    #[test]
    fn roundtrip() {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            let mut out = Vec::with_capacity(data.len());
            let mut json_out = Vec::with_capacity(data.len());
            let mut with_desc = 0;
            for (key, off, size) in offsets(&pabgh, data.len()) {
                let mut o = off;
                let rec = GlobalGameEventInfo::read_with_size(&data, &mut o, size).unwrap();
                assert_eq!(rec.key, key, "{name}: pabgh key");
                if !rec.event_desc.default.data.is_empty() {
                    with_desc += 1;
                }
                rec.write_to(&mut out).unwrap();
                GlobalGameEventInfo::write_from_json_dict(&mut json_out, &rec.to_json_dict()).unwrap();
            }
            assert!(out == data, "{name}: bytes differ after round-trip");
            assert!(json_out == data, "{name}: bytes differ after JSON round-trip");
            eprintln!("{name}: records with an event_desc text: {with_desc}");
        }
    }
}
