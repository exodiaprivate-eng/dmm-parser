//! `.uibinary` (compiled UI screen) codec + UI intents, game 2.04.
//!
//! A `.uibinary` is the compiled form of one `ui/xml/**.html` screen (plus its
//! templates and css list). Since 2.04 the game reads the `.uibinary`; the
//! `.html` / `.css` stay the source text. Mods that change a screen have to
//! change the `.uibinary` and keep the `.html` in step (element classes in the
//! binary = the sorted set of the html `css` tokens).
//!
//! Layout (one schema drives read and write, so encode(decode(b)) == b):
//! - File  = opt root Element, css paths, template paths, components,
//!   source html path, 5-byte end.
//! - Element = classes, opt children, opt attributes (key/value), opt params,
//!   opt events, opt text, opt override (selector, type), u8 0, id,
//!   local string key, component ref, component name, script,
//!   PropertyCondition, 3-byte node tail.
//! - Component = elements, events, name, classes, script, u8 flag, attributes.
//! - Event = Type, KeyGuideEventType, AnimationRulesetName, opt Condition,
//!   opt Execute, actions, PropertyCondition.
//! - Action = u8 kind, Panel, Target, opt Condition, child actions, then per
//!   kind: 0 Texture (path value, u8 resource type), 1 StyleClass (remove,
//!   add), 2 Text (string info), 3 Show (u8), 5 Sound (name, u32 fade ms,
//!   u8 play), 6 LocalString (u32 key hash, u32 param count, 3 value slots),
//!   7 CssPropertyOnlyTextureLayer (u8 property, u8 layer, 3 value slots),
//!   9 SendGameEvent (u32 hash). Hashes = hashlittle(lowercase text, 0xC5EDE).
//! - Value = attribute text split in parts (text, u8 1 = `$..$` expression)
//!   + one opt compiled expression per part.
//! - Compiled expression = terms (op / call / variable, register machine)
//!   + u32 result register + 1.
//!
//! Strings are u32 length + UTF-8. f32 values decode to JSON numbers (exact;
//! non-finite values as `"f32:xxxxxxxx"`), fixed byte runs to hex.
//!
//! Intents (see [`apply_ui_intents`], [`apply_ui_css_intents`]): element
//! classes / attributes / script, subtree clone and delete, with the `.html`
//! edited in step; css property set and new rules.

use crate::intents::ApplyStatus;
use serde_json::{json, Map, Number, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::sync::OnceLock;

fn bad(m: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, m.into())
}

// ── Schema ─────────────────────────────────────────────────────────────────

enum Ty {
    Str,
    U8,
    U32,
    I32,
    F32,
    Raw(usize),
    Lst(Box<Ty>),
    Arr(Box<Ty>, usize),
    Opt(Box<Ty>),
    St(Vec<(&'static str, Ty)>),
    /// Tagged union: key field name, u32 key (else u8), variants (all `St`).
    Tag(&'static str, bool, Vec<(u32, Ty)>),
    /// u8 that is always 0.
    Zero,
    /// Recursive references.
    Elem,
    Action,
}

struct Schema {
    file: Ty,
    elem: Ty,
    action: Ty,
}

fn lst(t: Ty) -> Ty {
    Ty::Lst(Box::new(t))
}
fn opt(t: Ty) -> Ty {
    Ty::Opt(Box::new(t))
}
fn arr(t: Ty, n: usize) -> Ty {
    Ty::Arr(Box::new(t), n)
}
fn st(f: Vec<(&'static str, Ty)>) -> Ty {
    Ty::St(f)
}

fn konst() -> Ty {
    // bool (u8), int32, float, string
    Ty::Tag(
        "tag",
        false,
        vec![
            (0, st(vec![("value", Ty::U8)])),
            (1, st(vec![("value", Ty::I32)])),
            (2, st(vec![("value", Ty::F32)])),
            (3, st(vec![("value", Ty::Str)])),
        ],
    )
}

fn operand() -> Ty {
    // 3 = register / unused, 1 = inline constant, 2 = enum (type, value name)
    Ty::Tag(
        "mode",
        true,
        vec![
            (3, st(vec![])),
            (1, st(vec![("const", konst())])),
            (2, st(vec![("enum_type", Ty::Str), ("enum_value", Ty::Str)])),
        ],
    )
}

fn term_head() -> Vec<(&'static str, Ty)> {
    // op: 0 ==, 1 !=, 2 <, 3 >, 6 &&, 7 ||, 14 !, 0x10 load, 0x11 call.
    // reg = register written; a/b = registers read (-1 = use lhs/rhs operand);
    // type = left operand's type; paren = 1 if parenthesised in the source.
    vec![
        ("op", Ty::U8),
        ("reg", Ty::U32),
        ("a", Ty::I32),
        ("b", Ty::I32),
        ("lhs", operand()),
        ("rhs", operand()),
        ("type", Ty::Str),
        ("paren", Ty::U8),
    ]
}

fn cexpr() -> Ty {
    let mut call = term_head();
    call.extend(vec![
        ("obj", Ty::U32),
        ("class", Ty::Str),
        ("method", Ty::U32),
        ("class2", Ty::Str),
        ("args", lst(Ty::U32)),
    ]);
    let mut var = term_head();
    var.extend(vec![("name", Ty::Str), ("var_type", Ty::Str)]);
    let term = Ty::Tag("kind", false, vec![(0, st(term_head())), (1, st(call)), (2, st(var))]);
    st(vec![("terms", lst(opt(term))), ("result", Ty::U32)])
}

fn part() -> Ty {
    st(vec![("text", Ty::Str), ("expr", Ty::U8)])
}
fn value() -> Ty {
    st(vec![("parts", lst(part())), ("exprs", lst(opt(cexpr())))])
}
fn vals3() -> Ty {
    st(vec![("parts", arr(lst(part()), 3)), ("exprs", arr(lst(opt(cexpr())), 3))])
}

fn action() -> Ty {
    let with = |extra: Vec<(&'static str, Ty)>| {
        let mut h = vec![
            ("panel", Ty::Str),
            ("target", Ty::Str),
            ("condition", opt(cexpr())),
            ("children", lst(opt(Ty::Action))),
        ];
        h.extend(extra);
        st(h)
    };
    Ty::Tag(
        "kind",
        false,
        vec![
            (0, with(vec![("path", value()), ("resource_type", Ty::U8)])),
            (1, with(vec![("remove", value()), ("add", value())])),
            (2, with(vec![("string_info", value())])),
            (3, with(vec![("show", Ty::U8)])),
            (5, with(vec![("sound", Ty::Str), ("fade_ms", Ty::U32), ("play", Ty::U8)])),
            (6, with(vec![("key_hash", Ty::U32), ("param_count", Ty::U32), ("params", vals3())])),
            (7, with(vec![("property", Ty::U8), ("layer", Ty::U8), ("values", vals3())])),
            (9, with(vec![("hash", Ty::U32)])),
        ],
    )
}

fn event() -> Ty {
    st(vec![
        ("type", Ty::Str),
        ("key_guide", Ty::Str),
        ("animation", Ty::Str),
        ("condition", opt(cexpr())),
        ("execute", opt(cexpr())),
        ("actions", lst(opt(Ty::Action))),
        ("property_condition", Ty::Str),
    ])
}

fn kv() -> Ty {
    st(vec![("key", Ty::Str), ("value", Ty::Str)])
}

fn elem() -> Ty {
    st(vec![
        ("classes", lst(Ty::Str)),
        ("children", opt(lst(opt(Ty::Elem)))),
        ("attributes", opt(lst(kv()))),
        ("params", opt(lst(Ty::Str))),
        ("events", opt(lst(opt(event())))),
        ("text", opt(lst(Ty::Str))),
        ("override", opt(st(vec![("selector", Ty::Str), ("type", Ty::U8)]))),
        ("reserved", Ty::Zero),
        ("id", Ty::Str),
        ("local_string", Ty::Str),
        ("component_ref", Ty::Str),
        ("component_name", Ty::Str),
        ("script", Ty::Str),
        ("property_condition", Ty::Str),
        ("node_tail", Ty::Raw(3)),
    ])
}

fn component() -> Ty {
    st(vec![
        ("elements", lst(opt(Ty::Elem))),
        ("events", lst(opt(event()))),
        ("name", Ty::Str),
        ("classes", lst(Ty::Str)),
        ("script", Ty::Str),
        ("flag", Ty::U8),
        ("attributes", lst(kv())),
    ])
}

fn schema() -> &'static Schema {
    static S: OnceLock<Schema> = OnceLock::new();
    S.get_or_init(|| Schema {
        file: st(vec![
            ("root", opt(Ty::Elem)),
            ("css", lst(Ty::Str)),
            ("templates", lst(Ty::Str)),
            ("components", lst(component())),
            ("source_html", Ty::Str),
            ("end", Ty::Raw(5)),
        ]),
        elem: elem(),
        action: action(),
    })
}

// ── Read / write ───────────────────────────────────────────────────────────

struct Rd<'a> {
    b: &'a [u8],
    o: usize,
}

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if n > self.b.len() - self.o {
            return Err(bad(format!("uibinary: eof @{:x}", self.o)));
        }
        let v = &self.b[self.o..self.o + n];
        self.o += n;
        Ok(v)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
}

fn hx(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

fn unhx(s: &str) -> io::Result<Vec<u8>> {
    if s.len() % 2 != 0 {
        return Err(bad(format!("uibinary: odd hex length '{}'", s)));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| bad(format!("uibinary: bad hex '{}'", s))))
        .collect()
}

fn f32_json(bits: u32) -> Value {
    let f = f32::from_bits(bits);
    match Number::from_f64(f as f64) {
        Some(n) if f.is_finite() => Value::Number(n),
        _ => Value::String(format!("f32:{:08x}", bits)),
    }
}

fn f32_bits(v: &Value) -> io::Result<u32> {
    if let Some(s) = v.as_str() {
        let h = s.strip_prefix("f32:").ok_or_else(|| bad(format!("uibinary: f32 string '{}'", s)))?;
        return u32::from_str_radix(h, 16).map_err(|_| bad(format!("uibinary: f32 bits '{}'", s)));
    }
    let f = v.as_f64().ok_or_else(|| bad(format!("uibinary: expected number, got {}", v)))?;
    Ok((f as f32).to_bits())
}

fn rd(t: &Ty, r: &mut Rd, s: &Schema) -> io::Result<Value> {
    Ok(match t {
        Ty::Str => {
            let at = r.o;
            let n = r.u32()? as usize;
            if n > r.b.len() - r.o {
                return Err(bad(format!("uibinary: bad string length {} @{:x}", n, at)));
            }
            let v = r.take(n)?;
            Value::String(
                String::from_utf8(v.to_vec()).map_err(|_| bad(format!("uibinary: string not utf-8 @{:x}", at)))?,
            )
        }
        Ty::U8 => json!(r.u8()?),
        Ty::U32 => json!(r.u32()?),
        Ty::I32 => json!(r.u32()? as i32),
        Ty::F32 => f32_json(r.u32()?),
        Ty::Raw(n) => Value::String(hx(r.take(*n)?)),
        Ty::Lst(t) => {
            let at = r.o;
            let n = r.u32()? as usize;
            if n > 100_000 {
                return Err(bad(format!("uibinary: list length {} @{:x}", n, at)));
            }
            let mut v = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                v.push(rd(t, r, s)?);
            }
            Value::Array(v)
        }
        Ty::Arr(t, n) => {
            let mut v = Vec::with_capacity(*n);
            for _ in 0..*n {
                v.push(rd(t, r, s)?);
            }
            Value::Array(v)
        }
        Ty::Opt(t) => match r.u8()? {
            0 => Value::Null,
            1 => rd(t, r, s)?,
            f => return Err(bad(format!("uibinary: option flag {} @{:x}", f, r.o - 1))),
        },
        Ty::St(fs) => {
            let mut m = Map::new();
            for (k, t) in fs {
                m.insert(k.to_string(), rd(t, r, s)?);
            }
            Value::Object(m)
        }
        Ty::Tag(key, wide, tags) => {
            let at = r.o;
            let k = if *wide { r.u32()? } else { r.u8()? as u32 };
            let t = tags
                .iter()
                .find(|(x, _)| *x == k)
                .map(|(_, t)| t)
                .ok_or_else(|| bad(format!("uibinary: unknown {} {} @{:x}", key, k, at)))?;
            let mut v = rd(t, r, s)?;
            v.as_object_mut().unwrap().insert(key.to_string(), json!(k));
            v
        }
        Ty::Zero => {
            let v = r.u8()?;
            if v != 0 {
                return Err(bad(format!("uibinary: reserved byte {} @{:x}", v, r.o - 1)));
            }
            json!(0)
        }
        Ty::Elem => rd(&s.elem, r, s)?,
        Ty::Action => rd(&s.action, r, s)?,
    })
}

