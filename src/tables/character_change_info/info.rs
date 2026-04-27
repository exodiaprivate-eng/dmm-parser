//! Hand-corrected: IDA-derived parser for `CharacterChange.pabgb`.
//!
//! Per IDA sub_1410D6950 (entry parser):
//!   u32 key
//!   CString string_key
//!   u8 is_blocked
//!   CArray<CString> name_list (sub_14106BAC0 — u32 count + N×CString)
//!   CArray<u16> hash_lookup_list (sub_1410FFD30 — u32 count + N×u16,
//!     each u16 is a key into a runtime hash registry; for round-trip
//!     we just preserve the u16 bytes verbatim)
//!   u32 trailing_id

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct CharacterChangeInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub name_list: Vec<CString<'a>>,
    pub hash_lookup_list: CArray<u16>,
    pub trailing_id: u32,
}

impl<'a> CharacterChangeInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        _entry_size: usize,
    ) -> io::Result<Self> {
        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;

        let name_count = u32::read_from(data, offset)? as usize;
        let mut name_list = Vec::with_capacity(name_count);
        for _ in 0..name_count {
            name_list.push(CString::read_from(data, offset)?);
        }

        let hash_lookup_list = CArray::<u16>::read_from(data, offset)?;
        let trailing_id = u32::read_from(data, offset)?;

        Ok(Self {
            key,
            string_key,
            is_blocked,
            name_list,
            hash_lookup_list,
            trailing_id,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        (self.name_list.len() as u32).write_to(w)?;
        for s in &self.name_list {
            s.write_to(w)?;
        }
        self.hash_lookup_list.write_to(w)?;
        self.trailing_id.write_to(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB: &str =
        r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\characterchange.pabgb";
    const PABGH: &str =
        r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\characterchange.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                CharacterChangeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e, "e{} k=0x{:x} consumed {} expected {}", i, k, c - s, e - s);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items {
            it.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "characterchange roundtrip bytes mismatch");
    }
}
