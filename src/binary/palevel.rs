// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.palevel` (and bare `.prefab`) — PA reflection documents, full
//! field-level codec plus object intents (game 2.04.03 / 2.04.08).
//!
//! Decodes every placed object of a level file down to its field values
//! and encodes back byte for byte (every count, offset, size and record
//! self-address is recomputed, so a byte match proves the model). Checked
//! on all 35,708 vanilla `.palevel` of 2.04.08 (build 26480954, base +
//! dlc_01): every file byte-exact; 4 objects in 4 files do not decode yet
//! (3 sector `SceneObject`s with 4 extra bytes after `_sceneObjectUuid`, 1
//! `EditorMeshComponent` record) and are kept as raw bytes, which blocks
//! clone/delete in those 4 docs only.
//!
//! ```text
//! .palevel = PARC(kind 0, hash) | PARC(kind 0, 0) | DOC
//!            | chunks: u8 0 | PARC(kind 2) | DOC       (0 or 1 seen)
//!            | footer: [u8 1 if no chunk] u32 counter | u16 n
//!                      | n x (u8 len, name\0)           (sub-level names)
//! PARC     = 'PARC' u8 kind | u8 0x08 | u64 hash | u16 0  (hashes are not
//!            checked by the game: a changed file mounted with the old
//!            hashes loads fine)
//! DOC      = ff ff 04 00 | u16 0 | u64 hash | u32 version (15) | u16 nclass
//!            (old: ff ff 03 00 | u16 0 | u32 version | u16 nclass)
//!            | classes: str name, u16 nfield, nfield x (str name, str type,
//!                       u16 kind, u16 size, u32 flags)
//!            | u32 n | n x str            string table (IndexedString)
//!            | [u32 n | n x (u32 id, str)] resource table (PARC docs only)
//!            | u32 nobj | u32 end (if nobj>0)
//!            | nobj x (u32 class, i64 parent, u32 start, u32 size)
//!            | object data back to back (offsets relative to the DOC start)
//! str      = u32 len | bytes
//! OBJECT   = u16 masklen | mask | u32 0 | VALUES
//! RECORD   = u16 masklen | mask | u16 class | u8 cflag | id (8 or 12 bytes)
//!            | u32 self (= DOC offset of the next byte) | u16 a | u16 n
//!            | n x (u16, str component name) | VALUES | u32 size (bytes
//!            from after `self` to here)
//! VALUES   = fields in declared order; written when the mask bit is set;
//!            kinds 3/6/7/10 without flag 0x2 are ALWAYS written (prefab
//!            instance component overrides use this)
//!   kind 0/2 = `size` bytes        kind 1 = string (PARC: u32 table index,
//!              0xffffffff + inline str; bare doc: inline str)
//!   kind 3   = u8 null | u32 n | n x size bytes
//!   kind 4   = RECORD              kind 5 = u8 1 + RECORD | u8 0 (null)
//!   kind 6/7 = u8 null | u32 n | u8 a | u32 b | u32 c | u32 m
//!              | m x id (8 bytes; 12 when it starts fe ff) | n x RECORD
//!   kind 10  = u8 null | u32 n | n x string
//!   (version < 15 docs: kinds 3/6/7/10 have no null byte)
//! ```
//!
//! Object identity: every placed scene object (top-level and nested
//! prefab-instance children) carries `_sceneObjectUid` (u32, unique per
//! file) and `_sceneObjectUuid` (16 bytes; children share the parent's
//! 4-byte prefix). `_worldTransform` / `_tiledTransform` = 10 f32 (scale
//! xyz, rotation quaternion xyzw, position xyz) + trailing bytes; nested
//! children hold LOCAL transforms (relative to their parent).
//!
//! Intents (`apply_palevel_intents`), objects selected by `key` =
//! `_sceneObjectUid`:
//! - `set`: `field` = path inside the object (`a.b[2].c`), `value` = the
//!   JSON form of [`decode_palevel`] (hex for fixed fields) or numbers for
//!   float / int / bool fields. Field must already be present.
//! - `move`: `value` = `{"x","y","z"}` deltas (or `[dx,dy,dz]`) and an
//!   optional `"scale"` factor, applied to `_worldTransform` and
//!   `_tiledTransform`. On a nested child the delta is in its parent's frame.
//! - `clone_record`: `source_key` = uid of a top-level object; the copy is
//!   appended to the same document; `value` = `{"move": [dx,dy,dz],
//!   "uids": [..] | "uid_start": n, "uuid_prefixes": {"old8hex": "new8hex"}}`.
//!   EVERY nested `_sceneObjectUid` gets a new unique uid (explicit list in
//!   traversal order, else the next free uids after the footer counter or
//!   `uid_start` / `new_key`), every `_sceneObjectUuid` prefix maps old ->
//!   new (one new prefix per old prefix, deterministic when not given). The
//!   footer counter is raised to the highest new uid. Uids used by OTHER
//!   files of the same level are not visible here: pass `uids`/`uid_start`.
//! - `delete_record`: `key` = uid of a top-level object without table
//!   children; later parent indexes are fixed.
//!
//! In game (proven by probes on 2.04.03, file overlay): moved / scaled /
//! cloned / removed placed objects show and survive a save reload.

use crate::intents::{ApplyOutcome, ApplyStatus, Intent};
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::io;

fn bad(m: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, m.into())
}

// ---------------------------------------------------------------- model

