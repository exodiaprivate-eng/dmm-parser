//! IDA-derived parser for `EquipTypeInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.
//!
//! ─── v3.1 closure analysis (iter 73) ────────────────────────────────────
//! Cross-check via `sub_1410A9500` (typeinfo→record-reader path per the
//! iter 55 typeinfo registry) confirms the prefix layout:
//!
//!   offset 0    4 bytes      → _key (u32)
//!   offset 8    CString      → _stringKey
//!   offset 16   1 byte       → _isBlocked
//!   offset 20   **12 bytes raw** → _destroyedAiEvent COMPOSITE
//!                  (split across 4 rust fields:
//!                   destroyed_ai_event_head u16,
//!                   destroyed_ai_event_pad  u16,
//!                   destroyed_ai_event_hash u32,
//!                   destroyed_ai_event_tail u32)
//!   offset 32   sub_1410CFA30 (u16) → ...
//!   offset 34   sub_1410CFA30 (u16) → ...
//!   ...
//!
//! NattKh schema lists `_destroyedAiEvent` as a single 12-byte
//! canonical at this position; the rust struct already documents the
//! 12-byte composite split (head=0xFFFF, pad=0, hash u32 varies,
//! tail=0xFFFFFFFF — empirically confirmed across all 111 entries).
//! Wire-level confirmation: **`_destroyedAiEvent` is a pure 1-to-4
//! wrapper** around the four `destroyed_ai_event_*` rust fields.
//! Closure path: 1-to-N alias entry. No new decoder work needed.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct EquipTypeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        // [u8;12] composite split into 4 fields per empirical sweep across
        // all 111 entries: head=0xFFFF, pad=0, hash u32 varies, tail=0xFFFFFFFF.
        pub destroyed_ai_event_head: u16,
        pub destroyed_ai_event_pad: u16,
        pub destroyed_ai_event_hash: u32,
        pub destroyed_ai_event_tail: u32,
        pub use_resource_item_type: u32,
        pub fake_equip_reserve_slot_data: u32,
        pub apply_status_group_info_on_activate: u32,
        pub apply_passive_skill_on_activate: u8,
        pub is_show_stamina: u8,
        pub decrease_endurance_percent: u64,
        pub on_guard_damage_reduction_percent: u64,
        pub is_critical_collidable: u8,
        pub enable_transfer: u8,
        pub enable_enchant: u8,
        pub use_action_on_quick_slot: u8,
        pub camera_preset_hash: u32,
        pub dye_rotation_value: u32,
        pub equip_able_hash_list: CArray<u32>,
        pub equip_type_name: LocalizableString<'a>,
        pub show_helm_on_battle_stance: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/equiptypeinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(EquipTypeInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "equiptypeinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(EquipTypeInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            EquipTypeInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
