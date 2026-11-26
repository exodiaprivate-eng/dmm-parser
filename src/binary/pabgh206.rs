// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Game 2.06 table container (named index).
//!
//! 2.06.00 (build 26949954) moved every table's record name out of the body
//! and into the `.pabgh`, and gave every body record a u32 after its key:
//!
//! ```text
//! 2.05 pabgh  = count + N x { key, u32 offset }
//! 2.05 record = u32 name_len + name + key + data
//!
//! 2.06 pabgh  = count + N x { key, u32 name_len, name, u32 offset, u32 size }
//! 2.06 record = key + u32 row_index + data
//! ```
//!
//! `count` is u16 or u32 and `key` is 1, 2, 4 or 8 bytes, as before. The
//! header entries tile the body exactly, and every record starts with its
//! header key (209/209 tables on 2.06.00).
//!
//! `row_index` is 0..n-1 in every base table (141 of 141). In the `.dlc_01`
//! tables it is strictly increasing but sparse: it looks like the row's index
//! in the merged base + DLC table (INFERRED from the values, not checked in
//! the exe). It is kept per record as the `_row_index` JSON field so a write
//! is byte-exact.
//!
//! `data` is unchanged for every table whose 2.05 layout still holds. So
//! this module translates at the container boundary: a 2.06 pair becomes a
//! 2.05 pair for the existing table readers, and records written by the
//! existing table writers become a 2.06 pair again. Record JSON keeps its
//! name field (`string_key` etc.), so intents that address it still work.

use std::io;

/// JSON field holding a record's 2.06 row index (u32 after the key).
pub const ROW_INDEX_FIELD: &str = "_row_index";
/// JSON field holding the table's key width in bytes (1, 2, 4 or 8).
pub const KEY_WIDTH_FIELD: &str = "_key_width";

/// Longest record name accepted while detecting the layout. Real names are
/// under 200 bytes; the bound only stops a 2.05 index being misread.
const MAX_NAME_LEN: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry206 {
    pub key: u64,
    pub name: Vec<u8>,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pabgh206 {
    /// Width of the leading record count: 2 or 4.
    pub count_width: u8,
    /// Width of every key: 1, 2, 4 or 8.
    pub key_width: u8,
    pub entries: Vec<Entry206>,
}

fn rd_uint(b: &[u8], p: usize, w: usize) -> Option<u64> {
    let s = b.get(p..p + w)?;
    let mut v = 0u64;
    for (i, x) in s.iter().enumerate() {
        v |= (*x as u64) << (8 * i);
    }
    Some(v)
}

fn rd_u32(b: &[u8], p: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(p..p + 4)?.try_into().ok()?))
}

fn put_uint(out: &mut Vec<u8>, v: u64, w: usize) {
    out.extend_from_slice(&v.to_le_bytes()[..w]);
}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

impl Pabgh206 {
    /// Try one (count width, key width) reading. Requires: the index is
    /// consumed exactly, every name is short, and the entries are contiguous
    /// from 0 in index order. With `body`: they also tile it exactly and every
    /// record starts with its key.
    fn try_parse(pabgh: &[u8], body: Option<&[u8]>, cw: usize, kw: usize) -> Option<Self> {
        let n = rd_uint(pabgh, 0, cw)? as usize;
        if n == 0 {
            return None;
        }
        let mut p = cw;
        let mut entries = Vec::with_capacity(n.min(1 << 20));
        let mut next = 0u64;
        for _ in 0..n {
            let key = rd_uint(pabgh, p, kw)?;
            p += kw;
            let len = rd_u32(pabgh, p)? as usize;
            p += 4;
            if len > MAX_NAME_LEN {
                return None;
            }
            let name = pabgh.get(p..p + len)?.to_vec();
            p += len;
            let offset = rd_u32(pabgh, p)?;
            let size = rd_u32(pabgh, p + 4)?;
            p += 8;
            if offset as u64 != next || (size as usize) < kw + 4 {
                return None;
            }
            next += size as u64;
            if let Some(body) = body {
                if rd_uint(body, offset as usize, kw)? != key {
                    return None;
                }
            }
            entries.push(Entry206 { key, name, offset, size });
        }
        if p != pabgh.len() || body.map_or(false, |b| next != b.len() as u64) {
            return None;
        }
        Some(Self { count_width: cw as u8, key_width: kw as u8, entries })
    }

