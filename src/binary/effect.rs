// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.pae` / `.paem` / `.parg` effect files — field-level codec plus field
//! intents (game 2.04.03 / 2.04.08).
//!
//! These are PA reflection documents in a one-header PARC container, so the
//! document itself is read and written by the `.palevel` codec
//! ([`super::palevel`]): every count, offset, size and record self-address is
//! recomputed on encode, so a byte match proves the model. Checked on every
//! vanilla effect file of 2.04.08 (build 26480954): 6,588 `.pae`, 560
//! `.paem`, 783 `.parg` (base + dlc_01), and on the 216 unique `.pae` /
//! `.paem` shipped by effect mods on Nexus (some carry zero padding after the
//! document, kept as `tail`). 16 vanilla `.pae` keep their top `EffectData`
//! object as raw bytes (a nested `Material` / curve record does not decode
//! yet): byte-exact, but `set` is refused in those files.
//!
//! ```text
//! file  = PARC | DOC | tail          (4 vanilla .pae are the 16-byte PARC
//!                                      header alone: doc = null)
//! PARC  = 'PARC' u8 kind (pae 6, paem 7, parg 0) | u8 version | u64 hash | u16 0
//! DOC   = reflection doc (see palevel) WITHOUT a resource table; strings are
//!         inline (u32 len + bytes)
//! ```
//!
//! Top classes: `.pae` = `EffectData` (lifetimes, `_emitterVariationDataArray`
//! of emitters with `_internalEmitterData._renderData` = colour, emissive
//! colour / brightness, light colour scale, preset flags), `.paem` =
//! `EmitterData` (`_renderData`, `_simulationData` scale / spawn / speed),
//! `.parg` = `EmitterRenderGroupData`.
//!
//! Field paths (same spelling as the leaf list of [`list_effect_fields`]):
//! `Class#n.field.field[i].field` — `Class` = top object class (or the part
//! after the last `::`), `#n` = n-th object of that class in the file (`#*`
//! or no `#` = every object), `[i]` = list item / array element, `[*]` =
//! every item. On a `floatN` field `[i]` selects one component.
//!
//! Intents (`apply_effect_intents`):
//! - `set`: `entry` = `Class#n` and `field` = path inside it, or `field` =
//!   the whole path. `value` = number / list of numbers / `"0.1, 0.9, 0.2"`
//!   for float, floatN, double and integer fields (integers range-checked,
//!   NaN / inf refused), `true`/`false` for bool, a string for string fields,
//!   `"hex:.."` for fields without a typed codec, or `"x1.5"` = multiply the
//!   current number(s). The field must already be present (mask bit set):
//!   adding a value or a list item is not supported. String edits may change
//!   the file size; everything is re-laid out by the codec.
//!
//! In game (C4 probe, 2.04.08, file overlay): a `set` of the emissive colour,
//! `_overridePresetColor`, `_particleLightColorScale` and
//! `_precomputeEmissiveLightLinearColor` of `fx_fire_common_b__fire0.5m_ies`
//! turned every brazier / firepot flame using that effect green, glow and
//! cast light.

use super::palevel::{self, bad, Body, ClassDef, Doc, FieldDef, Parc, Str, Val};
use crate::intents::{ApplyOutcome, ApplyStatus, Intent};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io;

// ---------------------------------------------------------------- container

/// A whole `.pae` / `.paem` / `.parg`.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectFile {
    pub parc: Parc,
    /// `None` = header-only file.
    pub doc: Option<Doc>,
    /// Bytes after the document (zero padding in some mod files).
    pub tail: Vec<u8>,
}

impl EffectFile {
    /// Objects whose values did not decode (kept as bytes).
    pub fn raw_objects(&self) -> usize {
        self.doc.as_ref().map_or(0, |d| {
            d.objects
                .iter()
                .filter(|o| matches!(o.body, Body::Raw { .. }))
                .count()
        })
    }
}

pub fn parse_effect(b: &[u8]) -> io::Result<EffectFile> {
    let (parc, o) = palevel::parse_parc(b, 0)?;
    if b.len() == o {
        return Ok(EffectFile {
            parc,
            doc: None,
            tail: Vec::new(),
        });
    }
    let mut first = None;
    let mut err = None;
    // effect docs carry no resource table; try the other layout as fallback
    for restab in [false, true] {
        match palevel::parse_doc(b, o, restab) {
            Ok((doc, e)) => {
                let mut out = Vec::new();
                if palevel::build_doc(&mut out, &doc).is_ok() && out[..] == b[o..e] {
                    return Ok(EffectFile {
                        parc,
                        doc: Some(doc),
                        tail: b[e..].to_vec(),
                    });
                }
                first.get_or_insert((doc, e));
            }
            Err(x) => {
                err.get_or_insert(x);
            }
        }
    }
    match first {
        Some((doc, e)) => Ok(EffectFile {
            parc,
            doc: Some(doc),
            tail: b[e..].to_vec(),
        }),
        None => Err(err.unwrap_or_else(|| bad("no reflection doc after PARC"))),
    }
}

