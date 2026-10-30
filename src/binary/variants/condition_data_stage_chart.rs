// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.


//! pa::ConditionData_StageChart / pa::ConditionData_StageChart_Event, 2.04.00.
//!
//! Before 2.04 both rode GameCondition node case 7 behind a presence byte. 2.04 gave each
//! its own node case (Mac node dispatch `sub_101D9C034`), with no presence byte:
//!
//!   case 7, StageChart (`sub_101C8DC8C`): CString label, u8, i64, GameExpression204.
//!   case 8, StageChart_Event (`sub_101C8E094`): u8 selector, IVariantItem payload for that
//!     selector (`sub_101C86764`), u32.
//!
//! JSON keeps the pre-2.04 `branch` key ("A" = case 7, "B" = case 8); the node writer picks
//! the case tag from it.

use super::game_expression_204::GameExpression204;
use super::ivariant_item::IVariantItem;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub enum ConditionDataStageChart<'a> {
    /// Node case 7.
    BranchA {
        label: CString<'a>,
        byte_b: u8,
        qword_c: u64,
        expression: GameExpression204<'a>,
    },
    /// Node case 8.
    BranchB {
        ivariant_selector: u8,
        item: IVariantItem<'a>,
        tail: u32,
    },
}

impl<'a> ConditionDataStageChart<'a> {
    /// Read the body of node case 7 or 8.
    pub fn read_from(data: &'a [u8], offset: &mut usize, case_tag: u8) -> io::Result<Self> {
        if case_tag == 7 {
            Ok(Self::BranchA {
                label: CString::read_from(data, offset)?,
                byte_b: u8::read_from(data, offset)?,
                qword_c: u64::read_from(data, offset)?,
                expression: GameExpression204::read_from(data, offset)?,
            })
        } else {
            let ivariant_selector = u8::read_from(data, offset)?;
            let item = IVariantItem::read_from_with_tag(data, offset, ivariant_selector)?;
            let tail = u32::read_from(data, offset)?;
            Ok(Self::BranchB { ivariant_selector, item, tail })
        }
    }

    /// The GameCondition node case this body belongs to.
    pub fn case_tag(&self) -> u8 {
        match self {
            Self::BranchA { .. } => 7,
            Self::BranchB { .. } => 8,
        }
    }

    /// Writes the body only; the node writes the case tag.
    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::BranchA { label, byte_b, qword_c, expression } => {
                label.write_to(w)?;
                byte_b.write_to(w)?;
                qword_c.write_to(w)?;
                expression.write_to(w)
            }
            Self::BranchB { ivariant_selector, item, tail } => {
                ivariant_selector.write_to(w)?;
                item.write_to(w)?;
                tail.write_to(w)
            }
        }
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        match self {
            Self::BranchA { label, byte_b, qword_c, expression } => {
                m.insert("branch".into(), Value::String("A".into()));
                m.insert("label".into(), label.to_json_value());
                m.insert("byte_b".into(), byte_b.to_json_value());
                m.insert("qword_c".into(), qword_c.to_json_value());
                m.insert("expression".into(), expression.to_json_value());
            }
            Self::BranchB { ivariant_selector, item, tail } => {
                m.insert("branch".into(), Value::String("B".into()));
                m.insert("ivariant_selector".into(), ivariant_selector.to_json_value());
                m.insert("item".into(), Value::Object(item.to_json_dict()));
                m.insert("tail".into(), tail.to_json_value());
            }
        }
        m
    }

    /// The node case tag a JSON body encodes (from its `branch`).
    pub fn case_tag_for_json(obj: &Map<String, Value>) -> io::Result<u8> {
        match json_get_field(obj, "branch")?.as_str() {
            Some("A") => Ok(7),
            Some("B") => Ok(8),
            other => Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("ConditionDataStageChart.branch: unknown {:?}", other))),
        }
    }

    /// Writes the body only; the node writes the case tag.
    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        if Self::case_tag_for_json(obj)? == 7 {
            <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "label")?)?;
            <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "byte_b")?)?;
            <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "qword_c")?)?;
            GameExpression204::write_from_json(w, json_get_field(obj, "expression")?)?;
        } else {
            <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ivariant_selector")?)?;
            let item_obj = json_get_field(obj, "item")?.as_object().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "StageChart.item: expected object")
            })?;
            IVariantItem::write_from_json_dict(w, item_obj)?;
            <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "tail")?)?;
        }
        Ok(())
    }
}
