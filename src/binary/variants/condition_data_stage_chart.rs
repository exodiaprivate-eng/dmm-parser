//! pa::ConditionData_StageChart / pa::ConditionData_StageChart_Event.
//! Case 7 of GameCondition tree (sub_141DAC600).
//!
//! Stream layout:
//!   [u8 outer_presence]
//!   if outer_presence != 0:  branch A (StageChart)
//!     [CString label][u8][u64][GameExpression body]
//!   else:                    branch B (StageChart_Event)
//!     [u8 ivariant_selector][IVariantItem payload per selector]

use super::game_expression::GameExpression;
use super::ivariant_item::IVariantItem;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub enum ConditionDataStageChart<'a> {
    /// outer_presence != 0
    BranchA {
        outer_presence: u8,
        label: CString<'a>,
        byte_b: u8,
        qword_c: u64,
        expression: GameExpression<'a>,
    },
    /// outer_presence == 0
    BranchB {
        ivariant_selector: u8,
        item: IVariantItem<'a>,
    },
}

impl<'a> ConditionDataStageChart<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let outer_presence = u8::read_from(data, offset)?;
        if outer_presence != 0 {
            let label = CString::read_from(data, offset)?;
            let byte_b = u8::read_from(data, offset)?;
            let qword_c = u64::read_from(data, offset)?;
            let expression = GameExpression::read_from(data, offset)?;
            Ok(Self::BranchA { outer_presence, label, byte_b, qword_c, expression })
        } else {
            let ivariant_selector = u8::read_from(data, offset)?;
            let item = IVariantItem::read_from_with_tag(data, offset, ivariant_selector)?;
            Ok(Self::BranchB { ivariant_selector, item })
        }
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::BranchA { outer_presence, label, byte_b, qword_c, expression } => {
                outer_presence.write_to(w)?;
                label.write_to(w)?;
                byte_b.write_to(w)?;
                qword_c.write_to(w)?;
                expression.write_to(w)
            }
            Self::BranchB { ivariant_selector, item } => {
                0u8.write_to(w)?;
                ivariant_selector.write_to(w)?;
                item.write_to(w)
            }
        }
    }

    /// JSON shape: discriminated union.
    /// - BranchA: {branch: "A", outer_presence, label, byte_b, qword_c,
    ///   expression_b64} — GameExpression body still rides as base64
    ///   pending its own per-variant JSON rollout.
    /// - BranchB: {branch: "B", ivariant_selector, item_b64}
    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        match self {
            Self::BranchA { outer_presence, label, byte_b, qword_c, expression } => {
                m.insert("branch".into(), Value::String("A".into()));
                m.insert("outer_presence".into(), outer_presence.to_json_value());
                m.insert("label".into(), label.to_json_value());
                m.insert("byte_b".into(), byte_b.to_json_value());
                m.insert("qword_c".into(), qword_c.to_json_value());
                let mut buf = Vec::new();
                expression.write_to(&mut buf).expect("write_to Vec");
                m.insert("expression_b64".into(), Value::String(B64.encode(&buf)));
            }
            Self::BranchB { ivariant_selector, item } => {
                m.insert("branch".into(), Value::String("B".into()));
                m.insert("ivariant_selector".into(), ivariant_selector.to_json_value());
                m.insert("item".into(), Value::Object(item.to_json_dict()));
            }
        }
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        let branch = json_get_field(obj, "branch")?.as_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData,
                "ConditionDataStageChart.branch: expected string")
        })?;
        match branch {
            "A" => {
                <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "outer_presence")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "label")?)?;
                <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "byte_b")?)?;
                <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "qword_c")?)?;
                let s = json_get_field(obj, "expression_b64")?.as_str().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData,
                        "expression_b64: expected base64 string")
                })?;
                let bytes = B64.decode(s).map_err(|e| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("expression_b64: invalid base64: {}", e),
                ))?;
                w.extend_from_slice(&bytes);
            }
            "B" => {
                w.push(0u8);  // outer_presence == 0 implies BranchB
                <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ivariant_selector")?)?;
                let item_obj = json_get_field(obj, "item")?.as_object().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData,
                        "StageChart.item: expected object")
                })?;
                IVariantItem::write_from_json_dict(w, item_obj)?;
            }
            other => return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("ConditionDataStageChart.branch: unknown {:?}", other))),
        }
        Ok(())
    }
}
