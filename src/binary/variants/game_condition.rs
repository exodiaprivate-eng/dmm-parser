//! Recursive GameCondition expression tree.
//!
//! Per the meta-dispatcher (sub_141E65330) decompiled from CrimsonDesert.exe,
//! the tree has 9 case tags. Cases 0/1/2 are recursive operator nodes;
//! cases 3-8 are leaf nodes (cases 3 and 4 are themselves polymorphic families,
//! case 7 dispatches into GameExpression + IVariantItem sub-families).
//!
//! Stream layout per node: [u8 case_tag][case_specific_payload].
//! The tree is depth-first: cases 0/1 recurse twice (left, right), case 2 once.

use super::branch_condition_data::BranchConditionData;
use super::condition_data::ConditionData;
use super::condition_data_stage_chart::ConditionDataStageChart;
use super::condition_gimmick_data::ConditionGimmickData;
use super::global_effect_condition_data::GlobalEffectConditionData;
use super::schedule_complete_condition_data::ScheduleCompleteConditionData;
use crate::binary::*;
use std::io::{self, Write};

/// Full GameCondition wire format: a recursive tree + a 3-byte footer.
///
/// Per IDA `sub_101021408` (the reader called from ConditionInfo's parser):
///   1. Construct the tree node and read it from the stream.
///   2. Read three u8 fields at struct offsets +8, +9, +10.
///
/// The footer applies to every table that consumes GameCondition (not just
/// ConditionInfo). Wrapping the tree + footer into a single type keeps the
/// recursive `GameConditionNode` clean of the table-level trailing bytes.
#[derive(Debug)]
pub struct GameCondition<'a> {
    pub tree: GameConditionNode<'a>,
    pub tail_a: u8,
    pub tail_b: u8,
    pub tail_c: u8,
}

impl<'a> GameCondition<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let tree = GameConditionNode::read_from(data, offset)?;
        let tail_a = u8::read_from(data, offset)?;
        let tail_b = u8::read_from(data, offset)?;
        let tail_c = u8::read_from(data, offset)?;
        Ok(Self { tree, tail_a, tail_b, tail_c })
    }
    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.tree.write_to(w)?;
        self.tail_a.write_to(w)?;
        self.tail_b.write_to(w)?;
        self.tail_c.write_to(w)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum GameConditionNode<'a> {
    /// Case 0: BinaryOp_A (operator constructor 0x141E65450, likely AND).
    BinaryOpA {
        left: Box<GameConditionNode<'a>>,
        right: Box<GameConditionNode<'a>>,
    },
    /// Case 1: BinaryOp_B (operator constructor 0x141E65B30, likely OR).
    BinaryOpB {
        left: Box<GameConditionNode<'a>>,
        right: Box<GameConditionNode<'a>>,
    },
    /// Case 2: UnaryOp (operator constructor 0x141E660F0, likely NOT).
    UnaryOp { child: Box<GameConditionNode<'a>> },
    /// Case 3: leaf ConditionData (405 variants).
    ConditionData(ConditionData<'a>),
    /// Case 4: leaf BranchConditionData (14 variants).
    BranchConditionData(BranchConditionData<'a>),
    /// Case 5: leaf ScheduleCompleteConditionData_CheckDeadOrRetreat.
    ScheduleCompleteConditionData(ScheduleCompleteConditionData<'a>),
    /// Case 6: leaf ConditionGimmickData_CheckProperty.
    ConditionGimmickData(ConditionGimmickData),
    /// Case 7: ConditionData_StageChart / ConditionData_StageChart_Event.
    StageChart(ConditionDataStageChart<'a>),
    /// Case 8: leaf GlobalEffectConditionData.
    GlobalEffectConditionData(GlobalEffectConditionData),
}

impl<'a> GameConditionNode<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let case_tag = u8::read_from(data, offset)?;
        match case_tag {
            0 => {
                let left = Box::new(Self::read_from(data, offset)?);
                let right = Box::new(Self::read_from(data, offset)?);
                Ok(Self::BinaryOpA { left, right })
            }
            1 => {
                let left = Box::new(Self::read_from(data, offset)?);
                let right = Box::new(Self::read_from(data, offset)?);
                Ok(Self::BinaryOpB { left, right })
            }
            2 => {
                let child = Box::new(Self::read_from(data, offset)?);
                Ok(Self::UnaryOp { child })
            }
            3 => Ok(Self::ConditionData(ConditionData::read_from(data, offset)?)),
            4 => Ok(Self::BranchConditionData(BranchConditionData::read_from(data, offset)?)),
            5 => Ok(Self::ScheduleCompleteConditionData(
                ScheduleCompleteConditionData::read_from(data, offset)?,
            )),
            6 => Ok(Self::ConditionGimmickData(ConditionGimmickData::read_from(data, offset)?)),
            7 => Ok(Self::StageChart(ConditionDataStageChart::read_from(data, offset)?)),
            8 => Ok(Self::GlobalEffectConditionData(GlobalEffectConditionData::read_from(data, offset)?)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown GameCondition case_tag: {}", other),
            )),
        }
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::BinaryOpA { left, right } => {
                0u8.write_to(w)?;
                left.write_to(w)?;
                right.write_to(w)
            }
            Self::BinaryOpB { left, right } => {
                1u8.write_to(w)?;
                left.write_to(w)?;
                right.write_to(w)
            }
            Self::UnaryOp { child } => {
                2u8.write_to(w)?;
                child.write_to(w)
            }
            Self::ConditionData(c) => {
                3u8.write_to(w)?;
                c.write_to(w)
            }
            Self::BranchConditionData(b) => {
                4u8.write_to(w)?;
                b.write_to(w)
            }
            Self::ScheduleCompleteConditionData(s) => {
                5u8.write_to(w)?;
                s.write_to(w)
            }
            Self::ConditionGimmickData(g) => {
                6u8.write_to(w)?;
                g.write_to(w)
            }
            Self::StageChart(s) => {
                7u8.write_to(w)?;
                s.write_to(w)
            }
            Self::GlobalEffectConditionData(g) => {
                8u8.write_to(w)?;
                g.write_to(w)
            }
        }
    }
}
