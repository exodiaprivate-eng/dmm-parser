//! Hand-corrected: fully typed `GlobalGameEvent.pabgb`, game 2.04.08.
//!
//! Record, in 2.04 wire order (decoded from the 2.04.08 data: 191 base +
//! 217 dlc_01 records, every byte typed, 0 fallbacks):
//!   1. CString string_key                (_stringKey; 2.04 writes it first)
//!   2. u16 key                           (_key, pabgh format 2)
//!   3. u8 is_blocked                     (_isBlocked)
//!   4. u16 global_game_event_group_info  (_globalGameEventGroupInfo)
//!   5. LocalizableString event_desc      (_eventDesc; 13 bytes when empty)
//!   6. u32 ui_icon_path                  (_uiIconPath, a path hash: 3 values;
//!                                         0x1F8AF380 on 405 records)
//!   7. CArray<u16> target_region_info_list (_targetRegionInfoList; empty in
//!                                         every 2.04.08 record, element type
//!                                         from the 1.x schema)
//!   8. CArray<GlobalGameEventConditionEntry> execute_data_list (13-byte
//!      entries; `condition_info` = conditioninfo key)
//!   9. GlobalGameEventExecuteData execute_data (u8 presence, u8 sub_tag,
//!      2.04 body: see `binary::variants::global_game_event_execute_data`)
//!
//! History: the 1.x reader read `_eventDesc` as 8 raw bytes, which left 2.04
//! records one byte out of step (all 408 had `execute_data` = raw). The D8b
//! change read the head correctly and kept fields 6-9 as `_tail_b64`; this
//! version types them.

use crate::binary::variants::global_game_event_execute_data::GlobalGameEventExecuteData;
use crate::binary::*;
use crate::json_traits::{get_field as json_get_field, ToJsonValue, WriteJsonValue};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// One element of `execute_data_list` (1.x reader `sub_1410F8F00`, 13
    /// bytes). `condition_info` is a conditioninfo key (e.g. 1000064
    /// Condition_MinusMaxPrice2 on RoyalSupply_Her, 1008514
    /// CompleteQuestTime(...) on FactionBlockEvent_Her_Node_Her_PervinFort,
    /// 1015378 CheckFactionManagementOpen(...) on
    /// ManagementSpecialEvent_Vellua_Pub_Maegu). `flag`, `lookup_b` and
    /// `lookup_c` are 0 in all 336 entries on 2.04.08.
    pub struct GlobalGameEventConditionEntry {
        pub condition_info: u32,
        pub flag: u8,
        pub lookup_b: u32,
        pub lookup_c: u32,
    }
}

#[derive(Debug)]
pub struct GlobalGameEventInfo<'a> {
    pub string_key: CString<'a>,
    pub key: u16,
    pub is_blocked: u8,
    pub global_game_event_group_info: u16,
    pub event_desc: LocalizableString<'a>,
    pub ui_icon_path: u32,
    pub target_region_info_list: CArray<u16>,
    pub execute_data_list: CArray<GlobalGameEventConditionEntry>,
    pub execute_data: GlobalGameEventExecuteData<'a>,
}