pub fn build_effect(m: &EffectFile) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    palevel::build_parc(&mut out, &m.parc);
    if let Some(d) = &m.doc {
        palevel::build_doc(&mut out, d)?;
    }
    out.extend_from_slice(&m.tail);
    Ok(out)
}

/// Whole file -> JSON (container, PARC header, doc with classes, strings
/// and every object's field values; fixed-size values as hex). Byte-exact
/// with [`encode_effect`]. Use [`list_effect_fields`] for typed values.
pub fn decode_effect(b: &[u8]) -> io::Result<Value> {
    let m = parse_effect(b)?;
    Ok(json!({
        "container": "parc",
        "parc": palevel::parc_json(&m.parc),
        "doc": m.doc.as_ref().map(palevel::doc_json),
        "tail": palevel::hx(&m.tail),
    }))
}

pub fn encode_effect(v: &Value) -> io::Result<Vec<u8>> {
    let parc = palevel::parc_from(v.get("parc").ok_or_else(|| bad("missing 'parc'"))?)?;
    let doc = match v.get("doc") {
        None | Some(Value::Null) => None,
        Some(d) => Some(palevel::doc_from(d)?),
    };
    let tail = match v.get("tail").and_then(|x| x.as_str()) {
        Some(s) => palevel::unhx(s)?,
        None => Vec::new(),
    };
    build_effect(&EffectFile { parc, doc, tail })
}

/// Byte round-trip check of one file: (exact, raw object count).
pub fn effect_roundtrip(b: &[u8]) -> io::Result<(bool, usize)> {
    let m = parse_effect(b)?;
    Ok((build_effect(&m)? == b, m.raw_objects()))
}

// ---------------------------------------------------------------- typed values

#[derive(Clone, Copy, Debug, PartialEq)]
enum Vt {
    Str,
    F32(usize),
    F64(usize),
    Bool,
    Int { signed: bool, bytes: usize },
    Raw,
}

fn is_int_type(t: &str) -> bool {
    if matches!(t, "byte" | "short" | "ushort" | "uint" | "ulong" | "long") {
        return true;
    }
    let t = t.strip_prefix('u').unwrap_or(t);
    matches!(t, "int" | "int8" | "int16" | "int32" | "int64")
}

/// Value type of a field (element size `sz` for arrays), from the doc's
/// own class table.
fn vtype(f: &FieldDef, sz: usize) -> Vt {
    let t = String::from_utf8_lossy(&f.type_name).to_ascii_lowercase();
    if matches!(f.kind, 1 | 10) {
        Vt::Str
    } else if t.contains("float") && sz % 4 == 0 {
        Vt::F32(sz / 4)
    } else if t.contains("double") && sz % 8 == 0 {
        Vt::F64(sz / 8)
    } else if t.contains("bool") && sz == 1 {
        Vt::Bool
    } else if is_int_type(&t) && matches!(sz, 1 | 2 | 4 | 8) {
        Vt::Int {
            signed: !(t.starts_with('u') || t.starts_with("byte")),
            bytes: sz,
        }
    } else {
        Vt::Raw
    }
}

fn vt_name(f: &FieldDef, vt: Vt) -> String {
    match vt {
        Vt::Str => "string".into(),
        Vt::F32(1) => "float".into(),
        Vt::F32(n) => format!("float{n}"),
        Vt::F64(1) => "double".into(),
        Vt::F64(n) => format!("double{n}"),
        Vt::Bool => "bool".into(),
        Vt::Int { signed, bytes } => format!("{}int{}", if signed { "" } else { "u" }, bytes * 8),
        Vt::Raw => format!("raw:{}", String::from_utf8_lossy(&f.type_name)),
    }
}

/// Non-finite numbers as "nan" / "inf" / "-inf" (JSON has no NaN).
fn nonfinite(x: f64) -> Value {
    Value::String(if x.is_nan() { "nan".into() } else if x > 0.0 { "inf".into() } else { "-inf".into() })
}

/// f32 as the shortest decimal that reads back to the same f32.
fn f32_json(x: f32) -> Value {
    if !x.is_finite() {
        return nonfinite(x as f64);
    }
    format!("{x}")
        .parse::<f64>()
        .ok()
        .and_then(|y| serde_json::Number::from_f64(y).map(Value::Number))
        .unwrap_or(Value::Null)
}

fn f64_json(x: f64) -> Value {
    serde_json::Number::from_f64(x)
        .map(Value::Number)
        .unwrap_or_else(|| nonfinite(x))
}

fn many(v: Vec<Value>) -> Value {
    if v.len() == 1 {
        v.into_iter().next().unwrap()
    } else {
        Value::Array(v)
    }
}

