// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Stream-mode wrapper around `GameConditionNode` (lane A's recursive
//! tree decoder) for tables that embed `sub_141103B30` — i.e., a
//! `u8 presence + (if presence: GameConditionNode + 3 footer bytes)`
//! wire pattern.
//!
//! Lane A's `GameCondition::read_from` uses a slice-bounded probe
//! (checks `probe == data.len()`) to fall back to `Raw` capture for
//! the 0.2% of ConditionInfo entries with anti-disassembly tags. That
//! API can't compose stream-style. This wrapper instead calls
//! `GameConditionNode::read_from` directly — every leaf variant is
//! self-delimiting via tag dispatch, so it parses correctly inside a
//! larger struct. The trade-off: if a vanilla entry hits one of the
//! anti-disassembly tags (54/286), parsing fails here. The roundtrip
//! tests on consumer tables surface that immediately if it ever
//! happens.

use crate::binary::variants::game_condition::GameConditionNode;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct OptionalGameCondition<'a> {
    pub inner: Option<GameConditionWrapper<'a>>,
    /// The presence byte as read. NOT a pure bool: 2.01.00 interactioninfo entry 307
    /// (key 0xf4322) carries 3 in `cond_b`; the engine only tests non-zero. Written
    /// back verbatim so round-trips are byte-exact. JSON carries it as `_presence`
    /// only when it is not the plain 1, so every existing mod file is unchanged.
    pub presence: u8,
    /// 2.04.00: the game's condition reader (Mac sub_101A31680) reads its OWN presence byte
    /// before the tree, so a COptional<GameCondition> (sub_101A5FB5C) is two presence bytes.
    /// 1 in vanilla whenever the outer one is set; JSON carries `_inner_presence` only when
    /// it is not 1 (0 = the condition is "present" but empty).
    pub inner_presence: u8,
}

#[derive(Debug)]
pub struct GameConditionWrapper<'a> {
    pub tree: GameConditionNode<'a>,
    pub tail_a: u8,
    pub tail_b: u8,
    pub tail_c: u8,
}

impl<'a> OptionalGameCondition<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let _ogc_start = *offset;
        let presence = u8::read_from(data, offset)?;
        let inner_presence = if presence != 0 { u8::read_from(data, offset)? } else { 0 };
        let inner = if presence != 0 && inner_presence != 0 {
            let tree = GameConditionNode::read_from(data, offset)?;
            let _ogc_after_tree = *offset;
            let tail_a = u8::read_from(data, offset)?;
            let tail_b = u8::read_from(data, offset)?;
            let tail_c = u8::read_from(data, offset)?;
            if std::env::var_os("OGCTRACE").is_some() {
                eprintln!(
                    "OGC start={} presence={} tree_end={} end={} tail={},{},{}",
                    _ogc_start, presence, _ogc_after_tree, *offset, tail_a, tail_b, tail_c
                );
            }
            Some(GameConditionWrapper { tree, tail_a, tail_b, tail_c })
        } else {
            if std::env::var_os("OGCTRACE").is_some() {
                eprintln!("OGC start={} presence=0 end={}", _ogc_start, *offset);
            }
            None
        };
        Ok(Self { inner, presence, inner_presence })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(g) => {
                (if self.presence == 0 { 1u8 } else { self.presence }).write_to(w)?;
                (if self.inner_presence == 0 { 1u8 } else { self.inner_presence }).write_to(w)?;
                g.tree.write_to(w)?;
                g.tail_a.write_to(w)?;
                g.tail_b.write_to(w)?;
                g.tail_c.write_to(w)?;
            }
            None if self.presence != 0 => {
                self.presence.write_to(w)?;
                0u8.write_to(w)?;
            }
            None => {
                0u8.write_to(w)?;
            }
        }
        Ok(())
    }

    pub fn to_json_value(&self) -> Value {
        match &self.inner {
            Some(g) => {
                let mut m = Map::new();
                m.insert("tree".to_string(), g.tree.to_json_value());
                m.insert("tail_a".to_string(), g.tail_a.to_json_value());
                m.insert("tail_b".to_string(), g.tail_b.to_json_value());
                m.insert("tail_c".to_string(), g.tail_c.to_json_value());
                if self.presence != 1 {
                    m.insert("_presence".to_string(), Value::from(self.presence));
                }
                if self.inner_presence != 1 {
                    m.insert("_inner_presence".to_string(), Value::from(self.inner_presence));
                }
                Value::Object(m)
            }
            None if self.presence != 0 => {
                let mut m = Map::new();
                m.insert("_inner_presence".to_string(), Value::from(0u8));
                if self.presence != 1 {
                    m.insert("_presence".to_string(), Value::from(self.presence));
                }
                Value::Object(m)
            }
            None => Value::Null,
        }
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            w.push(0);
            return Ok(());
        }
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "OptionalGameCondition: expected object or null",
        ))?;
        let presence = obj.get("_presence").and_then(|x| x.as_u64()).unwrap_or(1) as u8;
        w.push(if presence == 0 { 1 } else { presence });
        let inner_presence = obj.get("_inner_presence").and_then(|x| x.as_u64()).unwrap_or(1) as u8;
        w.push(inner_presence);
        if inner_presence == 0 {
            return Ok(());
        }
        GameConditionNode::write_from_json(w, json_get_field(obj, "tree")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "tail_a")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "tail_b")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "tail_c")?)?;
        Ok(())
    }
}

