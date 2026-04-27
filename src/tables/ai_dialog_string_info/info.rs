//! Hand-corrected: IDA-derived parser for `AIDialogStringInfo.pabgb`.
//!
//! Per IDA sub_1410D5210: 11 fields. The latter half contains a hashmap
//! (sub_141119210) and a COptional<polymorphic> (sub_141119080). Captured
//! the trailing fields as one byte-blob.

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct AIDialogStringInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub flag_a: u8,
    pub flag_b: u8,
    pub list_a: CArray<u16>,
    pub lookup_a: u16,  // 2 bytes on disk via sub_1410FF220
    pub lookup_b: u32,  // 4 bytes on disk via sub_1410FF2D0
    /// Hashmap + trailing fields captured as one byte-blob (variable-size
    /// internal structures defy linear-probe boundary detection).
    pub trailing_blob: Vec<u8>,
}

impl<'a> AIDialogStringInfo<'a> {
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
        let flag_a = u8::read_from(data, offset)?;
        let flag_b = u8::read_from(data, offset)?;
        let list_a = CArray::<u16>::read_from(data, offset)?;
        let lookup_a = u16::read_from(data, offset)?;
        let lookup_b = u32::read_from(data, offset)?;

        let trailing_blob = data[*offset..entry_end].to_vec();
        *offset = entry_end;

        Ok(Self {
            key, string_key, is_blocked, flag_a, flag_b,
            list_a, lookup_a, lookup_b, trailing_blob,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.flag_a.write_to(w)?;
        self.flag_b.write_to(w)?;
        self.list_a.write_to(w)?;
        self.lookup_a.write_to(w)?;
        self.lookup_b.write_to(w)?;
        w.write_all(&self.trailing_blob)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\aidialogstringinfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\aidialogstringinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = AIDialogStringInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "aidialogstringinfo roundtrip bytes mismatch");
    }
}
