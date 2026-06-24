//! IDA-derived parser for `npcactivitygroupinfo.pabgb`.
//! Reader: `sub_1410E4660` (CrimsonDesert.exe 1.0.8).
//! Sequential table, u32 key, 5 fields.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct NpcActivityGroupInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub npc_activity_tag: u32,
        pub activity_info_key: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str =
        r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/npcactivitygroupinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { return; };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(NpcActivityGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len());
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data);
    }
}
