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
        let inner = if presence != 0 {
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
        Ok(Self { inner, presence })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(g) => {
                (if self.presence == 0 { 1u8 } else { self.presence }).write_to(w)?;
                g.tree.write_to(w)?;
                g.tail_a.write_to(w)?;
                g.tail_b.write_to(w)?;
                g.tail_c.write_to(w)?;
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
