//! Hand-corrected: IDA-derived parser for `SequencerSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410F6AA0: 8 fields. _sequencerSpawnDataList is polymorphic
//! CArray (sub_141DAE6A0 with sub_141D8C6D0 deeply-nested PlaySequencer
//! variant). Captured as raw byte-blob with a 6-byte tail probe.

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct SequencerSpawnInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub description: CString<'a>,
    pub sequencer_spawn_data_list: Vec<u8>,
    pub stage_type: u8,
    pub is_random: u8,
    pub match_tag_hash: u32,
}

const TAIL_SIZE: usize = 6; // u8 + u8 + u32

impl<'a> SequencerSpawnInfo<'a> {
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
        let description = CString::read_from(data, offset)?;

        if *offset + TAIL_SIZE > entry_end {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "sequencerspawn: tail too short"));
        }
        let blob_end = entry_end - TAIL_SIZE;
        let sequencer_spawn_data_list = data[*offset..blob_end].to_vec();
        *offset = blob_end;

        let stage_type = u8::read_from(data, offset)?;
        let is_random = u8::read_from(data, offset)?;
        let match_tag_hash = u32::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, description,
            sequencer_spawn_data_list, stage_type, is_random, match_tag_hash,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.description.write_to(w)?;
        w.write_all(&self.sequencer_spawn_data_list)?;
        self.stage_type.write_to(w)?;
        self.is_random.write_to(w)?;
        self.match_tag_hash.write_to(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\sequencerspawninfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\sequencerspawninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = SequencerSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "sequencerspawninfo roundtrip bytes mismatch");
    }
}
