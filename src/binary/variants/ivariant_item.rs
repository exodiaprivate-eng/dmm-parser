//! pa::IVariantItem polymorphic reader (per sub_141DA38D0).
//! Used by ConditionData_StageChart_Event (case 7 branch B of GameCondition tree).
//!
//! The selector tag is read by the OUTER caller (StageChart_Event) and passed
//! in. Tag determines the byte layout:
//!
//!   0,2,3,4,5,14,15,16,17,18 → empty (no payload bytes)
//!   1                         → CString (typed staticstringA)
//!   6, 11                     → CString (typed staticstringA)
//!   7                         → CString + CString
//!   8                         → u32 + u32 + u32 (uint32 + InteractionKey + HashCode32)
//!   9                         → u32 + u32 (uint32 + InteractionKey)
//!   10                        → CString + CString (staticstringA × 2)
//!   12                        → u32 + u8 (uint32 + StageBranchType)
//!   13                        → u32 (uint32)
//!   19                        → u32 (HashCode32)
//!   default                   → error

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub enum IVariantItemPayload<'a> {
    Empty,
    OneCString(CString<'a>),
    TwoCString(CString<'a>, CString<'a>),
    ThreeU32(u32, u32, u32),
    TwoU32(u32, u32),
    U32U8(u32, u8),
    OneU32(u32),
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
            1 | 6 | 11 => IVariantItemPayload::OneCString(CString::read_from(data, offset)?),
            7 | 10 => {
                let a = CString::read_from(data, offset)?;
                let b = CString::read_from(data, offset)?;
                IVariantItemPayload::TwoCString(a, b)
            }
            8 => {
                let a = u32::read_from(data, offset)?;
                let b = u32::read_from(data, offset)?;
                let c = u32::read_from(data, offset)?;
                IVariantItemPayload::ThreeU32(a, b, c)
            }
            9 => {
                let a = u32::read_from(data, offset)?;
                let b = u32::read_from(data, offset)?;
                IVariantItemPayload::TwoU32(a, b)
            }
            12 => {
                let a = u32::read_from(data, offset)?;
                let b = u8::read_from(data, offset)?;
                IVariantItemPayload::U32U8(a, b)
            }
            13 | 19 => IVariantItemPayload::OneU32(u32::read_from(data, offset)?),
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
            IVariantItemPayload::OneCString(s) => s.write_to(w),
            IVariantItemPayload::TwoCString(a, b) => {
                a.write_to(w)?;
                b.write_to(w)
            }
            IVariantItemPayload::ThreeU32(a, b, c) => {
                a.write_to(w)?;
                b.write_to(w)?;
                c.write_to(w)
            }
            IVariantItemPayload::TwoU32(a, b) => {
                a.write_to(w)?;
                b.write_to(w)
            }
            IVariantItemPayload::U32U8(a, b) => {
                a.write_to(w)?;
                b.write_to(w)
            }
            IVariantItemPayload::OneU32(a) => a.write_to(w),
        }
    }
}
