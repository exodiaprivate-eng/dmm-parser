// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Field-level codec for `.paschedule` and `.paschedulepath` bodies
//! (game 2.04.03 - 2.04.08).
//!
//! [`super::paschedule::TypedPascheduleFile`] and
//! [`super::paschedulepath::TypedPaschedulePathFile`] keep the body as
//! opaque bytes. This module decodes the WHOLE file into named JSON
//! fields and encodes it back byte for byte, so a mod can change an NPC's
//! day (times, spots, actions), add or remove stops (size-changing), and
//! edit the walking paths that go with them.
//!
//! ## Checks (vanilla data)
//!
//! - Build 26345091 (2.04.03) and 26480954 (2.04.08), base + dlc_01:
//!   every `.paschedule` (4,407 / 4,408) and every `.paschedulepath`
//!   (3,808 / 3,809) decodes, re-encodes byte-exact, and survives a JSON
//!   text round trip.
//! - Header `hash` = [`hashlittle`] of the file stem, every file.
//! - Key index after the items = items sorted by `(k1, k2)`, every file
//!   (so the encoder can rebuild it after items are added or removed).
//! - Pair rules ([`check_schedule_path_pair`]) hold on every vanilla pair.
//! - In game (2.04.08): a size-changing write (one item 2 -> 4 top nodes,
//!   path record rebuilt to 4 paths) loaded and the NPC followed the new
//!   times and spots.
//!
//! ## `.paschedule` layout (sequential, no backtracking)
//!
//! ```text
//! u32 version | u32 hash (= hashlittle(file stem)) | u8 type | KEY key | str name
//! u32 a0 | u8 a1 | list<TRACK> tracks | list<ITEMREC> items | POST post | u64 stamp
//! u32 n (= item count) + n x KEY (item keys sorted by k1, k2)
//! list<PATHLIST> lists_a | list<PATHLIST> lists_b
//! ```
//!
//! `str` = u32 length + latin-1 bytes; `list<T>` = u32 count + items.
//! An ITEMREC is one spawned actor and its day: `key`, `name`, `type`,
//! then `fields` = the ITEM record (character, spawn transform, flags,
//! tags, refs, pose and the NODE tree = the stops of the day).
//!
//! ## `.paschedulepath` layout
//!
//! ```text
//! u64 id | u32 n (= item count of the .paschedule)
//! n x { u32 hash, u32 step (= item key hash, k1), u8 has, [BODY if has] }
//! BODY = u32 nodes (= node count of the item), u32 n_paths (= top-level
//!        nodes), u32 n_loops, n_loops x polyline,
//!        n_paths x { polyline pts, list<NREF> refs, list<NREF> refs2,
//!                    list<polyline> between (all paths but the last) }
//! polyline = list<vec3>
//! ```
//!
//! ## Field meanings
//!
//! Names are ours. Confirmed in game: node `start_h/start_m/end_h/end_m`
//! (time window; start == end = switch time, the NPC stays until the next
//! node), node `pos` (local to the schedule context origin), `yaw`,
//! `act_a/act_b/act_c` (action at the stop), `mode` (17 = pacing between
//! the children, 0 = stay). Checked on data: `seq_hash` = hashlittle of a
//! schedule name, item `start_state` = hashlittle(lower-case tag),
//! item `character` = characterinfo key. Other names (`u..`, `h..`,
//! `b..`, `f..`) are offsets/types only.
//!
//! ## JSON rules
//!
//! Raw byte runs are lower-case hex strings; an f32 that is NaN or
//! infinite is `{"f32bits": "<8 hex, little-endian>"}`; integers are
//! range-checked on encode. `index_order` (item number per key-index
//! entry) is optional on encode: omit it to rebuild the canonical order.
//!
//! ## Intents
//!
//! [`apply_paschedule_intents`] / [`apply_paschedulepath_intents`] take
//! the usual [`Intent`] list. Scope: `key` = item `k1` (`.paschedule`
//! item, or `.paschedulepath` record with `step == k1`, so ONE key
//! addresses an item and its path record); `entry` = item `name`
//! (`.paschedule` only); neither = file root. `field` = path inside the
//! scope, e.g. `fields.nodes[1].start_h`. Ops: `set` (`new` or `value`;
//! the target must exist), `array_append` (`field` = list, `value`),
//! `array_insert` / `array_remove` (`field` ends in `[i]`). Unknown
//! fields or indexes are reported as `Skipped` with a reason; a value of
//! the wrong type fails the encode with the field named.

use std::collections::{HashMap, VecDeque};
use std::io;

use serde_json::{Map, Number, Value};

use crate::intents::{ApplyOutcome, ApplyStatus, Intent};

const LIST_MAX: u32 = 200_000;
const STR_MAX: u32 = 4096;
const BODY_LIST_MAX: u32 = 10_000;