fn dec_bytes(f: &FieldDef, b: &[u8], sz: usize) -> Value {
    match vtype(f, sz) {
        Vt::F32(_) => many(
            b.chunks_exact(4)
                .map(|c| f32_json(f32::from_le_bytes(c.try_into().unwrap())))
                .collect(),
        ),
        Vt::F64(_) => many(
            b.chunks_exact(8)
                .map(|c| f64_json(f64::from_le_bytes(c.try_into().unwrap())))
                .collect(),
        ),
        Vt::Bool => Value::Bool(b.first().copied().unwrap_or(0) != 0),
        Vt::Int { signed, bytes } if b.len() == bytes => {
            let mut w = [0u8; 8];
            w[..bytes].copy_from_slice(b);
            if signed {
                let sh = 64 - 8 * bytes as u32;
                json!((i64::from_le_bytes(w) << sh) >> sh)
            } else {
                json!(u64::from_le_bytes(w))
            }
        }
        _ => Value::String(format!("hex:{}", palevel::hx(b))),
    }
}

fn dec_str(s: &Str, strings: &[Vec<u8>]) -> Value {
    let t = match s {
        Str::Index(i) => match strings.get(*i as usize) {
            Some(t) => t,
            None => return json!(i),
        },
        Str::Inline(t) | Str::Text(t) => t,
    };
    Value::String(String::from_utf8_lossy(t).into_owned())
}

// ---------------------------------------------------------------- leaf list

fn leaf(out: &mut Vec<Value>, path: String, ty: String, value: Value) {
    out.push(json!({"path": path, "type": ty, "value": value}));
}

fn walk(cl: &[ClassDef], strings: &[Vec<u8>], c: &ClassDef, vals: &[(usize, Val)], p: &str, out: &mut Vec<Value>) {
    for (i, v) in vals {
        let f = &c.fields[*i];
        let q = format!("{p}.{}", String::from_utf8_lossy(&f.name));
        let sz = f.size as usize;
        match v {
            Val::Null => leaf(out, q, vt_name(f, vtype(f, sz)), Value::Null),
            Val::Rec(r) => walk(cl, strings, &cl[r.cls as usize], &r.vals, &q, out),
            Val::List { items, .. } => {
                for (j, r) in items.iter().enumerate() {
                    walk(cl, strings, &cl[r.cls as usize], &r.vals, &format!("{q}[{j}]"), out)
                }
            }
            Val::Bytes(b) if f.kind == 3 => {
                if sz > 0 {
                    for (j, e) in b.chunks_exact(sz).enumerate() {
                        leaf(out, format!("{q}[{j}]"), vt_name(f, vtype(f, sz)), dec_bytes(f, e, sz))
                    }
                }
            }
            Val::Bytes(b) => leaf(out, q, vt_name(f, vtype(f, sz)), dec_bytes(f, b, sz)),
            Val::Str(s) => leaf(out, q, "string".into(), dec_str(s, strings)),
            Val::Strs(ss) => {
                for (j, s) in ss.iter().enumerate() {
                    leaf(out, format!("{q}[{j}]"), "string".into(), dec_str(s, strings))
                }
            }
        }
    }
}

