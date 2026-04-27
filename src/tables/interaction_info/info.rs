//! Hand-corrected: blob-tail parser for `InteractionInfo.pabgb`.
use crate::pabgh_blob_table;

pabgh_blob_table! {
    pub struct InteractionInfo<'a> {
        key: u32,
        blob_field: data_blob,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\interactioninfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\interactioninfo.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = InteractionInfo::read_with_size(&data, &mut c, e - s).unwrap_or_else(|er| panic!("entry {} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e); items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data);
    }
}
