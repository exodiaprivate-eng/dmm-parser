//! GameCondition tree case 5: pa::ScheduleCompleteConditionData_CheckDeadOrRetreat.
//!
//! Hand-written from IDA decompile of sub_141D8B1A0.
//! Stream layout: [u8 presence_flag] then if presence_flag == 0:
//!   CString + u8 + u64 + u8 + u8

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct ScheduleCompletePayload<'a> {
    pub label: CString<'a>,
    pub byte_a: u8,
    pub qword_b: u64,
    pub byte_c: u8,
    pub byte_d: u8,
}

impl<'a> ScheduleCompletePayload<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let label = CString::read_from(data, offset)?;
        let byte_a = u8::read_from(data, offset)?;
        let qword_b = u64::read_from(data, offset)?;
        let byte_c = u8::read_from(data, offset)?;
        let byte_d = u8::read_from(data, offset)?;
        Ok(Self { label, byte_a, qword_b, byte_c, byte_d })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.label.write_to(w)?;
        self.byte_a.write_to(w)?;
        self.qword_b.write_to(w)?;
        self.byte_c.write_to(w)?;
        self.byte_d.write_to(w)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ScheduleCompleteConditionData<'a> {
    pub presence_flag: u8,
    pub payload: Option<ScheduleCompletePayload<'a>>,
}

impl<'a> ScheduleCompleteConditionData<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let presence_flag = u8::read_from(data, offset)?;
        let payload = if presence_flag == 0 {
            Some(ScheduleCompletePayload::read_from(data, offset)?)
        } else {
            None
        };
        Ok(Self { presence_flag, payload })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.presence_flag.write_to(w)?;
        if let Some(p) = &self.payload {
            p.write_to(w)?;
        }
        Ok(())
    }
}