fn num(v: &Value, max: u64, what: &str) -> io::Result<u64> {
    v.as_u64().filter(|n| *n <= max).ok_or_else(|| bad(format!("uibinary: {} out of range: {}", what, v)))
}

fn wr(t: &Ty, v: &Value, out: &mut Vec<u8>, s: &Schema) -> io::Result<()> {
    match t {
        Ty::Str => {
            let x = v.as_str().ok_or_else(|| bad(format!("uibinary: expected string, got {}", v)))?;
            out.extend_from_slice(&(x.len() as u32).to_le_bytes());
            out.extend_from_slice(x.as_bytes());
        }
        Ty::U8 => out.push(num(v, 0xff, "u8")? as u8),
        Ty::U32 => out.extend_from_slice(&(num(v, u32::MAX as u64, "u32")? as u32).to_le_bytes()),
        Ty::I32 => {
            let n = v
                .as_i64()
                .filter(|n| *n >= i32::MIN as i64 && *n <= i32::MAX as i64)
                .ok_or_else(|| bad(format!("uibinary: i32 out of range: {}", v)))?;
            out.extend_from_slice(&(n as i32).to_le_bytes());
        }
        Ty::F32 => out.extend_from_slice(&f32_bits(v)?.to_le_bytes()),
        Ty::Raw(n) => {
            let b = unhx(v.as_str().ok_or_else(|| bad("uibinary: expected hex string"))?)?;
            if b.len() != *n {
                return Err(bad(format!("uibinary: fixed field needs {} bytes, got {}", n, b.len())));
            }
            out.extend_from_slice(&b);
        }
        Ty::Lst(t) => {
            let a = v.as_array().ok_or_else(|| bad(format!("uibinary: expected list, got {}", v)))?;
            out.extend_from_slice(&(a.len() as u32).to_le_bytes());
            for x in a {
                wr(t, x, out, s)?;
            }
        }
        Ty::Arr(t, n) => {
            let a = v.as_array().filter(|a| a.len() == *n).ok_or_else(|| bad(format!("uibinary: expected {} slots", n)))?;
            for x in a {
                wr(t, x, out, s)?;
            }
        }
        Ty::Opt(t) => {
            if v.is_null() {
                out.push(0);
            } else {
                out.push(1);
                wr(t, v, out, s)?;
            }
        }
        Ty::St(fs) => {
            let o = v.as_object().ok_or_else(|| bad(format!("uibinary: expected object, got {}", v)))?;
            for (k, t) in fs {
                let x = o.get(*k).ok_or_else(|| bad(format!("uibinary: missing field '{}'", k)))?;
                wr(t, x, out, s)?;
            }
        }
        Ty::Tag(key, wide, tags) => {
            let k = v.get(*key).and_then(|x| x.as_u64()).ok_or_else(|| bad(format!("uibinary: missing '{}'", key)))?;
            let t = tags
                .iter()
                .find(|(x, _)| *x as u64 == k)
                .map(|(_, t)| t)
                .ok_or_else(|| bad(format!("uibinary: unknown {} {}", key, k)))?;
            if *wide {
                out.extend_from_slice(&(k as u32).to_le_bytes());
            } else {
                out.push(k as u8);
            }
            wr(t, v, out, s)?;
        }
        Ty::Zero => {
            if num(v, 0, "reserved")? != 0 {
                return Err(bad("uibinary: reserved byte must be 0"));
            }
            out.push(0);
        }
        Ty::Elem => wr(&s.elem, v, out, s)?,
        Ty::Action => wr(&s.action, v, out, s)?,
    }
    Ok(())
}

/// Decode a whole `.uibinary` to JSON (byte-exact with [`encode_uibinary`]).
pub fn decode_uibinary(b: &[u8]) -> io::Result<Value> {
    let s = schema();
    let mut r = Rd { b, o: 0 };
    let v = rd(&s.file, &mut r, s)?;
    if r.o != b.len() {
        return Err(bad(format!("uibinary: {} bytes left @{:x}", b.len() - r.o, r.o)));
    }
    Ok(v)
}