#[derive(Clone, Debug, PartialEq)]
pub struct FieldDef {
    pub name: Vec<u8>,
    pub type_name: Vec<u8>,
    pub kind: u16,
    pub size: u16,
    pub flags: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassDef {
    pub name: Vec<u8>,
    pub fields: Vec<FieldDef>,
}

/// A kind-1 string value.
#[derive(Clone, Debug, PartialEq)]
pub enum Str {
    /// PARC doc: index into the doc string table.
    Index(u32),
    /// PARC doc: index 0xffffffff followed by an inline string.
    Inline(Vec<u8>),
    /// Bare doc: inline string.
    Text(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    /// kinds 0 / 2 (fixed size) and 3 (array payload, n x size bytes).
    Bytes(Vec<u8>),
    Str(Str),
    /// kinds 4 and 5 (present).
    Rec(Box<Rec>),
    /// kind 5 null (u8 0) or list null (u8 1).
    Null,
    /// kinds 6 / 7. `hdr` = the 13 header bytes (a, b, c, m).
    List {
        hdr: Vec<u8>,
        ids: Vec<Vec<u8>>,
        items: Vec<Rec>,
    },
    /// kind 10.
    Strs(Vec<Str>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rec {
    pub mask: Vec<u8>,
    pub cls: u16,
    pub cf: u8,
    pub id: Vec<u8>,
    pub a: u16,
    pub names: Vec<(u16, Vec<u8>)>,
    pub vals: Vec<(usize, Val)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    Top {
        mask: Vec<u8>,
        pre: Vec<u8>,
        vals: Vec<(usize, Val)>,
        tail: Vec<u8>,
    },
    /// Object whose values did not decode + re-encode exactly; kept as
    /// bytes and only valid at its original doc offset `start`.
    Raw { data: Vec<u8>, start: u32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Obj {
    pub cls: u32,
    pub parent: i64,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    pub format: u8,
    pub zero: u16,
    pub hash: Option<u64>,
    pub version: u32,
    pub classes: Vec<ClassDef>,
    pub strings: Vec<Vec<u8>>,
    pub resources: Option<Vec<(u32, Vec<u8>)>>,
    pub objects: Vec<Obj>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parc {
    pub kind: u8,
    pub v: u8,
    pub hash: u64,
    pub z: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub flag: u8,
    pub parc: Parc,
    pub doc: Doc,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Footer {
    pub end_flag: Option<u8>,
    pub counter: u32,
    pub sublevels: Vec<Vec<u8>>,
}

/// A whole `.palevel` (PARC container) or bare reflection doc.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelFile {
    /// `None` = bare doc (`.prefab`).
    pub parc: Option<[Parc; 2]>,
    pub doc: Doc,
    pub chunks: Vec<Chunk>,
    pub footer: Option<Footer>,
    pub tail: Vec<u8>,
}

impl LevelFile {
    pub fn docs(&self) -> Vec<&Doc> {
        let mut v = vec![&self.doc];
        v.extend(self.chunks.iter().map(|c| &c.doc));
        v
    }
    fn doc_mut(&mut self, i: usize) -> &mut Doc {
        if i == 0 {
            &mut self.doc
        } else {
            &mut self.chunks[i - 1].doc
        }
    }
    /// Objects whose values did not decode (0 on every vanilla .palevel).
    pub fn raw_objects(&self) -> usize {
        self.docs()
            .iter()
            .map(|d| {
                d.objects
                    .iter()
                    .filter(|o| matches!(o.body, Body::Raw { .. }))
                    .count()
            })
            .sum()
    }
}

// ---------------------------------------------------------------- reader

struct Rd<'a> {
    b: &'a [u8],
    p: usize,
}

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let s = self
            .b
            .get(self.p..self.p + n)
            .ok_or_else(|| bad(format!("short read at {:#x}", self.p)))?;
        self.p += n;
        Ok(s)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i64(&mut self) -> io::Result<i64> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn lstr(&mut self) -> io::Result<Vec<u8>> {
        let n = self.u32()? as usize;
        Ok(self.take(n)?.to_vec())
    }
    fn u32_at(&self, o: usize) -> Option<u32> {
        self.b
            .get(o..o + 4)
            .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
    }
}

fn wl(out: &mut Vec<u8>, s: &[u8]) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s);
}

struct Cx<'a> {
    classes: &'a [ClassDef],
    parc: bool,
    version: u32,
    /// doc offset of byte 0 of the buffer being read / written.
    base: usize,
}

fn always(kind: u16) -> bool {
    matches!(kind, 3 | 6 | 7 | 10)
}
fn mask_bit(mask: &[u8], i: usize) -> bool {
    mask.get(i / 8)
        .map(|b| b >> (i % 8) & 1 == 1)
        .unwrap_or(false)
}

fn read_str(r: &mut Rd, cx: &Cx) -> io::Result<Str> {
    if cx.parc {
        let v = r.u32()?;
        if v == 0xffff_ffff {
            Ok(Str::Inline(r.lstr()?))
        } else {
            Ok(Str::Index(v))
        }
    } else {
        Ok(Str::Text(r.lstr()?))
    }
}

fn write_str(out: &mut Vec<u8>, s: &Str) {
    match s {
        Str::Index(v) => out.extend_from_slice(&v.to_le_bytes()),
        Str::Inline(t) => {
            out.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
            wl(out, t)
        }
        Str::Text(t) => wl(out, t),
    }
}

fn read_values(c: &ClassDef, mask: &[u8], r: &mut Rd, cx: &Cx) -> io::Result<Vec<(usize, Val)>> {
    let mut out = Vec::new();
    for (i, f) in c.fields.iter().enumerate() {
        let k = f.kind;
        if !mask_bit(mask, i) && !(always(k) && f.flags & 2 == 0) {
            continue;
        }
        if always(k) || k == 5 {
            let fl = if k != 5 && cx.version < 15 {
                0
            } else {
                r.u8()?
            };
            if k == 5 {
                match fl {
                    0 => out.push((i, Val::Null)),
                    1 => out.push((i, Val::Rec(Box::new(read_rec(r, cx)?)))),
                    _ => return Err(bad(format!("kind 5 flag {fl} at {:#x}", r.p))),
                }
                continue;
            }
            if fl == 1 {
                out.push((i, Val::Null));
                continue;
            }
            if fl != 0 {
                return Err(bad(format!("list flag {fl} at {:#x}", r.p)));
            }
            let n = r.u32()? as usize;
            match k {
                3 => out.push((i, Val::Bytes(r.take(n * f.size as usize)?.to_vec()))),
                10 => {
                    let mut v = Vec::with_capacity(n.min(4096));
                    for _ in 0..n {
                        v.push(read_str(r, cx)?);
                    }
                    out.push((i, Val::Strs(v)))
                }
                _ => {
                    let hdr = r.take(13)?.to_vec();
                    let m = u32::from_le_bytes(hdr[9..13].try_into().unwrap());
                    let mut ids = Vec::new();
                    for _ in 0..(if m == 0xffff_ffff { 0 } else { m }) {
                        let idl = if r.b.get(r.p..r.p + 2) == Some(&[0xfe, 0xff][..]) {
                            12
                        } else {
                            8
                        };
                        ids.push(r.take(idl)?.to_vec());
                    }
                    let mut items = Vec::with_capacity(n.min(4096));
                    for _ in 0..n {
                        items.push(read_rec(r, cx)?);
                    }
                    out.push((i, Val::List { hdr, ids, items }))
                }
            }
            continue;
        }
        match k {
            0 | 2 => out.push((i, Val::Bytes(r.take(f.size as usize)?.to_vec()))),
            1 => out.push((i, Val::Str(read_str(r, cx)?))),
            4 => out.push((i, Val::Rec(Box::new(read_rec(r, cx)?)))),
            _ => {
                return Err(bad(format!(
                    "unsupported kind {k} ({})",
                    String::from_utf8_lossy(&f.type_name)
                )))
            }
        }
    }
    Ok(out)
}

fn read_rec(r: &mut Rd, cx: &Cx) -> io::Result<Rec> {
    let ml = r.u16()? as usize;
    let mask = r.take(ml)?.to_vec();
    let cls = r.u16()?;
    let cf = r.u8()?;
    if cls as usize >= cx.classes.len() {
        return Err(bad(format!("record class {cls} out of range")));
    }
    let o = r.p;
    let order: [usize; 2] = if r.b.get(o..o + 2) == Some(&[0xfe, 0xff][..]) {
        [12, 8]
    } else {
        [8, 12]
    };
    let idl = order
        .into_iter()
        .find(|&l| r.u32_at(o + l) == Some((cx.base + o + l + 4) as u32))
        .ok_or_else(|| {
            bad(format!(
                "record self-address not found at {:#x}",
                cx.base + o
            ))
        })?;
    let id = r.take(idl)?.to_vec();
    r.take(4)?;
    let sp = r.p;
    let a = r.u16()?;
    let n = r.u16()?;
    let mut names = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let bb = r.u16()?;
        names.push((bb, r.lstr()?));
    }
    let c = &cx.classes[cls as usize];
    let vals = read_values(c, &mask, r, cx)?;
    let size = r.u32()? as usize;
    if size != r.p - 4 - sp {
        return Err(bad(format!("record size {size} != {}", r.p - 4 - sp)));
    }
    Ok(Rec {
        mask,
        cls,
        cf,
        id,
        a,
        names,
        vals,
    })
}

fn write_values(out: &mut Vec<u8>, c: &ClassDef, vals: &[(usize, Val)], cx: &Cx) -> io::Result<()> {
    for (i, v) in vals {
        let f = c
            .fields
            .get(*i)
            .ok_or_else(|| bad("field index out of range"))?;
        let k = f.kind;
        let nf = k != 5 && cx.version < 15;
        match (k, v) {
            (5, Val::Null) => out.push(0),
            (3 | 6 | 7 | 10, Val::Null) => out.push(1),
            (5, Val::Rec(r)) => {
                out.push(1);
                write_rec(out, r, cx)?
            }
            (3, Val::Bytes(b)) => {
                if f.size == 0 || b.len() % f.size as usize != 0 {
                    return Err(bad("array payload not a multiple of the element size"));
                }
                if !nf {
                    out.push(0)
                }
                out.extend_from_slice(&((b.len() / f.size as usize) as u32).to_le_bytes());
                out.extend_from_slice(b);
            }
            (10, Val::Strs(ss)) => {
                if !nf {
                    out.push(0)
                }
                out.extend_from_slice(&(ss.len() as u32).to_le_bytes());
                for s in ss {
                    write_str(out, s)
                }
            }
            (6 | 7, Val::List { hdr, ids, items }) => {
                if hdr.len() != 13 {
                    return Err(bad("list header must be 13 bytes"));
                }
                if !nf {
                    out.push(0)
                }
                out.extend_from_slice(&(items.len() as u32).to_le_bytes());
                if ids.is_empty() {
                    out.extend_from_slice(hdr)
                } else {
                    out.extend_from_slice(&hdr[..9]);
                    out.extend_from_slice(&(ids.len() as u32).to_le_bytes());
                }
                for id in ids {
                    out.extend_from_slice(id)
                }
                for r in items {
                    write_rec(out, r, cx)?
                }
            }
            (0 | 2, Val::Bytes(b)) => {
                if b.len() != f.size as usize {
                    return Err(bad(format!(
                        "field {} needs {} bytes, got {}",
                        String::from_utf8_lossy(&f.name),
                        f.size,
                        b.len()
                    )));
                }
                out.extend_from_slice(b)
            }
            (1, Val::Str(s)) => write_str(out, s),
            (4, Val::Rec(r)) => write_rec(out, r, cx)?,
            _ => {
                return Err(bad(format!(
                    "value does not fit field {} (kind {k})",
                    String::from_utf8_lossy(&f.name)
                )))
            }
        }
    }
    Ok(())
}

fn write_rec(out: &mut Vec<u8>, r: &Rec, cx: &Cx) -> io::Result<()> {
    out.extend_from_slice(&(r.mask.len() as u16).to_le_bytes());
    out.extend_from_slice(&r.mask);
    out.extend_from_slice(&r.cls.to_le_bytes());
    out.push(r.cf);
    out.extend_from_slice(&r.id);
    let me = (cx.base + out.len() + 4) as u32;
    out.extend_from_slice(&me.to_le_bytes());
    let sp = out.len();
    out.extend_from_slice(&r.a.to_le_bytes());
    out.extend_from_slice(&(r.names.len() as u16).to_le_bytes());
    for (bb, s) in &r.names {
        out.extend_from_slice(&bb.to_le_bytes());
        wl(out, s);
    }
    let c = cx
        .classes
        .get(r.cls as usize)
        .ok_or_else(|| bad("record class out of range"))?;
    write_values(out, c, &r.vals, cx)?;
    let size = (out.len() - sp) as u32;
    out.extend_from_slice(&size.to_le_bytes());
    Ok(())
}

fn read_top(data: &[u8], c: &ClassDef, cx: &Cx) -> io::Result<Body> {
    let mut r = Rd { b: data, p: 0 };
    let ml = r.u16()? as usize;
    let mask = r.take(ml)?.to_vec();
    let pre = r.take(4)?.to_vec();
    let vals = read_values(c, &mask, &mut r, cx)?;
    Ok(Body::Top {
        mask,
        pre,
        vals,
        tail: data[r.p..].to_vec(),
    })
}

fn write_body(b: &Body, c: &ClassDef, cx: &Cx) -> io::Result<Vec<u8>> {
    match b {
        Body::Raw { data, start } => {
            if *start as usize != cx.base {
                return Err(bad(
                    "an undecoded object would move; structural edits are not possible in this doc",
                ));
            }
            Ok(data.clone())
        }
        Body::Top {
            mask,
            pre,
            vals,
            tail,
        } => {
            let mut out = Vec::new();
            out.extend_from_slice(&(mask.len() as u16).to_le_bytes());
            out.extend_from_slice(mask);
            out.extend_from_slice(pre);
            write_values(&mut out, c, vals, cx)?;
            out.extend_from_slice(tail);
            Ok(out)
        }
    }
}

// ---------------------------------------------------------------- doc / container

fn parse_doc(b: &[u8], o: usize, restab: bool) -> io::Result<(Doc, usize)> {
    let d0 = o;
    let mut r = Rd { b, p: o };
    let h = r.take(4)?;
    if h[0] != 0xff || h[1] != 0xff || h[3] != 0 || !(h[2] == 3 || h[2] == 4) {
        return Err(bad(format!("no reflection doc (ffff0400/0300) at {o:#x}")));
    }
    let format = h[2];
    let zero = r.u16()?;
    let hash = if format == 4 { Some(r.u64()?) } else { None };
    let version = r.u32()?;
    let nc = r.u16()?;
    let mut classes = Vec::with_capacity(nc as usize);
    for _ in 0..nc {
        let name = r.lstr()?;
        let nf = r.u16()?;
        let mut fields = Vec::with_capacity(nf as usize);
        for _ in 0..nf {
            let fname = r.lstr()?;
            let tname = r.lstr()?;
            let (kind, size, flags) = (r.u16()?, r.u16()?, r.u32()?);
            fields.push(FieldDef {
                name: fname,
                type_name: tname,
                kind,
                size,
                flags,
            });
        }
        classes.push(ClassDef { name, fields });
    }
    let ns = r.u32()?;
    let mut strings = Vec::with_capacity(ns.min(1 << 16) as usize);
    for _ in 0..ns {
        strings.push(r.lstr()?);
    }
    let resources = if restab {
        let n = r.u32()?;
        let mut v = Vec::with_capacity(n.min(1 << 16) as usize);
        for _ in 0..n {
            let id = r.u32()?;
            v.push((id, r.lstr()?));
        }
        Some(v)
    } else {
        None
    };
    let (n, endall) = if r.b.get(r.p..r.p + 4) != Some(&[0, 0, 0, 0][..]) {
        let n = r.u32()?;
        (n, Some(r.u32()?))
    } else {
        r.u32()?;
        (0, None)
    };
    let mut recs = Vec::with_capacity(n.min(1 << 20) as usize);
    for _ in 0..n {
        recs.push((r.u32()?, r.i64()?, r.u32()?, r.u32()?));
    }
    let mut doc = Doc {
        format,
        zero,
        hash,
        version,
        classes,
        strings,
        resources,
        objects: Vec::with_capacity(recs.len()),
    };
    for (i, (cls, parent, start, size)) in recs.into_iter().enumerate() {
        if start as usize != r.p - d0 {
            return Err(bad(format!(
                "object {i} start {start:#x} != {:#x}",
                r.p - d0
            )));
        }
        let data = r.take(size as usize)?;
        let c = doc
            .classes
            .get(cls as usize)
            .ok_or_else(|| bad(format!("object {i} class {cls} out of range")))?;
        let cx = Cx {
            classes: &doc.classes,
            parc: doc.resources.is_some(),
            version,
            base: start as usize,
        };
        // keep the decoded form only when it re-encodes to the same bytes
        let body = match read_top(data, c, &cx) {
            Ok(t) if write_body(&t, c, &cx).ok().as_deref() == Some(data) => t,
            _ => Body::Raw {
                data: data.to_vec(),
                start,
            },
        };
        doc.objects.push(Obj { cls, parent, body });
    }
    if let Some(e) = endall {
        if e as usize != r.p - d0 {
            return Err(bad(format!("object table end {e:#x} != {:#x}", r.p - d0)));
        }
    }
    Ok((doc, r.p))
}

fn build_doc(out: &mut Vec<u8>, d: &Doc) -> io::Result<()> {
    let d0 = out.len();
    out.extend_from_slice(&[0xff, 0xff, d.format, 0]);
    out.extend_from_slice(&d.zero.to_le_bytes());
    match (d.format, d.hash) {
        (4, Some(h)) => out.extend_from_slice(&h.to_le_bytes()),
        (3, None) => {}
        _ => return Err(bad("doc format 4 needs a hash, format 3 none")),
    }
    out.extend_from_slice(&d.version.to_le_bytes());
    out.extend_from_slice(&(d.classes.len() as u16).to_le_bytes());
    for c in &d.classes {
        wl(out, &c.name);
        out.extend_from_slice(&(c.fields.len() as u16).to_le_bytes());
        for f in &c.fields {
            wl(out, &f.name);
            wl(out, &f.type_name);
            out.extend_from_slice(&f.kind.to_le_bytes());
            out.extend_from_slice(&f.size.to_le_bytes());
            out.extend_from_slice(&f.flags.to_le_bytes());
        }
    }
    out.extend_from_slice(&(d.strings.len() as u32).to_le_bytes());
    for s in &d.strings {
        wl(out, s)
    }
    if let Some(res) = &d.resources {
        out.extend_from_slice(&(res.len() as u32).to_le_bytes());
        for (id, s) in res {
            out.extend_from_slice(&id.to_le_bytes());
            wl(out, s)
        }
    }
    let n = d.objects.len();
    let mut start = out.len() - d0 + if n > 0 { 8 } else { 4 } + 20 * n;
    let mut datas = Vec::with_capacity(n);
    let mut table = Vec::with_capacity(20 * n);
    for o in &d.objects {
        let c = d
            .classes
            .get(o.cls as usize)
            .ok_or_else(|| bad("object class out of range"))?;
        let cx = Cx {
            classes: &d.classes,
            parc: d.resources.is_some(),
            version: d.version,
            base: start,
        };
        let data = write_body(&o.body, c, &cx)?;
        table.extend_from_slice(&o.cls.to_le_bytes());
        table.extend_from_slice(&o.parent.to_le_bytes());
        table.extend_from_slice(&(start as u32).to_le_bytes());
        table.extend_from_slice(&(data.len() as u32).to_le_bytes());
        start += data.len();
        datas.push(data);
    }
    out.extend_from_slice(&(n as u32).to_le_bytes());
    if n > 0 {
        out.extend_from_slice(&(start as u32).to_le_bytes());
    }
    out.extend_from_slice(&table);
    for x in datas {
        out.extend_from_slice(&x)
    }
    Ok(())
}

fn parse_parc(b: &[u8], o: usize) -> io::Result<(Parc, usize)> {
    if b.get(o..o + 4) != Some(&b"PARC"[..]) {
        return Err(bad(format!("no PARC at {o:#x}")));
    }
    let mut r = Rd { b, p: o + 4 };
    Ok((
        Parc {
            kind: r.u8()?,
            v: r.u8()?,
            hash: r.u64()?,
            z: r.u16()?,
        },
        r.p,
    ))
}

fn build_parc(out: &mut Vec<u8>, p: &Parc) {
    out.extend_from_slice(b"PARC");
    out.push(p.kind);
    out.push(p.v);
    out.extend_from_slice(&p.hash.to_le_bytes());
    out.extend_from_slice(&p.z.to_le_bytes());
}

fn parse_footer(b: &[u8], o: usize, no_chunks: bool) -> Option<(Footer, usize)> {
    let mut p = o;
    let mut end_flag = None;
    if no_chunks && *b.get(p)? == 1 {
        end_flag = Some(1);
        p += 1;
    }
    let mut r = Rd { b, p };
    let counter = r.u32().ok()?;
    let n = r.u16().ok()?;
    let mut subs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let ln = r.u8().ok()? as usize;
        let t = r.take(ln).ok()?;
        if t.last() != Some(&0) {
            return None;
        }
        subs.push(t[..ln - 1].to_vec());
    }
    if end_flag.is_none() && no_chunks {
        return None;
    }
    Some((
        Footer {
            end_flag,
            counter,
            sublevels: subs,
        },
        r.p,
    ))
}