    fn try_all(pabgh: &[u8], body: Option<&[u8]>) -> Option<Self> {
        for cw in [2usize, 4] {
            for kw in [4usize, 2, 8, 1] {
                if let Some(h) = Self::try_parse(pabgh, body, cw, kw) {
                    return Some(h);
                }
            }
        }
        None
    }

    /// Detect a 2.06 index for `body`. `None` = not this layout (a 2.05 or
    /// older index, an empty table, or a pair that does not match).
    pub fn detect(pabgh: &[u8], body: &[u8]) -> Option<Self> {
        Self::try_all(pabgh, Some(body))
    }

    /// Read a 2.06 index without its body (a write that only has the
    /// original index). Same checks as [`Self::detect`] minus the body ones.
    pub fn parse_index(pabgh: &[u8]) -> Option<Self> {
        Self::try_all(pabgh, None)
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let cw = self.count_width as usize;
        let kw = self.key_width as usize;
        let max_n = if cw == 2 { u16::MAX as usize } else { u32::MAX as usize };
        if self.entries.len() > max_n {
            return Err(invalid(format!(
                "2.06 index counts records in {} bits, so it holds at most {}; asked to write {}",
                cw * 8, max_n, self.entries.len())));
        }
        let mut out = Vec::new();
        put_uint(&mut out, self.entries.len() as u64, cw);
        for e in &self.entries {
            put_uint(&mut out, e.key, kw);
            out.extend_from_slice(&(e.name.len() as u32).to_le_bytes());
            out.extend_from_slice(&e.name);
            out.extend_from_slice(&e.offset.to_le_bytes());
            out.extend_from_slice(&e.size.to_le_bytes());
        }
        Ok(out)
    }
}

/// A 2.06 pair rewritten in the 2.05 layout, plus what the rewrite dropped.
pub struct As205 {
    pub pabgh: Vec<u8>,
    pub body: Vec<u8>,
    /// Row index of each record, in index order.
    pub row_index: Vec<u32>,
}

/// Rewrite a 2.06 pair in the 2.05 layout: names go back to the front of
/// each record, the row index is taken out, and the index becomes
/// `count + N x {key, offset}` in the 2.05 format for this key width
/// (u8 / u16 / u32 keys; 8-byte keys = the 12-byte u32 count format, as
/// characterappearanceindexinfo used in 2.05).
pub fn to_205(h: &Pabgh206, body: &[u8]) -> io::Result<As205> {
    use crate::binary::pabgh::{Pabgh, PabghEntry, PabghFormat};
    let kw = h.key_width as usize;
    let mut out = Vec::with_capacity(body.len() + h.entries.len() * 24);
    let mut entries = Vec::with_capacity(h.entries.len());
    let mut rows = Vec::with_capacity(h.entries.len());
    for e in &h.entries {
        let o = e.offset as usize;
        let rec = &body[o..o + e.size as usize];
        rows.push(rd_u32(rec, kw).unwrap());
        let off205 = u32::try_from(out.len())
            .map_err(|_| invalid("2.05 body over 4 GB".into()))?;
        out.extend_from_slice(&(e.name.len() as u32).to_le_bytes());
        out.extend_from_slice(&e.name);
        out.extend_from_slice(&rec[..kw]);
        out.extend_from_slice(&rec[kw + 4..]);
        let (key, extra_bytes) = if kw == 8 {
            ((e.key & 0xFFFF_FFFF) as u32, ((e.key >> 32) as u32).to_le_bytes().to_vec())
        } else {
            (e.key as u32, Vec::new())
        };
        entries.push(PabghEntry { key, offset: off205, extra_bytes });
    }
    let format = match (h.count_width, h.key_width) {
        (2, 4) => Some(PabghFormat::U16CountU32Key),
        (2, 2) => Some(PabghFormat::U16CountU16Key),
        (4, 4) => Some(PabghFormat::U32CountU32Key),
        (2, 1) => Some(PabghFormat::U16CountU8Key),
        (4, 8) => Some(PabghFormat::U32CountU32KeyExtra4),
        _ => None,
    };
    let pabgh = match format {
        Some(format) => Pabgh { format, entries }.to_bytes()?,
        // No 2.05 index format for this width pair (seen: u16 count + 8-byte
        // keys, npcfriendlyinfo.dlc_01, a sequential table that never reads
        // its index). Write count + {key, offset} at the native widths.
        None => {
            let mut b = Vec::new();
            put_uint(&mut b, h.entries.len() as u64, h.count_width as usize);
            for (e, pe) in h.entries.iter().zip(&entries) {
                put_uint(&mut b, e.key, kw);
                b.extend_from_slice(&pe.offset.to_le_bytes());
            }
            b
        }
    };
    Ok(As205 { pabgh, body: out, row_index: rows })
}