/// Encode JSON from [`decode_uibinary`] back to file bytes.
pub fn encode_uibinary(v: &Value) -> io::Result<Vec<u8>> {
    let s = schema();
    let mut out = Vec::new();
    wr(&s.file, v, &mut out, s)?;
    Ok(out)
}

/// `encode(decode(b)) == b`.
pub fn roundtrip_check(b: &[u8]) -> io::Result<bool> {
    Ok(encode_uibinary(&decode_uibinary(b)?)? == b)
}

/// Bob Jenkins lookup3 `hashlittle`; LocalString keys and SendGameEvent
/// names are `hashlittle(lowercase(text), 0xC5EDE)`.
pub fn hashlittle(data: &[u8], init: u32) -> u32 {
    let mut a = 0xdeadbeefu32.wrapping_add(data.len() as u32).wrapping_add(init);
    let (mut b, mut c) = (a, a);
    let w = |d: &[u8], i: usize| u32::from_le_bytes(d[i..i + 4].try_into().unwrap());
    let mut i = 0;
    let mut n = data.len();
    while n > 12 {
        a = a.wrapping_add(w(data, i));
        b = b.wrapping_add(w(data, i + 4));
        c = c.wrapping_add(w(data, i + 8));
        a = a.wrapping_sub(c); a ^= c.rotate_left(4); c = c.wrapping_add(b);
        b = b.wrapping_sub(a); b ^= a.rotate_left(6); a = a.wrapping_add(c);
        c = c.wrapping_sub(b); c ^= b.rotate_left(8); b = b.wrapping_add(a);
        a = a.wrapping_sub(c); a ^= c.rotate_left(16); c = c.wrapping_add(b);
        b = b.wrapping_sub(a); b ^= a.rotate_left(19); a = a.wrapping_add(c);
        c = c.wrapping_sub(b); c ^= b.rotate_left(4); b = b.wrapping_add(a);
        i += 12;
        n -= 12;
    }
    if n == 0 {
        return c;
    }
    let mut tail = [0u8; 12];
    tail[..n].copy_from_slice(&data[i..]);
    a = a.wrapping_add(w(&tail, 0));
    b = b.wrapping_add(w(&tail, 4));
    c = c.wrapping_add(w(&tail, 8));
    c ^= b; c = c.wrapping_sub(b.rotate_left(14));
    a ^= c; a = a.wrapping_sub(c.rotate_left(11));
    b ^= a; b = b.wrapping_sub(a.rotate_left(25));
    c ^= b; c = c.wrapping_sub(b.rotate_left(16));
    a ^= c; a = a.wrapping_sub(c.rotate_left(4));
    b ^= a; b = b.wrapping_sub(a.rotate_left(14));
    c ^= b; c = c.wrapping_sub(b.rotate_left(24));
    c
}

// ── Element walk + selectors ───────────────────────────────────────────────

/// Element location: `comp` None = the root tree, else component index;
/// `path` = child indexes (empty + comp = the component itself).
#[derive(Clone, Debug, PartialEq)]
struct Loc {
    comp: Option<usize>,
    path: Vec<usize>,
}

impl Loc {
    fn ptr(&self) -> String {
        let mut p = match self.comp {
            None => "/root".to_string(),
            Some(ci) => format!("/components/{}", ci),
        };
        for (n, i) in self.path.iter().enumerate() {
            if n == 0 && self.comp.is_some() {
                p.push_str(&format!("/elements/{}", i));
            } else {
                p.push_str(&format!("/children/{}", i));
            }
        }
        p
    }
    fn is_component(&self) -> bool {
        self.comp.is_some() && self.path.is_empty()
    }
}

fn walk(doc: &Value) -> Vec<(String, Loc)> {
    fn rec(e: &Value, ctx: &str, loc: Loc, out: &mut Vec<(String, Loc)>) {
        if e.is_null() {
            return;
        }
        out.push((ctx.to_string(), loc.clone()));
        if let Some(kids) = e.get("children").and_then(|k| k.as_array()) {
            for (i, k) in kids.iter().enumerate() {
                let mut l = loc.clone();
                l.path.push(i);
                rec(k, ctx, l, out);
            }
        }
    }
    let mut out = Vec::new();
    rec(&doc["root"], "root", Loc { comp: None, path: vec![] }, &mut out);
    if let Some(comps) = doc["components"].as_array() {
        for (ci, c) in comps.iter().enumerate() {
            let ctx = format!("comp:{}", c["name"].as_str().unwrap_or(""));
            out.push((ctx.clone(), Loc { comp: Some(ci), path: vec![] }));
            if let Some(els) = c["elements"].as_array() {
                for (i, k) in els.iter().enumerate() {
                    rec(k, &ctx, Loc { comp: Some(ci), path: vec![i] }, &mut out);
                }
            }
        }
    }
    out
}

enum Nth {
    One,
    Index(usize),
    All,
}

/// `#Id` | `#Id@n` | `#Id@*` | `comp:Name` | `comp:Name#Id[@n|@*]` | `root`.
fn select(doc: &Value, sel: &str) -> Result<Vec<(String, Loc)>, String> {
    if sel == "root" {
        if doc["root"].is_null() {
            return Err("this uibinary has no root (component file): use comp:Name".into());
        }
        return Ok(vec![("root".into(), Loc { comp: None, path: vec![] })]);
    }
    let badsel = || format!("bad element selector '{}'", sel);
    let mut rest = sel;
    let mut comp = None;
    if let Some(r) = rest.strip_prefix("comp:") {
        let end = r.find(|c| c == '#' || c == '@').unwrap_or(r.len());
        if end == 0 {
            return Err(badsel());
        }
        comp = Some(&r[..end]);
        rest = &r[end..];
    }
    let mut eid = None;
    if let Some(r) = rest.strip_prefix('#') {
        let end = r.find('@').unwrap_or(r.len());
        if end == 0 {
            return Err(badsel());
        }
        eid = Some(&r[..end]);
        rest = &r[end..];
    }
    let mut nth = Nth::One;
    if let Some(r) = rest.strip_prefix('@') {
        nth = if r == "*" {
            Nth::All
        } else if !r.is_empty() && r.bytes().all(|c| c.is_ascii_digit()) {
            Nth::Index(r.parse().map_err(|_| badsel())?)
        } else {
            return Err(badsel());
        };
        rest = "";
    }
    if !rest.is_empty() {
        return Err(badsel());
    }
    let mut hits = Vec::new();
    for (ctx, loc) in walk(doc) {
        if let Some(c) = comp {
            if ctx != format!("comp:{}", c) {
                continue;
            }
        }
        let e = doc.pointer(&loc.ptr()).unwrap_or(&Value::Null);
        match eid {
            None => {
                if comp.is_some() && loc.is_component() && e["name"].as_str() == comp {
                    hits.push((ctx, loc));
                }
            }
            Some(id) => {
                if !loc.is_component() && e.get("id").and_then(|x| x.as_str()) == Some(id) {
                    hits.push((ctx, loc));
                }
            }
        }
    }
    if hits.is_empty() {
        return Err(format!("{}: no such element in this uibinary", sel));
    }
    match nth {
        Nth::All => Ok(hits),
        Nth::Index(n) => {
            if n >= hits.len() {
                Err(format!("{}: only {} matches", sel, hits.len()))
            } else {
                Ok(vec![hits.swap_remove(n)])
            }
        }
        Nth::One => {
            if hits.len() > 1 {
                Err(format!("{}: {} matches (template expansion) - add @n or @*", sel, hits.len()))
            } else {
                Ok(hits)
            }
        }
    }
}