/// Parse a `.palevel` (PARC container) or a bare reflection doc.
pub fn parse_level(b: &[u8]) -> io::Result<LevelFile> {
    if b.starts_with(&[0xff, 0xff]) {
        let (doc, o) = parse_doc(b, 0, false)?;
        return Ok(LevelFile {
            parc: None,
            doc,
            chunks: vec![],
            footer: None,
            tail: b[o..].to_vec(),
        });
    }
    let (h1, o) = parse_parc(b, 0)?;
    let (h2, o) = parse_parc(b, o)?;
    let (doc, mut o) = parse_doc(b, o, true)?;
    let mut chunks = Vec::new();
    while o + 17 <= b.len() && &b[o + 1..o + 5] == b"PARC" {
        let flag = b[o];
        let (ph, o2) = parse_parc(b, o + 1)?;
        let (sd, o3) = parse_doc(b, o2, true)?;
        chunks.push(Chunk {
            flag,
            parc: ph,
            doc: sd,
        });
        o = o3;
    }
    let (footer, tail) = match parse_footer(b, o, chunks.is_empty()) {
        Some((f, p)) => (Some(f), b[p..].to_vec()),
        None => (None, b[o..].to_vec()),
    };
    Ok(LevelFile {
        parc: Some([h1, h2]),
        doc,
        chunks,
        footer,
        tail,
    })
}

/// Encode a [`LevelFile`] back to bytes (recomputes every offset).
pub fn build_level(m: &LevelFile) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    match &m.parc {
        None => build_doc(&mut out, &m.doc)?,
        Some([h1, h2]) => {
            build_parc(&mut out, h1);
            build_parc(&mut out, h2);
            build_doc(&mut out, &m.doc)?;
            for c in &m.chunks {
                out.push(c.flag);
                build_parc(&mut out, &c.parc);
                build_doc(&mut out, &c.doc)?;
            }
            if let Some(f) = &m.footer {
                if let Some(e) = f.end_flag {
                    out.push(e)
                }
                out.extend_from_slice(&f.counter.to_le_bytes());
                out.extend_from_slice(&(f.sublevels.len() as u16).to_le_bytes());
                for s in &f.sublevels {
                    out.push((s.len() + 1) as u8);
                    out.extend_from_slice(s);
                    out.push(0);
                }
            }
        }
    }
    out.extend_from_slice(&m.tail);
    Ok(out)
}

// ---------------------------------------------------------------- JSON

fn hx(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

fn unhx(s: &str) -> io::Result<Vec<u8>> {
    if s.len() % 2 != 0 {
        return Err(bad("odd hex length"));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| bad(format!("bad hex '{s}'"))))
        .collect()
}

/// Text as a JSON string when it is UTF-8, else `{"hex": ..}`.
fn bj(b: &[u8]) -> Value {
    match std::str::from_utf8(b) {
        Ok(s) => Value::String(s.to_string()),
        Err(_) => json!({ "hex": hx(b) }),
    }
}

fn jb(v: &Value) -> io::Result<Vec<u8>> {
    match v {
        Value::String(s) => Ok(s.as_bytes().to_vec()),
        Value::Object(o) => unhx(
            o.get("hex")
                .and_then(|x| x.as_str())
                .ok_or_else(|| bad("expected string or {hex}"))?,
        ),
        _ => Err(bad("expected string")),
    }
}