/// Tag parsed records with their row index and the key width. `items` must
/// be one record per index entry, in index order.
pub fn annotate(items: &mut [serde_json::Value], h: &Pabgh206, rows: &[u32]) -> io::Result<()> {
    if items.len() != rows.len() {
        return Err(invalid(format!(
            "2.06 table: the reader returned {} records for {} index entries",
            items.len(), rows.len())));
    }
    for (item, row) in items.iter_mut().zip(rows) {
        if let serde_json::Value::Object(m) = item {
            m.insert(ROW_INDEX_FIELD.into(), (*row).into());
            m.insert(KEY_WIDTH_FIELD.into(), h.key_width.into());
        }
    }
    Ok(())
}

/// True when the records carry the 2.06 tags written by [`annotate`].
pub fn items_are_206(items: &[serde_json::Value]) -> bool {
    items.first().map_or(false, |v| v.get(KEY_WIDTH_FIELD).is_some())
}

/// Key width for a write: the caller's index wins, else the records' tag.
pub fn key_width_of(items: &[serde_json::Value], known: Option<u8>) -> io::Result<usize> {
    if let Some(k) = known {
        return Ok(k as usize);
    }
    let k = items
        .iter()
        .find_map(|v| v.get(KEY_WIDTH_FIELD).and_then(|x| x.as_u64()))
        .ok_or_else(|| invalid(format!("2.06 write: no '{}' on any record", KEY_WIDTH_FIELD)))?;
    match k {
        1 | 2 | 4 | 8 => Ok(k as usize),
        _ => Err(invalid(format!("2.06 write: key width {} is not 1, 2, 4 or 8", k))),
    }
}

/// Row index per record for a write.
///
/// A record keeps its `_row_index` when no earlier record already holds that
/// value. A record without one (new) or with a taken one (a clone of an
/// existing record) gets the next free value above the table's highest.
/// Removed records leave their value unused; nothing is renumbered.
///
/// For a base table (indices 0..n-1) that makes an appended record n, n+1..,
/// which is its position. For a `.dlc_01` table the indices are the merged
/// base + DLC row numbers (INFERRED), and a new record goes after the highest
/// one in this file: whether the game then wants base-table additions
/// counted in is NOT known.
pub fn assign_row_index(items: &[serde_json::Value]) -> Vec<u32> {
    assign_rows(
        items
            .iter()
            .map(|v| v.get(ROW_INDEX_FIELD).and_then(|x| x.as_u64()).map(|x| x as u32))
            .collect(),
    )
}

/// [`assign_row_index`] on plain values: `given[i]` = record i's row index
/// if it has one.
pub fn assign_rows(given: Vec<Option<u32>>) -> Vec<u32> {
    let mut next = given.iter().flatten().max().map_or(0, |m| m.saturating_add(1));
    let mut seen = std::collections::HashSet::with_capacity(given.len());
    given
        .into_iter()
        .map(|g| match g {
            Some(x) if seen.insert(x) => x,
            _ => {
                let x = next;
                next = next.saturating_add(1);
                seen.insert(x);
                x
            }
        })
        .collect()
}