/// Every leaf value of the file as `{"path", "type", "value"}` rows, typed
/// (floats, floatN lists, ints, bools, strings; `"nan"`/`"inf"`/`"-inf"` for
/// non-finite floats; `"hex:.."` otherwise; null = field present but null).
/// `path` is what a `set` intent takes. Raw (undecoded) objects have no rows.
pub fn list_effect_fields(b: &[u8]) -> io::Result<Vec<Value>> {
    let m = parse_effect(b)?;
    let Some(d) = &m.doc else {
        return Ok(Vec::new());
    };
    let mut seen: HashMap<&[u8], usize> = HashMap::new();
    let mut out = Vec::new();
    for o in &d.objects {
        let c = &d.classes[o.cls as usize];
        let n = seen.entry(&c.name[..]).or_insert(0);
        let key = format!("{}#{}", String::from_utf8_lossy(&c.name), n);
        *n += 1;
        if let Body::Top { vals, .. } = &o.body {
            walk(&d.classes, &d.strings, c, vals, &key, &mut out)
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------- set

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ix {
    One(usize),
    All,
}

#[derive(Debug)]
struct Seg {
    name: String,
    ix: Option<Ix>,
}

/// `Class#n.a.b[2].c` -> (class, n (None = every object), steps).
fn parse_path(p: &str) -> Result<(String, Option<usize>, Vec<Seg>), String> {
    let (cls, n, rest) = match p.find('#') {
        Some(h) => {
            let after = &p[h + 1..];
            let (num, rest) = after.split_once('.').unwrap_or((after, ""));
            let n = match num {
                "*" => None,
                s => Some(s.parse::<usize>().map_err(|_| format!("bad object index '#{s}' in {p}"))?),
            };
            (&p[..h], n, rest)
        }
        None => {
            let (c, rest) = p.split_once('.').unwrap_or((p, ""));
            (c, None, rest)
        }
    };
    if rest.is_empty() {
        return Err(format!("path needs a field: {p}"));
    }
    let mut steps = Vec::new();
    for seg in rest.split('.') {
        let (name, ix) = match seg.find('[') {
            Some(b) if seg.ends_with(']') => {
                let s = &seg[b + 1..seg.len() - 1];
                let ix = if s == "*" {
                    Ix::All
                } else {
                    Ix::One(s.parse().map_err(|_| format!("bad index in '{seg}'"))?)
                };
                (&seg[..b], Some(ix))
            }
            _ => (seg, None),
        };
        if name.is_empty() {
            return Err(format!("empty field name in {p}"));
        }
        steps.push(Seg {
            name: name.to_string(),
            ix,
        });
    }
    Ok((cls.to_string(), n, steps))
}

fn num_list(v: &Value) -> Result<Vec<f64>, String> {
    match v {
        Value::Number(n) => Ok(vec![n.as_f64().unwrap_or(f64::NAN)]),
        Value::Array(a) => a
            .iter()
            .map(|x| x.as_f64().ok_or_else(|| format!("not a number: {x}")))
            .collect(),
        Value::String(s) => s
            .trim()
            .trim_matches(|c| matches!(c, '[' | ']' | '(' | ')'))
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|x| !x.is_empty())
            .map(|x| x.parse::<f64>().map_err(|_| format!("not a number: '{x}'")))
            .collect(),
        _ => Err(format!("number(s) expected, got {v}")),
    }
}

/// `"x1.5"` / `"*2"` -> factor.
fn factor(v: &Value) -> Option<f64> {
    let s = v.as_str()?.trim();
    let r = s.strip_prefix('x').or_else(|| s.strip_prefix('*'))?;
    if r.is_empty() || !r.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | 'e')) {
        return None;
    }
    r.parse().ok()
}

