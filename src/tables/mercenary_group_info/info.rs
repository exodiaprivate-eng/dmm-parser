//! IDA-derived parser for `MercenaryGroupInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! **T0-V verification (iter 1+6 of T0 verification loop, IDA Win 1.06):**
//! MercenaryGroupInfo is NOT in NattKh's pabgb_complete_schema.json
//! (one of 4 schema-missing T0 tables). IDA cross-references the rust
//! field names against the in-binary metaobject at 0x144b0a300+.
//! **Status: T0-V FULL (4/4 named CArrays verified canonical):**
//!
//! | rust field | canonical PA name | IDA address |
//! |---|---|---|
//! | `is_blocked` | `_isBlocked` | 0x144b0a4c5 ✓ |
//! | `parent_mercenary_group_info` | `_parentMercenaryGroupInfo` | 0x144b0a375 ✓ |
//! | `child_mercenary_group_info_list` | `_childMercenaryGroupInfoList` | 0x144b0a565 ✓ |
//! | `allow_operation_type_list` (renamed iter 6) | `_allowOperationTypeList` | 0x144b0a315 ✓ |
//! | `mercenarye_info_list` | `_mercenaryeInfoList` (typo!) | 0x144b0a515 ✓ |
//!
//! Iter 6 cleanup: renamed rust field `mercenary_key_list` →
//! `allow_operation_type_list` after iter-5 discovery confirmed the
//! 5-name vocabulary in IDA. Both `_mercenaryeInfoList` (typo'd in PA
//! binary) and `mercenarye_info_list` (typo'd in rust) match — typo
//! is intentional and preserved.
//!
//! All 10 records parse + round-trip byte-identical on 1.06 install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct MercenaryGroupInfo<'a> {
        pub key: u8,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        // ★ ORDER CORRECTED 2.00.00. These three were listed
        // allow / mercenarye / child; the reader takes them
        // mercenarye / child / allow (`field_order.py MercenaryGroupInfo`,
        // read sites 0x10202aabc / aac8 / aad4).
        //
        // All three are `CArray<u8>`, so the wire could not tell us: every
        // permutation round-trips byte-identically. The names were simply
        // attached to the wrong lists, which is invisible until a mod edits
        // one by name. Field names are the mod contract, so this is a fix,
        // not a tidy-up.
        //
        // Typo preserved on purpose: `_mercenaryeInfoList` (extra 'e') is
        // spelled that way in the game binary.
        pub mercenarye_info_list: CArray<u8>,
        pub child_mercenary_group_info_list: CArray<u8>,
        pub allow_operation_type_list: CArray<u8>,
        pub parent_mercenary_group_info: u8,
        // ── 2.00.00: nine new trailing fields ─────────────────────
        // Appended after `_parentMercenaryGroupInfo`, read sites
        // 0x10202aaec..0x10202ab4c at a regular 0xc stride.
        pub is_dead_state_updatable: u8,
        pub is_using_portrait_texture: u8,
        pub is_item_contribution_countable: u8,
        pub is_hyosi_usable: u8,
        // Typo preserved: `_useChanageMainMercenary` in the binary.
        pub use_chanage_main_mercenary: u8,
        pub is_using_spawned_usable: u8,
        pub use_reserve_or_main: u8,
        pub use_vehicle_key: u8,
        pub is_return_main_mercenary_data: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("mercenarygroupinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(MercenaryGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "mercenarygroupinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(MercenaryGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            MercenaryGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