/// Same as OptionalGameCondition but WITHOUT the 3-byte tail.
/// Used by InventoryMoveData where the condition has no footer.
#[derive(Debug)]
pub struct OptionalGameConditionNoTail<'a> {
    pub inner: Option<GameConditionNode<'a>>,
}

impl<'a> OptionalGameConditionNoTail<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let presence = u8::read_from(data, offset)?;
        let inner = if presence != 0 {
            Some(GameConditionNode::read_from(data, offset)?)
        } else {
            None
        };
        Ok(Self { inner })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(tree) => {
                1u8.write_to(w)?;
                tree.write_to(w)?;
            }
            None => {
                0u8.write_to(w)?;
            }
        }
        Ok(())
    }

    pub fn to_json_value(&self) -> Value {
        match &self.inner {
            Some(tree) => {
                let mut m = Map::new();
                m.insert("tree".to_string(), tree.to_json_value());
                Value::Object(m)
            }
            None => Value::Null,
        }
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            w.push(0);
            return Ok(());
        }
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "OptionalGameConditionNoTail: expected object or null",
        ))?;
        w.push(1);
        GameConditionNode::write_from_json(w, json_get_field(obj, "tree")?)?;
        Ok(())
    }
}

// ── Trait impls so the type can sit inside a `py_binary_struct!` ──────────────
//
// The inherent read_from/write_to already match the trait signatures exactly;
// these delegate so a macro-generated struct can hold an OptionalGameCondition
// as a plain field. Added for 2.00.00's `GimmickLevelAliasData`, whose
// `_activeExpression` is one of these.
//
// `read_tracked` records the whole condition as ONE field range rather than
// walking into the tree. The tracked reader exists to give field-path offsets
// for mods, and no mod addresses a node inside a condition tree — it edits the
// condition wholesale, so a single range is the useful granularity.
impl<'a> crate::binary::BinaryRead<'a> for OptionalGameCondition<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        OptionalGameCondition::read_from(data, offset)
    }
}

impl crate::binary::BinaryWrite for OptionalGameCondition<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        OptionalGameCondition::write_to(self, w)
    }
}

impl<'a> crate::binary::BinaryReadTracked<'a> for OptionalGameCondition<'a> {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<crate::binary::FieldRange>,
    ) -> io::Result<Self> {
        let start = *offset;
        let v = OptionalGameCondition::read_from(data, offset)?;
        ranges.push(crate::binary::FieldRange {
            path: path.clone(),
            start,
            end: *offset,
            ty: "OptionalGameCondition",
        });
        Ok(v)
    }
}

// JSON is the supported path for condition trees (same as FilterCondition and
// the other variant families) — the Python surface would have to mirror the
// whole recursive node type for no benefit.
impl crate::json_traits::ToJsonValue for OptionalGameCondition<'_> {
    fn to_json_value(&self) -> Value {
        OptionalGameCondition::to_json_value(self)
    }
}

impl crate::json_traits::WriteJsonValue for OptionalGameCondition<'_> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        OptionalGameCondition::write_from_json(w, v)
    }
}

impl crate::python_traits::ToPyValue for OptionalGameCondition<'_> {
    fn to_py_value(&self, _py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "OptionalGameCondition: use JSON path"))
    }
}

impl crate::python_traits::WritePyValue for OptionalGameCondition<'_> {
    fn write_from_py(_w: &mut Vec<u8>, _obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "OptionalGameCondition: use JSON path"))
    }
}