fn bad(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

fn ctx<T>(r: io::Result<T>, name: &str) -> io::Result<T> {
    r.map_err(|e| bad(format!("{}: {}", name, e)))
}

// ── Spec engine ──────────────────────────────────────────────────────

/// Wire type of one field.
#[derive(Clone, Copy, Debug)]
enum Ty {
    U8,
    I16,
    U32,
    I32,
    U64,
    F32,
    /// `n` f32 as a JSON array (vec3 = 3, quat = 4, aabb = 6).
    Floats(usize),
    /// 10 f32 as `{"pos": [3], "rot": [4], "scale": [3]}`.
    Tf,
    /// Fixed byte run as a hex string.
    Raw(usize),
    /// u32 length + latin-1 bytes.
    Str,
    /// u32 count + elements.
    List(&'static Ty),
    /// Named record (see [`spec`]).
    Rec(&'static str),
    /// Present only when the named earlier u8 field of the same record
    /// is non-zero.
    If(&'static str, &'static Ty),
}

const VEC3: Ty = Ty::Floats(3);
const QUAT: Ty = Ty::Floats(4);

#[derive(Debug)]
struct Field(&'static str, Ty);

use Ty::*;

const KEY: &[Field] = &[Field("hash", U32), Field("k1", U32), Field("k2", U32)];

// .paschedule frame
const TRACK: &[Field] = &[
    Field("hash", U32),
    Field("splines", List(&List(&Raw(30)))),
    Field("kind", U8),
    Field("begin", U32),
    Field("end", U32),
    Field("aabb", Raw(24)),
    Field("p", U8),
    Field("q", U8),
    Field("qv", If("q", &U32)),
    Field("r", U32),
    Field("s", U32),
    Field("fps", U32),
    Field("t", U32),
];
const ITEMREC: &[Field] = &[
    Field("key", Rec("KEY")),
    Field("name", Str),
    Field("type", U8),
    Field("fields", Rec("ITEM")),
];
const PATHLIST: &[Field] = &[Field("flag", U8), Field("points", List(&Raw(66)))];

// schedule item payload
const NODE: &[Field] = &[
    Field("tag", U32),
    Field("u04", U8),
    Field("yaw", F32),
    Field("act_a", U32),
    Field("act_b", U32),
    Field("act_c", U32),
    Field("h15", U32),
    Field("h19", U32),
    Field("u1d", U32),
    Field("seq_hash", U32),
    Field("seq_index", I32),
    Field("start_h", U8),
    Field("start_m", U8),
    Field("end_h", U8),
    Field("end_m", U8),
    Field("u2d", U8),
    Field("mode", U8),
    Field("u2f", U8),
    Field("u30", U8),
    Field("u31", U32),
    Field("h35", U32),
    Field("r39", Raw(6)),
    Field("obj", Raw(12)),
    Field("u4b", U32),
    Field("i4f", I32),
    Field("b53", U8),
    Field("b54", U8),
    Field("b55", U8),
    Field("b56", U8),
    Field("b57", U8),
    Field("h58", U32),
    Field("b5c", U8),
    Field("b5d", U8),
    Field("b5e", U8),
    Field("b5f", U8),
    Field("links", List(&Rec("LINK"))),
    Field("u9a", U8),
    Field("u9b", U32),
    Field("u9c", U32),
    Field("children", List(&Rec("NODE"))),
    Field("ids", List(&U32)),
    Field("pos", VEC3),
    Field("u_end", U32),
];
const LINK: &[Field] = &[
    Field("u0", U32),
    Field("index", I32),
    Field("obj", Raw(12)),
    Field("n", U32),
];
const TAG: &[Field] = &[Field("name", Str), Field("value", U32)];
const REF: &[Field] = &[Field("u", U8), Field("seq_hash", U32), Field("value", U32)];
const PAIR: &[Field] = &[Field("hash", U32), Field("value", U32)];
const ITEM: &[Field] = &[
    Field("u0", U32),
    Field("nodes", List(&Rec("NODE"))),
    Field("character", U32),
    Field("s_a", U32),
    Field("s_b", U32),
    Field("has_tf", U8),
    Field("spawn", Tf),
    Field("b_index", I32),
    Field("b_hash", U32),
    Field("f08", U8),
    Field("f09", U8),
    Field("f0a", U8),
    Field("f0b", U8),
    Field("f0c", U8),
    Field("f0d", U8),
    Field("f0e", U8),
    Field("f0f", U8),
    Field("f10", U8),
    Field("f11", U8),
    Field("f12", U8),
    Field("start_state", U32),
    Field("h17", U32),
    Field("z1b", Raw(24)),
    Field("b33", U8),
    Field("b34", U32),
    Field("tags", List(&Rec("TAG"))),
    Field("refs", List(&Rec("REF"))),
    Field("t_a", U8),
    Field("t_b", U8),
    Field("t_tf", Tf),
    Field("pose_a", I16),
    Field("pose_b", I16),
    Field("pose_off", VEC3),
    Field("pose", List(&QUAT)),
    Field("q_a", U32),
    Field("q_hash", U32),
    Field("q_index", I32),
    Field("q_h", U32),
    Field("q_character", U32),
    Field("q_hash2", U32),
    Field("q_index2", I32),
    Field("pairs", List(&Rec("PAIR"))),
];
// file-level lists after the last item
const POST: &[Field] = &[
    Field("gimmicks", List(&Rec("GIMMICK"))),
    Field("decals", List(&Rec("DECAL"))),
    Field("areas", List(&Rec("AREA"))),
    Field("characters", List(&U32)),
];
const GIMMICK: &[Field] = &[
    Field("u", U8),
    Field("hash", U32),
    Field("index", U32),
    Field("tf", Tf),
    Field("kind", U32),
    Field("id", U32),
    Field("r8", Raw(8)),
    Field("tags", List(&Rec("TAG"))),
];
const DECAL: &[Field] = &[
    Field("index", U32),
    Field("flag", U8),
    Field("tf", Tf),
    Field("prefab", Str),
    Field("i7f", I32),
    Field("flag2", U8),
    Field("tf2", Tf),
];
const AREA: &[Field] = &[
    Field("shape", U32),
    Field("u", U32),
    Field("flag", U8),
    Field("pos", VEC3),
    Field("rot", QUAT),
    Field("ext", VEC3),
    Field("flag2", U8),
];
// .paschedulepath
const PREC: &[Field] = &[Field("hash", U32), Field("step", U32), Field("has", U8)];
const NREF: &[Field] = &[Field("kind", U8), Field("index", U32)];

const SCHEDULE_HEAD: &[Field] = &[
    Field("version", U32),
    Field("hash", U32),
    Field("type", U8),
    Field("key", Rec("KEY")),
    Field("name", Str),
    Field("a0", U32),
    Field("a1", U8),
    Field("tracks", List(&Rec("TRACK"))),
    Field("items", List(&Rec("ITEMREC"))),
    Field("post", Rec("POST")),
    Field("stamp", U64),
];

fn spec(name: &str) -> &'static [Field] {
    match name {
        "KEY" => KEY,
        "TRACK" => TRACK,
        "ITEMREC" => ITEMREC,
        "PATHLIST" => PATHLIST,
        "NODE" => NODE,
        "LINK" => LINK,
        "TAG" => TAG,
        "REF" => REF,
        "PAIR" => PAIR,
        "ITEM" => ITEM,
        "POST" => POST,
        "GIMMICK" => GIMMICK,
        "DECAL" => DECAL,
        "AREA" => AREA,
        "PREC" => PREC,
        "NREF" => NREF,
        _ => &[],
    }
}

// ── Reader ───────────────────────────────────────────────────────────

struct Rd<'a> {
    b: &'a [u8],
    o: usize,
}

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self
            .o
            .checked_add(n)
            .filter(|&e| e <= self.b.len())
            .ok_or_else(|| bad(format!("unexpected end of data at 0x{:x}", self.o)))?;
        let s = &self.b[self.o..end];
        self.o = end;
        Ok(s)
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn count(&mut self, max: u32) -> io::Result<usize> {
        let at = self.o;
        let n = self.u32()?;
        if n > max {
            return Err(bad(format!("count {} at 0x{:x} over limit {}", n, at, max)));
        }
        Ok(n as usize)
    }
}

fn hex(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 15) as usize] as char);
    }
    s
}

