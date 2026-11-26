// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Game 2.06 moved every record's name out of the body and into the index.
//!
//! | | 2.05 and older | 2.06 |
//! |---|---|---|
//! | `.pabgh` entry | `prefix, u32 offset` | `prefix, CString name, u32 offset, u32 size` |
//! | `.pabgb` record | `CString name, prefix, rest` | `prefix, u32 staticInfoIndex, rest` |
//!
//! The count width (u8/u16/u32) and the prefix (the u8/u16/u32 key, or the 8-byte /
//! 12-byte prefixes of characterappearanceindexinfo / aieventtableinfo) did not change, and
//! the body record starts with the WHOLE prefix. Sizes tile the body in index order.
//!
//! `staticInfoIndex` is the record's ordinal in a base table. In a DLC copy it is the record's
//! position in the game's merged base+DLC table: increasing in file order, but not derivable
//! from the DLC file alone, so [`LegacySidecar`] keeps it by prefix.
//!
//! [`to_legacy`] turns a 2.06 pair into the 2.05 layout every table reader (and DMM) was
//! written for; [`from_legacy`] turns it back. A 2.06 pair round-trips byte-identically, and a
//! 2.05.01-vs-2.06.00 check gave the old bytes exactly for every table whose content did not
//! change (72 base + 15 DLC tables).

use std::collections::HashMap;
use std::io;

fn bad(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

fn read_uint(b: &[u8], width: usize) -> u64 {
    let mut v = 0u64;
    for (i, x) in b[..width].iter().enumerate() {
        v |= (*x as u64) << (8 * i);
    }
    v
}

fn push_uint(out: &mut Vec<u8>, v: u64, width: usize) {
    for i in 0..width {
        out.push((v >> (8 * i)) as u8);
    }
}

fn u32_at(b: &[u8], p: usize) -> Option<u32> {
    b.get(p..p + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}

/// Count widths and prefix widths seen in retail data, in the order they are tried.
const COUNT_WIDTHS: [usize; 3] = [2, 4, 1];
const PREFIX_WIDTHS: [usize; 5] = [4, 2, 1, 8, 12];

/// One 2.06 index entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedEntry {
    pub prefix: Vec<u8>,
    pub name: Vec<u8>,
    pub offset: u32,
    pub size: u32,
}

/// A parsed 2.06 index (`.pabgh`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedIndex {
    pub count_width: usize,
    pub prefix_width: usize,
    pub entries: Vec<NamedEntry>,
}

fn parse_with(h: &[u8], cw: usize, pw: usize) -> Option<Vec<NamedEntry>> {
    if h.len() < cw {
        return None;
    }
    let n = read_uint(h, cw) as usize;
    let mut p = cw;
    let mut out = Vec::with_capacity(n.min(1 << 20));
    for _ in 0..n {
        let prefix = h.get(p..p + pw)?.to_vec();
        p += pw;
        let len = u32_at(h, p)? as usize;
        p += 4;
        if len > 1 << 16 {
            return None;
        }
        let name = h.get(p..p + len)?.to_vec();
        p += len;
        let offset = u32_at(h, p)?;
        let size = u32_at(h, p + 4)?;
        p += 8;
        out.push(NamedEntry { prefix, name, offset, size });
    }
    (p == h.len()).then_some(out)
}