fn jhex(v: &Value) -> io::Result<Vec<u8>> {
    unhx(v.as_str().ok_or_else(|| bad("expected hex string"))?)
}
fn ju(v: Option<&Value>, what: &str) -> io::Result<u64> {
    v.and_then(|x| x.as_u64())
        .ok_or_else(|| bad(format!("missing/invalid '{what}'")))
}

fn str_json(s: &Str) -> Value {
    match s {
        Str::Index(i) => json!(i),
        Str::Inline(t) => json!({ "inline": bj(t) }),
        Str::Text(t) => bj(t),
    }
}

fn str_from(v: &Value, parc: bool) -> io::Result<Str> {
    if parc {
        if let Some(i) = v.as_u64() {
            return Ok(Str::Index(i as u32));
        }
        let t = v
            .get("inline")
            .ok_or_else(|| bad("PARC doc string must be an index or {inline}"))?;
        Ok(Str::Inline(jb(t)?))
    } else {
        Ok(Str::Text(jb(v)?))
    }
}

fn vals_json(c: &ClassDef, vals: &[(usize, Val)], cl: &[ClassDef]) -> Value {
    Value::Array(
        vals.iter()
            .map(|(i, v)| json!([bj(&c.fields[*i].name), val_json(v, cl)]))
            .collect(),
    )
}

fn rec_json(r: &Rec, cl: &[ClassDef]) -> Value {
    json!({
        "class": r.cls, "cf": r.cf, "id": hx(&r.id), "mask": hx(&r.mask), "a": r.a,
        "names": r.names.iter().map(|(b, s)| json!([b, bj(s)])).collect::<Vec<_>>(),
        "fields": vals_json(&cl[r.cls as usize], &r.vals, cl),
    })
}

fn val_json(v: &Val, cl: &[ClassDef]) -> Value {
    match v {
        Val::Bytes(b) => Value::String(hx(b)),
        Val::Str(s) => str_json(s),
        Val::Rec(r) => rec_json(r, cl),
        Val::Null => Value::Null,
        Val::List { hdr, ids, items } => json!({
            "hdr": hx(hdr),
            "ids": ids.iter().map(|x| hx(x)).collect::<Vec<_>>(),
            "items": items.iter().map(|r| rec_json(r, cl)).collect::<Vec<_>>(),
        }),
        Val::Strs(ss) => Value::Array(ss.iter().map(str_json).collect()),
    }
}

fn field_index(c: &ClassDef, name: &[u8]) -> Option<usize> {
    c.fields.iter().position(|f| f.name == name)
}

fn vals_from(v: &Value, c: &ClassDef, cx: &Cx) -> io::Result<Vec<(usize, Val)>> {
    let arr = v
        .as_array()
        .ok_or_else(|| bad("fields must be a list of [name, value]"))?;
    let mut out = Vec::with_capacity(arr.len());
    for p in arr {
        let pair = p
            .as_array()
            .filter(|a| a.len() == 2)
            .ok_or_else(|| bad("field entry must be [name, value]"))?;
        let name = jb(&pair[0])?;
        let i = field_index(c, &name).ok_or_else(|| {
            bad(format!(
                "class has no field {}",
                String::from_utf8_lossy(&name)
            ))
        })?;
        out.push((i, val_from(&pair[1], &c.fields[i], cx)?));
    }
    Ok(out)
}

fn rec_from(v: &Value, cx: &Cx) -> io::Result<Rec> {
    let cls = ju(v.get("class"), "class")? as u16;
    let c = cx
        .classes
        .get(cls as usize)
        .ok_or_else(|| bad("record class out of range"))?;
    let mut names = Vec::new();
    for n in v
        .get("names")
        .and_then(|x| x.as_array())
        .ok_or_else(|| bad("record needs names"))?
    {
        let a = n
            .as_array()
            .filter(|a| a.len() == 2)
            .ok_or_else(|| bad("name entry must be [u16, text]"))?;
        names.push((ju(Some(&a[0]), "name u16")? as u16, jb(&a[1])?));
    }
    Ok(Rec {
        mask: jhex(v.get("mask").ok_or_else(|| bad("record needs mask"))?)?,
        cls,
        cf: ju(v.get("cf"), "cf")? as u8,
        id: jhex(v.get("id").ok_or_else(|| bad("record needs id"))?)?,
        a: ju(v.get("a"), "a")? as u16,
        names,
        vals: vals_from(
            v.get("fields").ok_or_else(|| bad("record needs fields"))?,
            c,
            cx,
        )?,
    })
}

fn val_from(v: &Value, f: &FieldDef, cx: &Cx) -> io::Result<Val> {
    if v.is_null() {
        return if matches!(f.kind, 3 | 5 | 6 | 7 | 10) {
            Ok(Val::Null)
        } else {
            Err(bad("null only for kinds 3/5/6/7/10"))
        };
    }
    Ok(match f.kind {
        0 | 2 | 3 => Val::Bytes(jhex(v)?),
        1 => Val::Str(str_from(v, cx.parc)?),
        4 | 5 => Val::Rec(Box::new(rec_from(v, cx)?)),
        6 | 7 => {
            let ids = v
                .get("ids")
                .and_then(|x| x.as_array())
                .ok_or_else(|| bad("list needs ids"))?;
            let items = v
                .get("items")
                .and_then(|x| x.as_array())
                .ok_or_else(|| bad("list needs items"))?;
            Val::List {
                hdr: jhex(v.get("hdr").ok_or_else(|| bad("list needs hdr"))?)?,
                ids: ids.iter().map(jhex).collect::<io::Result<_>>()?,
                items: items
                    .iter()
                    .map(|x| rec_from(x, cx))
                    .collect::<io::Result<_>>()?,
            }
        }
        10 => Val::Strs(
            v.as_array()
                .ok_or_else(|| bad("kind 10 needs a list"))?
                .iter()
                .map(|x| str_from(x, cx.parc))
                .collect::<io::Result<_>>()?,
        ),
        k => return Err(bad(format!("unsupported kind {k}"))),
    })
}

