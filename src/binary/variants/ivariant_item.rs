//! pa::IVariantItem polymorphic reader (per sub_141DA38D0).
//! Used by ConditionData_StageChart_Event (case 7 branch B of GameCondition tree).
//!
//! The selector tag is read by the OUTER caller (StageChart_Event) and passed
//! in. Tag determines the byte layout (semantic field names from the IDA
//! decompile of sub_141DA38D0):
//!
//!   0,2,3,4,5,14,15,16,17,18 → empty (no payload bytes)
//!   1                         → CString (staticstringA)
//!   6, 11                     → CString (staticstringA)
//!   7                         → CString + CString (staticstringA × 2)
//!   8                         → u32 + u32 + u32 (uint32, InteractionKey, HashCode32)
//!   9                         → u32 + u32 (uint32, InteractionKey)
//!   10                        → CString + CString (staticstringA × 2)
//!   12                        → u32 + u8 (uint32, StageBranchType)
//!   13                        → u32 (uint32)
//!   19                        → u32 (HashCode32)
//!   default                   → error
//!
//! Variants are split per-tag-with-distinct-semantics so JSON consumers see
//! semantic field names (e.g. `interaction_key` vs `hash_code`) instead of
//! positional `0`/`1`/`2`. Wire layout is unchanged — round-trip preserved.

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub enum IVariantItemPayload<'a> {
    /// Tags 0, 2, 3, 4, 5, 14, 15, 16, 17, 18 — no payload bytes.
    Empty,
    /// Tags 1, 6, 11 — single staticstringA.
    StaticString { value: CString<'a> },
    /// Tag 7 — pair of staticstringA.
    StaticStringPair {
        first: CString<'a>,
        second: CString<'a>,
    },
    /// Tag 10 — pair of staticstringA (separate from tag 7 since the
    /// caller's semantic differs even though the wire layout matches).
    StaticStringPairAlt {
        first: CString<'a>,
        second: CString<'a>,
    },
    /// Tag 8 — uint32 value + InteractionKey + HashCode32.
    InteractionWithHash {
        value: u32,
        interaction_key: u32,
        hash_code: u32,
    },
    /// Tag 9 — uint32 value + InteractionKey.
    Interaction { value: u32, interaction_key: u32 },
    /// Tag 12 — uint32 value + StageBranchType (u8 enum).
    StageBranch { value: u32, branch_type: u8 },
    /// Tag 13 — bare uint32 numeric value.
    Uint32 { value: u32 },
    /// Tag 19 — bare HashCode32.
    HashCode { hash_code: u32 },
}

#[derive(Debug)]
pub struct IVariantItem<'a> {
    pub tag: u8,
    pub payload: IVariantItemPayload<'a>,
}

impl<'a> IVariantItem<'a> {
    pub fn read_from_with_tag(
        data: &'a [u8],
        offset: &mut usize,
        tag: u8,
    ) -> io::Result<Self> {
        let payload = match tag {
            0 | 2 | 3 | 4 | 5 | 14 | 15 | 16 | 17 | 18 => IVariantItemPayload::Empty,
            1 | 6 | 11 => IVariantItemPayload::StaticString {
                value: CString::read_from(data, offset)?,
            },
            7 => IVariantItemPayload::StaticStringPair {
                first: CString::read_from(data, offset)?,
                second: CString::read_from(data, offset)?,
            },
            10 => IVariantItemPayload::StaticStringPairAlt {
                first: CString::read_from(data, offset)?,
                second: CString::read_from(data, offset)?,
            },
            8 => IVariantItemPayload::InteractionWithHash {
                value: u32::read_from(data, offset)?,
                interaction_key: u32::read_from(data, offset)?,
                hash_code: u32::read_from(data, offset)?,
            },
            9 => IVariantItemPayload::Interaction {
                value: u32::read_from(data, offset)?,
                interaction_key: u32::read_from(data, offset)?,
            },
            12 => IVariantItemPayload::StageBranch {
                value: u32::read_from(data, offset)?,
                branch_type: u8::read_from(data, offset)?,
            },
            13 => IVariantItemPayload::Uint32 {
                value: u32::read_from(data, offset)?,
            },
            19 => IVariantItemPayload::HashCode {
                hash_code: u32::read_from(data, offset)?,
            },
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown IVariantItem tag: {}", other),
                ))
            }
        };
        Ok(Self { tag, payload })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.payload {
            IVariantItemPayload::Empty => Ok(()),
            IVariantItemPayload::StaticString { value } => value.write_to(w),
            IVariantItemPayload::StaticStringPair { first, second }
            | IVariantItemPayload::StaticStringPairAlt { first, second } => {
                first.write_to(w)?;
                second.write_to(w)
            }
            IVariantItemPayload::InteractionWithHash {
                value,
                interaction_key,
                hash_code,
            } => {
                value.write_to(w)?;
                interaction_key.write_to(w)?;
                hash_code.write_to(w)
            }
            IVariantItemPayload::Interaction {
                value,
                interaction_key,
            } => {
                value.write_to(w)?;
                interaction_key.write_to(w)
            }
            IVariantItemPayload::StageBranch { value, branch_type } => {
                value.write_to(w)?;
                branch_type.write_to(w)
            }
            IVariantItemPayload::Uint32 { value } => value.write_to(w),
            IVariantItemPayload::HashCode { hash_code } => hash_code.write_to(w),
        }
    }
}