/// Game 2.06.00: a field that held a ConditionInfo KEY (u32) holds the condition INLINE
/// instead: an [`OptionalGameCondition`] on the wire (Mac 2.06 reader sub_10191A474: u8 presence,
/// then sub_1019EAE58: u8 presence + condition tree (sub_101D4472C) + 3 u8). conditioninfo lost
/// ~2,500 records to this (10,831 -> 8,316). First seen in `DropInfoData` (`_ownerCondition`,
/// `_playerCondition`, `_gimmickCachedTargetCondition`, formerly `...ConditionInfo`).
///
/// Owned (the wire bytes), so the structs that embed it keep no lifetime. JSON is the
/// structured [`OptionalGameCondition`] form; `null` = no condition. A legacy number from a
/// pre-2.06 mod is accepted only when it meant "none" (0 or 0xFFFFFFFF): a real key no longer
/// names anything the game can read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineCondition {
    pub bytes: Vec<u8>,
}

impl Default for InlineCondition {
    fn default() -> Self {
        Self { bytes: vec![0] }
    }
}

impl InlineCondition {
    pub fn is_none(&self) -> bool {
        self.bytes == [0]
    }
}

impl<'a> crate::binary::BinaryRead<'a> for InlineCondition {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let start = *offset;
        OptionalGameCondition::read_from(data, offset)?;
        Ok(Self { bytes: data[start..*offset].to_vec() })
    }
}

impl crate::binary::BinaryWrite for InlineCondition {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.bytes)
    }
}

impl<'a> crate::binary::BinaryReadTracked<'a> for InlineCondition {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<crate::binary::FieldRange>,
    ) -> io::Result<Self> {
        let start = *offset;
        let v = <Self as crate::binary::BinaryRead>::read_from(data, offset)?;
        ranges.push(crate::binary::FieldRange { path: path.clone(), start, end: *offset, ty: "InlineCondition" });
        Ok(v)
    }
}

/// Turns a pre-2.06 ConditionInfo KEY (a number a 2.05-era mod still carries) into the inline
/// condition 2.06 writes in its place. Global and replaceable: DMM installs one per mount
/// (built from the live conditioninfo plus the removed-key map), `None` clears it. With no
/// resolver, or no answer for a key, the JSON reader keeps refusing the number.
pub type LegacyConditionResolver = std::sync::Arc<dyn Fn(u32) -> Option<InlineCondition> + Send + Sync>;

static LEGACY_CONDITION_RESOLVER: std::sync::RwLock<Option<LegacyConditionResolver>> =
    std::sync::RwLock::new(None);

pub fn set_legacy_condition_resolver(r: Option<LegacyConditionResolver>) {
    *LEGACY_CONDITION_RESOLVER.write().unwrap_or_else(|e| e.into_inner()) = r;
}

pub fn resolve_legacy_condition(key: u32) -> Option<InlineCondition> {
    let r = LEGACY_CONDITION_RESOLVER.read().unwrap_or_else(|e| e.into_inner()).clone();
    r.and_then(|f| f(key))
}

impl InlineCondition {
    /// The inline condition 2.06 writes in place of a conditioninfo KEY, from that conditioninfo
    /// record (LEGACY layout, i.e. after `named_index::to_legacy`: name, key, is_blocked,
    /// parser_type, has_game_condition, tree + 3 tail bytes, original_string).
    ///
    /// Rule: `0x01` ++ the record's bytes from `has_game_condition` up to `original_string`; a
    /// `CheckNone()` record is "no condition" (`0x00`). Confirmed on 2.06.00 vanilla: every key
    /// 2.06 still has that 2.05.01 used in an inlined field (46 + CheckNone) matches exactly, and
    /// the same rule on 2.05.01's records reproduces 1,271 of the 1,286 removed keys vanilla
    /// inlined (the 15 others are conditions Pearl Abyss edited in 2.06).
    pub fn from_conditioninfo_record(rec: &[u8]) -> io::Result<Self> {
        let mut o = 0;
        let ci = crate::tables::condition_info::ConditionInfo::read_with_size(rec, &mut o, rec.len())?;
        if ci.string_key.data.starts_with("CheckNone(") {
            return Ok(Self::default());
        }
        let start = 4 + ci.string_key.length as usize + 4 + 2;
        let end = rec.len() - 4 - ci.original_string.length as usize;
        let mut bytes = Vec::with_capacity(end - start + 1);
        bytes.push(1);
        bytes.extend_from_slice(&rec[start..end]);
        Ok(Self { bytes })
    }
}