/// Turn records written in the 2.05 layout (one `u32 name_len + name + key +
/// data` record per entry of `recs`) into a 2.06 body and index.
pub fn from_205_records(
    recs: &[Vec<u8>],
    rows: &[u32],
    count_width: u8,
    key_width: usize,
) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let mut body = Vec::with_capacity(recs.iter().map(|r| r.len()).sum());
    let mut entries = Vec::with_capacity(recs.len());
    for (i, (rec, row)) in recs.iter().zip(rows).enumerate() {
        let len = rd_u32(rec, 0)
            .ok_or_else(|| invalid(format!("2.06 write: record {} shorter than its name length", i)))?
            as usize;
        let name_end = 4 + len;
        let key_end = name_end + key_width;
        let key = rd_uint(rec, name_end, key_width).ok_or_else(|| invalid(format!(
            "2.06 write: record {} ({} bytes) ends inside its name or key", i, rec.len())))?;
        let offset = u32::try_from(body.len()).map_err(|_| invalid("2.06 body over 4 GB".into()))?;
        body.extend_from_slice(&rec[name_end..key_end]);
        body.extend_from_slice(&row.to_le_bytes());
        body.extend_from_slice(&rec[key_end..]);
        let size = (rec.len() - 4 - len + 4) as u32;
        entries.push(Entry206 { key, name: rec[4..name_end].to_vec(), offset, size });
    }
    let pabgh = Pabgh206 { count_width, key_width: key_width as u8, entries }.to_bytes()?;
    Ok((body, pabgh))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (Vec<u8>, Vec<u8>) {
        // Two records, u16 count, u32 keys: "ab" key 7 row 0 data [1,2];
        // "" key 9 row 1 data [].
        let mut body = Vec::new();
        body.extend_from_slice(&7u32.to_le_bytes());
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&[1, 2]);
        body.extend_from_slice(&9u32.to_le_bytes());
        body.extend_from_slice(&1u32.to_le_bytes());
        let h = Pabgh206 {
            count_width: 2,
            key_width: 4,
            entries: vec![
                Entry206 { key: 7, name: b"ab".to_vec(), offset: 0, size: 10 },
                Entry206 { key: 9, name: Vec::new(), offset: 10, size: 8 },
            ],
        };
        (h.to_bytes().unwrap(), body)
    }

    #[test]
    fn detect_convert_and_back() {
        let (ph, body) = pair();
        let h = Pabgh206::detect(&ph, &body).expect("2.06 detected");
        assert_eq!((h.count_width, h.key_width, h.entries.len()), (2, 4, 2));
        let a = to_205(&h, &body).unwrap();
        assert_eq!(a.row_index, vec![0, 1]);
        // 2.05 record 0 = len 2 + "ab" + key 7 + [1,2]
        assert_eq!(&a.body[..12], &[2, 0, 0, 0, b'a', b'b', 7, 0, 0, 0, 1, 2]);
        let old = crate::binary::pabgh::Pabgh::parse(&a.pabgh).unwrap();
        assert_eq!(old.entries[1].offset, 12);
        let recs = vec![a.body[..12].to_vec(), a.body[12..].to_vec()];
        let (b2, h2) = from_205_records(&recs, &a.row_index, 2, 4).unwrap();
        assert_eq!(b2, body);
        assert_eq!(h2, ph);
    }

    #[test]
    fn old_index_is_not_206() {
        // u16 count 1 + {u32 key 7, u32 offset 0} over a 2.05 record.
        let ph = [1u8, 0, 7, 0, 0, 0, 0, 0, 0, 0];
        let body = [2u8, 0, 0, 0, b'a', b'b', 7, 0, 0, 0];
        assert!(Pabgh206::detect(&ph, &body).is_none());
    }

    /// Every 2.06 pair in `DMM_PARSER_PABGB_DIR`: parse -> write must give
    /// back both files byte for byte, through the JSON API and through the
    /// intent pipeline (no intents). Prints one line per table and the totals;
    /// `P206_REPORT=<file>` also writes them as JSON, `P206_OUT205=<dir>`
    /// writes each pair in the 2.05 layout (for the per-table reader tests),
    /// `P206_STRICT=1` fails on any table that is not byte-exact.
    #[test]
    fn gate_206_fixture_round_trip() {
        use crate::dispatch::{apply_intents_to_table_body, normalize_target_name,
                              parse_table_to_json, serialize_table_from_json_with_pabgh};
        let Ok(dir) = std::env::var("DMM_PARSER_PABGB_DIR") else {
            eprintln!("SKIP: DMM_PARSER_PABGB_DIR not set");
            return;
        };
        let out205 = std::env::var("P206_OUT205").ok();
        let mut names: Vec<String> = std::fs::read_dir(&dir).unwrap()
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter_map(|n| n.strip_suffix(".pabgh").map(str::to_string))
            .collect();
        names.sort();
        let mut report = serde_json::Map::new();
        let mut totals = std::collections::BTreeMap::<String, usize>::new();
        for stem in &names {
            let Ok(body) = std::fs::read(format!("{}/{}.pabgb", dir, stem)) else { continue };
            let ph = std::fs::read(format!("{}/{}.pabgh", dir, stem)).unwrap();
            let base = stem.strip_suffix(".dlc_01").unwrap_or(stem);
            let table = normalize_target_name(base).unwrap_or(base).to_string();
            let verdict: String = (|| {
                let Some(h) = Pabgh206::detect(&ph, &body) else {
                    // Older pair: report its blob fallback count for comparison.
                    return match parse_table_to_json(&table, &body, Some(&ph)) {
                        Ok(v) => format!("not_206 fallback {}",
                                         v.iter().filter(|x| x.get("_blob_b64").is_some()).count()),
                        Err(e) => format!("not_206 parse_error: {}", e).chars().take(120).collect(),
                    };
                };
                if let Some(d) = &out205 {
                    if let Ok(a) = to_205(&h, &body) {
                        let _ = std::fs::write(format!("{}/{}.pabgb", d, stem), &a.body);
                        let _ = std::fs::write(format!("{}/{}.pabgh", d, stem), &a.pabgh);
                    }
                }
                let items = match parse_table_to_json(&table, &body, Some(&ph)) {
                    Ok(v) => v,
                    Err(e) => return format!("parse_error: {}", e).chars().take(200).collect(),
                };
                let fallback = items.iter().filter(|v| v.get("_blob_b64").is_some()).count();
                match serialize_table_from_json_with_pabgh(&table, &items, &ph) {
                    Ok((b2, h2)) if b2 == body && h2 == ph => {}
                    Ok((b2, h2)) => return format!("mismatch body {} header {}",
                                                   b2 == body, h2 == ph),
                    Err(e) => return format!("serialize_error: {}", e).chars().take(200).collect(),
                }
                match apply_intents_to_table_body(&table, &body, Some(&ph), &[]) {
                    Ok((b3, Some(h3), _)) if b3 == body && h3 == ph => {}
                    Ok(_) => return "apply_mismatch".into(),
                    Err(e) => return format!("apply_error: {}", e).chars().take(200).collect(),
                }
                if fallback > 0 { format!("roundtrip_ok_blob_fallback {}", fallback) }
                else { "roundtrip_ok".into() }
            })();
            let kind = verdict.split([':', ' ']).next().unwrap().to_string();
            *totals.entry(kind).or_default() += 1;
            println!("P206 {:<44} {}", stem, verdict);
            report.insert(stem.clone(), serde_json::json!({"table": table, "v": verdict}));
        }
        println!("P206 TOTALS {:?} of {}", totals, names.len());
        if let Ok(p) = std::env::var("P206_REPORT") {
            let j = serde_json::json!({"totals": totals, "tables": report});
            std::fs::write(p, serde_json::to_string_pretty(&j).unwrap()).unwrap();
        }
        if std::env::var("P206_STRICT").is_ok() {
            let ok = totals.iter().filter(|(k, _)| k.starts_with("roundtrip_ok")).map(|(_, n)| n).sum::<usize>();
            assert_eq!(ok, names.len(), "P206 gate: not every table is byte-exact");
        }
    }

    #[test]
    fn row_index_keeps_unique_and_numbers_new_and_clones() {
        use serde_json::json;
        let items = vec![
            json!({"_row_index": 0}), json!({"_row_index": 5}),
            json!({"_row_index": 5}), json!({}), json!({"_row_index": 2}),
        ];
        assert_eq!(assign_row_index(&items), vec![0, 5, 6, 7, 2]);
    }
}