impl NamedIndex {
    /// Parse a 2.06 index, checked against its body: the sizes tile the body in index order
    /// and every record starts with its entry's prefix. `None` = not a 2.06 pair (a 2.05
    /// index never passes: it has no names and no sizes).
    pub fn parse(pabgh: &[u8], pabgb: &[u8]) -> Option<Self> {
        for cw in COUNT_WIDTHS {
            for pw in PREFIX_WIDTHS {
                let Some(entries) = parse_with(pabgh, cw, pw) else { continue };
                let mut pos = 0usize;
                let ok = entries.iter().all(|e| {
                    let fits = e.offset as usize == pos
                        && (e.size as usize) >= pw + 4
                        && pabgb.get(pos..pos + pw) == Some(&e.prefix[..]);
                    pos += e.size as usize;
                    fits
                });
                if ok && pos == pabgb.len() {
                    return Some(Self { count_width: cw, prefix_width: pw, entries });
                }
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        push_uint(&mut out, self.entries.len() as u64, self.count_width);
        for e in &self.entries {
            out.extend_from_slice(&e.prefix);
            out.extend_from_slice(&(e.name.len() as u32).to_le_bytes());
            out.extend_from_slice(&e.name);
            out.extend_from_slice(&e.offset.to_le_bytes());
            out.extend_from_slice(&e.size.to_le_bytes());
        }
        out
    }
}

/// True when `pabgh`/`pabgb` is a 2.06 pair (names in the index). An empty table (count 0,
/// empty body) is the same in both layouts and reports false.
pub fn is_named_pair(pabgh: &[u8], pabgb: &[u8]) -> bool {
    !pabgb.is_empty() && NamedIndex::parse(pabgh, pabgb).is_some()
}

/// What [`from_legacy`] needs to rebuild the 2.06 pair: the widths, and each record's
/// `staticInfoIndex` by prefix.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LegacySidecar {
    pub count_width: usize,
    pub prefix_width: usize,
    /// True when every record's index equals its ordinal (every base table). Then the index is
    /// recomputed on write, so an added or removed record keeps the numbering right.
    pub ordinal: bool,
    /// `staticInfoIndex` by prefix (used when `ordinal` is false: the DLC copies).
    pub index_by_prefix: HashMap<Vec<u8>, u32>,
}

const SIDECAR_MAGIC: &[u8; 8] = b"DMMLEG01";

impl LegacySidecar {
    /// Compact bytes, to keep next to a cached legacy copy.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = SIDECAR_MAGIC.to_vec();
        out.push(self.count_width as u8);
        out.push(self.prefix_width as u8);
        out.push(self.ordinal as u8);
        let mut rows: Vec<_> = self.index_by_prefix.iter().collect();
        rows.sort_by_key(|(_, idx)| **idx);
        out.extend_from_slice(&(rows.len() as u32).to_le_bytes());
        for (prefix, idx) in rows {
            out.extend_from_slice(prefix);
            out.extend_from_slice(&idx.to_le_bytes());
        }
        out
    }

    pub fn from_bytes(b: &[u8]) -> io::Result<Self> {
        if b.len() < 15 || &b[..8] != SIDECAR_MAGIC {
            return Err(bad("not a DMM legacy sidecar".into()));
        }
        let (cw, pw, ordinal) = (b[8] as usize, b[9] as usize, b[10] != 0);
        let n = u32_at(b, 11).unwrap() as usize;
        let mut p = 15;
        let mut index_by_prefix = HashMap::with_capacity(n);
        for _ in 0..n {
            let prefix = b.get(p..p + pw).ok_or_else(|| bad("sidecar truncated".into()))?.to_vec();
            let idx = u32_at(b, p + pw).ok_or_else(|| bad("sidecar truncated".into()))?;
            index_by_prefix.insert(prefix, idx);
            p += pw + 4;
        }
        if p != b.len() {
            return Err(bad("sidecar has trailing bytes".into()));
        }
        Ok(Self { count_width: cw, prefix_width: pw, ordinal, index_by_prefix })
    }
}

/// A 2.06 pair -> the 2.05 layout (`CString name, prefix, rest` records; `prefix, u32 offset`
/// index entries), plus the sidecar [`from_legacy`] needs. Errors when the pair is not 2.06.
pub fn to_legacy(pabgh: &[u8], pabgb: &[u8]) -> io::Result<(Vec<u8>, Vec<u8>, LegacySidecar)> {
    let idx = NamedIndex::parse(pabgh, pabgb)
        .ok_or_else(|| bad("not a game 2.06 table pair (no names in the index)".into()))?;
    let pw = idx.prefix_width;
    let mut head = Vec::with_capacity(idx.count_width + idx.entries.len() * (pw + 4));
    push_uint(&mut head, idx.entries.len() as u64, idx.count_width);
    let mut body = Vec::with_capacity(pabgb.len() + idx.entries.len() * 16);
    let mut index_by_prefix = HashMap::with_capacity(idx.entries.len());
    let mut ordinal = true;
    for (i, e) in idx.entries.iter().enumerate() {
        let rec = &pabgb[e.offset as usize..(e.offset + e.size) as usize];
        let sii = u32_at(rec, pw).unwrap();
        ordinal &= sii as usize == i;
        index_by_prefix.insert(e.prefix.clone(), sii);
        head.extend_from_slice(&e.prefix);
        head.extend_from_slice(&(body.len() as u32).to_le_bytes());
        body.extend_from_slice(&(e.name.len() as u32).to_le_bytes());
        body.extend_from_slice(&e.name);
        body.extend_from_slice(&e.prefix);
        body.extend_from_slice(&rec[pw + 4..]);
    }
    let sidecar = LegacySidecar {
        count_width: idx.count_width,
        prefix_width: pw,
        ordinal,
        index_by_prefix: if ordinal { HashMap::new() } else { index_by_prefix },
    };
    Ok((head, body, sidecar))
}