/// A resolver for [`set_legacy_condition_resolver`]: a key the live (legacy-layout) conditioninfo
/// still has resolves by [`InlineCondition::from_conditioninfo_record`]; a key 2.06 removed
/// resolves from `removed` (the patch-day map, `tools/patchday/condition_map_206_from_old.py`).
pub fn resolver_from_conditioninfo(
    pabgh: &[u8],
    pabgb: &[u8],
    removed: std::collections::HashMap<u32, InlineCondition>,
) -> io::Result<LegacyConditionResolver> {
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets_from_bytes};
    let entries = load_pabgh_offsets_from_bytes(pabgh)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "conditioninfo: unreadable index"))?;
    let mut live = std::collections::HashMap::with_capacity(entries.len());
    for (k, s, e) in entry_ranges(&entries, pabgb.len()) {
        if let Ok(c) = InlineCondition::from_conditioninfo_record(&pabgb[s..e]) {
            live.insert(k, c);
        }
    }
    let map = std::sync::Arc::new((live, removed));
    Ok(std::sync::Arc::new(move |k: u32| map.0.get(&k).or_else(|| map.1.get(&k)).cloned()))
}

/// Parse the map file `condition_map_206_from_old.py` writes (`{"map": {"<key>": "hex:.."}}`).
pub fn removed_condition_map_from_json(v: &Value) -> io::Result<std::collections::HashMap<u32, InlineCondition>> {
    let m = v.get("map").and_then(|m| m.as_object())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "condition map: no \"map\" object"))?;
    let mut out = std::collections::HashMap::with_capacity(m.len());
    for (k, h) in m {
        let key: u32 = k.parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("condition map key {k}")))?;
        let mut w = Vec::new();
        <InlineCondition as WriteJsonValue>::write_from_json(&mut w, h)?;
        out.insert(key, InlineCondition { bytes: w });
    }
    Ok(out)
}

impl crate::json_traits::ToJsonValue for InlineCondition {
    fn to_json_value(&self) -> Value {
        let mut off = 0;
        match OptionalGameCondition::read_from(&self.bytes, &mut off) {
            Ok(c) if off == self.bytes.len() => c.to_json_value(),
            _ => Value::String(format!("hex:{}", hex_string(&self.bytes))),
        }
    }
}

fn hex_string(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

impl crate::json_traits::WriteJsonValue for InlineCondition {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        match v {
            Value::Number(n) => match n.as_u64() {
                Some(0) | Some(0xFFFF_FFFF) => {
                    w.push(0);
                    Ok(())
                }
                Some(k) if k <= u32::MAX as u64 => match resolve_legacy_condition(k as u32) {
                    Some(c) => {
                        w.extend_from_slice(&c.bytes);
                        Ok(())
                    }
                    None => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "condition {n}: a ConditionInfo key from before game 2.06 that no longer \
                             resolves; since 2.06 this condition is written inline (an object, or null \
                             for none)"
                        ),
                    )),
                },
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("condition {n}: not a ConditionInfo key"),
                )),
            },
            Value::String(s) if s.starts_with("hex:") => {
                let h = &s[4..];
                if h.len() % 2 != 0 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "InlineCondition: odd hex"));
                }
                for i in (0..h.len()).step_by(2) {
                    w.push(u8::from_str_radix(&h[i..i + 2], 16).map_err(|e| {
                        io::Error::new(io::ErrorKind::InvalidData, format!("InlineCondition hex: {e}"))
                    })?);
                }
                Ok(())
            }
            _ => OptionalGameCondition::write_from_json(w, v),
        }
    }
}

impl crate::python_traits::ToPyValue for InlineCondition {
    fn to_py_value(&self, _py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err("InlineCondition: use JSON path"))
    }
}

impl crate::python_traits::WritePyValue for InlineCondition {
    fn write_from_py(_w: &mut Vec<u8>, _obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err("InlineCondition: use JSON path"))
    }
}

