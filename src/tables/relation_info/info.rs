//! Hand-corrected: IDA-derived parser for `RelationInfo.pabgb`.
//!
//! Per IDA sub_1410F4C70 + sub_14110AA70:
//!   - 11 outer fields (u8 key, CString, then 9 more)
//!   - gimmick_tag_data_list element = { u32 hash + CArray<u32> + CArray<u32> }
//!     (NB: mac symbols list only 2 inner fields but binary reads 3 — last
//!      one is unnamed in current Mac depot)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct RelationGimmickTagData {
        pub gimmick_tag_hash: u32,
        pub spawn_reason_hash_list: CArray<u32>,
        pub extra_list: CArray<u32>,
    }
}

py_binary_struct! {
    pub struct RelationInfo<'a> {
        pub key: u8,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub relation_reaction_type: u8,
        pub order: u8,
        pub detect_restrict_count: u8,
        pub detect_memorize_time: u64,
        pub do_complete_not_priority_actor: u8,
        pub detect_value_ratio: f32,
        pub is_detect_event_only: u8,
        pub gimmick_tag_data_list: CArray<RelationGimmickTagData>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\relationinfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\relationinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        // RelationInfo's pabgh uses u16 count + (u8 key + u32 offset) per entry.
        let Ok(pabgh) = std::fs::read(PABGH_PATH) else {
            eprintln!("SKIP: missing pabgh fixture");
            return;
        };
        let count = u16::from_le_bytes(pabgh[0..2].try_into().unwrap()) as usize;
        let mut offsets = Vec::with_capacity(count);
        for i in 0..count {
            let pos = 2 + i * 5;
            let off = u32::from_le_bytes(pabgh[pos + 1..pos + 5].try_into().unwrap()) as usize;
            offsets.push(off);
        }
        offsets.sort();

        let mut items = Vec::new();
        for i in 0..offsets.len() {
            let mut o = offsets[i];
            let item = RelationInfo::read_from(&data, &mut o).unwrap();
            let next_off = if i + 1 < offsets.len() { offsets[i + 1] } else { data.len() };
            assert_eq!(o, next_off, "entry {} under/over-read: stopped at {} expected {}", i, o, next_off);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "relationinfo roundtrip bytes mismatch");
    }
}