/// The 2.05-layout pair (as DMM edited it) -> the 2.06 layout the game reads.
///
/// Records are written in index order. `staticInfoIndex` = the ordinal for a base table; for a
/// DLC copy the sidecar's value by prefix, and a record the sidecar does not know (added by a
/// mod) takes the next number after the highest one.
pub fn from_legacy(
    legacy_pabgh: &[u8],
    legacy_pabgb: &[u8],
    sidecar: &LegacySidecar,
) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let (cw, pw) = (sidecar.count_width, sidecar.prefix_width);
    if legacy_pabgh.len() < cw {
        return Err(bad("legacy index shorter than its count".into()));
    }
    let n = read_uint(legacy_pabgh, cw) as usize;
    if legacy_pabgh.len() != cw + n * (pw + 4) {
        return Err(bad(format!(
            "legacy index is {} bytes, expected {} for {n} entries of {} bytes",
            legacy_pabgh.len(),
            cw + n * (pw + 4),
            pw + 4
        )));
    }
    let entries: Vec<(&[u8], usize)> = (0..n)
        .map(|i| {
            let p = cw + i * (pw + 4);
            (&legacy_pabgh[p..p + pw], u32_at(legacy_pabgh, p + pw).unwrap() as usize)
        })
        .collect();
    // A record ends where the next record (by offset) starts.
    let mut starts: Vec<usize> = entries.iter().map(|e| e.1).collect();
    starts.sort_unstable();
    starts.dedup();
    let end_of = |off: usize| -> usize {
        match starts.binary_search(&off) {
            Ok(i) if i + 1 < starts.len() => starts[i + 1],
            _ => legacy_pabgb.len(),
        }
    };
    let mut next_new = sidecar.index_by_prefix.values().max().map_or(0, |m| m + 1);
    let mut head = Vec::with_capacity(legacy_pabgh.len() + n * 40);
    let mut body = Vec::with_capacity(legacy_pabgb.len());
    push_uint(&mut head, n as u64, cw);
    for (i, (prefix, off)) in entries.iter().enumerate() {
        let end = end_of(*off);
        let rec = legacy_pabgb
            .get(*off..end)
            .ok_or_else(|| bad(format!("record {i} at {off} is outside the body")))?;
        let len = u32_at(rec, 0).ok_or_else(|| bad(format!("record {i} too short for a name")))? as usize;
        let name = rec.get(4..4 + len).ok_or_else(|| bad(format!("record {i}: name runs past the record")))?;
        let after = 4 + len;
        if rec.get(after..after + pw) != Some(*prefix) {
            return Err(bad(format!(
                "record {i}: the key after the name does not match the index entry ({:02x?})",
                prefix
            )));
        }
        let sii = if sidecar.ordinal {
            i as u32
        } else if let Some(v) = sidecar.index_by_prefix.get(*prefix) {
            *v
        } else {
            next_new += 1;
            next_new - 1
        };
        let start = body.len() as u32;
        body.extend_from_slice(prefix);
        body.extend_from_slice(&sii.to_le_bytes());
        body.extend_from_slice(&rec[after + pw..]);
        head.extend_from_slice(prefix);
        head.extend_from_slice(&(name.len() as u32).to_le_bytes());
        head.extend_from_slice(name);
        head.extend_from_slice(&start.to_le_bytes());
        head.extend_from_slice(&(body.len() as u32 - start).to_le_bytes());
    }
    Ok((head, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named_pair(names: &[&str], keys: &[u16], idx: &[u32]) -> (Vec<u8>, Vec<u8>) {
        let mut h = (names.len() as u16).to_le_bytes().to_vec();
        let mut b = Vec::new();
        for ((n, k), s) in names.iter().zip(keys).zip(idx) {
            let start = b.len() as u32;
            b.extend_from_slice(&k.to_le_bytes());
            b.extend_from_slice(&s.to_le_bytes());
            b.extend_from_slice(&[7, 7, 7]);
            h.extend_from_slice(&k.to_le_bytes());
            h.extend_from_slice(&(n.len() as u32).to_le_bytes());
            h.extend_from_slice(n.as_bytes());
            h.extend_from_slice(&start.to_le_bytes());
            h.extend_from_slice(&(b.len() as u32 - start).to_le_bytes());
        }
        (h, b)
    }

    #[test]
    fn base_pair_round_trips_and_numbers_by_ordinal() {
        let (h, b) = named_pair(&["Alpha", "Be"], &[10, 11], &[0, 1]);
        let (lh, lb, sc) = to_legacy(&h, &b).unwrap();
        assert!(sc.ordinal && sc.index_by_prefix.is_empty());
        assert_eq!(lh, [2, 0, 10, 0, 0, 0, 0, 0, 11, 0, 14, 0, 0, 0]);
        assert_eq!(&lb[..9], b"\x05\0\0\0Alpha");
        assert_eq!(from_legacy(&lh, &lb, &sc).unwrap(), (h, b));
    }

    #[test]
    fn dlc_pair_keeps_its_indexes_and_numbers_an_added_record_after_them() {
        let (h, b) = named_pair(&["A", "B"], &[1, 2], &[5, 9]);
        let (mut lh, mut lb, sc) = to_legacy(&h, &b).unwrap();
        assert!(!sc.ordinal);
        assert_eq!(from_legacy(&lh, &lb, &sc).unwrap(), (h, b));
        // a mod appends record key 3
        let off = lb.len() as u32;
        lb.extend_from_slice(b"\x01\0\0\0C\x03\0\x07\x07\x07");
        lh[0] = 3;
        lh.extend_from_slice(&3u16.to_le_bytes());
        lh.extend_from_slice(&off.to_le_bytes());
        let (nh, nb) = from_legacy(&lh, &lb, &sc).unwrap();
        let idx = NamedIndex::parse(&nh, &nb).unwrap();
        assert_eq!(idx.entries[2].name, b"C");
        assert_eq!(u32_at(&nb, idx.entries[2].offset as usize + 2), Some(10));
    }

    #[test]
    fn a_legacy_pair_is_not_named() {
        let (h, b) = named_pair(&["A"], &[1], &[0]);
        let (lh, lb, _) = to_legacy(&h, &b).unwrap();
        assert!(!is_named_pair(&lh, &lb));
        assert!(is_named_pair(&h, &b));
    }

    #[test]
    fn sidecar_bytes_round_trip() {
        let (h, b) = named_pair(&["A", "B"], &[1, 2], &[5, 9]);
        let sc = to_legacy(&h, &b).unwrap().2;
        assert_eq!(LegacySidecar::from_bytes(&sc.to_bytes()).unwrap(), sc);
    }

    /// Every pair in a 2.06 fixture dir: to_legacy -> from_legacy gives the same bytes.
    /// `DMM_PARSER_NAMED_DIRS` = `;`-separated dirs (base and DLC sets).
    #[test]
    #[ignore]
    fn fixture_dirs_round_trip() {
        let dirs = std::env::var("DMM_PARSER_NAMED_DIRS").expect("set DMM_PARSER_NAMED_DIRS");
        let mut n = 0;
        for dir in dirs.split(';').filter(|d| !d.is_empty()) {
            for e in std::fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                if p.extension().and_then(|x| x.to_str()) != Some("pabgh") {
                    continue;
                }
                let h = std::fs::read(&p).unwrap();
                let b = std::fs::read(p.with_extension("pabgb")).unwrap();
                let (lh, lb, sc) = to_legacy(&h, &b).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
                let sc = LegacySidecar::from_bytes(&sc.to_bytes()).unwrap();
                let (nh, nb) = from_legacy(&lh, &lb, &sc).unwrap();
                assert!(nh == h && nb == b, "{}: round trip differs", p.display());
                n += 1;
            }
        }
        eprintln!("{n} pairs round-tripped");
        assert!(n > 0);
    }
}