#[cfg(test)]
mod dump_206 {
    //! Patch-day helper for the 2.06 condition map (tools/patchday/condition_map_206.py).
    //! `DMM_DUMP_IN` = a 2.06 LEGACY fixture dir, `DMM_DUMP_OUT` = output dir. Writes
    //! `<table>.json` (records, every inline condition as `"hex:.."`) for DMM_DUMP_TABLES
    //! (comma list), and `conditioninfo_trees.json` (key -> has_game_condition, tree hex).
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets_from_bytes};

    fn is_condition_object(m: &Map<String, Value>) -> bool {
        (m.contains_key("tree") && m.contains_key("tail_a")) || (m.len() <= 2 && m.contains_key("_inner_presence"))
    }

    fn hexify(v: &mut Value) {
        match v {
            Value::Object(m) if is_condition_object(m) => {
                let mut w = Vec::new();
                if OptionalGameCondition::write_from_json(&mut w, v).is_ok() {
                    *v = Value::String(format!("hex:{}", hex_string(&w)));
                }
            }
            Value::Object(m) => m.values_mut().for_each(hexify),
            Value::Array(a) => a.iter_mut().for_each(hexify),
            _ => {}
        }
    }

    #[test]
    #[ignore]
    fn dump() {
        let (Ok(inp), Ok(out)) = (std::env::var("DMM_DUMP_IN"), std::env::var("DMM_DUMP_OUT")) else {
            eprintln!("set DMM_DUMP_IN / DMM_DUMP_OUT");
            return;
        };
        std::fs::create_dir_all(&out).unwrap();
        let tables = std::env::var("DMM_DUMP_TABLES").unwrap_or_default();
        for t in tables.split(',').filter(|t| !t.is_empty()) {
            let b = std::fs::read(format!("{inp}/{t}.pabgb")).unwrap();
            let h = std::fs::read(format!("{inp}/{t}.pabgh")).unwrap();
            match crate::dispatch::parse_table_to_json(t, &b, Some(&h)) {
                Ok(mut recs) => {
                    recs.iter_mut().for_each(hexify);
                    std::fs::write(format!("{out}/{t}.json"), serde_json::to_vec(&recs).unwrap()).unwrap();
                    eprintln!("{t}: {} records", recs.len());
                }
                Err(e) => eprintln!("{t}: FAILED {e}"),
            }
        }
        // conditioninfo trees
        let b = std::fs::read(format!("{inp}/conditioninfo.pabgb")).unwrap();
        let h = std::fs::read(format!("{inp}/conditioninfo.pabgh")).unwrap();
        let entries = load_pabgh_offsets_from_bytes(&h).unwrap();
        let mut m = Map::new();
        for (k, s, e) in entry_ranges(&entries, b.len()) {
            let mut o = s;
            let Ok(ci) = crate::tables::condition_info::ConditionInfo::read_with_size(&b, &mut o, e - s) else {
                continue;
            };
            let tree_start = s + 4 + ci.string_key.length as usize + 4 + 3;
            let tree_end = e - 4 - ci.original_string.length as usize;
            let mut r = Map::new();
            r.insert("has".into(), Value::from(ci.has_game_condition));
            r.insert("parser_type".into(), Value::from(ci.parser_type));
            r.insert("tree".into(), Value::String(hex_string(&b[tree_start..tree_end])));
            r.insert("name".into(), Value::String(ci.string_key.data.to_string()));
            m.insert(k.to_string(), Value::Object(r));
        }
        eprintln!("conditioninfo: {} records", m.len());
        std::fs::write(format!("{out}/conditioninfo_trees.json"), serde_json::to_vec(&Value::Object(m)).unwrap()).unwrap();
    }
}

#[cfg(test)]
mod legacy_resolver_206 {
    use super::*;

    /// `DMM_COND_MAP` = condition_map_206_full.json. Builds the resolver from the 2.06 legacy
    /// conditioninfo + the map and checks keys from the 5.3.0 mount failures.
    #[test]
    #[ignore]
    fn resolves_live_removed_and_checknone() {
        let Ok(mp) = std::env::var("DMM_COND_MAP") else { eprintln!("set DMM_COND_MAP"); return; };
        let h = std::fs::read(crate::testenv::resolve("conditioninfo.pabgh")).unwrap();
        let b = std::fs::read(crate::testenv::resolve("conditioninfo.pabgb")).unwrap();
        let removed = removed_condition_map_from_json(&serde_json::from_slice(&std::fs::read(mp).unwrap()).unwrap()).unwrap();
        let r = resolver_from_conditioninfo(&h, &b, removed).unwrap();
        // live keys, values as 2.06 vanilla inlines them
        assert_eq!(hex_string(&r(1000317).unwrap().bytes), "01010203a301020000000001");
        assert_eq!(hex_string(&r(1000534).unwrap().bytes), "01010373010000000001");
        assert_eq!(r(1000531).unwrap().bytes, vec![0]); // CheckNone() = no condition
        assert!(r(1000083).is_some(), "a key 2.06 removed resolves from the map");
        // and the JSON reader takes a legacy number once the resolver is installed
        set_legacy_condition_resolver(Some(r));
        let mut w = Vec::new();
        <InlineCondition as WriteJsonValue>::write_from_json(&mut w, &Value::from(1000317u32)).unwrap();
        assert_eq!(hex_string(&w), "01010203a301020000000001");
        set_legacy_condition_resolver(None);
        assert!(<InlineCondition as WriteJsonValue>::write_from_json(&mut Vec::new(), &Value::from(1000317u32)).is_err());
    }
}