fn unhex(s: &str, n: usize) -> io::Result<Vec<u8>> {
    let d = s.as_bytes();
    if d.len() != n * 2 {
        return Err(bad(format!("hex string needs {} bytes", n)));
    }
    let nib = |c: u8| -> io::Result<u8> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(bad("bad hex digit")),
        }
    };
    (0..n)
        .map(|i| Ok(nib(d[2 * i])? << 4 | nib(d[2 * i + 1])?))
        .collect()
}

fn f32_value(b: &[u8]) -> Value {
    let v = f32::from_le_bytes(b.try_into().unwrap());
    match Number::from_f64(v as f64) {
        Some(n) if v.is_finite() => Value::Number(n),
        _ => {
            let mut m = Map::new();
            m.insert("f32bits".into(), Value::String(hex(b)));
            Value::Object(m)
        }
    }
}

fn floats(r: &mut Rd, n: usize) -> io::Result<Vec<Value>> {
    let b = r.take(4 * n)?;
    Ok(b.chunks_exact(4).map(f32_value).collect())
}

fn read_ty(r: &mut Rd, t: &Ty) -> io::Result<Value> {
    Ok(match *t {
        U8 => Value::from(r.take(1)?[0]),
        I16 => Value::from(i16::from_le_bytes(r.take(2)?.try_into().unwrap())),
        U32 => Value::from(r.u32()?),
        I32 => Value::from(i32::from_le_bytes(r.take(4)?.try_into().unwrap())),
        U64 => Value::from(u64::from_le_bytes(r.take(8)?.try_into().unwrap())),
        F32 => f32_value(r.take(4)?),
        Floats(n) => Value::Array(floats(r, n)?),
        Tf => {
            let v = floats(r, 10)?;
            let mut m = Map::new();
            m.insert("pos".into(), Value::Array(v[0..3].to_vec()));
            m.insert("rot".into(), Value::Array(v[3..7].to_vec()));
            m.insert("scale".into(), Value::Array(v[7..10].to_vec()));
            Value::Object(m)
        }
        Raw(n) => Value::String(hex(r.take(n)?)),
        Str => {
            let n = r.count(STR_MAX)?;
            Value::String(r.take(n)?.iter().map(|&c| c as char).collect())
        }
        List(et) => {
            let n = r.count(LIST_MAX)?;
            let mut v = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                v.push(read_ty(r, et)?);
            }
            Value::Array(v)
        }
        Rec(name) => Value::Object(read_fields(r, spec(name))?),
        If(_, et) => read_ty(r, et)?,
    })
}

fn flag_set(m: &Map<String, Value>, flag: &str) -> bool {
    m.get(flag).and_then(Value::as_u64).unwrap_or(0) != 0
}

fn read_fields(r: &mut Rd, fields: &[Field]) -> io::Result<Map<String, Value>> {
    let mut m = Map::new();
    for Field(name, t) in fields {
        if let If(flag, _) = t {
            if !flag_set(&m, flag) {
                continue;
            }
        }
        let v = ctx(read_ty(r, t), name)?;
        m.insert((*name).into(), v);
    }
    Ok(m)
}

// ── Writer ───────────────────────────────────────────────────────────

fn int(v: &Value, lo: i128, hi: i128) -> io::Result<i128> {
    let x = v
        .as_i64()
        .map(i128::from)
        .or_else(|| v.as_u64().map(i128::from))
        .ok_or_else(|| bad(format!("expected an integer, got {}", v)))?;
    if x < lo || x > hi {
        return Err(bad(format!("{} out of range {}..={}", x, lo, hi)));
    }
    Ok(x)
}

fn write_f32(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
    if let Some(bits) = v.get("f32bits").and_then(Value::as_str) {
        w.extend(unhex(bits, 4)?);
    } else {
        let x = v
            .as_f64()
            .ok_or_else(|| bad(format!("expected a number, got {}", v)))?;
        w.extend((x as f32).to_le_bytes());
    }
    Ok(())
}

fn write_floats(w: &mut Vec<u8>, v: &Value, n: usize) -> io::Result<()> {
    let a = v
        .as_array()
        .filter(|a| a.len() == n)
        .ok_or_else(|| bad(format!("expected an array of {} numbers", n)))?;
    for (i, x) in a.iter().enumerate() {
        ctx(write_f32(w, x), &format!("[{}]", i))?;
    }
    Ok(())
}

fn write_ty(w: &mut Vec<u8>, t: &Ty, v: &Value) -> io::Result<()> {
    match *t {
        U8 => w.push(int(v, 0, u8::MAX as i128)? as u8),
        I16 => w.extend((int(v, i16::MIN as i128, i16::MAX as i128)? as i16).to_le_bytes()),
        U32 => w.extend((int(v, 0, u32::MAX as i128)? as u32).to_le_bytes()),
        I32 => w.extend((int(v, i32::MIN as i128, i32::MAX as i128)? as i32).to_le_bytes()),
        U64 => w.extend((int(v, 0, u64::MAX as i128)? as u64).to_le_bytes()),
        F32 => write_f32(w, v)?,
        Floats(n) => write_floats(w, v, n)?,
        Tf => {
            ctx(
                write_floats(w, v.get("pos").unwrap_or(&Value::Null), 3),
                "pos",
            )?;
            ctx(
                write_floats(w, v.get("rot").unwrap_or(&Value::Null), 4),
                "rot",
            )?;
            ctx(
                write_floats(w, v.get("scale").unwrap_or(&Value::Null), 3),
                "scale",
            )?;
        }
        Raw(n) => {
            let s = v.as_str().ok_or_else(|| bad("expected a hex string"))?;
            w.extend(unhex(s, n)?);
        }
        Str => {
            let s = v.as_str().ok_or_else(|| bad("expected a string"))?;
            let b: Vec<u8> = s
                .chars()
                .map(|c| u8::try_from(c as u32).map_err(|_| bad("string is not latin-1")))
                .collect::<io::Result<_>>()?;
            if b.len() > STR_MAX as usize {
                return Err(bad("string too long"));
            }
            w.extend((b.len() as u32).to_le_bytes());
            w.extend(b);
        }
        List(et) => {
            let a = v.as_array().ok_or_else(|| bad("expected an array"))?;
            if a.len() > LIST_MAX as usize {
                return Err(bad("list too long"));
            }
            w.extend((a.len() as u32).to_le_bytes());
            for (i, x) in a.iter().enumerate() {
                ctx(write_ty(w, et, x), &format!("[{}]", i))?;
            }
        }
        Rec(name) => {
            let m = v.as_object().ok_or_else(|| bad("expected an object"))?;
            write_fields(w, spec(name), m)?;
        }
        If(_, et) => write_ty(w, et, v)?,
    }
    Ok(())
}