/// New bytes (same size) for a non-string value.
fn enc(f: &FieldDef, val: &Value, old: &[u8], sz: usize) -> Result<Vec<u8>, String> {
    let fname = String::from_utf8_lossy(&f.name).into_owned();
    let vt = vtype(f, sz);
    let mut val = val.clone();
    if let (Some(k), false) = (factor(&val), vt == Vt::Str) {
        val = match dec_bytes(f, old, sz) {
            Value::Number(n) => {
                let x = n.as_f64().unwrap_or(0.0) * k;
                if matches!(vt, Vt::Int { .. }) {
                    json!(x.round_ties_even())
                } else {
                    json!(x)
                }
            }
            Value::Array(a) => Value::Array(a.iter().map(|x| json!(x.as_f64().unwrap_or(0.0) * k)).collect()),
            other => other,
        };
    }
    match vt {
        Vt::Str => Err(format!("{fname}: string field")),
        Vt::F32(n) | Vt::F64(n) => {
            let xs = num_list(&val).map_err(|e| format!("{fname}: {e}"))?;
            if xs.len() != n {
                return Err(format!("{fname}: {n} number(s) expected, got {}", xs.len()));
            }
            if xs.iter().any(|x| !x.is_finite()) {
                return Err(format!("{fname}: NaN/inf refused"));
            }
            Ok(if matches!(vt, Vt::F32(_)) {
                xs.iter().flat_map(|x| (*x as f32).to_le_bytes()).collect()
            } else {
                xs.iter().flat_map(|x| x.to_le_bytes()).collect()
            })
        }
        Vt::Bool => {
            let b = match &val {
                Value::Bool(b) => *b,
                Value::Number(n) => n.as_f64() == Some(1.0) && (n.is_i64() || n.is_u64()),
                Value::String(s) => matches!(s.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
                _ => false,
            };
            Ok(vec![b as u8])
        }
        Vt::Int { signed, bytes } => {
            let x: i128 = match &val {
                Value::Number(n) => match (n.as_i64(), n.as_u64(), n.as_f64()) {
                    (Some(i), _, _) => i as i128,
                    (_, Some(u), _) => u as i128,
                    (_, _, Some(f)) if f.is_finite() => f.trunc() as i128,
                    _ => return Err(format!("{fname}: integer expected")),
                },
                Value::String(s) => match s.trim().parse::<f64>() {
                    Ok(f) if f.is_finite() => f.trunc() as i128,
                    _ => return Err(format!("{fname}: integer expected")),
                },
                _ => return Err(format!("{fname}: integer expected")),
            };
            let bits = 8 * bytes as u32;
            let (lo, hi) = if signed {
                (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
            } else {
                (0, (1i128 << bits) - 1)
            };
            if x < lo || x > hi {
                return Err(format!(
                    "{fname}: {x} outside {}int{bits} range",
                    if signed { "" } else { "u" }
                ));
            }
            Ok(x.to_le_bytes()[..bytes].to_vec())
        }
        Vt::Raw => match val.as_str().and_then(|s| s.strip_prefix("hex:")) {
            Some(h) => {
                let b = palevel::unhx(h).map_err(|e| format!("{fname}: {e}"))?;
                if b.len() != sz {
                    return Err(format!("{fname}: {sz} bytes expected"));
                }
                Ok(b)
            }
            None => Err(format!(
                "{fname}: type {} has no typed codec - give \"hex:<{sz} bytes>\"",
                String::from_utf8_lossy(&f.type_name)
            )),
        },
    }
}

struct SetCx<'a> {
    cl: &'a [ClassDef],
    strings: &'a mut Vec<Vec<u8>>,
    /// leaves changed
    n: usize,
}

/// A string value in the same form as `old` (table strings reuse or
/// append a string-table entry).
fn new_str(old: &Str, t: &str, strings: &mut Vec<Vec<u8>>) -> Str {
    match old {
        Str::Inline(_) => Str::Inline(t.as_bytes().to_vec()),
        Str::Text(_) => Str::Text(t.as_bytes().to_vec()),
        Str::Index(_) => {
            let i = match strings.iter().position(|s| s == t.as_bytes()) {
                Some(i) => i,
                None => {
                    strings.push(t.as_bytes().to_vec());
                    strings.len() - 1
                }
            };
            Str::Index(i as u32)
        }
    }
}

fn pick(ix: Option<Ix>, n: usize, here: &str) -> Result<Vec<usize>, String> {
    match ix {
        None | Some(Ix::All) => Ok((0..n).collect()),
        Some(Ix::One(j)) if j < n => Ok(vec![j]),
        Some(Ix::One(j)) => Err(format!("{here}[{j}]: only {n} items")),
    }
}

fn set_in(cls: usize, vals: &mut [(usize, Val)], steps: &[Seg], val: &Value, path: &str, sc: &mut SetCx) -> Result<(), String> {
    let cl = sc.cl;
    let c = &cl[cls];
    let st = &steps[0];
    let rest = &steps[1..];
    let Some(pos) = vals
        .iter()
        .position(|(i, _)| c.fields[*i].name == st.name.as_bytes())
    else {
        return Err(format!(
            "{path}.{}: field not present in this object (unknown name, or not set = adding a value is not supported)",
            st.name
        ));
    };
    let f = &c.fields[vals[pos].0];
    let here = format!("{path}.{}", st.name);
    let sz = f.size as usize;
    match &mut vals[pos].1 {
        Val::Null => Err(format!("{here}: null/unset here - adding a value is not supported")),
        Val::Rec(r) => {
            if st.ix.is_some() {
                return Err(format!("{here}: record, no [index]"));
            }
            if rest.is_empty() {
                return Err(format!("{here}: path ends on a record; name a field"));
            }
            let rc = r.cls as usize;
            set_in(rc, &mut r.vals, rest, val, &here, sc)
        }
        Val::List { items, .. } => {
            let Some(ix) = st.ix else {
                return Err(format!("{here}: list - give [i] or [*]"));
            };
            if rest.is_empty() {
                return Err(format!("{here}: path ends on a list item; name a field"));
            }
            let before = sc.n;
            for j in pick(Some(ix), items.len(), &here)? {
                let rc = items[j].cls as usize;
                if let Err(e) = set_in(rc, &mut items[j].vals, rest, val, &format!("{here}[{j}]"), sc) {
                    if ix != Ix::All {
                        return Err(e);
                    }
                }
            }
            if sc.n == before {
                let names: Vec<&str> = rest.iter().map(|s| s.name.as_str()).collect();
                return Err(format!("{here}[*]: no item carries {}", names.join(".")));
            }
            Ok(())
        }
        _ if !rest.is_empty() => Err(format!("{here}: leaf field, cannot go deeper")),
        Val::Bytes(b) if f.kind == 3 => {
            if sz == 0 {
                return Err(format!("{here}: zero-size elements"));
            }
            let cnt = b.len() / sz;
            for j in pick(st.ix, cnt, &here)? {
                let nb = enc(f, val, &b[j * sz..(j + 1) * sz], sz)?;
                b[j * sz..(j + 1) * sz].copy_from_slice(&nb);
                sc.n += 1;
            }
            Ok(())
        }
        Val::Strs(ss) => {
            let t = val
                .as_str()
                .ok_or_else(|| format!("{here}: string expected"))?;
            for j in pick(st.ix, ss.len(), &here)? {
                ss[j] = new_str(&ss[j], t, sc.strings);
                sc.n += 1;
            }
            Ok(())
        }
        Val::Str(s) => {
            if st.ix.is_some() {
                return Err(format!("{here}: string, no [index]"));
            }
            let t = val
                .as_str()
                .ok_or_else(|| format!("{here}: string expected"))?;
            *s = new_str(s, t, sc.strings);
            sc.n += 1;
            Ok(())
        }
        Val::Bytes(b) => {
            if let Some(ix) = st.ix {
                // one component of a floatN field
                let Vt::F32(n) = vtype(f, sz) else {
                    return Err(format!("{here}: [index] only on floatN fields"));
                };
                if n < 2 {
                    return Err(format!("{here}: [index] only on floatN fields"));
                }
                let one = FieldDef {
                    name: f.name.clone(),
                    type_name: b"float".to_vec(),
                    kind: 0,
                    size: 4,
                    flags: f.flags,
                };
                for j in pick(Some(ix), n, &here)? {
                    let nb = enc(&one, val, &b[4 * j..4 * j + 4], 4)?;
                    b[4 * j..4 * j + 4].copy_from_slice(&nb);
                }
                sc.n += 1;
                return Ok(());
            }
            let nb = enc(f, val, b, sz)?;
            *b = nb;
            sc.n += 1;
            Ok(())
        }
    }
}

enum Step {
    Done,
    Skip(String),
}

fn class_matches(name: &[u8], want: &str) -> bool {
    let n = String::from_utf8_lossy(name);
    n == want || n.rsplit("::").next() == Some(want)
}

fn op_set(m: &mut EffectFile, it: &Intent) -> Step {
    let path = match (it.entry.as_deref(), it.field.as_deref()) {
        (Some(e), Some(f)) => format!("{e}.{f}"),
        (None, Some(f)) => f.to_string(),
        _ => return Step::Skip("set needs field (and entry = Class#n, or a full path in field)".into()),
    };
    let Some(value) = it.value.as_ref().or(it.new.as_ref()) else {
        return Step::Skip("set needs value".into());
    };
    let (cls, want, steps) = match parse_path(&path) {
        Ok(x) => x,
        Err(e) => return Step::Skip(e),
    };
    let Some(Doc {
        classes,
        strings,
        objects,
        ..
    }) = &mut m.doc
    else {
        return Step::Skip("header-only file: no document to edit".into());
    };
    let mut sc = SetCx {
        cl: classes,
        strings,
        n: 0,
    };
    let mut seen: HashMap<u32, usize> = HashMap::new();
    let mut matched = 0;
    for o in objects.iter_mut() {
        let cn = &sc.cl[o.cls as usize].name;
        let k = {
            let e = seen
                .entry(
                    sc.cl
                        .iter()
                        .position(|c| &c.name == cn)
                        .unwrap_or(o.cls as usize) as u32,
                )
                .or_insert(0);
            *e += 1;
            *e - 1
        };
        if !class_matches(cn, &cls) || want.is_some_and(|w| w != k) {
            continue;
        }
        matched += 1;
        let key = format!("{}#{k}", String::from_utf8_lossy(cn));
        match &mut o.body {
            Body::Raw { .. } => {
                if want.is_some() {
                    return Step::Skip(format!("{key}: object did not decode; not editable"));
                }
            }
            Body::Top { vals, tail, .. } => {
                if !tail.is_empty() {
                    if want.is_some() {
                        return Step::Skip(format!("{key}: undecoded bytes after the values; not editable"));
                    }
                    continue;
                }
                if let Err(e) = set_in(o.cls as usize, vals, &steps, value, &key, &mut sc) {
                    if want.is_some() {
                        return Step::Skip(e);
                    }
                }
            }
        }
    }
    if matched == 0 {
        return Step::Skip(format!(
            "{path}: no object {cls}#{} in this file",
            want.map(|w| w.to_string()).unwrap_or_else(|| "*".into())
        ));
    }
    if sc.n == 0 {
        return Step::Skip(format!("{path}: no object carries this field"));
    }
    Step::Done
}

/// Apply field intents to a `.pae` / `.paem` / `.parg` (see module docs).
/// Returns the new file bytes and one outcome per intent; failed intents
/// are `Skipped` with a reason and change nothing.
pub fn apply_effect_intents(data: &[u8], intents: &[Intent]) -> io::Result<(Vec<u8>, Vec<ApplyOutcome>)> {
    let mut m = parse_effect(data)?;
    let mut outcomes = Vec::with_capacity(intents.len());
    for it in intents {
        let op = it.op.clone().unwrap_or_else(|| "set".into());
        let before = m.clone();
        let step = match op.as_str() {
            "set" => op_set(&mut m, it),
            other => Step::Skip(format!("unknown effect op '{other}' (supported: set)")),
        };
        let status = match step {
            Step::Done => match build_effect(&m) {
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
    Ok((build_effect(&m)?, outcomes))
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::palevel::{Obj, Rec};

    fn f(name: &str, ty: &str, kind: u16, size: u16) -> FieldDef {
        FieldDef {
            name: name.as_bytes().to_vec(),
            type_name: ty.as_bytes().to_vec(),
            kind,
            size,
            flags: 0,
        }
    }
    fn fl(xs: &[f32]) -> Val {
        Val::Bytes(xs.iter().flat_map(|x| x.to_le_bytes()).collect())
    }
    fn emitter(id: u8, c: [f32; 3], scale: f32) -> Rec {
        Rec {
            mask: vec![0x03],
            cls: 1,
            cf: 0,
            id: vec![id, 0, 0, 0, 0, 0, 0, 0],
            a: 0,
            names: vec![],
            vals: vec![(0, fl(&c)), (1, fl(&[scale]))],
        }
    }

    fn sample() -> Vec<u8> {
        let classes = vec![
            ClassDef {
                name: b"EffectData".to_vec(),
                fields: vec![
                    f("_life", "float", 0, 4),
                    f("_color", "float3", 0, 12),
                    f("_effectDataName", "IndexedStringA", 1, 0),
                    f("_times", "float", 3, 4),
                    f("_emitters", "EmitterRenderData", 7, 0),
                    f("_override", "bool", 0, 1),
                    f("_count", "uint8", 0, 1),
                    f("_opt", "EmitterRenderData", 5, 0),
                    f("_blob", "Guid", 0, 4),
                ],
            },
            ClassDef {
                name: b"EmitterRenderData".to_vec(),
                fields: vec![f("_emissiveColor", "float3", 0, 12), f("_scale", "float", 0, 4)],
            },
        ];
        let mut hdr = vec![0u8; 9];
        hdr.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
        let vals = vec![
            (0, fl(&[0.3])),
            (1, fl(&[1.0, 0.5, 0.25])),
            (2, Val::Str(Str::Text(b"fx_test".to_vec()))),
            (3, fl(&[1.0, 2.0, 3.0])),
            (
                4,
                Val::List {
                    hdr,
                    ids: vec![],
                    items: vec![emitter(1, [1.0, 0.0, 0.0], 2.0), emitter(2, [0.0, 0.0, 1.0], 4.0)],
                },
            ),
            (5, Val::Bytes(vec![0])),
            (6, Val::Bytes(vec![200])),
            (7, Val::Null),
            (8, Val::Bytes(vec![1, 2, 3, 4])),
        ];
        let doc = Doc {
            format: 4,
            zero: 0,
            hash: Some(0x1122_3344_5566_7788),
            version: 15,
            classes,
            strings: vec![],
            resources: None,
            objects: vec![Obj {
                cls: 0,
                parent: -1,
                body: Body::Top {
                    mask: vec![0xff, 0x01],
                    pre: vec![0; 4],
                    vals,
                    tail: vec![],
                },
            }],
        };
        build_effect(&EffectFile {
            parc: Parc {
                kind: 6,
                v: 8,
                hash: 42,
                z: 0,
            },
            doc: Some(doc),
            tail: vec![],
        })
        .unwrap()
    }

    fn set(path: &str, v: Value) -> Intent {
        Intent::from_value(&json!({"op": "set", "field": path, "value": v})).unwrap()
    }
    fn leaves(b: &[u8]) -> HashMap<String, Value> {
        list_effect_fields(b)
            .unwrap()
            .into_iter()
            .map(|r| (r["path"].as_str().unwrap().to_string(), r["value"].clone()))
            .collect()
    }

    #[test]
    fn synthetic_round_trip_bytes_and_json() {
        let b = sample();
        assert_eq!(effect_roundtrip(&b).unwrap(), (true, 0));
        let j = decode_effect(&b).unwrap();
        assert_eq!(encode_effect(&j).unwrap(), b);
        let mut padded = b.clone();
        padded.extend_from_slice(&[0; 32]);
        assert_eq!(effect_roundtrip(&padded).unwrap(), (true, 0));
        let l = leaves(&b);
        assert_eq!(l["EffectData#0._life"], json!(0.3));
        assert_eq!(l["EffectData#0._color"], json!([1.0, 0.5, 0.25]));
        assert_eq!(l["EffectData#0._effectDataName"], json!("fx_test"));
        assert_eq!(l["EffectData#0._times[2]"], json!(3.0));
        assert_eq!(l["EffectData#0._emitters[1]._scale"], json!(4.0));
        assert_eq!(l["EffectData#0._override"], json!(false));
        assert_eq!(l["EffectData#0._count"], json!(200));
        assert_eq!(l["EffectData#0._opt"], Value::Null);
        assert_eq!(l["EffectData#0._blob"], json!("hex:01020304"));
    }

    #[test]
    fn set_values_by_path() {
        let b = sample();
        let its = vec![
            set("EffectData#0._life", json!("x2")),
            set("EffectData#0._color", json!([0.15, 0.95, 0.2])),
            set("EffectData#0._emitters[*]._emissiveColor[1]", json!(0.75)),
            set("EffectData._emitters[0]._scale", json!("x1.5")),
            set("EffectData#0._override", json!(true)),
            set("EffectData#0._times", json!(9.0)),
            set("EffectData#0._effectDataName", json!("fx_test_green")),
            set("EffectData#0._blob", json!("hex:0a0b0c0d")),
        ];
        let (nb, out) = apply_effect_intents(&b, &its).unwrap();
        assert!(out.iter().all(|o| matches!(o.status, ApplyStatus::Applied)), "{out:?}");
        assert_eq!(effect_roundtrip(&nb).unwrap(), (true, 0));
        assert_eq!(nb.len(), b.len() + 6);
        let l = leaves(&nb);
        assert_eq!(l["EffectData#0._life"], json!(0.6));
        assert_eq!(l["EffectData#0._color"], json!([0.15, 0.95, 0.2]));
        assert_eq!(l["EffectData#0._emitters[0]._emissiveColor"], json!([1.0, 0.75, 0.0]));
        assert_eq!(l["EffectData#0._emitters[1]._emissiveColor"], json!([0.0, 0.75, 1.0]));
        assert_eq!(l["EffectData#0._emitters[0]._scale"], json!(3.0));
        assert_eq!(l["EffectData#0._emitters[1]._scale"], json!(4.0));
        assert_eq!(l["EffectData#0._override"], json!(true));
        assert_eq!(l["EffectData#0._times[1]"], json!(9.0));
        assert_eq!(l["EffectData#0._effectDataName"], json!("fx_test_green"));
        assert_eq!(l["EffectData#0._blob"], json!("hex:0a0b0c0d"));
        // entry + field spelling = same result
        let e = Intent::from_value(&json!({"op": "set", "entry": "EffectData#0", "field": "_life", "value": 0.6})).unwrap();
        let (nb2, _) = apply_effect_intents(&b, &[e]).unwrap();
        assert_eq!(leaves(&nb2)["EffectData#0._life"], json!(0.6));
    }

    #[test]
    fn refused_sets_change_nothing() {
        let b = sample();
        let bad_ones = vec![
            set("EffectData#0._count", json!(300)),
            set("EffectData#0._color", json!([1.0, 2.0])),
            set("EffectData#0._life", json!("nan")),
            set("EffectData#0._opt._scale", json!(1.0)),
            set("EffectData#0._missing", json!(1.0)),
            set("EffectData#1._life", json!(1.0)),
            set("EffectData#0._emitters", json!(1.0)),
            set("EffectData#0._emitters[5]._scale", json!(1.0)),
            set("EffectData#0._blob", json!(1.0)),
            set("EffectData#0._life[0]", json!(1.0)),
            Intent::from_value(&json!({"op": "delete_record", "field": "EffectData#0._life"})).unwrap(),
        ];
        let (nb, out) = apply_effect_intents(&b, &bad_ones).unwrap();
        assert_eq!(nb, b);
        for o in &out {
            assert!(matches!(o.status, ApplyStatus::Skipped(_)), "{o:?}");
        }
    }

    #[test]
    fn bad_input_is_an_error() {
        assert!(parse_effect(b"PARC").is_err());
        assert!(parse_effect(b"nope").is_err());
        let mut b = sample();
        b.truncate(b.len() - 3);
        assert!(effect_roundtrip(&b).is_err());
    }

    #[test]
    fn header_only_file() {
        let b = sample()[..16].to_vec();
        assert_eq!(effect_roundtrip(&b).unwrap(), (true, 0));
        let j = decode_effect(&b).unwrap();
        assert!(j["doc"].is_null());
        assert_eq!(encode_effect(&j).unwrap(), b);
        assert!(list_effect_fields(&b).unwrap().is_empty());
        let (nb, out) = apply_effect_intents(&b, &[set("EffectData#0._life", json!(1.0))]).unwrap();
        assert_eq!(nb, b);
        assert!(matches!(out[0].status, ApplyStatus::Skipped(_)));
    }

    /// Set DMM_PARSER_EFFECT_DIR to a folder of extracted `.pae/.paem/.parg` files
    /// to check every one round-trips byte-exact.
    #[test]
    fn vanilla_effect_round_trip_env() {
        let Ok(dir) = std::env::var("DMM_PARSER_EFFECT_DIR") else {
            return;
        };
        let mut stack = vec![std::path::PathBuf::from(dir)];
        let (mut n, mut bad) = (0, Vec::new());
        while let Some(p) = stack.pop() {
            for e in std::fs::read_dir(&p).unwrap().flatten() {
                let q = e.path();
                if q.is_dir() {
                    stack.push(q);
                } else if matches!(q.extension().and_then(|x| x.to_str()), Some("pae" | "paem" | "parg")) {
                    n += 1;
                    let b = std::fs::read(&q).unwrap();
                    match effect_roundtrip(&b) {
                        Ok((true, _)) => {}
                        r => bad.push(format!("{}: {r:?}", q.display())),
                    }
                }
            }
        }
        assert!(bad.is_empty(), "{} of {n} not exact: {:?}", bad.len(), &bad[..bad.len().min(5)]);
    }
}
