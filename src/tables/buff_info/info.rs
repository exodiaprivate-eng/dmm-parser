//! Hand-corrected: IDA-derived parser for `BuffInfo.pabgb`.
//!
//! Per IDA sub_1410D6510: 13 fields matching mac binary __cstring order.
//! _buffDataList is a CArray<BuffDataEntry>. Each entry is:
//!   - u32 leading_lookup
//!   - u8 absent_flag (1=absent, 0=present)  ← INVERTED COptional
//!   - if !absent: typed BuffData (see binary::variants::buff_data)
//!
//! BuffData is a 120-variant polymorphic family, fully field-decoded
//! from Mac-binary symbols + Win-binary parser introspection.
//! All 48 variant tags observed in vanilla data validate cleanly.

use crate::binary::variants::buff_data::BuffData;
use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct BuffDataEntry<'a> {
    pub leading_lookup: u32,
    pub absent_flag: u8,
    pub data: Option<BuffData<'a>>,
}

impl<'a> BinaryRead<'a> for BuffDataEntry<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let leading_lookup = u32::read_from(data, offset)?;
        let absent_flag = u8::read_from(data, offset)?;
        let payload = if absent_flag == 0 {
            Some(BuffData::read_from(data, offset)?)
        } else {
            None
        };
        Ok(Self { leading_lookup, absent_flag, data: payload })
    }
}

impl<'a> BinaryWrite for BuffDataEntry<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.leading_lookup.write_to(w)?;
        self.absent_flag.write_to(w)?;
        if let Some(d) = &self.data {
            d.write_to(w)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct BuffInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub buff_data_list: CArray<BuffDataEntry<'a>>,
    pub min_level: u32,
    pub max_level: u32,
    pub sequencer_file_name: CString<'a>,
    pub buff_level_calculate_type: u8,
    pub ui_template_name: u32,
    pub ui_component_name: u32,
    pub elemental_status_info: u32,
    pub is_use_skill_info_pattern_description: u8,
    pub use_counting_by_global_timer: u8,
}

impl<'a> BuffInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        _entry_size: usize,
    ) -> io::Result<Self> {
        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let buff_data_list = CArray::<BuffDataEntry>::read_from(data, offset)?;
        let min_level = u32::read_from(data, offset)?;
        let max_level = u32::read_from(data, offset)?;
        let sequencer_file_name = CString::read_from(data, offset)?;
        let buff_level_calculate_type = u8::read_from(data, offset)?;
        let ui_template_name = u32::read_from(data, offset)?;
        let ui_component_name = u32::read_from(data, offset)?;
        let elemental_status_info = u32::read_from(data, offset)?;
        let is_use_skill_info_pattern_description = u8::read_from(data, offset)?;
        let use_counting_by_global_timer = u8::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, buff_data_list,
            min_level, max_level, sequencer_file_name,
            buff_level_calculate_type, ui_template_name, ui_component_name,
            elemental_status_info,
            is_use_skill_info_pattern_description, use_counting_by_global_timer,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.buff_data_list.write_to(w)?;
        self.min_level.write_to(w)?;
        self.max_level.write_to(w)?;
        self.sequencer_file_name.write_to(w)?;
        self.buff_level_calculate_type.write_to(w)?;
        self.ui_template_name.write_to(w)?;
        self.ui_component_name.write_to(w)?;
        self.elemental_status_info.write_to(w)?;
        self.is_use_skill_info_pattern_description.write_to(w)?;
        self.use_counting_by_global_timer.write_to(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str =
        r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\buffinfo.pabgb";
    const PABGH_PATH: &str =
        r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\buffinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing pabgb fixture {}", PABGB_PATH);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else {
            eprintln!("SKIP: missing/unparseable pabgh fixture {}", PABGH_PATH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = BuffInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| {
                    panic!(
                        "parse failed at entry {} (key=0x{:x}, offset 0x{:x}, size {}): {}",
                        i, key, start, end - start, e
                    )
                });
            assert_eq!(
                cursor, *end,
                "entry {} (key 0x{:x}) under/over-consumed: read {} bytes, expected {}",
                i, key, cursor - start, end - start
            );
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out.len(), data.len(), "buffinfo roundtrip size mismatch");
        assert_eq!(out, data, "buffinfo roundtrip bytes mismatch");
    }
}