fn write_fields(w: &mut Vec<u8>, fields: &[Field], m: &Map<String, Value>) -> io::Result<()> {
    for Field(name, t) in fields {
        if let If(flag, _) = t {
            if !flag_set(m, flag) {
                continue;
            }
        }
        let v = m
            .get(*name)
            .ok_or_else(|| bad(format!("missing field '{}'", name)))?;
        ctx(write_ty(w, t, v), name)?;
    }
    Ok(())
}

/// A default (all-zero, empty-list) JSON value for a named record:
/// `NODE`, `LINK`, `TAG`, `REF`, `PAIR`, `ITEM`, `ITEMREC`, `GIMMICK`,
/// `DECAL`, `AREA`, `TRACK`, `PATHLIST`, `KEY`, `NREF`. Useful as an
/// `array_append` template; vanilla copies are usually a better start.
pub fn paschedule_template(record: &str) -> Option<Value> {
    fn dflt(t: &Ty) -> Value {
        match *t {
            U8 | I16 | U32 | I32 | U64 => Value::from(0),
            F32 => Value::from(0.0),
            Floats(n) => Value::Array(vec![Value::from(0.0); n]),
            Tf => {
                let mut m = Map::new();
                m.insert("pos".into(), Value::Array(vec![Value::from(0.0); 3]));
                m.insert(
                    "rot".into(),
                    Value::Array(
                        vec![0.0, 0.0, 0.0, 1.0]
                            .into_iter()
                            .map(Value::from)
                            .collect(),
                    ),
                );
                m.insert("scale".into(), Value::Array(vec![Value::from(1.0); 3]));
                Value::Object(m)
            }
            Raw(n) => Value::String("00".repeat(n)),
            Str => Value::String(String::new()),
            List(_) => Value::Array(vec![]),
            Rec(name) => {
                let mut m = Map::new();
                for Field(n, t) in spec(name) {
                    if !matches!(t, If(..)) {
                        m.insert((*n).into(), dflt(t));
                    }
                }
                Value::Object(m)
            }
            If(_, et) => dflt(et),
        }
    }
    const NAMES: &[&str] = &[
        "KEY", "TRACK", "ITEMREC", "PATHLIST", "NODE", "LINK", "TAG", "REF", "PAIR", "ITEM",
        "POST", "GIMMICK", "DECAL", "AREA", "PREC", "NREF",
    ];
    NAMES.iter().find(|n| **n == record).map(|n| dflt(&Rec(n)))
}

// ── .paschedule ──────────────────────────────────────────────────────

fn key_bytes(item: &Value) -> io::Result<[u8; 12]> {
    let k = item.get("key").ok_or_else(|| bad("item without key"))?;
    let mut out = [0u8; 12];
    for (i, n) in ["hash", "k1", "k2"].iter().enumerate() {
        let x = int(k.get(*n).unwrap_or(&Value::Null), 0, u32::MAX as i128)? as u32;
        out[4 * i..4 * i + 4].copy_from_slice(&x.to_le_bytes());
    }
    Ok(out)
}

