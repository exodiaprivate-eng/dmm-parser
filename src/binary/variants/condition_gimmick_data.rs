//! GameCondition tree case 6: pa::ConditionGimmickData_CheckProperty.
//!
//! Hand-written from IDA decompile of sub_141CB6480.
//! Stream layout: [u32 property_id]

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct ConditionGimmickData {
    pub property_id: u32,
}

impl ConditionGimmickData {
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let property_id = u32::read_from(data, offset)?;
        Ok(Self { property_id })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.property_id.write_to(w)
    }
}
