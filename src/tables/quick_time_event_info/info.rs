//! Hand-corrected: IDA-derived parser for `QuickTimeEventInfo.pabgb`.
//!
//! Per IDA sub_14081E190 (outer): u32 key, CString string_key, u8 is_blocked,
//! CArray<QuickTimeEventInfoData> (sub_14110A790, stride 48 bytes).
//!
//! Per IDA sub_1410F5550 (element parser): each entry has 14 fixed leading
//! fields followed by a polymorphic QuickTimeEventData payload via
//! sub_141F96FB0.
//!
//! Per IDA sub_141F96FB0: 10 known QuickTimeEventData variants dispatched on
//! a u8 tag (case 0..=9):
//!   0  SingleClick   — empty
//!   1  RepeatClick   — u16 + u32  (6 bytes)
//!   2  MultiClick    — u16        (2 bytes)
//!   3  DoubleClick   — empty
//!   4  Press         — empty
//!   5  Timing        — u32+u32+u32+u32+u16+u8 (19 bytes)
//!   6  Indicator     — empty
//!   7  Spin          — u32+u8 (5 bytes)
//!   8  Balance       — u32+u32+u32 (12 bytes, disk read order)
//!   9  BarTiming     — u32+u32+u32+u32+u16 (18 bytes)
//!
//! DO NOT REGENERATE. Hand-written; bulk_process.py guards via the
//! "Hand-corrected" header marker on line 1.

use crate::binary::*;
use crate::py_binary_struct;
use std::io::{self, Write};

py_binary_struct! {
    pub struct RepeatClickPayload {
        pub field_a: u16,
        pub field_b: u32,
    }
}

py_binary_struct! {
    pub struct MultiClickPayload {
        pub field_a: u16,
    }
}

py_binary_struct! {
    pub struct TimingPayload {
        pub field_a: u32,
        pub field_b: u32,
        pub field_c: u32,
        pub field_d: u32,
        pub field_e: u16,
        pub field_f: u8,
    }
}

py_binary_struct! {
    pub struct SpinPayload {
        pub field_a: u32,
        pub field_b: u8,
    }
}

py_binary_struct! {
    pub struct BalancePayload {
        pub field_a: u32,
        pub field_b: u32,
        pub field_c: u32,
    }
}

py_binary_struct! {
    pub struct BarTimingPayload {
        pub field_a: u32,
        pub field_b: u32,
        pub field_c: u32,
        pub field_d: u32,
        pub field_e: u16,
    }
}

#[derive(Debug)]
pub enum QuickTimeEventDataVariant {
    SingleClick,
    RepeatClick(RepeatClickPayload),
    MultiClick(MultiClickPayload),
    DoubleClick,
    Press,
    Timing(TimingPayload),
    Indicator,
    Spin(SpinPayload),
    Balance(BalancePayload),
    BarTiming(BarTimingPayload),
}

impl QuickTimeEventDataVariant {
    pub fn discriminator(&self) -> u8 {
        match self {
            Self::SingleClick => 0,
            Self::RepeatClick(_) => 1,
            Self::MultiClick(_) => 2,
            Self::DoubleClick => 3,
            Self::Press => 4,
            Self::Timing(_) => 5,
            Self::Indicator => 6,
            Self::Spin(_) => 7,
            Self::Balance(_) => 8,
            Self::BarTiming(_) => 9,
        }
    }

    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let disc = u8::read_from(data, offset)?;
        let result = match disc {
            0 => Self::SingleClick,
            1 => Self::RepeatClick(RepeatClickPayload::read_from(data, offset)?),
            2 => Self::MultiClick(MultiClickPayload::read_from(data, offset)?),
            3 => Self::DoubleClick,
            4 => Self::Press,
            5 => Self::Timing(TimingPayload::read_from(data, offset)?),
            6 => Self::Indicator,
            7 => Self::Spin(SpinPayload::read_from(data, offset)?),
            8 => Self::Balance(BalancePayload::read_from(data, offset)?),
            9 => Self::BarTiming(BarTimingPayload::read_from(data, offset)?),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown QuickTimeEventData discriminator: {}", disc),
                ));
            }
        };
        Ok(result)
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.discriminator().write_to(w)?;
        match self {
            Self::SingleClick | Self::DoubleClick | Self::Press | Self::Indicator => Ok(()),
            Self::RepeatClick(p) => p.write_to(w),
            Self::MultiClick(p) => p.write_to(w),
            Self::Timing(p) => p.write_to(w),
            Self::Spin(p) => p.write_to(w),
            Self::Balance(p) => p.write_to(w),
            Self::BarTiming(p) => p.write_to(w),
        }
    }
}

#[derive(Debug)]
pub struct QuickTimeEventInfoData {
    pub field_a: u8,
    pub field_b: u8,
    pub hash_a: u32,
    pub hash_b: u32,
    pub hash_c: u32,
    pub field_c: u32,
    pub field_d: u32,
    pub field_e: u32,
    pub field_f: u32,
    pub block: [u8; 8],
    pub flag_a: u8,
    pub flag_b: u8,
    pub field_g: u32,
    pub variant: QuickTimeEventDataVariant,
}

impl QuickTimeEventInfoData {
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let field_a = u8::read_from(data, offset)?;
        let field_b = u8::read_from(data, offset)?;
        let hash_a = u32::read_from(data, offset)?;
        let hash_b = u32::read_from(data, offset)?;
        let hash_c = u32::read_from(data, offset)?;
        let field_c = u32::read_from(data, offset)?;
        let field_d = u32::read_from(data, offset)?;
        let field_e = u32::read_from(data, offset)?;
        let field_f = u32::read_from(data, offset)?;
        let mut block = [0u8; 8];
        for b in &mut block {
            *b = u8::read_from(data, offset)?;
        }
        let flag_a = u8::read_from(data, offset)?;
        let flag_b = u8::read_from(data, offset)?;
        let field_g = u32::read_from(data, offset)?;
        let variant = QuickTimeEventDataVariant::read_from(data, offset)?;
        Ok(Self {
            field_a, field_b, hash_a, hash_b, hash_c,
            field_c, field_d, field_e, field_f, block,
            flag_a, flag_b, field_g, variant,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.field_a.write_to(w)?;
        self.field_b.write_to(w)?;
        self.hash_a.write_to(w)?;
        self.hash_b.write_to(w)?;
        self.hash_c.write_to(w)?;
        self.field_c.write_to(w)?;
        self.field_d.write_to(w)?;
        self.field_e.write_to(w)?;
        self.field_f.write_to(w)?;
        w.write_all(&self.block)?;
        self.flag_a.write_to(w)?;
        self.flag_b.write_to(w)?;
        self.field_g.write_to(w)?;
        self.variant.write_to(w)
    }
}

#[derive(Debug)]
pub struct QuickTimeEventInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub quick_time_event_data_list: Vec<QuickTimeEventInfoData>,
}

impl<'a> QuickTimeEventInfo<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let count = u32::read_from(data, offset)? as usize;
        let mut quick_time_event_data_list = Vec::with_capacity(count);
        for _ in 0..count {
            quick_time_event_data_list.push(QuickTimeEventInfoData::read_from(data, offset)?);
        }
        Ok(Self { key, string_key, is_blocked, quick_time_event_data_list })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        (self.quick_time_event_data_list.len() as u32).write_to(w)?;
        for entry in &self.quick_time_event_data_list {
            entry.write_to(w)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\quicktimeeventinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(QuickTimeEventInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "quicktimeeventinfo roundtrip bytes mismatch");
    }
}