fn item_k(item: &Value, n: &str) -> u64 {
    item.get("key")
        .and_then(|k| k.get(n))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

/// Canonical key-index order: item numbers sorted by `(k1, k2)` (stable).
/// Every vanilla file follows it.
pub fn canonical_index_order(items: &[Value]) -> Vec<usize> {
    let mut o: Vec<usize> = (0..items.len()).collect();
    o.sort_by_key(|&i| (item_k(&items[i], "k1"), item_k(&items[i], "k2")));
    o
}

/// Decode a whole `.paschedule` file to JSON.
pub fn decode_paschedule(data: &[u8]) -> io::Result<Value> {
    let mut r = Rd { b: data, o: 0 };
    let mut m = read_fields(&mut r, SCHEDULE_HEAD)?;
    let keys: Vec<[u8; 12]> = m["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(key_bytes)
        .collect::<io::Result<_>>()?;
    let n = r.u32()? as usize;
    if n != keys.len() {
        return Err(bad(format!(
            "key index count {} != item count {}",
            n,
            keys.len()
        )));
    }
    let mut by_key: HashMap<[u8; 12], VecDeque<usize>> = HashMap::new();
    for (i, k) in keys.into_iter().enumerate() {
        by_key.entry(k).or_default().push_back(i);
    }
    let mut order = Vec::with_capacity(n);
    for _ in 0..n {
        let k: [u8; 12] = r.take(12)?.try_into().unwrap();
        let i = by_key
            .get_mut(&k)
            .and_then(VecDeque::pop_front)
            .ok_or_else(|| bad("key index entry is not an item key"))?;
        order.push(Value::from(i));
    }
    m.insert("index_order".into(), Value::Array(order));
    let pl = List(&Rec("PATHLIST"));
    m.insert("lists_a".into(), ctx(read_ty(&mut r, &pl), "lists_a")?);
    m.insert("lists_b".into(), ctx(read_ty(&mut r, &pl), "lists_b")?);
    if r.o != data.len() {
        return Err(bad(format!("{} trailing bytes", data.len() - r.o)));
    }
    Ok(Value::Object(m))
}

/// Encode a `.paschedule` from the JSON of [`decode_paschedule`].
pub fn encode_paschedule(doc: &Value) -> io::Result<Vec<u8>> {
    let m = doc.as_object().ok_or_else(|| bad("expected an object"))?;
    let mut w = Vec::new();
    write_fields(&mut w, SCHEDULE_HEAD, m)?;
    let items = m["items"].as_array().unwrap();
    let given: Option<Vec<usize>> = m
        .get("index_order")
        .and_then(Value::as_array)
        .and_then(|a| a.iter().map(|x| x.as_u64().map(|x| x as usize)).collect());
    let order = match given {
        Some(o)
            if {
                let mut s = o.clone();
                s.sort_unstable();
                s == (0..items.len()).collect::<Vec<_>>()
            } =>
        {
            o
        }
        _ => canonical_index_order(items),
    };
    w.extend((items.len() as u32).to_le_bytes());
    for i in order {
        w.extend(key_bytes(&items[i])?);
    }
    let pl = List(&Rec("PATHLIST"));
    for n in ["lists_a", "lists_b"] {
        let v = m
            .get(n)
            .ok_or_else(|| bad(format!("missing field '{}'", n)))?;
        ctx(write_ty(&mut w, &pl, v), n)?;
    }
    Ok(w)
}

// ── .paschedulepath ──────────────────────────────────────────────────

const POLY: Ty = List(&VEC3);
const NREFS: Ty = List(&Rec("NREF"));
const BETWEEN: Ty = List(&POLY);

/// Decode a whole `.paschedulepath` file to JSON.
pub fn decode_paschedulepath(data: &[u8]) -> io::Result<Value> {
    let mut r = Rd { b: data, o: 0 };
    let id = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
    let n = r.count(LIST_MAX)?;
    let mut recs = Vec::with_capacity(n.min(4096));
    for i in 0..n {
        let mut rec = ctx(read_fields(&mut r, PREC), &format!("records[{}]", i))?;
        let has = rec["has"].as_u64().unwrap();
        if has > 1 {
            return Err(bad(format!("records[{}]: has = {}", i, has)));
        }
        if has == 1 {
            let body = ctx(read_body(&mut r), &format!("records[{}].body", i))?;
            rec.insert("body".into(), body);
        }
        recs.push(Value::Object(rec));
    }
    if r.o != data.len() {
        return Err(bad(format!("{} trailing bytes", data.len() - r.o)));
    }
    let mut m = Map::new();
    m.insert("id".into(), Value::from(id));
    m.insert("records".into(), Value::Array(recs));
    Ok(Value::Object(m))
}

fn read_body(r: &mut Rd) -> io::Result<Value> {
    let nodes = r.u32()?;
    let np = r.count(BODY_LIST_MAX)?;
    let nl = r.count(BODY_LIST_MAX)?;
    let mut loops = Vec::with_capacity(nl);
    for _ in 0..nl {
        loops.push(read_ty(r, &POLY)?);
    }
    let mut paths = Vec::with_capacity(np);
    for i in 0..np {
        let mut e = Map::new();
        e.insert("pts".into(), read_ty(r, &POLY)?);
        e.insert("refs".into(), read_ty(r, &NREFS)?);
        e.insert("refs2".into(), read_ty(r, &NREFS)?);
        if i + 1 < np {
            e.insert("between".into(), read_ty(r, &BETWEEN)?);
        }
        paths.push(Value::Object(e));
    }
    let mut b = Map::new();
    b.insert("nodes".into(), Value::from(nodes));
    b.insert("loops".into(), Value::Array(loops));
    b.insert("paths".into(), Value::Array(paths));
    Ok(Value::Object(b))
}

fn write_body(w: &mut Vec<u8>, b: &Value) -> io::Result<()> {
    let nodes = int(b.get("nodes").unwrap_or(&Value::Null), 0, u32::MAX as i128)? as u32;
    let empty = Vec::new();
    let loops = b.get("loops").and_then(Value::as_array).unwrap_or(&empty);
    let paths = b
        .get("paths")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing 'paths'"))?;
    if paths.len() > BODY_LIST_MAX as usize || loops.len() > BODY_LIST_MAX as usize {
        return Err(bad("too many paths or loops"));
    }
    w.extend(nodes.to_le_bytes());
    w.extend((paths.len() as u32).to_le_bytes());
    w.extend((loops.len() as u32).to_le_bytes());
    for (i, p) in loops.iter().enumerate() {
        ctx(write_ty(w, &POLY, p), &format!("loops[{}]", i))?;
    }
    for (i, e) in paths.iter().enumerate() {
        let at = format!("paths[{}]", i);
        let get = |n: &str| {
            e.get(n)
                .ok_or_else(|| bad(format!("{}: missing '{}'", at, n)))
        };
        ctx(write_ty(w, &POLY, get("pts")?), &at)?;
        ctx(write_ty(w, &NREFS, get("refs")?), &at)?;
        ctx(write_ty(w, &NREFS, get("refs2")?), &at)?;
        let between = e.get("between");
        if i + 1 < paths.len() {
            let empty = Value::Array(vec![]);
            ctx(write_ty(w, &BETWEEN, between.unwrap_or(&empty)), &at)?;
        } else if between
            .and_then(Value::as_array)
            .is_some_and(|a| !a.is_empty())
        {
            return Err(bad(format!(
                "{}: the last path cannot have 'between' polylines",
                at
            )));
        }
    }
    Ok(())
}

/// Encode a `.paschedulepath` from the JSON of [`decode_paschedulepath`].
/// Path and loop counts come from the list lengths; `between` is written
/// for every path but the last (empty if absent).
pub fn encode_paschedulepath(doc: &Value) -> io::Result<Vec<u8>> {
    let id = int(doc.get("id").unwrap_or(&Value::Null), 0, u64::MAX as i128)? as u64;
    let recs = doc
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing 'records'"))?;
    let mut w = Vec::new();
    w.extend(id.to_le_bytes());
    w.extend((recs.len() as u32).to_le_bytes());
    for (i, rec) in recs.iter().enumerate() {
        let at = format!("records[{}]", i);
        let m = rec
            .as_object()
            .ok_or_else(|| bad(format!("{}: expected an object", at)))?;
        ctx(write_fields(&mut w, PREC, m), &at)?;
        if flag_set(m, "has") {
            let body = m
                .get("body")
                .ok_or_else(|| bad(format!("{}: has = 1 but no body", at)))?;
            ctx(write_body(&mut w, body), &format!("{}.body", at))?;
        }
    }
    Ok(w)
}

// ── Cross-file rules ─────────────────────────────────────────────────

fn count_nodes(nodes: &[Value]) -> u64 {
    nodes
        .iter()
        .map(|n| {
            1 + count_nodes(
                n.get("children")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
            )
        })
        .sum()
}

/// Rules between a decoded `.paschedule` and its `.paschedulepath`
/// (same stem; `binary__` vs `pathbinary__`). Empty = all hold. Every
/// vanilla pair passes: record count = item count; record `(hash, step)`
/// = item key `(hash, k1)`; with a body, `body.nodes` = node count of the
/// item (all depths) and path count = top-level node count.
pub fn check_schedule_path_pair(schedule: &Value, path: &Value) -> Vec<String> {
    let empty = Vec::new();
    let items = schedule
        .get("items")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    let recs = path
        .get("records")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if items.len() != recs.len() {
        return vec![format!(
            "record count {} != item count {}",
            recs.len(),
            items.len()
        )];
    }
    let mut out = Vec::new();
    for (i, (it, rec)) in items.iter().zip(recs).enumerate() {
        let rk = (
            rec.get("hash").and_then(Value::as_u64),
            rec.get("step").and_then(Value::as_u64),
        );
        if rk != (Some(item_k(it, "hash")), Some(item_k(it, "k1"))) {
            out.push(format!(
                "records[{}]: (hash, step) != item key (hash, k1)",
                i
            ));
        }
        if let Some(body) = rec.get("body") {
            let nodes = it
                .get("fields")
                .and_then(|f| f.get("nodes"))
                .and_then(Value::as_array)
                .unwrap_or(&empty);
            if body.get("nodes").and_then(Value::as_u64) != Some(count_nodes(nodes)) {
                out.push(format!(
                    "records[{}]: body.nodes != node count of item {}",
                    i, i
                ));
            }
            let np = body
                .get("paths")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            if np != nodes.len() {
                out.push(format!(
                    "records[{}]: {} paths != {} top-level nodes",
                    i,
                    np,
                    nodes.len()
                ));
            }
        }
    }
    out
}

// ── Names ────────────────────────────────────────────────────────────

/// Jenkins lookup3 `hashlittle`. The game uses `initval = 0xC5EDE`:
/// `.paschedule` header hash = `hashlittle(file stem)`, node `seq_hash` =
/// `hashlittle(schedule name)`, item `start_state` =
/// `hashlittle(lower-case tag)`.
pub fn hashlittle(data: &[u8], initval: u32) -> u32 {
    let mut a = 0xdead_beefu32
        .wrapping_add(data.len() as u32)
        .wrapping_add(initval);
    let (mut b, mut c) = (a, a);
    let rd = |s: &[u8], i: usize| u32::from_le_bytes(s[i..i + 4].try_into().unwrap());
    let mut k = data;
    while k.len() > 12 {
        a = a.wrapping_add(rd(k, 0));
        b = b.wrapping_add(rd(k, 4));
        c = c.wrapping_add(rd(k, 8));
        a = a.wrapping_sub(c);
        a ^= c.rotate_left(4);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= a.rotate_left(6);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= b.rotate_left(8);
        b = b.wrapping_add(a);
        a = a.wrapping_sub(c);
        a ^= c.rotate_left(16);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= a.rotate_left(19);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= b.rotate_left(4);
        b = b.wrapping_add(a);
        k = &k[12..];
    }
    if k.is_empty() {
        return c;
    }
    let mut t = [0u8; 12];
    t[..k.len()].copy_from_slice(k);
    a = a.wrapping_add(rd(&t, 0));
    b = b.wrapping_add(rd(&t, 4));
    c = c.wrapping_add(rd(&t, 8));
    c ^= b;
    c = c.wrapping_sub(b.rotate_left(14));
    a ^= c;
    a = a.wrapping_sub(c.rotate_left(11));
    b ^= a;
    b = b.wrapping_sub(a.rotate_left(25));
    c ^= b;
    c = c.wrapping_sub(b.rotate_left(16));
    a ^= c;
    a = a.wrapping_sub(c.rotate_left(4));
    b ^= a;
    b = b.wrapping_sub(a.rotate_left(14));
    c ^= b;
    c = c.wrapping_sub(b.rotate_left(24));
    c
}

// ── Intents ──────────────────────────────────────────────────────────

enum Seg {
    Key(String),
    Idx(usize),
}

fn parse_field_path(s: &str) -> Result<Vec<Seg>, String> {
    let mut out = Vec::new();
    for part in s.split('.') {
        let (name, mut rest) = match part.find('[') {
            Some(i) => (&part[..i], &part[i..]),
            None => (part, ""),
        };
        if !name.is_empty() {
            out.push(Seg::Key(name.to_string()));
        } else if rest.is_empty() {
            return Err(format!("empty segment in '{}'", s));
        }
        while !rest.is_empty() {
            let close = rest
                .find(']')
                .ok_or_else(|| format!("missing ']' in '{}'", s))?;
            let idx = rest[1..close]
                .parse::<usize>()
                .map_err(|_| format!("bad index in '{}'", s))?;
            out.push(Seg::Idx(idx));
            rest = &rest[close + 1..];
            if !rest.is_empty() && !rest.starts_with('[') {
                return Err(format!("bad path '{}'", s));
            }
        }
    }
    Ok(out)
}

fn walk_mut<'a>(mut v: &'a mut Value, segs: &[Seg]) -> Result<&'a mut Value, String> {
    for s in segs {
        v = match s {
            Seg::Key(k) => v
                .get_mut(k.as_str())
                .ok_or_else(|| format!("unknown field '{}'", k))?,
            Seg::Idx(i) => {
                let a = v
                    .as_array_mut()
                    .ok_or_else(|| format!("[{}] on a non-list", i))?;
                let n = a.len();
                a.get_mut(*i)
                    .ok_or_else(|| format!("index {} out of range (len {})", i, n))?
            }
        };
    }
    Ok(v)
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Schedule,
    Path,
}