/// One row per element / component: ctx, selector (unique, `@n` added for
/// repeated ids), id, classes, script, depth.
pub fn list_uibinary_elements(b: &[u8]) -> io::Result<Vec<Value>> {
    let doc = decode_uibinary(b)?;
    let w = walk(&doc);
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut total: BTreeMap<String, usize> = BTreeMap::new();
    let key = |ctx: &str, id: &str, global: bool| if global { format!("#{}", id) } else { format!("{}#{}", ctx, id) };
    for (ctx, loc) in &w {
        let e = doc.pointer(&loc.ptr()).unwrap();
        if let Some(id) = e.get("id").and_then(|x| x.as_str()).filter(|x| !x.is_empty()) {
            if !loc.is_component() {
                *total.entry(key(ctx, id, true)).or_default() += 1;
                if ctx != "root" {
                    *total.entry(key(ctx, id, false)).or_default() += 1;
                }
            }
        }
    }
    let mut rows = Vec::new();
    for (ctx, loc) in &w {
        let e = doc.pointer(&loc.ptr()).unwrap();
        let id = e.get("id").and_then(|x| x.as_str()).unwrap_or("");
        let sel = if loc.is_component() {
            Value::String(ctx.clone())
        } else if id.is_empty() {
            Value::Null
        } else {
            let k = key(ctx, id, ctx == "root");
            let n = seen.entry(k.clone()).or_default();
            let s = if total[&k] > 1 { format!("{}@{}", k, n) } else { k };
            *n += 1;
            Value::String(s)
        };
        rows.push(json!({
            "ctx": ctx,
            "selector": sel,
            "id": if loc.is_component() { Value::Null } else { json!(id) },
            "classes": e["classes"].clone(),
            "script": e["script"].clone(),
            "depth": loc.path.len(),
        }));
    }
    Ok(rows)
}

// ── Element edits (uibinary side) ──────────────────────────────────────────

fn str_list(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default()
}

/// classes = sorted set of (old - remove) | add  (the html css token set).
fn set_classes(e: &mut Value, add: &[String], rem: &[String]) -> (Vec<String>, Vec<String>) {
    let old = str_list(&e["classes"]);
    let mut s: BTreeSet<String> = old.iter().cloned().collect();
    for r in rem {
        s.remove(r);
    }
    for a in add {
        s.insert(a.clone());
    }
    let new: Vec<String> = s.into_iter().collect();
    e["classes"] = json!(new);
    (old, new)
}

fn set_attr(e: &mut Value, key: &str, val: &str) -> Result<Option<String>, String> {
    if key == "script" {
        let old = e["script"].as_str().map(|s| s.to_string());
        e["script"] = json!(val);
        return Ok(old);
    }
    if ["id", "css", "class", "name"].contains(&key.to_lowercase().as_str()) {
        return Err(format!("attribute '{}' is not editable (use array_append/array_remove on classes)", key));
    }
    if e["attributes"].is_null() {
        e["attributes"] = json!([]);
    }
    let attrs = e["attributes"].as_array_mut().ok_or("attributes is not a list")?;
    let kl = key.to_lowercase();
    for kv in attrs.iter_mut() {
        if kv["key"].as_str().map(|k| k.to_lowercase()) == Some(kl.clone()) {
            let old = kv["value"].as_str().map(|s| s.to_string());
            kv["value"] = json!(val);
            return Ok(old);
        }
    }
    attrs.push(json!({"key": key, "value": val}));
    Ok(None)
}

fn parent_list<'a>(doc: &'a mut Value, loc: &Loc) -> Option<&'a mut Vec<Value>> {
    let (last, parent) = loc.path.split_last()?;
    let _ = last;
    let p = Loc { comp: loc.comp, path: parent.to_vec() };
    let ptr = if p.is_component() {
        format!("{}/elements", p.ptr())
    } else {
        format!("{}/children", p.ptr())
    };
    doc.pointer_mut(&ptr)?.as_array_mut()
}

// ── html (text edits, everything else byte-identical) ──────────────────────

#[derive(Clone, Debug)]
struct HAttr {
    key: String,
    value: String,
    /// value span incl. quotes (absolute)
    vspan: Option<(usize, usize)>,
}

#[derive(Clone, Debug)]
struct HTag {
    start: usize,
    end: usize,
    name: String,
    selfclose: bool,
    attrs: Vec<HAttr>,
}

impl HTag {
    fn attr(&self, k: &str) -> Option<&HAttr> {
        self.attrs.iter().rev().find(|a| a.key == k)
    }
    fn get(&self, k: &str) -> &str {
        self.attr(k).map(|a| a.value.as_str()).unwrap_or("")
    }
}

fn is_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn try_tag(t: &str, s: usize) -> Option<HTag> {
    let b = t.as_bytes();
    let n = b.len();
    let mut p = s + 1;
    if p >= n || !b[p].is_ascii_alphabetic() {
        return None;
    }
    p += 1;
    while p < n && (b[p].is_ascii_alphanumeric() || b[p] == b'_' || b[p] == b'-' || b[p] >= 0x80) {
        p += 1;
    }
    let name = t[s + 1..p].to_string();
    let skip = |mut i: usize| {
        while i < n && is_ws(b[i]) {
            i += 1;
        }
        i
    };
    let mut attrs = Vec::new();
    loop {
        let w = skip(p);
        if w == p {
            break;
        }
        let mut q = w;
        while q < n && !is_ws(b[q]) && b[q] != b'=' && b[q] != b'>' && b[q] != b'/' {
            q += 1;
        }
        if q == w {
            break;
        }
        let key = t[w..q].to_lowercase();
        let mut vspan = None;
        let r = skip(q);
        if r < n && b[r] == b'=' {
            let v0 = skip(r + 1);
            if v0 < n && (b[v0] == b'"' || b[v0] == b'\'') {
                if let Some(cl) = t[v0 + 1..].find(b[v0] as char) {
                    vspan = Some((v0, v0 + 1 + cl + 1));
                }
            } else {
                let mut v1 = v0;
                while v1 < n && !is_ws(b[v1]) && b[v1] != b'>' && b[v1] != b'"' && b[v1] != b'\'' {
                    v1 += 1;
                }
                if v1 > v0 {
                    vspan = Some((v0, v1));
                }
            }
        }
        let value = match vspan {
            Some((a, e)) if b[a] == b'"' || b[a] == b'\'' => t[a + 1..e - 1].to_string(),
            Some((a, e)) => t[a..e].to_string(),
            None => String::new(),
        };
        p = vspan.map(|v| v.1).unwrap_or(q);
        attrs.push(HAttr { key, value, vspan });
    }
    let mut e = skip(p);
    let mut selfclose = false;
    if e < n && b[e] == b'/' {
        selfclose = true;
        e += 1;
    }
    if e < n && b[e] == b'>' {
        Some(HTag { start: s, end: e + 1, name, selfclose, attrs })
    } else {
        None
    }
}

