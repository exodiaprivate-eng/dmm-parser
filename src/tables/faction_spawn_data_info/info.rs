//! Hand-corrected: IDA-derived parser for `FactionSpawnDataInfo.pabgb`.
//!
//! Per IDA sub_1410DF1D0: 7 fields. _patrolSpawnData / _gimmickSpawnDataList /
//! _scheduleSpawnInfo / _sequencerSpawnInfo are all COptional<polymorphic>
//! with variable inner sizes. Captured the four-field tail as one byte-blob.

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct FactionSpawnDataInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    /// _patrolSpawnData + _gimmickSpawnDataList + _scheduleSpawnInfo +
    /// _sequencerSpawnInfo as combined byte-blob (each is COptional with
    /// variable polymorphic inner content).
    pub spawn_data_blob: Vec<u8>,
}

impl<'a> FactionSpawnDataInfo<'a> {
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

        let spawn_data_blob = data[*offset..entry_end].to_vec();
        *offset = entry_end;

        Ok(Self { key, string_key, is_blocked, spawn_data_blob })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        w.write_all(&self.spawn_data_blob)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\factionspawndatainfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\factionspawndatainfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = FactionSpawnDataInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionspawndatainfo roundtrip bytes mismatch");
    }
}