impl<'a> GlobalGameEventInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let entry_end = *offset + entry_size;
        if entry_end > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "GlobalGameEventInfo: entry past end of data"));
        }
        // Bound every read to this entry.
        let entry = &data[..entry_end];
        let string_key = CString::read_from(entry, offset)?;
        let key = u16::read_from(entry, offset)?;
        let is_blocked = u8::read_from(entry, offset)?;
        let global_game_event_group_info = u16::read_from(entry, offset)?;
        let event_desc = LocalizableString::read_from(entry, offset)?;
        let ui_icon_path = u32::read_from(entry, offset)?;
        let target_region_info_list = <CArray<u16>>::read_from(entry, offset)?;
        let execute_data_list = <CArray<GlobalGameEventConditionEntry>>::read_from(entry, offset)?;
        // Reads to the entry end; anything it cannot type stays `Raw`.
        let execute_data = GlobalGameEventExecuteData::read_from(entry, offset)?;
        Ok(Self {
            string_key,
            key,
            is_blocked,
            global_game_event_group_info,
            event_desc,
            ui_icon_path,
            target_region_info_list,
            execute_data_list,
            execute_data,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.string_key.write_to(w)?;
        self.key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.global_game_event_group_info.write_to(w)?;
        self.event_desc.write_to(w)?;
        self.ui_icon_path.write_to(w)?;
        self.target_region_info_list.write_to(w)?;
        self.execute_data_list.write_to(w)?;
        self.execute_data.write_to(w)
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("string_key".into(), self.string_key.to_json_value());
        m.insert("key".into(), self.key.to_json_value());
        m.insert("is_blocked".into(), self.is_blocked.to_json_value());
        m.insert("global_game_event_group_info".into(), self.global_game_event_group_info.to_json_value());
        m.insert("event_desc".into(), self.event_desc.to_json_value());
        m.insert("ui_icon_path".into(), self.ui_icon_path.to_json_value());
        m.insert("target_region_info_list".into(), self.target_region_info_list.to_json_value());
        m.insert("execute_data_list".into(), self.execute_data_list.to_json_value());
        m.insert("execute_data".into(), self.execute_data.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "global_game_event_group_info")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "event_desc")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ui_icon_path")?)?;
        <CArray<u16> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "target_region_info_list")?)?;
        <CArray<GlobalGameEventConditionEntry> as WriteJsonValue>::write_from_json(
            w,
            json_get_field(obj, "execute_data_list")?,
        )?;
        GlobalGameEventExecuteData::write_from_json(w, json_get_field(obj, "execute_data")?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variants::global_game_event_execute_data::GlobalGameEventExecuteDataBody as B;

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

    fn each(mut f: impl FnMut(&str, &GlobalGameEventInfo)) {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            for (key, off, size) in offsets(&pabgh, data.len()) {
                let mut o = off;
                let rec = GlobalGameEventInfo::read_with_size(&data, &mut o, size).unwrap();
                assert_eq!(o, off + size, "{name} {key}: read did not end on the record end");
                assert_eq!(rec.key, key, "{name}: pabgh key");
                f(name, &rec);
            }
        }
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
            for (_key, off, size) in offsets(&pabgh, data.len()) {
                let mut o = off;
                let rec = GlobalGameEventInfo::read_with_size(&data, &mut o, size).unwrap();
                rec.write_to(&mut out).unwrap();
                GlobalGameEventInfo::write_from_json_dict(&mut json_out, &rec.to_json_dict()).unwrap();
            }
            assert!(out == data, "{name}: bytes differ after round-trip");
            assert!(json_out == data, "{name}: bytes differ after JSON round-trip");
        }
    }

    /// Every record is typed to the last byte: no raw execute_data, no
    /// base64 anywhere in the JSON, and the body head tag matches sub_tag.
    #[test]
    fn fully_typed() {
        let mut n = 0;
        each(|name, rec| {
            n += 1;
            assert!(!rec.execute_data.is_raw(), "{name} {}: execute_data raw", rec.key);
            let js = Value::Object(rec.to_json_dict()).to_string();
            assert!(!js.contains("_b64"), "{name} {}: base64 left in JSON", rec.key);
            let v = rec.execute_data.to_json_value();
            assert_eq!(v["body"]["body_tag"], v["sub_tag"], "{name} {}: body_tag != sub_tag", rec.key);
        });
        eprintln!("records checked: {n}");
    }

    /// The sub_tag families line up with the record names, and the texts
    /// carry the record key in the high half of the index.
    #[test]
    fn named_fields_match_record_names() {
        use crate::binary::variants::global_game_event_execute_data::GlobalGameEventExecuteData as E;
        let mut building_group = std::collections::HashMap::new();
        each(|name, rec| {
            let sk = rec.string_key.data;
            let E::Present(body) = &rec.execute_data else { panic!("{name} {sk}: not present") };
            let text = |t: &LocalizableString| {
                if t.index != 0 {
                    assert_eq!((t.index >> 32) as u16, rec.key, "{name} {sk}: text index high half != key");
                    assert_eq!(t.default.data, t.index.to_string(), "{name} {sk}: default != index");
                }
            };
            match body {
                B::VaryTradeItemPrice(p) => {
                    text(&p.text_1);
                    assert_eq!(p.description.index, p.text_1.index, "{name} {sk}: description != text_1");
                    for e in &p.price_list.items {
                        assert!(e.min_price <= e.max_price, "{name} {sk}: min > max");
                    }
                }
                B::OpenRoyalSupply(_) => assert!(sk.starts_with("RoyalSupply_"), "{sk}"),
                B::FactionBlock(_) => assert!(sk.starts_with("FactionBlockEvent_"), "{sk}"),
                B::PartyHashList(_) | B::PartyKeyList(_) => assert!(sk.starts_with("Graymane_Party_"), "{sk}"),
                B::ManagementSpecialEvent(p) => {
                    assert!(sk.starts_with("ManagementSpecialEvent_"), "{sk}");
                    text(&p.text_1);
                    let g = building_group
                        .entry(p.management_building_key)
                        .or_insert(rec.global_game_event_group_info);
                    assert_eq!(*g, rec.global_game_event_group_info, "{sk}: building key in two groups");
                }
            }
        });
    }
    /// Negative control: an unknown sub_tag keeps the bytes as `Raw` and
    /// still writes back identically.
    #[test]
    fn unknown_sub_tag_falls_back_to_raw() {
        let Some((data, pabgh)) = load(FIXTURES[0]) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let (_, off, size) = offsets(&pabgh, data.len())[0];
        let mut o = off;
        let rec = GlobalGameEventInfo::read_with_size(&data, &mut o, size).unwrap();
        let mut ex = Vec::new();
        rec.execute_data.write_to(&mut ex).unwrap();
        let sub_tag_at = off + size - ex.len() + 1;
        let mut bad = data.clone();
        bad[sub_tag_at] = 9;
        let mut o = off;
        let rec2 = GlobalGameEventInfo::read_with_size(&bad, &mut o, size).unwrap();
        assert!(rec2.execute_data.is_raw(), "unknown sub_tag must give Raw");
        let mut out = Vec::new();
        rec2.write_to(&mut out).unwrap();
        assert!(out == bad[off..off + size], "Raw must write back the same bytes");
    }
}
