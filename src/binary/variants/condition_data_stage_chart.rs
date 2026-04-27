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
}