fn doc_json(d: &Doc) -> Value {
    let objects: Vec<Value> = d
        .objects
        .iter()
        .map(|o| match &o.body {
            Body::Top {
                mask,
                pre,
                vals,
                tail,
            } => {
                let mut m = json!({
                    "class": o.cls, "parent": o.parent, "mask": hx(mask), "pre": hx(pre),
                    "fields": vals_json(&d.classes[o.cls as usize], vals, &d.classes),
                });
                if !tail.is_empty() {
                    m["tail"] = Value::String(hx(tail));
                }
                m
            }
            Body::Raw { data, start } => {
                json!({"class": o.cls, "parent": o.parent, "raw": hx(data), "start": start})
            }
        })
        .collect();
    json!({
        "format": d.format, "zero": d.zero, "hash": d.hash.map(|h| format!("{h:016x}")), "version": d.version,
        "classes": d.classes.iter().map(|c| json!({
            "name": bj(&c.name),
            "fields": c.fields.iter().map(|f| json!({"name": bj(&f.name), "type": bj(&f.type_name), "kind": f.kind, "size": f.size, "flags": f.flags})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "strings": d.strings.iter().map(|s| bj(s)).collect::<Vec<_>>(),
        "resources": d.resources.as_ref().map(|r| r.iter().map(|(i, s)| json!([i, bj(s)])).collect::<Vec<_>>()),
        "objects": objects,
    })
}

fn doc_from(v: &Value) -> io::Result<Doc> {
    let mut classes = Vec::new();
    for c in v
        .get("classes")
        .and_then(|x| x.as_array())
        .ok_or_else(|| bad("doc needs classes"))?
    {
        let mut fields = Vec::new();
        for f in c
            .get("fields")
            .and_then(|x| x.as_array())
            .ok_or_else(|| bad("class needs fields"))?
        {
            fields.push(FieldDef {
                name: jb(f.get("name").ok_or_else(|| bad("field needs name"))?)?,
                type_name: jb(f.get("type").ok_or_else(|| bad("field needs type"))?)?,
                kind: ju(f.get("kind"), "kind")? as u16,
                size: ju(f.get("size"), "size")? as u16,
                flags: ju(f.get("flags"), "flags")? as u32,
            });
        }
        classes.push(ClassDef {
            name: jb(c.get("name").ok_or_else(|| bad("class needs name"))?)?,
            fields,
        });
    }
    let hash = match v.get("hash") {
        Some(Value::String(s)) => {
            Some(u64::from_str_radix(s, 16).map_err(|_| bad("bad doc hash"))?)
        }
        _ => None,
    };
    let resources = match v.get("resources") {
        Some(Value::Array(a)) => Some(
            a.iter()
                .map(|p| {
                    let p = p
                        .as_array()
                        .filter(|x| x.len() == 2)
                        .ok_or_else(|| bad("resource must be [id, path]"))?;
                    Ok((ju(Some(&p[0]), "resource id")? as u32, jb(&p[1])?))
                })
                .collect::<io::Result<Vec<_>>>()?,
        ),
        _ => None,
    };
    let version = ju(v.get("version"), "version")? as u32;
    let strings = v
        .get("strings")
        .and_then(|x| x.as_array())
        .ok_or_else(|| bad("doc needs strings"))?
        .iter()
        .map(jb)
        .collect::<io::Result<_>>()?;
    let mut doc = Doc {
        format: ju(v.get("format"), "format")? as u8,
        zero: ju(v.get("zero"), "zero")? as u16,
        hash,
        version,
        classes,
        strings,
        resources,
        objects: vec![],
    };
    let mut objects = Vec::new();
    for o in v
        .get("objects")
        .and_then(|x| x.as_array())
        .ok_or_else(|| bad("doc needs objects"))?
    {
        let cls = ju(o.get("class"), "class")? as u32;
        let parent = o
            .get("parent")
            .and_then(|x| x.as_i64())
            .ok_or_else(|| bad("object needs parent"))?;
        let body = if let Some(raw) = o.get("raw") {
            Body::Raw {
                data: jhex(raw)?,
                start: ju(o.get("start"), "start")? as u32,
            }
        } else {
            let c = doc
                .classes
                .get(cls as usize)
                .ok_or_else(|| bad("object class out of range"))?;
            let cx = Cx {
                classes: &doc.classes,
                parc: doc.resources.is_some(),
                version,
                base: 0,
            };
            Body::Top {
                mask: jhex(o.get("mask").ok_or_else(|| bad("object needs mask"))?)?,
                pre: jhex(o.get("pre").ok_or_else(|| bad("object needs pre"))?)?,
                vals: vals_from(
                    o.get("fields").ok_or_else(|| bad("object needs fields"))?,
                    c,
                    &cx,
                )?,
                tail: match o.get("tail") {
                    Some(t) => jhex(t)?,
                    None => vec![],
                },
            }
        };
        objects.push(Obj { cls, parent, body });
    }
    doc.objects = objects;
    Ok(doc)
}

fn parc_json(p: &Parc) -> Value {
    json!({"kind": p.kind, "v": p.v, "hash": format!("{:016x}", p.hash), "z": p.z})
}
fn parc_from(v: &Value) -> io::Result<Parc> {
    Ok(Parc {
        kind: ju(v.get("kind"), "kind")? as u8,
        v: ju(v.get("v"), "v")? as u8,
        hash: u64::from_str_radix(
            v.get("hash")
                .and_then(|x| x.as_str())
                .ok_or_else(|| bad("PARC needs hash"))?,
            16,
        )
        .map_err(|_| bad("bad PARC hash"))?,
        z: ju(v.get("z"), "z")? as u16,
    })
}

pub fn level_to_json(m: &LevelFile) -> Value {
    let mut o = Map::new();
    o.insert(
        "container".into(),
        json!(if m.parc.is_some() { "parc" } else { "bare" }),
    );
    if let Some([a, b]) = &m.parc {
        o.insert("parc".into(), json!([parc_json(a), parc_json(b)]));
    }
    o.insert("doc".into(), doc_json(&m.doc));
    o.insert(
        "chunks".into(),
        Value::Array(m.chunks.iter().map(|c| json!({"flag": c.flag, "parc": parc_json(&c.parc), "doc": doc_json(&c.doc)})).collect()),
    );
    o.insert(
        "footer".into(),
        match &m.footer {
            None => Value::Null,
            Some(f) => json!({"end_flag": f.end_flag, "counter": f.counter, "sublevels": f.sublevels.iter().map(|s| bj(s)).collect::<Vec<_>>()}),
        },
    );
    o.insert("tail".into(), Value::String(hx(&m.tail)));
    Value::Object(o)
}

pub fn level_from_json(v: &Value) -> io::Result<LevelFile> {
    let parc = match v.get("container").and_then(|x| x.as_str()) {
        Some("bare") => None,
        Some("parc") => {
            let p = v
                .get("parc")
                .and_then(|x| x.as_array())
                .filter(|a| a.len() == 2)
                .ok_or_else(|| bad("parc needs 2 headers"))?;
            Some([parc_from(&p[0])?, parc_from(&p[1])?])
        }
        _ => return Err(bad("container must be 'parc' or 'bare'")),
    };
    let mut chunks = Vec::new();
    for c in v
        .get("chunks")
        .and_then(|x| x.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[])
    {
        chunks.push(Chunk {
            flag: ju(c.get("flag"), "flag")? as u8,
            parc: parc_from(c.get("parc").ok_or_else(|| bad("chunk needs parc"))?)?,
            doc: doc_from(c.get("doc").ok_or_else(|| bad("chunk needs doc"))?)?,
        });
    }
    let footer = match v.get("footer") {
        Some(f) if !f.is_null() => Some(Footer {
            end_flag: f.get("end_flag").and_then(|x| x.as_u64()).map(|x| x as u8),
            counter: ju(f.get("counter"), "counter")? as u32,
            sublevels: f
                .get("sublevels")
                .and_then(|x| x.as_array())
                .ok_or_else(|| bad("footer needs sublevels"))?
                .iter()
                .map(jb)
                .collect::<io::Result<_>>()?,
        }),
        _ => None,
    };
    Ok(LevelFile {
        parc,
        doc: doc_from(v.get("doc").ok_or_else(|| bad("missing doc"))?)?,
        chunks,
        footer,
        tail: match v.get("tail") {
            Some(t) => jhex(t)?,
            None => vec![],
        },
    })
}

/// Decode a `.palevel` / bare reflection doc to JSON (byte-exact with
/// [`encode_palevel`]).
pub fn decode_palevel(b: &[u8]) -> io::Result<Value> {
    Ok(level_to_json(&parse_level(b)?))
}

/// Encode JSON from [`decode_palevel`] back to file bytes.
pub fn encode_palevel(v: &Value) -> io::Result<Vec<u8>> {
    build_level(&level_from_json(v)?)
}

// ---------------------------------------------------------------- object helpers

const UID: &[u8] = b"_sceneObjectUid";
const UUID: &[u8] = b"_sceneObjectUuid";

fn fname<'a>(cl: &'a [ClassDef], cls: usize, i: usize) -> &'a [u8] {
    &cl[cls].fields[i].name
}

fn direct_uid(cl: &[ClassDef], cls: usize, vals: &[(usize, Val)]) -> Option<u32> {
    vals.iter().find_map(|(i, v)| match v {
        Val::Bytes(b) if fname(cl, cls, *i) == UID && b.len() == 4 => {
            Some(u32::from_le_bytes(b[..4].try_into().unwrap()))
        }
        _ => None,
    })
}

/// Step into a value: (index in vals, list item).
type Path = Vec<(usize, Option<usize>)>;

fn find_uid(
    cl: &[ClassDef],
    cls: usize,
    vals: &[(usize, Val)],
    uid: u32,
    path: &mut Path,
    hits: &mut Vec<(Path, usize)>,
) {
    if direct_uid(cl, cls, vals) == Some(uid) {
        hits.push((path.clone(), cls));
    }
    for (k, (_, v)) in vals.iter().enumerate() {
        match v {
            Val::Rec(r) => {
                path.push((k, None));
                find_uid(cl, r.cls as usize, &r.vals, uid, path, hits);
                path.pop();
            }
            Val::List { items, .. } => {
                for (j, r) in items.iter().enumerate() {
                    path.push((k, Some(j)));
                    find_uid(cl, r.cls as usize, &r.vals, uid, path, hits);
                    path.pop();
                }
            }
            _ => {}
        }
    }
}

fn nav<'a>(
    mut vals: &'a mut Vec<(usize, Val)>,
    path: &[(usize, Option<usize>)],
) -> Option<&'a mut Vec<(usize, Val)>> {
    for (k, j) in path {
        vals = match (&mut vals.get_mut(*k)?.1, j) {
            (Val::Rec(r), None) => &mut r.vals,
            (Val::List { items, .. }, Some(j)) => &mut items.get_mut(*j)?.vals,
            _ => return None,
        };
    }
    Some(vals)
}

fn collect(
    cl: &[ClassDef],
    cls: usize,
    vals: &[(usize, Val)],
    uids: &mut Vec<u32>,
    pres: &mut HashSet<[u8; 4]>,
) {
    for (i, v) in vals {
        match v {
            Val::Bytes(b) if fname(cl, cls, *i) == UID && b.len() == 4 => {
                uids.push(u32::from_le_bytes(b[..4].try_into().unwrap()))
            }
            Val::Bytes(b) if fname(cl, cls, *i) == UUID && b.len() == 16 => {
                pres.insert(b[..4].try_into().unwrap());
            }
            Val::Rec(r) => collect(cl, r.cls as usize, &r.vals, uids, pres),
            Val::List { items, .. } => items
                .iter()
                .for_each(|r| collect(cl, r.cls as usize, &r.vals, uids, pres)),
            _ => {}
        }
    }
}

fn file_ids(m: &LevelFile) -> (Vec<u32>, HashSet<[u8; 4]>) {
    let (mut u, mut p) = (Vec::new(), HashSet::new());
    for d in m.docs() {
        for o in &d.objects {
            if let Body::Top { vals, .. } = &o.body {
                collect(&d.classes, o.cls as usize, vals, &mut u, &mut p)
            }
        }
    }
    (u, p)
}

/// Every uid of the scene objects in a file, in traversal order.
pub fn palevel_uids(b: &[u8]) -> io::Result<Vec<u32>> {
    Ok(file_ids(&parse_level(b)?).0)
}

/// Rewrite every `_sceneObjectUid` (traversal order) and uuid prefix.
fn remap(
    cl: &[ClassDef],
    cls: usize,
    vals: &mut [(usize, Val)],
    uid: &mut dyn FnMut(u32) -> u32,
    pre: &mut dyn FnMut([u8; 4]) -> [u8; 4],
) {
    for (i, v) in vals.iter_mut() {
        let n = fname(cl, cls, *i);
        match v {
            Val::Bytes(b) if n == UID && b.len() == 4 => {
                let u = uid(u32::from_le_bytes(b[..4].try_into().unwrap()));
                b.copy_from_slice(&u.to_le_bytes())
            }
            Val::Bytes(b) if n == UUID && b.len() == 16 => {
                let p = pre(b[..4].try_into().unwrap());
                b[..4].copy_from_slice(&p)
            }
            Val::Rec(r) => remap(cl, r.cls as usize, &mut r.vals, uid, pre),
            Val::List { items, .. } => items
                .iter_mut()
                .for_each(|r| remap(cl, r.cls as usize, &mut r.vals, uid, pre)),
            _ => {}
        }
    }
}