fn select_scope<'a>(doc: &'a mut Value, it: &Intent, kind: Kind) -> Result<&'a mut Value, String> {
    if it.entry.is_none() && it.key.is_none() {
        return Ok(doc);
    }
    let list = if kind == Kind::Schedule {
        "items"
    } else {
        "records"
    };
    let arr = doc
        .get_mut(list)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("no '{}' list", list))?;
    let hits: Vec<usize> = arr
        .iter()
        .enumerate()
        .filter(|(_, x)| {
            let by_entry = match &it.entry {
                Some(e) => {
                    kind == Kind::Schedule
                        && x.get("name").and_then(Value::as_str) == Some(e.as_str())
                }
                None => true,
            };
            let by_key = match it.key {
                Some(k) => {
                    let got = if kind == Kind::Schedule {
                        Some(item_k(x, "k1"))
                    } else {
                        x.get("step").and_then(Value::as_u64)
                    };
                    got.map(|g| g as i64) == Some(k)
                }
                None => true,
            };
            by_entry && by_key
        })
        .map(|(i, _)| i)
        .collect();
    match hits.as_slice() {
        [i] => Ok(&mut arr[*i]),
        [] if kind == Kind::Path && it.entry.is_some() => {
            Err("entry (item name) is not stored in .paschedulepath: use key = item k1".into())
        }
        [] => Err("no item matches entry/key".into()),
        _ => Err(format!(
            "{} items match entry/key: add key = item k1",
            hits.len()
        )),
    }
}

