//! Hand-corrected: IDA-derived parser for `FactionNodeSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410DED80: 6 fields. _patrolAISplineDataList is a CArray
//! with COptional<sub_1413F8A20> inner elements (variable per-element size).
//! Captured as raw byte-blob; no tail.

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct FactionNodeSpawnInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub faction_node_info: u32,
    pub boundary_box: [u8; 24],
    pub patrol_ai_spline_data_list: Vec<u8>,
}

impl<'a> FactionNodeSpawnInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start + entry_size;

        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let faction_node_info = u32::read_from(data, offset)?;
        let mut boundary_box = [0u8; 24];
        for b in &mut boundary_box { *b = u8::read_from(data, offset)?; }

        let patrol_ai_spline_data_list = data[*offset..entry_end].to_vec();
        *offset = entry_end;

        Ok(Self {
            key, string_key, is_blocked, faction_node_info,
            boundary_box, patrol_ai_spline_data_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.faction_node_info.write_to(w)?;
        w.write_all(&self.boundary_box)?;
        w.write_all(&self.patrol_ai_spline_data_list)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\factionnodespawninfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\factionnodespawninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = FactionNodeSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionnodespawninfo roundtrip bytes mismatch");
    }
}
