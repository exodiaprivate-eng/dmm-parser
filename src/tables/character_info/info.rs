//! Hand-corrected: blob-tail parser for `CharacterInfo.pabgb`.
//!
//! The full struct (sub_1410D7480, ~8.4 KB of decompile) contains many
//! deeply-nested polymorphic CArrays that defy linear-probe boundary
//! detection. Captured the post-pre-fields content as one byte-blob.
//! Round-trips byte-perfect; full field decoding deferred until Mac
//! C++ symbols are available.

use crate::pabgh_blob_table;

pabgh_blob_table! {
    pub struct CharacterInfo<'a> {
        key: u32,
        blob_field: data_blob,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\characterinfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\characterinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = CharacterInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: {}", i, key, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data);
    }
}