fn apply_one(doc: &mut Value, it: &Intent, kind: Kind) -> Result<(), String> {
    let op = it.op.as_deref().unwrap_or("set");
    let field = it.field.as_deref().ok_or("missing 'field'")?;
    let segs = parse_field_path(field)?;
    let scope = select_scope(doc, it, kind)?;
    let payload = || {
        it.new
            .clone()
            .or_else(|| it.value.clone())
            .ok_or("missing 'new'/'value'")
    };
    match op {
        "set" => {
            let target = walk_mut(scope, &segs)?;
            *target = payload()?;
        }
        "array_append" => {
            let target = walk_mut(scope, &segs)?;
            let v = payload()?;
            target
                .as_array_mut()
                .ok_or_else(|| format!("'{}' is not a list", field))?
                .push(v);
        }
        "array_insert" | "array_remove" => {
            let (last, parent) = segs.split_last().ok_or("empty field")?;
            let Seg::Idx(i) = last else {
                return Err(format!("'{}' must end in [index]", field));
            };
            let a = walk_mut(scope, parent)?
                .as_array_mut()
                .ok_or_else(|| format!("'{}' is not in a list", field))?;
            if op == "array_insert" {
                if *i > a.len() {
                    return Err(format!("insert index {} past end (len {})", i, a.len()));
                }
                a.insert(*i, payload()?);
            } else {
                if *i >= a.len() {
                    return Err(format!("remove index {} out of range (len {})", i, a.len()));
                }
                a.remove(*i);
            }
        }
        other => return Err(format!("op '{}' is not supported here", other)),
    }
    Ok(())
}

fn apply_all(doc: &mut Value, intents: &[Intent], kind: Kind) -> Vec<ApplyOutcome> {
    intents
        .iter()
        .map(|it| ApplyOutcome {
            op: it.op.clone().unwrap_or_else(|| "set".into()),
            status: match apply_one(doc, it, kind) {
                Ok(()) => ApplyStatus::Applied,
                Err(e) => ApplyStatus::Skipped(e),
            },
        })
        .collect()
}

/// Apply intents to a `.paschedule` file and re-encode it. The key index
/// is rebuilt in canonical order. Errors only when the edited JSON does
/// not encode (wrong type, out of range); the message names the field.
pub fn apply_paschedule_intents(
    data: &[u8],
    intents: &[Intent],
) -> io::Result<(Vec<u8>, Vec<ApplyOutcome>)> {
    let mut doc = decode_paschedule(data)?;
    let outcomes = apply_all(&mut doc, intents, Kind::Schedule);
    if let Some(m) = doc.as_object_mut() {
        m.remove("index_order");
    }
    Ok((encode_paschedule(&doc)?, outcomes))
}