fn html_tags(t: &str) -> Vec<HTag> {
    let b = t.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'<' {
            if let Some(tag) = try_tag(t, i) {
                i = tag.end;
                out.push(tag);
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Tags with this id inside the element's context (`comp:Name` = inside that
/// `<component name="Name">`; id None = the component tag itself).
fn html_find(t: &str, ctx: &str, eid: Option<&str>) -> Vec<HTag> {
    let tags = html_tags(t);
    let (mut lo, mut hi) = (0, t.len());
    if let Some(name) = ctx.strip_prefix("comp:") {
        match tags.iter().find(|g| g.name.eq_ignore_ascii_case("component") && g.get("name") == name) {
            None => return vec![],
            Some(g) => {
                lo = g.start;
                hi = t[g.end..].find("</component>").map(|x| x + g.end).unwrap_or(t.len());
            }
        }
        if eid.is_none() {
            return tags.into_iter().filter(|g| g.start == lo).collect();
        }
    }
    match eid {
        None => vec![],
        Some(id) => tags.into_iter().filter(|g| g.start >= lo && g.start < hi && g.get("id") == id).collect(),
    }
}

fn html_set_attr(t: &str, g: &HTag, key: &str, val: &str) -> String {
    let k = key.to_lowercase();
    if let Some((vs, ve)) = g.attr(&k).and_then(|a| a.vspan) {
        let q = match t.as_bytes()[vs] {
            b'\'' => "'",
            _ => "\"",
        };
        return format!("{}{}{}{}{}", &t[..vs], q, val, q, &t[ve..]);
    }
    let mut ins = g.end - if g.selfclose { 2 } else { 1 };
    while ins > 0 && is_ws(t.as_bytes()[ins - 1]) {
        ins -= 1;
    }
    format!("{} {}=\"{}\"{}", &t[..ins], key, val, &t[ins..])
}

fn css_delta(old: &str, add: &[String], rem: &[String]) -> String {
    let old: Vec<&str> = old.split_whitespace().collect();
    let mut new: Vec<&str> = old.iter().copied().filter(|c| !rem.iter().any(|r| r == c)).collect();
    for a in add {
        if !old.contains(&a.as_str()) {
            new.push(a);
        }
    }
    new.join(" ")
}

fn html_one(t: &str, ctx: &str, eid: Option<&str>) -> Result<HTag, String> {
    let hits = html_find(t, ctx, eid);
    if hits.len() != 1 {
        return Err(format!(
            "html: {} tags match {} #{} - html not edited (element from a template, or duplicate id)",
            hits.len(),
            ctx,
            eid.unwrap_or("")
        ));
    }
    Ok(hits.into_iter().next().unwrap())
}

fn html_edit_class(t: &str, ctx: &str, eid: Option<&str>, add: &[String], rem: &[String]) -> Result<String, String> {
    let g = html_one(t, ctx, eid)?;
    Ok(html_set_attr(t, &g, "css", &css_delta(g.get("css"), add, rem)))
}

fn html_edit_attr(t: &str, ctx: &str, eid: Option<&str>, key: &str, val: &str) -> Result<String, String> {
    let g = html_one(t, ctx, eid)?;
    Ok(html_set_attr(t, &g, key, val))
}

/// Whole element span incl. its closing tag (same-name nesting counted).
fn html_elem_span(t: &str, g: &HTag) -> Result<(usize, usize), String> {
    if g.selfclose {
        return Ok((g.start, g.end));
    }
    let b = t.as_bytes();
    let nm = g.name.as_bytes();
    let (mut depth, mut pos) = (1, g.end);
    while depth > 0 {
        let mut found = None;
        let mut x = pos;
        while let Some(off) = t[x..].find('<') {
            let at = x + off;
            let close = b.get(at + 1) == Some(&b'/');
            let n0 = at + 1 + close as usize;
            let n1 = n0 + nm.len();
            if n1 < b.len()
                && b[n0..n1].eq_ignore_ascii_case(nm)
                && (is_ws(b[n1]) || b[n1] == b'>' || b[n1] == b'/')
            {
                found = Some((close, n1));
                break;
            }
            x = at + 1;
        }
        let (close, n1) = found.ok_or_else(|| format!("html: no closing tag for <{}> @{}", g.name, g.start))?;
        let gt = t[n1..].find('>').map(|o| o + n1).ok_or("html: unterminated tag")?;
        if close {
            depth -= 1;
        } else if b[gt - 1] != b'/' {
            depth += 1;
        }
        pos = gt + 1;
    }
    Ok((g.start, pos))
}

fn html_clone(
    t: &str,
    ctx: &str,
    src_id: &str,
    new_id: &str,
    add: &[String],
    rem: &[String],
    after_id: Option<&str>,
) -> Result<String, String> {
    let g = html_one(t, ctx, Some(src_id))?;
    if !html_find(t, ctx, Some(new_id)).is_empty() {
        return Err(format!("html: id {} exists - html not edited", new_id));
    }
    let (s0, e0) = html_elem_span(t, &g)?;
    let mut blk = t[s0..e0].to_string();
    let g0 = html_tags(&blk).into_iter().next().ok_or("html: no tag")?;
    blk = html_set_attr(&blk, &g0, "id", new_id);
    let g0 = html_tags(&blk).into_iter().next().ok_or("html: no tag")?;
    let css = css_delta(g0.get("css"), add, rem);
    blk = html_set_attr(&blk, &g0, "css", &css);
    let (mut a, mut b) = (s0, e0);
    if let Some(aid) = after_id.filter(|x| *x != src_id) {
        let ag = html_one(t, ctx, Some(aid))?;
        let sp = html_elem_span(t, &ag)?;
        a = sp.0;
        b = sp.1;
    }
    let ls = t[..a].rfind('\n').map(|x| x + 1).unwrap_or(0);
    let ind = if t[ls..a].trim().is_empty() { &t[ls..a] } else { "" };
    let nl = if t.contains("\r\n") { "\r\n" } else { "\n" };
    Ok(format!("{}{}{}{}{}", &t[..b], nl, ind, blk, &t[b..]))
}

fn html_delete(t: &str, ctx: &str, eid: &str) -> Result<String, String> {
    let g = html_one(t, ctx, Some(eid))?;
    let (s, e) = html_elem_span(t, &g)?;
    let mut start = s;
    if let Some(nl) = t[..s].rfind('\n') {
        if t[nl + 1..s].trim().is_empty() {
            start = if nl > 0 && t.as_bytes()[nl - 1] == b'\r' { nl - 1 } else { nl };
        }
    }
    Ok(format!("{}{}", &t[..start], &t[e..]))
}

/// Per id that is unique in both files: html `css` token set vs uibinary
/// classes, `script`, and attribute values present in both.
/// -> `{id: {"classes": [html, uibinary], "script": [..], "attributes": {k: [..]}}}`
/// (only ids with a difference; template / override elements differ in vanilla too).
pub fn check_uibinary_html_pair(b: &[u8], html: &str) -> io::Result<Value> {
    let doc = decode_uibinary(b)?;
    let mut hid: BTreeMap<String, Vec<HTag>> = BTreeMap::new();
    for g in html_tags(html) {
        if let Some(a) = g.attr("id") {
            hid.entry(a.value.clone()).or_default().push(g.clone());
        }
    }
    let mut uid: BTreeMap<String, Vec<Loc>> = BTreeMap::new();
    for (_, loc) in walk(&doc) {
        if loc.is_component() {
            continue;
        }
        let e = doc.pointer(&loc.ptr()).unwrap();
        if let Some(id) = e["id"].as_str().filter(|x| !x.is_empty()) {
            uid.entry(id.to_string()).or_default().push(loc);
        }
    }
    let mut out = Map::new();
    for (id, hs) in &hid {
        let us = match uid.get(id) {
            Some(u) if u.len() == 1 && hs.len() == 1 => u,
            _ => continue,
        };
        let (g, e) = (&hs[0], doc.pointer(&us[0].ptr()).unwrap());
        let mut iss = Map::new();
        let hc: BTreeSet<&str> = g.get("css").split_whitespace().collect();
        let hc: Vec<&str> = hc.into_iter().collect();
        if json!(hc) != e["classes"] {
            iss.insert("classes".into(), json!([hc, e["classes"]]));
        }
        let us_script = e["script"].as_str().unwrap_or("");
        if g.get("script") != us_script {
            iss.insert("script".into(), json!([g.get("script"), us_script]));
        }
        let mut ua: BTreeMap<String, String> = BTreeMap::new();
        for kv in e["attributes"].as_array().into_iter().flatten() {
            ua.insert(kv["key"].as_str().unwrap_or("").to_lowercase(), kv["value"].as_str().unwrap_or("").to_string());
        }
        let mut keys: Vec<&str> = Vec::new();
        for a in &g.attrs {
            if !keys.contains(&a.key.as_str()) {
                keys.push(&a.key);
            }
        }
        let mut am = Map::new();
        for k in keys {
            if let Some(u) = ua.get(k) {
                if g.get(k) != u {
                    am.insert(k.to_string(), json!([g.get(k), u]));
                }
            }
        }
        if !am.is_empty() {
            iss.insert("attributes".into(), Value::Object(am));
        }
        if !iss.is_empty() {
            out.insert(id.clone(), Value::Object(iss));
        }
    }
    Ok(Value::Object(out))
}

// ── css (text edits) ───────────────────────────────────────────────────────

fn mask_comments(t: &str) -> Vec<u8> {
    let mut b = t.as_bytes().to_vec();
    let mut i = 0;
    while let Some(o) = t[i..].find("/*") {
        let s = i + o;
        match t[s + 2..].find("*/") {
            Some(e) => {
                let e = s + 2 + e + 2;
                b[s..e].iter_mut().for_each(|c| *c = b' ');
                i = e;
            }
            None => break,
        }
    }
    b
}

/// Rules as (selector start, selector end, body start, body end) on the masked text.
fn css_rules(mk: &[u8]) -> Vec<(usize, usize, usize, usize)> {
    let mut out = Vec::new();
    let n = mk.len();
    let mut p = 0;
    while p < n {
        if mk[p] == b'{' || mk[p] == b'}' {
            p += 1;
            continue;
        }
        let mut q = p;
        while q < n && mk[q] != b'{' && mk[q] != b'}' {
            q += 1;
        }
        if q >= n || mk[q] == b'}' {
            p = q + 1;
            continue;
        }
        let mut r = q + 1;
        while r < n && mk[r] != b'{' && mk[r] != b'}' {
            r += 1;
        }
        if r < n && mk[r] == b'}' {
            out.push((p, q, q + 1, r));
            p = r + 1;
        } else {
            p = q + 1;
        }
    }
    out
}

fn cnorm(s: &str) -> String {
    s.replace('\u{feff}', "").split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn mstr(mk: &[u8], a: usize, b: usize) -> String {
    String::from_utf8_lossy(&mk[a..b]).into_owned()
}

/// First `prop: value` in a rule body -> value span (trailing spaces trimmed).
fn css_prop(mk: &[u8], bs: usize, be: usize, prop: &str) -> Option<(usize, usize)> {
    let pl = prop.to_ascii_lowercase();
    let pb = pl.as_bytes();
    let mut i = bs;
    while i + pb.len() <= be {
        let hit = mk[i..i + pb.len()].eq_ignore_ascii_case(pb)
            && (i == bs || !(mk[i - 1].is_ascii_alphanumeric() || mk[i - 1] == b'_' || mk[i - 1] == b'-' || mk[i - 1] >= 0x80));
        if hit {
            let mut j = i + pb.len();
            while j < be && is_ws(mk[j]) {
                j += 1;
            }
            if j < be && mk[j] == b':' {
                j += 1;
                while j < be && is_ws(mk[j]) {
                    j += 1;
                }
                let mut e = j;
                while e < be && mk[e] != b';' && mk[e] != b'}' {
                    e += 1;
                }
                while e > j && is_ws(mk[e - 1]) {
                    e -= 1;
                }
                return Some((j, e));
            }
        }
        i += 1;
    }
    None
}

/// Set `prop` in every rule whose selector equals `selector` (whitespace and
/// case normalised) where it exists, else add it to the last such rule.
/// -> (new text, old values).
pub fn css_set(t: &str, selector: &str, prop: &str, val: &str) -> Result<(String, Vec<Option<String>>), String> {
    let mk = mask_comments(t);
    let rules: Vec<_> = css_rules(&mk).into_iter().filter(|r| cnorm(&mstr(&mk, r.0, r.1)) == cnorm(selector)).collect();
    if rules.is_empty() {
        return Err(format!("css: no rule '{}'", selector));
    }
    let mut out = t.to_string();
    let mut done = Vec::new();
    for r in rules.iter().rev() {
        if let Some((vs, ve)) = css_prop(&mk, r.2, r.3, prop) {
            done.push(Some(out[vs..ve].to_string()));
            out = format!("{}{}{}", &out[..vs], val, &out[ve..]);
        }
    }
    if done.is_empty() {
        let r = rules.last().unwrap();
        let body = &out[r.2..r.3];
        let j = body.trim_end().len();
        let sep = if body.trim().is_empty() || body.trim_end().ends_with(';') { "" } else { ";" };
        out = format!("{}{} {}: {};{}", &out[..r.2 + j], sep, prop, val, &out[r.2 + j..]);
        done.push(None);
    }
    Ok((out, done))
}

/// Add one rule `selector { prop: value; }` on a new line after the last rule
/// `after` (default: end of file). Refuses a selector that already exists.
pub fn css_add(t: &str, rule: &str, after: Option<&str>) -> Result<String, String> {
    if !rule.contains('{') || !rule.trim_end().ends_with('}') {
        return Err("css add_rule: rule must be 'selector { prop: value; }'".into());
    }
    let sel = rule.split('{').next().unwrap();
    let mk = mask_comments(t);
    let rules = css_rules(&mk);
    if rules.iter().any(|r| cnorm(&mstr(&mk, r.0, r.1)) == cnorm(sel)) {
        return Err(format!("css add_rule: rule '{}' exists - use set", sel.trim()));
    }
    let nl = if t.contains("\r\n") { "\r\n" } else { "\n" };
    let after = match after {
        None => return Ok(format!("{}{}{}{}", t.trim_end(), nl, rule, nl)),
        Some(a) => a,
    };
    let last = rules
        .iter()
        .filter(|r| cnorm(&mstr(&mk, r.0, r.1)) == cnorm(after))
        .last()
        .ok_or_else(|| format!("css add_rule: no rule '{}'", after))?;
    let end = last.3 + 1;
    let le = t[end..].find('\n').map(|x| end + x + 1).unwrap_or(t.len());
    let pre = if le > 0 && t.as_bytes()[le - 1] == b'\n' { "" } else { nl };
    Ok(format!("{}{}{}{}{}", &t[..le], pre, rule, nl, &t[le..]))
}

// ── Intents ────────────────────────────────────────────────────────────────

/// Outcome of one UI intent: `status` as for table intents; `html` = what
/// happened to the `.html` (None when no html was given or not touched).
#[derive(Debug, Clone)]
pub struct UiOutcome {
    pub op: String,
    pub status: ApplyStatus,
    pub html: Option<String>,
}

fn strs(v: &Value) -> Result<Vec<String>, String> {
    match v {
        Value::String(s) => Ok(vec![s.clone()]),
        Value::Array(a) => a.iter().map(|x| x.as_str().map(|s| s.to_string()).ok_or("expected strings".to_string())).collect(),
        Value::Null => Ok(vec![]),
        _ => Err("expected a string or a list of strings".into()),
    }
}

fn text_of(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn one_ui_intent(doc: &mut Value, html: &mut Option<String>, it: &Value) -> Result<Option<String>, String> {
    let op = it["op"].as_str().unwrap_or("");
    let entry = it["entry"].as_str().ok_or("missing 'entry' (element selector)")?;
    let field = it["field"].as_str();
    match op {
        "set" | "array_append" | "array_remove" => {
            let hits = select(doc, entry)?;
            let (key, val, add, rem) = if op == "set" {
                let k = field.ok_or("set: missing 'field' (attribute name or 'script')")?;
                let v = text_of(it.get("value").or(it.get("new")).unwrap_or(&Value::Null)).ok_or("set: 'value' must be text")?;
                (Some(k.to_string()), v, vec![], vec![])
            } else {
                if field != Some("classes") {
                    return Err(format!("{}: only field 'classes' is supported", op));
                }
                let v = strs(it.get("value").unwrap_or(&Value::Null))?;
                if v.is_empty() {
                    return Err(format!("{}: 'value' is empty", op));
                }
                if op == "array_append" { (None, String::new(), v, vec![]) } else { (None, String::new(), vec![], v) }
            };
            // validate every hit first (no partial edits)
            if let Some(k) = &key {
                if k != "script" && ["id", "css", "class", "classes", "name"].contains(&k.to_lowercase().as_str()) {
                    return Err(format!("attribute '{}' is not editable (use array_append/array_remove on classes)", k));
                }
            }
            for (_, loc) in &hits {
                let e = doc.pointer_mut(&loc.ptr()).ok_or("element vanished")?;
                match &key {
                    Some(k) => {
                        set_attr(e, k, &val)?;
                    }
                    None => {
                        set_classes(e, &add, &rem);
                    }
                }
            }
            let mut note = None;
            if let Some(h) = html.as_mut() {
                if hits.len() != 1 {
                    note = Some(format!("html not edited: {} elements (template expansion)", hits.len()));
                } else {
                    let (ctx, loc) = &hits[0];
                    let e = doc.pointer(&loc.ptr()).unwrap();
                    let eid = if loc.is_component() { None } else { e["id"].as_str() };
                    let r = match &key {
                        Some(k) => html_edit_attr(h, ctx, eid, k, &val),
                        None => html_edit_class(h, ctx, eid, &add, &rem),
                    };
                    note = Some(match r {
                        Ok(nt) => {
                            *h = nt;
                            "html edited".to_string()
                        }
                        Err(e) => e,
                    });
                }
            }
            Ok(note)
        }
        "clone_record" => {
            let opts = it.get("value").filter(|v| v.is_object()).ok_or("clone_record: 'value' must be {\"id\", \"add\", \"remove\", \"after\"}")?;
            let new_id = opts["id"].as_str().filter(|s| !s.is_empty()).ok_or("clone_record: missing value.id (new element id)")?;
            let add = strs(&opts["add"])?;
            let rem = strs(&opts["remove"])?;
            let hits = select(doc, entry)?;
            if hits.len() != 1 {
                return Err(format!("{}: clone needs exactly one source element (use @n)", entry));
            }
            let (ctx, src) = hits[0].clone();
            if src.path.is_empty() {
                return Err("root / component cannot be cloned".into());
            }
            for (_, loc) in walk(doc) {
                if !loc.is_component() && doc.pointer(&loc.ptr()).unwrap()["id"].as_str() == Some(new_id) {
                    return Err(format!("id {} already exists", new_id));
                }
            }
            let mut anchor = src.clone();
            if let Some(after) = opts["after"].as_str() {
                let ah = select(doc, after)?;
                if ah.len() != 1 {
                    return Err(format!("{}: needs exactly one anchor element", after));
                }
                if ah[0].0 != ctx {
                    return Err(format!("anchor {} is in {}, source in {}", after, ah[0].0, ctx));
                }
                anchor = ah[0].1.clone();
                if anchor.path.is_empty() {
                    return Err("anchor cannot be the root or a component".into());
                }
            }
            let src_id = doc.pointer(&src.ptr()).unwrap()["id"].as_str().unwrap_or("").to_string();
            let anc_id = doc.pointer(&anchor.ptr()).unwrap()["id"].as_str().unwrap_or("").to_string();
            let mut new_html = None;
            if let Some(h) = html.as_ref() {
                new_html = Some(html_clone(h, &ctx, &src_id, new_id, &add, &rem, Some(anc_id.as_str()))?);
            }
            let mut ne = doc.pointer(&src.ptr()).unwrap().clone();
            ne["id"] = json!(new_id);
            set_classes(&mut ne, &add, &rem);
            let idx = *anchor.path.last().unwrap();
            let lst = parent_list(doc, &anchor).ok_or("anchor has no parent list")?;
            lst.insert(idx + 1, ne);
            Ok(new_html.map(|nh| {
                *html = Some(nh);
                "html edited".to_string()
            }))
        }
        "delete_record" => {
            let hits = select(doc, entry)?;
            if hits.len() != 1 {
                return Err(format!("{}: delete needs exactly one element (use @n)", entry));
            }
            let (ctx, loc) = hits[0].clone();
            if loc.path.is_empty() {
                return Err("root / component cannot be deleted".into());
            }
            let eid = doc.pointer(&loc.ptr()).unwrap()["id"].as_str().unwrap_or("").to_string();
            let mut new_html = None;
            if let Some(h) = html.as_ref() {
                new_html = Some(html_delete(h, &ctx, &eid)?);
            }
            let idx = *loc.path.last().unwrap();
            parent_list(doc, &loc).ok_or("element has no parent list")?.remove(idx);
            Ok(new_html.map(|nh| {
                *html = Some(nh);
                "html edited".to_string()
            }))
        }
        other => Err(format!("unknown ui op '{}' (set, array_append, array_remove, clone_record, delete_record)", other)),
    }
}

/// Apply element intents to a `.uibinary`, editing the matching `.html` in step
/// when given. Elements are selected by `entry` = `#Id`, `#Id@n` (n-th match,
/// document order), `#Id@*` (all; html then left alone), `comp:Name`,
/// `comp:Name#Id`, `root`.
///
/// | op | fields | effect |
/// |---|---|---|
/// | `set` | `entry`, `field` (attribute name or `script`), `value` (text) | replace or add one attribute |
/// | `array_append` / `array_remove` | `entry`, `field` = `classes`, `value` (class or list) | class add / remove (classes kept as the sorted set) |
/// | `clone_record` | `entry` (source), `value` `{"id", "add", "remove", "after"}` | deep copy of the element subtree as a new sibling after `after` (default the source), new id + class delta |
/// | `delete_record` | `entry` | remove the element subtree |
///
/// Misses come back as `Skipped` with a reason and change nothing.
/// Returns (uibinary bytes, html text, outcomes).
pub fn apply_ui_intents(b: &[u8], html: Option<&str>, intents: &[Value]) -> io::Result<(Vec<u8>, Option<String>, Vec<UiOutcome>)> {
    let mut doc = decode_uibinary(b)?;
    let mut h = html.map(|s| s.to_string());
    let mut outs = Vec::new();
    for it in intents {
        let op = it["op"].as_str().unwrap_or("").to_string();
        let mut trial = doc.clone();
        let mut th = h.clone();
        match one_ui_intent(&mut trial, &mut th, it) {
            Ok(note) => {
                doc = trial;
                h = th;
                outs.push(UiOutcome { op, status: ApplyStatus::Applied, html: note });
            }
            Err(r) => outs.push(UiOutcome { op, status: ApplyStatus::Skipped(r), html: None }),
        }
    }
    Ok((encode_uibinary(&doc)?, h, outs))
}

/// Apply css intents to a `.css` text:
/// `{"op": "set", "entry": <selector>, "field": <property>, "value": <value>}`
/// and `{"op": "add_rule", "value": "sel { prop: value; }", "after": <selector>?}`.
pub fn apply_ui_css_intents(css: &str, intents: &[Value]) -> (String, Vec<UiOutcome>) {
    let mut t = css.to_string();
    let mut outs = Vec::new();
    for it in intents {
        let op = it["op"].as_str().unwrap_or("").to_string();
        let r: Result<String, String> = match op.as_str() {
            "set" => match (it["entry"].as_str(), it["field"].as_str(), text_of(it.get("value").unwrap_or(&Value::Null))) {
                (Some(s), Some(p), Some(v)) => css_set(&t, s, p, &v).map(|x| x.0),
                _ => Err("set: needs entry (selector), field (property), value".into()),
            },
            "add_rule" => match it["value"].as_str() {
                Some(rule) => css_add(&t, rule, it["after"].as_str()),
                None => Err("add_rule: 'value' must be the rule text".into()),
            },
            o => Err(format!("unknown css op '{}' (set, add_rule)", o)),
        };
        match r {
            Ok(nt) => {
                t = nt;
                outs.push(UiOutcome { op, status: ApplyStatus::Applied, html: None });
            }
            Err(e) => outs.push(UiOutcome { op, status: ApplyStatus::Skipped(e), html: None }),
        }
    }
    (t, outs)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn elem_json(id: &str, cls: &[&str], kids: Vec<Value>) -> Value {
        json!({
            "classes": cls, "children": if kids.is_empty() { Value::Null } else { json!(kids) },
            "attributes": [{"key": "delay-time", "value": "0.1"}], "params": null,
            "events": null, "text": null, "override": null, "reserved": 0, "id": id,
            "local_string": "", "component_ref": "", "component_name": "", "script": "",
            "property_condition": "", "node_tail": "000000"
        })
    }

    fn sample() -> Value {
        let cx = json!({"terms": [{"kind": 2, "op": 16, "reg": 0, "a": -1, "b": -1,
            "lhs": {"mode": 1, "const": {"tag": 2, "value": 1.5}}, "rhs": {"mode": 3},
            "type": "float", "paren": 0, "name": "self", "var_type": "CommonActor"}, null], "result": 1});
        let act = json!({"kind": 1, "panel": "", "target": "#A", "condition": cx, "children": [null],
            "remove": {"parts": [{"text": "a", "expr": 0}], "exprs": [null]},
            "add": {"parts": [], "exprs": []}});
        let mut root = elem_json("Root", &["root"], vec![
            elem_json("A", &["a", "x"], vec![elem_json("A1", &[], vec![])]),
            elem_json("B", &["b"], vec![]),
        ]);
        root["events"] = json!([{"type": "show", "key_guide": "", "animation": "", "condition": null,
            "execute": null, "actions": [act], "property_condition": ""}]);
        json!({"root": root, "css": ["ui/x.css"], "templates": [], "components": [{
            "elements": [elem_json("C1", &["c"], vec![])], "events": [], "name": "Comp",
            "classes": [], "script": "S", "flag": 1, "attributes": []}],
            "source_html": "ui/x.html", "end": "0000000000"})
    }

    const HTML: &str = "<html>\r\n<div id=\"Root\" css=\"root\">\r\n  <div id=\"A\" css=\"x a\" delay-time=\"0.1\">\r\n    <div id=\"A1\"/>\r\n  </div>\r\n  <div id=\"B\" css=\"b\"></div>\r\n</div>\r\n<component name=\"Comp\" script=\"S\"><div id=\"C1\" css=\"c\"/></component>\r\n</html>\r\n";

    #[test]
    fn roundtrip_synthetic() {
        let v = sample();
        let b = encode_uibinary(&v).unwrap();
        let d = decode_uibinary(&b).unwrap();
        assert_eq!(d, v);
        assert!(roundtrip_check(&b).unwrap());
        assert!(decode_uibinary(&b[..b.len() - 1]).is_err());
    }

    #[test]
    fn hash_matches_reference() {
        // lookup3 known value: hashlittle("Four score and seven years ago", 0) = 0x17770551
        assert_eq!(hashlittle(b"Four score and seven years ago", 0), 0x17770551);
        assert_eq!(hashlittle(b"", 0), 0xdeadbeef);
    }

    #[test]
    fn select_and_class_attr_with_html() {
        let b = encode_uibinary(&sample()).unwrap();
        let intents = vec![
            json!({"op": "array_append", "entry": "#A", "field": "classes", "value": ["!none", "b0"]}),
            json!({"op": "array_remove", "entry": "#A", "field": "classes", "value": "x"}),
            json!({"op": "set", "entry": "#A", "field": "delay-time", "value": "5"}),
            json!({"op": "set", "entry": "#B", "field": "time-scale", "value": 0.5}),
            json!({"op": "set", "entry": "#Nope", "field": "x", "value": "1"}),
            json!({"op": "set", "entry": "#A", "field": "id", "value": "1"}),
        ];
        let (nb, nh, outs) = apply_ui_intents(&b, Some(HTML), &intents).unwrap();
        let st: Vec<bool> = outs.iter().map(|o| matches!(o.status, ApplyStatus::Applied)).collect();
        assert_eq!(st, vec![true, true, true, true, false, false]);
        let d = decode_uibinary(&nb).unwrap();
        let a = &d["root"]["children"][0];
        assert_eq!(a["classes"], json!(["!none", "a", "b0"]));
        assert_eq!(a["attributes"][0]["value"], json!("5"));
        assert_eq!(d["root"]["children"][1]["attributes"][1], json!({"key": "time-scale", "value": "0.5"}));
        let nh = nh.unwrap();
        assert!(nh.contains("<div id=\"A\" css=\"a !none b0\" delay-time=\"5\">"), "{}", nh);
        assert!(nh.contains("<div id=\"B\" css=\"b\" time-scale=\"0.5\"></div>"), "{}", nh);
        assert!(check_uibinary_html_pair(&nb, &nh).unwrap().as_object().unwrap().is_empty());
    }

    #[test]
    fn clone_and_delete_with_html() {
        let b = encode_uibinary(&sample()).unwrap();
        let intents = vec![
            json!({"op": "clone_record", "entry": "#A", "value": {"id": "A2", "add": ["y"], "remove": ["x"]}}),
            json!({"op": "clone_record", "entry": "#A", "value": {"id": "A3", "add": "z", "after": "#A2"}}),
            json!({"op": "clone_record", "entry": "#A", "value": {"id": "A3"}}),
            json!({"op": "clone_record", "entry": "comp:Comp#C1", "value": {"id": "C2", "after": "#B"}}),
            json!({"op": "delete_record", "entry": "#B"}),
        ];
        let (nb, nh, outs) = apply_ui_intents(&b, Some(HTML), &intents).unwrap();
        let st: Vec<bool> = outs.iter().map(|o| matches!(o.status, ApplyStatus::Applied)).collect();
        assert_eq!(st, vec![true, true, false, false, true]);
        let d = decode_uibinary(&nb).unwrap();
        let ids: Vec<&str> = d["root"]["children"].as_array().unwrap().iter().map(|e| e["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["A", "A2", "A3"]);
        assert_eq!(d["root"]["children"][1]["classes"], json!(["a", "y"]));
        assert_eq!(d["root"]["children"][2]["children"][0]["id"], json!("A1"));
        let nh = nh.unwrap();
        let a2 = nh.find("<div id=\"A2\" css=\"a y\"").unwrap();
        let a3 = nh.find("<div id=\"A3\" css=\"x a z\"").unwrap();
        assert!(a2 < a3 && nh.find("id=\"B\"").is_none(), "{}", nh);
        assert!(nh.contains("\r\n  <div id=\"A2\""));
        assert!(check_uibinary_html_pair(&nb, &nh).unwrap().as_object().unwrap().is_empty());
        let rows = list_uibinary_elements(&nb).unwrap();
        assert!(rows.iter().any(|r| r["selector"] == json!("comp:Comp#C1")));
    }

    #[test]
    fn css_edits() {
        let css = "/* {x} */\r\n#L { height: 320px; }\r\n.w.a3 { top: 140px; }\r\n.w.a5{top:1px}\r\n";
        let (t, o) = apply_ui_css_intents(css, &[
            json!({"op": "set", "entry": "#L", "field": "height", "value": "390px"}),
            json!({"op": "add_rule", "value": ".w.a4 { top: 210px; } ", "after": ".w.a3"}),
            json!({"op": "add_rule", "value": ".w.a3 { top: 0; }"}),
            json!({"op": "set", "entry": ".w.a5", "field": "left", "value": "2px"}),
            json!({"op": "set", "entry": ".nope", "field": "top", "value": "1px"}),
        ]);
        let st: Vec<bool> = o.iter().map(|o| matches!(o.status, ApplyStatus::Applied)).collect();
        assert_eq!(st, vec![true, true, false, true, false]);
        assert_eq!(t, "/* {x} */\r\n#L { height: 390px; }\r\n.w.a3 { top: 140px; }\r\n.w.a4 { top: 210px; } \r\n.w.a5{top:1px; left: 2px;}\r\n");
    }

    fn files_with_ext(dir: &std::path::Path, ext: &str, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    files_with_ext(&p, ext, out);
                } else if p.extension().map(|x| x.eq_ignore_ascii_case(ext)).unwrap_or(false) {
                    out.push(p);
                }
            }
        }
    }

    /// `DMM_PARSER_UIBINARY_DIR=<dir with extracted .uibinary> cargo test --release --lib uibinary::tests::vanilla -- --nocapture`
    #[test]
    fn vanilla() {
        let Ok(dir) = std::env::var("DMM_PARSER_UIBINARY_DIR") else { return };
        let mut files = Vec::new();
        files_with_ext(std::path::Path::new(&dir), "uibinary", &mut files);
        let mut bad = Vec::new();
        for f in &files {
            let b = std::fs::read(f).unwrap();
            match roundtrip_check(&b) {
                Ok(true) => {}
                Ok(false) => bad.push(format!("{}: not byte-exact", f.display())),
                Err(e) => bad.push(format!("{}: {}", f.display(), e)),
            }
        }
        eprintln!("uibinary vanilla: {}/{} byte-exact", files.len() - bad.len(), files.len());
        assert!(bad.is_empty(), "{:?}", &bad[..bad.len().min(10)]);
    }

    /// Replays an in-game-proven mod (P2: item log 5 slots) as intents.
    /// `DMM_PARSER_UI_P2_DIR=<dir with van_* and p2_* itemlogview2 files>`.
    #[test]
    fn p2_replay() {
        let Ok(dir) = std::env::var("DMM_PARSER_UI_P2_DIR") else { return };
        let rd = |n: &str| std::fs::read(std::path::Path::new(&dir).join(n)).unwrap();
        let (ub, ht, cs) = (rd("van_itemlogview2_game.uibinary"), rd("van_itemlogview2.html"), rd("van_itemlogview2.css"));
        let ui: Vec<Value> = serde_json::from_str(P2_UI).unwrap();
        let css: Vec<Value> = serde_json::from_str(P2_CSS).unwrap();
        let (nb, nh, o1) = apply_ui_intents(&ub, Some(std::str::from_utf8(&ht).unwrap()), &ui).unwrap();
        let (nc, o2) = apply_ui_css_intents(std::str::from_utf8(&cs).unwrap(), &css);
        assert!(o1.iter().chain(o2.iter()).all(|o| matches!(o.status, ApplyStatus::Applied)));
        assert_eq!(nb, rd("p2_itemlogview2_game.uibinary"));
        assert_eq!(nh.unwrap().as_bytes(), &rd("p2_itemlogview2.html")[..]);
        assert_eq!(nc.as_bytes(), &rd("p2_itemlogview2.css")[..]);
    }

    pub(crate) const P2_UI: &str = r##"[
        {"op": "clone_record", "entry": "#ItemLog2", "value": {"id": "ItemLog3", "add": ["item-log-4"], "remove": ["item-log-3"]}},
        {"op": "clone_record", "entry": "#ItemLog2", "value": {"id": "ItemLog4", "add": ["item-log-5"], "remove": ["item-log-3"], "after": "#ItemLog3"}}
    ]"##;
    pub(crate) const P2_CSS: &str = r##"[
        {"op": "add_rule", "value": ".item-log-wrap.item-log-4 { top: 210px; } ", "after": ".item-log-wrap.item-log-3"},
        {"op": "add_rule", "value": ".item-log-wrap.item-log-5 { top: 280px; } ", "after": ".item-log-wrap.item-log-4"},
        {"op": "set", "entry": "#ItemLogList", "field": "height", "value": "390px"}
    ]"##;
}
