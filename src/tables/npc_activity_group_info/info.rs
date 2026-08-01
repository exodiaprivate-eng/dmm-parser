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
        // 1.16.00: the binary's 7-field list resolves this tail exactly —
        // _selfConditionInfo(u32) + _useContentsOnly(u8) + _npcActivityInfoList
        // (CArray<u32>) + _totalRate(f32). Decisive on rec4 (key 0xf426c): count=1
        // with activity key 1000103 and totalRate = 1.0f, ending at 51/51; rec0
        // 52/52 and rec3 40/40 with empty lists.
        //
        // This u32 IS the binary's `_selfConditionInfo` (values 1010696 / 1010697 /
        // 1000531 are all ConditionInfo keys — the 1.12 pass mis-read it as an
        // activity reference). Name kept as-is: parser field names are a MOD
        // CONTRACT and renaming silently skips any mod that sets it.
        pub activity_info_key: u32,
        pub use_contents_only: u8,
        pub npc_activity_info_list: CArray<u32>,
        pub total_rate: f32,
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
