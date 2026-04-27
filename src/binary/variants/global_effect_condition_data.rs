//! GameCondition tree case 8: pa::GlobalEffectConditionData.
//!
//! Hand-written from IDA decompile of sub_14114FF40.
//! Stream layout: [u32 effect_id][u8 byte_a (default 8)][u8 byte_b (default 6)]
//! Note: the f32 field at object+8 is zero-initialized from xmm0 in memory only —
//! NOT read from the stream.

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct GlobalEffectConditionData {
    pub effect_id: u32,
    pub byte_a: u8,
    pub byte_b: u8,
}

impl GlobalEffectConditionData {
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let effect_id = u32::read_from(data, offset)?;
        let byte_a = u8::read_from(data, offset)?;
        let byte_b = u8::read_from(data, offset)?;
        Ok(Self { effect_id, byte_a, byte_b })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.effect_id.write_to(w)?;
        self.byte_a.write_to(w)?;
        self.byte_b.write_to(w)?;
        Ok(())
    }
}