/// Add (dx, dy, dz) to the position and multiply the scale of
/// `_worldTransform` / `_tiledTransform`. Returns how many were changed.
fn transform(
    cl: &[ClassDef],
    cls: usize,
    vals: &mut [(usize, Val)],
    d: [f64; 3],
    scale: f64,
) -> usize {
    let mut n = 0;
    for (i, v) in vals.iter_mut() {
        let name = fname(cl, cls, *i);
        if name != b"_worldTransform" && name != b"_tiledTransform" {
            continue;
        }
        if let Val::Bytes(b) = v {
            if b.len() < 40 {
                continue;
            }
            let mut edit = |k: usize, f: &dyn Fn(f64) -> f64| {
                let x = f32::from_le_bytes(b[4 * k..4 * k + 4].try_into().unwrap()) as f64;
                b[4 * k..4 * k + 4].copy_from_slice(&(f(x) as f32).to_le_bytes());
            };
            if scale != 1.0 {
                (0..3).for_each(|k| edit(k, &|x| x * scale));
            }
            for (k, dk) in d.iter().enumerate() {
                if *dk != 0.0 {
                    edit(7 + k, &|x| x + dk);
                }
            }
            n += 1;
        }
    }
    n
}

/// One line per top-level object: doc, index, class, parent, uid, world
/// position, nested uid count. For finding the uids that intents use.
pub fn list_palevel_objects(b: &[u8]) -> io::Result<Vec<Value>> {
    let m = parse_level(b)?;
    let mut out = Vec::new();
    for (di, d) in m.docs().into_iter().enumerate() {
        for (oi, o) in d.objects.iter().enumerate() {
            let mut row = json!({"doc": di, "index": oi, "class": bj(&d.classes[o.cls as usize].name), "parent": o.parent});
            if let Body::Top { vals, .. } = &o.body {
                let cls = o.cls as usize;
                row["uid"] = json!(direct_uid(&d.classes, cls, vals));
                let (mut u, mut p) = (Vec::new(), HashSet::new());
                collect(&d.classes, cls, vals, &mut u, &mut p);
                row["uids"] = json!(u.len());
                if let Some(Val::Bytes(w)) = vals
                    .iter()
                    .find(|(i, _)| fname(&d.classes, cls, *i) == b"_worldTransform")
                    .map(|x| &x.1)
                {
                    if w.len() >= 40 {
                        let f =
                            |k: usize| f32::from_le_bytes(w[4 * k..4 * k + 4].try_into().unwrap());
                        row["position"] = json!([f(7), f(8), f(9)]);
                    }
                }
            } else {
                row["raw"] = json!(true);
            }
            out.push(row);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------- intents

fn num(v: Option<&Value>) -> Option<f64> {
    v.and_then(|x| x.as_f64())
}

fn vec3(v: &Value) -> Option<[f64; 3]> {
    if let Some(a) = v.as_array() {
        if a.len() == 3 {
            return Some([a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?]);
        }
        return None;
    }
    if v.is_object() && (v.get("x").is_some() || v.get("y").is_some() || v.get("z").is_some()) {
        return Some([
            num(v.get("x")).unwrap_or(0.0),
            num(v.get("y")).unwrap_or(0.0),
            num(v.get("z")).unwrap_or(0.0),
        ]);
    }
    None
}

/// Typed convenience for `set`: numbers for float/int fields, bool for
/// bool; otherwise the decode JSON form.
fn set_value(f: &FieldDef, v: &Value, cx: &Cx) -> io::Result<Val> {
    let t = String::from_utf8_lossy(&f.type_name).to_ascii_lowercase();
    let size = f.size as usize;
    if f.kind == 0 {
        let nums: Option<Vec<f64>> = match v {
            Value::Number(n) => n.as_f64().map(|x| vec![x]),
            Value::Array(a) if a.iter().all(|x| x.is_number()) => {
                Some(a.iter().filter_map(|x| x.as_f64()).collect())
            }
            _ => None,
        };
        if let (Some(ns), true) = (&nums, t.starts_with("float")) {
            if ns.len() * 4 != size {
                return Err(bad(format!("{t} needs {} numbers", size / 4)));
            }
            return Ok(Val::Bytes(
                ns.iter().flat_map(|x| (*x as f32).to_le_bytes()).collect(),
            ));
        }
        if let (Some(ns), true) = (&nums, t.contains("int")) {
            let i = v
                .as_i64()
                .or_else(|| v.as_u64().map(|u| u as i64))
                .filter(|_| ns.len() == 1)
                .ok_or_else(|| bad("integer field needs one integer"))?;
            return Ok(Val::Bytes(i.to_le_bytes()[..size.min(8)].to_vec()));
        }
        if let (Value::Bool(x), true) = (v, t == "bool") {
            return Ok(Val::Bytes(vec![*x as u8]));
        }
    }
    val_from(v, f, cx)
}

enum Step {
    Done,
    Skip(String),
}

fn find_one(m: &LevelFile, uid: u32) -> Result<(usize, usize, Path, usize), String> {
    let mut found = Vec::new();
    for (di, d) in m.docs().into_iter().enumerate() {
        for (oi, o) in d.objects.iter().enumerate() {
            if let Body::Top { vals, .. } = &o.body {
                let mut hits = Vec::new();
                find_uid(
                    &d.classes,
                    o.cls as usize,
                    vals,
                    uid,
                    &mut vec![],
                    &mut hits,
                );
                found.extend(hits.into_iter().map(|(p, c)| (di, oi, p, c)));
            }
        }
    }
    match found.len() {
        0 => Err(format!("no object with _sceneObjectUid {uid}")),
        1 => Ok(found.pop().unwrap()),
        n => Err(format!("_sceneObjectUid {uid} found {n} times")),
    }
}

fn key_u32(k: Option<i64>, what: &str) -> Result<u32, String> {
    k.and_then(|x| u32::try_from(x).ok())
        .ok_or_else(|| format!("{what} must be a _sceneObjectUid (u32)"))
}

fn op_set(m: &mut LevelFile, it: &Intent) -> Step {
    let uid = match key_u32(it.key, "key") {
        Ok(u) => u,
        Err(e) => return Step::Skip(e),
    };
    let (Some(field), Some(value)) = (it.field.as_deref(), it.value.as_ref().or(it.new.as_ref()))
    else {
        return Step::Skip("set needs field and value".into());
    };
    let (di, oi, path, mut cls) = match find_one(m, uid) {
        Ok(x) => x,
        Err(e) => return Step::Skip(e),
    };
    let d = m.doc_mut(di);
    let raw = d.objects.iter().any(|o| matches!(o.body, Body::Raw { .. }));
    let cx = Cx {
        classes: &d.classes,
        parc: d.resources.is_some(),
        version: d.version,
        base: 0,
    };
    let Body::Top { vals, .. } = &mut d.objects[oi].body else {
        return Step::Skip("object not decoded".into());
    };
    let Some(mut cur) = nav(vals, &path) else {
        return Step::Skip("path to uid lost".into());
    };
    let segs: Vec<&str> = field.split('.').collect();
    for (si, seg) in segs.iter().enumerate() {
        let (name, idx) = match seg.find('[') {
            Some(p) if seg.ends_with(']') => match seg[p + 1..seg.len() - 1].parse::<usize>() {
                Ok(i) => (&seg[..p], Some(i)),
                Err(_) => return Step::Skip(format!("bad index in '{seg}'")),
            },
            _ => (*seg, None),
        };
        let Some(fi) = field_index(&cx.classes[cls], name.as_bytes()) else {
            return Step::Skip(format!(
                "class {} has no field {name}",
                String::from_utf8_lossy(&cx.classes[cls].name)
            ));
        };
        let Some(k) = cur.iter().position(|(i, _)| *i == fi) else {
            return Step::Skip(format!(
                "field {name} is not present in this object (mask bit clear)"
            ));
        };
        if si + 1 == segs.len() && idx.is_none() {
            let f = &cx.classes[cls].fields[fi];
            let nv = match set_value(f, value, &cx) {
                Ok(v) => v,
                Err(e) => return Step::Skip(e.to_string()),
            };
            if raw {
                let mut a = Vec::new();
                let mut b = Vec::new();
                let c = &cx.classes[cls];
                let ok = write_values(&mut a, c, &[(fi, cur[k].1.clone())], &cx).is_ok()
                    && write_values(&mut b, c, &[(fi, nv.clone())], &cx).is_ok();
                if !ok || a.len() != b.len() {
                    return Step::Skip(
                        "doc has undecoded objects: only same-size sets allowed".into(),
                    );
                }
            }
            cur[k].1 = nv;
            return Step::Done;
        }
        cur = match (&mut cur[k].1, idx) {
            (Val::Rec(r), None) => {
                cls = r.cls as usize;
                &mut r.vals
            }
            (Val::List { items, .. }, Some(j)) if j < items.len() => {
                cls = items[j].cls as usize;
                &mut items[j].vals
            }
            _ => return Step::Skip(format!("cannot step into '{seg}'")),
        };
    }
    Step::Skip("field path ends on a record; name a field".into())
}

fn op_move(m: &mut LevelFile, it: &Intent) -> Step {
    let uid = match key_u32(it.key, "key") {
        Ok(u) => u,
        Err(e) => return Step::Skip(e),
    };
    let Some(v) = it.value.as_ref().or(it.new.as_ref()) else {
        return Step::Skip("move needs value {x,y,z[,scale]}".into());
    };
    let d3 = vec3(v).unwrap_or([0.0; 3]);
    let scale = num(v.get("scale")).unwrap_or(1.0);
    let (di, oi, path, cls) = match find_one(m, uid) {
        Ok(x) => x,
        Err(e) => return Step::Skip(e),
    };
    let d = m.doc_mut(di);
    let Body::Top { vals, .. } = &mut d.objects[oi].body else {
        return Step::Skip("object not decoded".into());
    };
    let Some(cur) = nav(vals, &path) else {
        return Step::Skip("path to uid lost".into());
    };
    if transform(&d.classes, cls, cur, d3, scale) == 0 {
        return Step::Skip("object has no _worldTransform/_tiledTransform".into());
    }
    Step::Done
}

fn top_index(m: &LevelFile, uid: u32) -> Result<(usize, usize), String> {
    let (di, oi, path, _) = find_one(m, uid)?;
    if !path.is_empty() {
        return Err(format!(
            "uid {uid} is a nested child, not a top-level object"
        ));
    }
    let d = m.docs()[di];
    if d.objects.iter().any(|o| matches!(o.body, Body::Raw { .. })) {
        return Err("doc has undecoded objects; structural edits refused".into());
    }
    if d.objects.iter().any(|o| o.parent == oi as i64) {
        return Err(format!("uid {uid} has child objects in the object table"));
    }
    Ok((di, oi))
}

fn op_delete(m: &mut LevelFile, it: &Intent) -> Step {
    let uid = match key_u32(it.key, "key") {
        Ok(u) => u,
        Err(e) => return Step::Skip(e),
    };
    let (di, oi) = match top_index(m, uid) {
        Ok(x) => x,
        Err(e) => return Step::Skip(e),
    };
    let d = m.doc_mut(di);
    d.objects.remove(oi);
    for o in d.objects.iter_mut() {
        if o.parent > oi as i64 {
            o.parent -= 1;
        }
    }
    Step::Done
}

fn fnv(b: &[u8]) -> u32 {
    b.iter().fold(0x811c_9dc5u32, |h, x| {
        (h ^ *x as u32).wrapping_mul(0x0100_0193)
    })
}

fn op_clone(m: &mut LevelFile, it: &Intent) -> Step {
    let uid = match key_u32(it.source_key.or(it.key), "source_key") {
        Ok(u) => u,
        Err(e) => return Step::Skip(e),
    };
    let (di, oi) = match top_index(m, uid) {
        Ok(x) => x,
        Err(e) => return Step::Skip(e),
    };
    let empty = json!({});
    let v = it.value.as_ref().or(it.new.as_ref()).unwrap_or(&empty);
    let (used, mut pres) = file_ids(m);
    let mut used: HashSet<u32> = used.into_iter().collect();
    let d = &m.docs()[di];
    let mut obj = d.objects[oi].clone();
    if obj.parent != -1 {
        return Step::Skip("only root objects (parent -1) can be cloned".into());
    }
    let cls = obj.cls as usize;
    let Body::Top { vals, .. } = &mut obj.body else {
        return Step::Skip("object not decoded".into());
    };
    let (mut old, mut tmp) = (Vec::new(), HashSet::new());
    collect(&d.classes, cls, vals, &mut old, &mut tmp);
    // new uids: explicit list, or sequential free uids
    let explicit: Option<Vec<u32>> = match v.get("uids") {
        None => None,
        Some(a) => match a.as_array().map(|a| {
            a.iter()
                .map(|x| x.as_u64().and_then(|u| u32::try_from(u).ok()))
                .collect::<Option<Vec<_>>>()
        }) {
            Some(Some(l)) => Some(l),
            _ => return Step::Skip("uids must be a list of u32".into()),
        },
    };
    let mut new_uids = Vec::new();
    if let Some(l) = explicit {
        if l.len() != old.len() {
            return Step::Skip(format!("object has {} uids, {} given", old.len(), l.len()));
        }
        for u in &l {
            if !used.insert(*u) {
                return Step::Skip(format!("uid {u} already used in this file or listed twice"));
            }
        }
        new_uids = l;
    } else {
        let counter = m.footer.as_ref().map(|f| f.counter).unwrap_or(0);
        let start = v
            .get("uid_start")
            .and_then(|x| x.as_u64())
            .map(|x| x as u32)
            .or(it.new_key.map(|x| x as u32))
            .unwrap_or(counter.saturating_add(1));
        let mut u = start;
        for _ in &old {
            while used.contains(&u) {
                u += 1;
            }
            used.insert(u);
            new_uids.push(u);
        }
    }
    // uuid prefixes: explicit map, else deterministic from (old prefix, first new uid)
    let mut pmap: HashMap<[u8; 4], [u8; 4]> = HashMap::new();
    if let Some(mp) = v.get("uuid_prefixes").and_then(|x| x.as_object()) {
        for (a, b) in mp {
            match (unhx(a), b.as_str().map(unhx)) {
                (Ok(a), Some(Ok(b))) if a.len() == 4 && b.len() == 4 => {
                    pmap.insert(a.try_into().unwrap(), b.try_into().unwrap());
                }
                _ => return Step::Skip("uuid_prefixes must map 8-hex to 8-hex".into()),
            }
        }
    }
    let seed = new_uids.first().copied().unwrap_or(0);
    let mut it_uids = new_uids.clone().into_iter();
    let mut uid_fn = |_: u32| it_uids.next().unwrap_or(0);
    let mut pre_fn = |o: [u8; 4]| {
        if let Some(p) = pmap.get(&o) {
            return *p;
        }
        let mut k = [o[0], o[1], o[2], o[3], 0, 0, 0, 0];
        k[4..].copy_from_slice(&seed.to_le_bytes());
        let mut p = fnv(&k).to_le_bytes();
        while pres.contains(&p) {
            p = fnv(&p).to_le_bytes();
        }
        pres.insert(p);
        pmap.insert(o, p);
        p
    };
    remap(&d.classes, cls, vals, &mut uid_fn, &mut pre_fn);
    if let Some(mv) = v.get("move") {
        let Some(d3) = vec3(mv) else {
            return Step::Skip("move must be [dx,dy,dz] or {x,y,z}".into());
        };
        transform(
            &d.classes,
            cls,
            vals,
            d3,
            num(mv.get("scale")).unwrap_or(1.0),
        );
    }
    let top = new_uids.iter().copied().max();
    m.doc_mut(di).objects.push(obj);
    if let (Some(f), Some(t)) = (m.footer.as_mut(), top) {
        f.counter = f.counter.max(t);
    }
    Step::Done
}

/// Apply object intents to a `.palevel` (see module docs). Returns the new
/// file bytes and one outcome per intent; failed intents are `Skipped`
/// with a reason and change nothing.
pub fn apply_palevel_intents(
    data: &[u8],
    intents: &[Intent],
) -> io::Result<(Vec<u8>, Vec<ApplyOutcome>)> {
    let mut m = parse_level(data)?;
    let mut outcomes = Vec::with_capacity(intents.len());
    for it in intents {
        let op = it.op.clone().unwrap_or_else(|| "set".into());
        let before = m.clone();
        let step = match op.as_str() {
            "set" => op_set(&mut m, it),
            "move" => op_move(&mut m, it),
            "clone_record" | "clone" => op_clone(&mut m, it),
            "delete_record" | "delete" | "remove" => op_delete(&mut m, it),
            other => Step::Skip(format!("unknown palevel op '{other}'")),
        };
        let status = match step {
            Step::Done => match build_level(&m) {
                Ok(_) => ApplyStatus::Applied,
                Err(e) => {
                    m = before;
                    ApplyStatus::Skipped(format!("result does not encode: {e}"))
                }
            },
            Step::Skip(r) => {
                m = before;
                ApplyStatus::Skipped(r)
            }
        };
        outcomes.push(ApplyOutcome { op, status });
    }
    Ok((build_level(&m)?, outcomes))
}

/// Byte round-trip check of one file: (exact, raw object count).
pub fn roundtrip_check(b: &[u8]) -> io::Result<(bool, usize)> {
    let m = parse_level(b)?;
    let exact = build_level(&m)? == b;
    Ok((exact, m.raw_objects()))
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn f(name: &str, ty: &str, kind: u16, size: u16, flags: u32) -> FieldDef {
        FieldDef {
            name: name.into(),
            type_name: ty.into(),
            kind,
            size,
            flags,
        }
    }

    fn tr(x: f32, z: f32) -> Vec<u8> {
        let mut v: Vec<u8> = [1.0f32, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, x, 5.0, z]
            .iter()
            .flat_map(|a| a.to_le_bytes())
            .collect();
        v.extend_from_slice(&[0u8; 4]);
        v
    }

    fn uuid(p: u8, sub: u8) -> Vec<u8> {
        let mut v = vec![p, p, p, p];
        v.extend_from_slice(&[0xaa; 11]);
        v.push(sub);
        v
    }

    /// Synthetic level: root doc + one chunk with 3 placed prefab
    /// instances (each with a nested light child) + footer.
    fn sample() -> Vec<u8> {
        let classes = vec![
            ClassDef {
                name: "SceneLevelDataReflect".into(),
                fields: vec![
                    f("_name", "staticstringA", 1, 1, 3),
                    f("_isLoadable", "bool", 0, 1, 0),
                ],
            },
            ClassDef {
                name: "object/lamp_01.prefab".into(),
                fields: vec![
                    f("_sceneObjectUid", "uint32", 0, 4, 0),
                    f("_sceneObjectUuid", "uuid", 0, 16, 0),
                    f("_worldTransform", "Transform", 0, 44, 0),
                    f("_tiledTransform", "Transform", 0, 44, 0),
                    f("_childSceneObjects", "ReflectObjectPtr", 7, 8, 0),
                    f("_tags", "staticstringA", 10, 1, 0),
                    f("_range", "float", 0, 4, 0),
                ],
            },
            ClassDef {
                name: "LightObject".into(),
                fields: vec![
                    f("_sceneObjectUid", "uint32", 0, 4, 0),
                    f("_sceneObjectUuid", "uuid", 0, 16, 0),
                    f("_worldTransform", "Transform", 0, 44, 0),
                    f("_colour", "float3", 0, 12, 0),
                    f("_note", "staticstringA", 1, 1, 0),
                ],
            },
        ];
        let child = |uid: u32, p: u8| Rec {
            mask: vec![0b0001_1111],
            cls: 2,
            cf: 1,
            id: vec![1, 0, 2, 0, 3, 0, 4, 0],
            a: 0,
            names: vec![(0, b"Light".to_vec())],
            vals: vec![
                (0, Val::Bytes(uid.to_le_bytes().to_vec())),
                (1, Val::Bytes(uuid(p, 1))),
                (2, Val::Bytes(tr(0.0, 1.0))),
                (
                    3,
                    Val::Bytes(
                        [1.0f32, 0.5, 0.2]
                            .iter()
                            .flat_map(|a| a.to_le_bytes())
                            .collect(),
                    ),
                ),
                (4, Val::Str(Str::Inline(b"warm".to_vec()))),
            ],
        };
        let lamp = |uid: u32, cu: u32, p: u8, x: f32| Obj {
            cls: 1,
            parent: -1,
            body: Body::Top {
                mask: vec![0b0100_1111],
                pre: vec![0; 4],
                vals: vec![
                    (0, Val::Bytes(uid.to_le_bytes().to_vec())),
                    (1, Val::Bytes(uuid(p, 0))),
                    (2, Val::Bytes(tr(x, -40.0))),
                    (3, Val::Bytes(tr(x + 1000.0, 960.0))),
                    (
                        4,
                        Val::List {
                            hdr: vec![0, 1, 0, 0, 0, 2, 0, 0, 0, 0xff, 0xff, 0xff, 0xff],
                            ids: vec![],
                            items: vec![child(cu, p)],
                        },
                    ),
                    (5, Val::Null),
                    (6, Val::Bytes(2.5f32.to_le_bytes().to_vec())),
                ],
                tail: vec![],
            },
        };
        let mut chunk_doc = Doc {
            format: 4,
            zero: 0,
            hash: Some(0x1122_3344_5566_7788),
            version: 15,
            classes: classes.clone(),
            strings: vec![b"camp".to_vec()],
            resources: Some(vec![(7, b"object/lamp_01.prefab".to_vec())]),
            objects: vec![
                lamp(10, 11, 0x10, -100.0),
                lamp(20, 21, 0x20, -110.0),
                lamp(30, 31, 0x30, -120.0),
            ],
        };
        chunk_doc.objects[2].parent = -1;
        let root = Doc {
            format: 4,
            zero: 0,
            hash: Some(5),
            version: 15,
            classes,
            strings: vec![b"camp".to_vec()],
            resources: Some(vec![]),
            objects: vec![Obj {
                cls: 0,
                parent: -1,
                body: Body::Top {
                    mask: vec![0b11],
                    pre: vec![0; 4],
                    vals: vec![(0, Val::Str(Str::Index(0))), (1, Val::Bytes(vec![1]))],
                    tail: vec![],
                },
            }],
        };
        let m = LevelFile {
            parc: Some([
                Parc {
                    kind: 0,
                    v: 8,
                    hash: 9,
                    z: 0,
                },
                Parc {
                    kind: 0,
                    v: 8,
                    hash: 0,
                    z: 0,
                },
            ]),
            doc: root,
            chunks: vec![Chunk {
                flag: 0,
                parc: Parc {
                    kind: 2,
                    v: 8,
                    hash: 3,
                    z: 0,
                },
                doc: chunk_doc,
            }],
            footer: Some(Footer {
                end_flag: None,
                counter: 40,
                sublevels: vec![b"camp_sub_01".to_vec()],
            }),
            tail: vec![],
        };
        build_level(&m).unwrap()
    }

    fn intent(v: Value) -> Intent {
        Intent::from_value(&v).unwrap()
    }

    fn pos(b: &[u8], uid: u32) -> [f32; 3] {
        let rows = list_palevel_objects(b).unwrap();
        let r = rows.iter().find(|r| r["uid"] == json!(uid)).unwrap();
        let p = r["position"].as_array().unwrap();
        [
            p[0].as_f64().unwrap() as f32,
            p[1].as_f64().unwrap() as f32,
            p[2].as_f64().unwrap() as f32,
        ]
    }

    #[test]
    fn synthetic_round_trip_bytes_and_json() {
        let b = sample();
        let m = parse_level(&b).unwrap();
        assert_eq!(m.raw_objects(), 0);
        assert_eq!(build_level(&m).unwrap(), b);
        let j = decode_palevel(&b).unwrap();
        let text = serde_json::to_string(&j).unwrap();
        let back: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(encode_palevel(&back).unwrap(), b);
        assert_eq!(roundtrip_check(&b).unwrap(), (true, 0));
        assert_eq!(palevel_uids(&b).unwrap(), vec![10, 11, 20, 21, 30, 31]);
    }

    #[test]
    fn self_address_is_checked() {
        let mut b = sample();
        // corrupt the first child record's self-address: the object falls back to raw
        let light = b
            .windows(9)
            .position(|w| w == b"\x05\x00\x00\x00Light")
            .unwrap()
            + 4;
        let self_at = light - 4 - 2 - 2 - 2 - 4; // len, u16 b, u16 n, u16 a, u32 self
        b[self_at] ^= 0xff;
        let m2 = parse_level(&b).unwrap();
        assert_eq!(m2.raw_objects(), 1);
        assert_eq!(build_level(&m2).unwrap(), b);
        // structural edits are refused while a raw object exists
        let (out, oc) =
            apply_palevel_intents(&b, &[intent(json!({"op": "delete_record", "key": 20}))])
                .unwrap();
        assert!(matches!(oc[0].status, ApplyStatus::Skipped(_)));
        assert_eq!(out, b);
    }

    #[test]
    fn move_set_clone_delete() {
        let b = sample();
        let ints = vec![
            intent(json!({"op": "move", "key": 10, "value": {"x": 3.0, "z": -1.5}})),
            intent(json!({"op": "set", "key": 21, "field": "_colour", "value": [0.0, 0.0, 1.0]})),
            intent(
                json!({"op": "set", "key": 20, "field": "_childSceneObjects[0]._note", "value": {"inline": "blue"}}),
            ),
            intent(
                json!({"op": "clone_record", "source_key": 30, "value": {"move": [2.0, 0.0, 0.0]}}),
            ),
            intent(json!({"op": "delete_record", "key": 20})),
            intent(json!({"op": "delete_record", "key": 11})),
            intent(json!({"op": "set", "key": 10, "field": "_nope", "value": 1})),
            intent(json!({"op": "move", "key": 999, "value": [1, 0, 0]})),
        ];
        let (out, oc) = apply_palevel_intents(&b, &ints).unwrap();
        let st: Vec<bool> = oc
            .iter()
            .map(|o| matches!(o.status, ApplyStatus::Applied))
            .collect();
        assert_eq!(
            st,
            vec![true, true, true, true, true, false, false, false],
            "{oc:?}"
        );
        // still a valid, byte-stable file
        assert_eq!(roundtrip_check(&out).unwrap(), (true, 0));
        assert_eq!(pos(&out, 10), [-97.0, 5.0, -41.5]);
        // clone: uids 41, 42 (after counter 40), placed +2 m, counter raised
        assert_eq!(palevel_uids(&out).unwrap(), vec![10, 11, 30, 31, 41, 42]);
        assert_eq!(pos(&out, 41), [-118.0, 5.0, -40.0]);
        let m = parse_level(&out).unwrap();
        assert_eq!(m.footer.as_ref().unwrap().counter, 42);
        // child uuid keeps the parent's (new) prefix
        let (_, pres) = file_ids(&m);
        assert_eq!(pres.len(), 3);
        // the unchanged object is byte-identical in JSON
        let a = decode_palevel(&b).unwrap();
        let z = decode_palevel(&out).unwrap();
        assert_eq!(
            a["chunks"][0]["doc"]["objects"][2],
            z["chunks"][0]["doc"]["objects"][1]
        );
    }

    #[test]
    fn clone_with_explicit_ids_is_exact() {
        let b = sample();
        let (out, oc) = apply_palevel_intents(
            &b,
            &[intent(json!({"op": "clone_record", "source_key": 10, "value": {"uids": [500, 77], "uuid_prefixes": {"10101010": "deadbeef"}}}))],
        )
        .unwrap();
        assert!(matches!(oc[0].status, ApplyStatus::Applied));
        assert_eq!(
            palevel_uids(&out).unwrap(),
            vec![10, 11, 20, 21, 30, 31, 500, 77]
        );
        assert_eq!(parse_level(&out).unwrap().footer.unwrap().counter, 500);
        assert!(out.windows(4).any(|w| w == [0xde, 0xad, 0xbe, 0xef]));
        // duplicate uid refused
        let (_, oc) = apply_palevel_intents(
            &b,
            &[intent(
                json!({"op": "clone_record", "source_key": 10, "value": {"uids": [20, 77]}}),
            )],
        )
        .unwrap();
        assert!(matches!(oc[0].status, ApplyStatus::Skipped(_)));
    }

    #[test]
    fn bad_input_is_an_error() {
        assert!(parse_level(b"PARC").is_err());
        assert!(parse_level(&[0xff, 0xff, 0x05, 0x00, 0, 0]).is_err());
        let b = sample();
        assert!(parse_level(&b[..b.len() / 2]).is_err());
        assert!(encode_palevel(&json!({"container": "zip"})).is_err());
    }

    /// Env-gated vanilla sweep: every `.palevel` / `.prefab` under
    /// `DMM_PARSER_PALEVEL_DIR` must rebuild byte-exact (also via JSON
    /// text) with no undecoded objects.
    #[test]
    fn vanilla_palevel_round_trip_env() {
        let Ok(root) = std::env::var("DMM_PARSER_PALEVEL_DIR") else {
            return;
        };
        let mut stack = vec![std::path::PathBuf::from(root)];
        let (mut n, mut bad_files, mut raw) = (0usize, Vec::new(), 0usize);
        while let Some(p) = stack.pop() {
            for e in std::fs::read_dir(&p).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
                if ext != "palevel" && ext != "prefab" {
                    continue;
                }
                let b = std::fs::read(&p).unwrap();
                n += 1;
                let ok = match parse_level(&b) {
                    Ok(m) => {
                        raw += m.raw_objects();
                        let j = serde_json::to_string(&level_to_json(&m)).unwrap();
                        build_level(&m).ok().as_deref() == Some(&b[..])
                            && encode_palevel(&serde_json::from_str(&j).unwrap())
                                .ok()
                                .as_deref()
                                == Some(&b[..])
                    }
                    Err(_) => false,
                };
                if !ok {
                    bad_files.push(p.display().to_string());
                }
            }
        }
        eprintln!(
            "palevel sweep: {n} files, {} not exact, {raw} raw objects",
            bad_files.len()
        );
        assert!(n > 0);
        assert!(
            bad_files.is_empty(),
            "{:?}",
            &bad_files[..bad_files.len().min(10)]
        );
        assert_eq!(raw, 0);
    }
}
