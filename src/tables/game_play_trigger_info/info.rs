use crate::pabgh_blob_table;
pabgh_blob_table! { pub struct GamePlayTriggerInfo<'a> { key: u32, blob_field: data_blob, } }
#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameplaytrigger.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameplaytrigger.pabgh";
    #[test] fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s; items.push(GamePlayTriggerInfo::read_with_size(&data, &mut c, e-s).unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)));
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len()); for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data);
    }
}