/// Apply intents to a `.paschedulepath` file and re-encode it.
pub fn apply_paschedulepath_intents(
    data: &[u8],
    intents: &[Intent],
) -> io::Result<(Vec<u8>, Vec<ApplyOutcome>)> {
    let mut doc = decode_paschedulepath(data)?;
    let outcomes = apply_all(&mut doc, intents, Kind::Path);
    Ok((encode_paschedulepath(&doc)?, outcomes))
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const STEM: &str = "cd_seq_basecamp_greyfur_family_lv2_schedule";

    fn node(h: u64, children: Vec<Value>) -> Value {
        let mut n = paschedule_template("NODE").unwrap();
        n["start_h"] = json!(h);
        n["end_h"] = json!(h);
        n["children"] = Value::Array(children);
        n
    }

    fn synthetic() -> (Value, Value) {
        let mut item = paschedule_template("ITEMREC").unwrap();
        item["key"] = json!({"hash": 7, "k1": 8, "k2": 1});
        item["name"] = json!("Item_0001");
        item["fields"]["nodes"] = json!([
            node(3, vec![node(3, vec![]), node(3, vec![])]),
            node(15, vec![])
        ]);
        let mut item2 = item.clone();
        item2["key"] = json!({"hash": 7, "k1": 4, "k2": 2});
        item2["name"] = json!("Item_0002");
        let mut track = paschedule_template("TRACK").unwrap();
        track["q"] = json!(1);
        track["qv"] = json!(99);
        track["splines"] = json!([["00".repeat(30)]]);
        let s = json!({
            "version": 1, "hash": hashlittle(STEM.as_bytes(), 0xC5EDE), "type": 5,
            "key": {"hash": 7, "k1": 0, "k2": 0}, "name": STEM, "a0": 0, "a1": 0,
            "tracks": [track], "items": [item, item2],
            "post": paschedule_template("POST").unwrap(), "stamp": 1234567890123u64,
            "lists_a": [{"flag": 1, "points": ["11".repeat(66)]}], "lists_b": [],
        });
        let body = |np: usize| {
            let paths: Vec<Value> = (0..np)
                .map(|i| {
                    let mut p = json!({"pts": [[1.0, 2.0, 3.0], [4.5, 5.0, -6.25]], "refs": [{"kind": 5, "index": 9}], "refs2": []});
                    if i + 1 < np {
                        p["between"] = json!([[[0.0, 0.0, 0.0]]]);
                    }
                    p
                })
                .collect();
            json!({"nodes": 4, "loops": [[[0.0, 1.0, 2.0]]], "paths": paths})
        };
        let p = json!({"id": 42u64, "records": [
            {"hash": 7, "step": 8, "has": 1, "body": body(2)},
            {"hash": 7, "step": 4, "has": 1, "body": body(2)},
        ]});
        (s, p)
    }

    #[test]
    fn hashlittle_matches_reference() {
        // values from the Python reference (sp1_paschedule.py)
        assert_eq!(hashlittle(b"", 0xC5EDE), 0xDEBA1DCD);
        assert_eq!(hashlittle(STEM.as_bytes(), 0xC5EDE), 0xE64CC506);
    }

    #[test]
    fn synthetic_round_trip_and_canonical_index() {
        let (s, p) = synthetic();
        let sb = encode_paschedule(&s).unwrap();
        let d = decode_paschedule(&sb).unwrap();
        assert_eq!(d["index_order"], json!([1, 0]), "sorted by k1");
        assert_eq!(encode_paschedule(&d).unwrap(), sb);
        assert_eq!(d["tracks"][0]["qv"], json!(99));
        let pb = encode_paschedulepath(&p).unwrap();
        let pd = decode_paschedulepath(&pb).unwrap();
        assert_eq!(pd, p);
        assert_eq!(encode_paschedulepath(&pd).unwrap(), pb);
        assert!(check_schedule_path_pair(&d, &pd).is_empty());
    }

    #[test]
    fn nan_survives_as_bits() {
        let (mut s, _) = synthetic();
        s["items"][0]["fields"]["nodes"][0]["yaw"] = json!({"f32bits": "0000c07f"});
        let b = encode_paschedule(&s).unwrap();
        let d = decode_paschedule(&b).unwrap();
        assert_eq!(
            d["items"][0]["fields"]["nodes"][0]["yaw"],
            json!({"f32bits": "0000c07f"})
        );
    }

    #[test]
    fn rejects_bad_input() {
        let (s, _) = synthetic();
        let b = encode_paschedule(&s).unwrap();
        assert!(decode_paschedule(&b[..b.len() - 1]).is_err());
        let mut extra = b.clone();
        extra.push(0);
        assert!(decode_paschedule(&extra).is_err());
        let mut s2 = s.clone();
        s2["items"][0]["fields"]["nodes"][0]["start_h"] = json!(300);
        let e = encode_paschedule(&s2).unwrap_err().to_string();
        assert!(e.contains("start_h"), "{}", e);
    }

    #[test]
    fn intents_size_changing_stops() {
        let (s, p) = synthetic();
        let sb = encode_paschedule(&s).unwrap();
        let pb = encode_paschedulepath(&p).unwrap();
        let mut new_stop = node(19, vec![]);
        new_stop["pos"] = json!([1.0, 2.0, 3.0]);
        let ints = vec![
            Intent {
                op: Some("set".into()),
                key: Some(8),
                field: Some("fields.nodes[0].start_h".into()),
                new: Some(json!(8)),
                ..Default::default()
            },
            Intent {
                op: Some("array_insert".into()),
                key: Some(8),
                field: Some("fields.nodes[1]".into()),
                value: Some(new_stop),
                ..Default::default()
            },
            Intent {
                op: Some("set".into()),
                entry: Some("Item_0002".into()),
                field: Some("fields.character".into()),
                new: Some(json!(1002975)),
                ..Default::default()
            },
            Intent {
                op: Some("set".into()),
                key: Some(8),
                field: Some("fields.no_such_field".into()),
                new: Some(json!(1)),
                ..Default::default()
            },
            Intent {
                op: Some("array_remove".into()),
                key: Some(8),
                field: Some("fields.nodes[9]".into()),
                ..Default::default()
            },
        ];
        let (nb, out) = apply_paschedule_intents(&sb, &ints).unwrap();
        let st: Vec<bool> = out
            .iter()
            .map(|o| matches!(o.status, ApplyStatus::Applied))
            .collect();
        assert_eq!(st, vec![true, true, true, false, false]);
        assert!(nb.len() > sb.len());
        let d = decode_paschedule(&nb).unwrap();
        let nodes = d["items"][0]["fields"]["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0]["start_h"], json!(8));
        assert_eq!(nodes[1]["start_h"], json!(19));
        assert_eq!(d["items"][1]["fields"]["character"], json!(1002975));
        // the path record must follow: one more path, one more node
        let pd = decode_paschedulepath(&pb).unwrap();
        assert_eq!(check_schedule_path_pair(&d, &pd).len(), 2);
        let mut extra = pd["records"][0]["body"]["paths"][0].clone();
        extra["between"] = json!([]);
        let pints = vec![
            Intent {
                op: Some("array_insert".into()),
                key: Some(8),
                field: Some("body.paths[1]".into()),
                value: Some(extra),
                ..Default::default()
            },
            Intent {
                op: Some("set".into()),
                key: Some(8),
                field: Some("body.nodes".into()),
                new: Some(json!(5)),
                ..Default::default()
            },
            Intent {
                op: Some("set".into()),
                entry: Some("Item_0001".into()),
                field: Some("body.nodes".into()),
                new: Some(json!(5)),
                ..Default::default()
            },
        ];
        let (npb, pout) = apply_paschedulepath_intents(&pb, &pints).unwrap();
        assert!(matches!(pout[2].status, ApplyStatus::Skipped(_)));
        let pd2 = decode_paschedulepath(&npb).unwrap();
        assert!(check_schedule_path_pair(&d, &pd2).is_empty());
    }

    #[test]
    fn last_path_rejects_between() {
        let (_, mut p) = synthetic();
        p["records"][0]["body"]["paths"][1]["between"] = json!([[[0.0, 0.0, 0.0]]]);
        assert!(encode_paschedulepath(&p).is_err());
    }

    /// Byte-exact on every vanilla file under `DMM_PARSER_SEQUENCER_DIR`
    /// (extracted `.paschedule` / `.paschedulepath`, any layout; pairs are
    /// matched by `binary__` -> `pathbinary__`). Skips when unset.
    #[test]
    fn vanilla_round_trip_from_env() {
        let Ok(root) = std::env::var("DMM_PARSER_SEQUENCER_DIR") else {
            eprintln!("DMM_PARSER_SEQUENCER_DIR not set: skipped");
            return;
        };
        let mut files = Vec::new();
        let mut stack = vec![std::path::PathBuf::from(root)];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                } else if matches!(
                    p.extension().and_then(|x| x.to_str()),
                    Some("paschedule" | "paschedulepath")
                ) {
                    files.push(p);
                }
            }
        }
        files.sort();
        let (mut ok, mut pairs, mut fails) = (0, 0, Vec::new());
        let mut sched = std::collections::HashMap::new();
        for f in &files {
            let b = std::fs::read(f).unwrap();
            let is_s = f.extension().unwrap() == "paschedule";
            let dec = if is_s {
                decode_paschedule(&b)
            } else {
                decode_paschedulepath(&b)
            };
            let res = dec.and_then(|d| {
                let txt = serde_json::to_string(&d).unwrap();
                let back: Value = serde_json::from_str(&txt).unwrap();
                let enc = if is_s {
                    encode_paschedule(&back)?
                } else {
                    encode_paschedulepath(&back)?
                };
                Ok((d, enc))
            });
            match res {
                Ok((d, enc)) if enc == b => {
                    ok += 1;
                    if is_s {
                        let stem = f.file_stem().unwrap().to_str().unwrap();
                        assert_eq!(
                            d["hash"].as_u64(),
                            Some(hashlittle(stem.as_bytes(), 0xC5EDE) as u64),
                            "{:?}",
                            f
                        );
                        sched.insert(
                            f.to_string_lossy().replace("binary__", "pathbinary__") + "path",
                            d,
                        );
                    } else if let Some(s) = sched.get(&*f.to_string_lossy()) {
                        pairs += 1;
                        let probs = check_schedule_path_pair(s, &d);
                        assert!(probs.is_empty(), "{:?}: {:?}", f, probs);
                    }
                }
                Ok(_) => fails.push(format!("{:?}: re-encode differs", f)),
                Err(e) => fails.push(format!("{:?}: {}", f, e)),
            }
        }
        eprintln!(
            "sequencer files byte-exact {}/{}, pairs checked {}",
            ok,
            files.len(),
            pairs
        );
        assert!(fails.is_empty(), "{:?}", &fails[..fails.len().min(10)]);
    }
}
