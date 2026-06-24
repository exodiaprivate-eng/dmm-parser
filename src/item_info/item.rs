use super::keys::*;
use super::structs::*;
use crate::binary::*;
use crate::json_traits::ToJsonValue;
use crate::py_binary_struct;
use std::io;

/// Parse the entire iteminfo binary into a `Vec<serde_json::Value>` of
/// item dicts. Mirrors the Python `parse_iteminfo_from_bytes(data)` function
/// but runs without a Python interpreter — used by Rust mod managers (DMM)
/// for v3 mod application.
///
/// Each dict's shape matches `ItemInfo::to_json_dict()`. Field names match
/// the v3 mod format spec verbatim.
pub fn parse_iteminfo_to_json(data: &[u8]) -> io::Result<Vec<::serde_json::Value>> {
    let mut items = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let item = ItemInfo::read_from(data, &mut offset)?;
        items.push(item.to_json_value());
    }
    Ok(items)
}

/// Inverse of `parse_iteminfo_to_json`: write a sequence of item dicts back
/// to bytes. Each value must be an object whose shape matches what
/// `ItemInfo::to_json_dict()` produces.
pub fn serialize_iteminfo_from_json(items: &[::serde_json::Value]) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(items.len() * 256);
    for (i, v) in items.iter().enumerate() {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("item[{}]: expected object, got {:?}", i, v),
        ))?;
        ItemInfo::write_from_json_dict(&mut out, obj).map_err(|e| io::Error::new(
            e.kind(),
            format!("item[{}]: {}", i, e),
        ))?;
    }
    Ok(out)
}

// Schema reverted to the dmm-api-test 1.3.3 layout (108 fields). The earlier
// 117-field schema added 12 speculative fields based on the
// `/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1` test fixture, but the
// game version users actually run (1.05.01 — Steam app 3321460) reads only
// 107 wire fields per ItemInfo per the IDA decompile of `sub_101885C38` in
// the Mac binary `CrimsonDesert_Steam`. The 117-field reader overran the
// per-entry byte ranges, misaligned downstream length prefixes, and produced
// the silent-close that took DMM down at `analyze/parse_vanilla` on item
// 1002151. The 108-field version below was the last-known-good schema for
// 1.05.01 (1 over the binary, but the trailing field gracefully fails on
// items that don't have it — no misalignment cascade).
py_binary_struct! {
    pub struct ItemInfo<'a> {
        pub key: ItemKey,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub max_stack_count: u64,
        pub item_name: LocalizableString<'a>,
        pub broken_item_prefix_string: LocalStringInfoKey,
        pub inventory_info: InventoryKey,
        pub equip_type_info: EquipTypeKey,
        pub occupied_equip_slot_data_list: CArray<OccupiedEquipSlotData>,
        pub item_tag_list: CArray<u32>,
        pub equipable_hash: u32,
        pub consumable_type_list: CArray<u32>,
        pub item_use_info_list: CArray<ItemUseKey>,
        pub item_icon_list: CArray<ItemIconData>,
        pub map_icon_path: StringInfoKey,
        // 1.10: _moneyIconPath (StringInfoKey, u32) removed from ItemInfo.
        // The wire region between item_icon_list and item_desc shrank by 4
        // bytes (18→14): map_icon_path, use_map_icon_alert, item_type,
        // material_key, material_match_info. Verified byte-identically across
        // all 6325 records in the 2026-6-4 (1.10) iteminfo fixture
        // (use_map_icon_alert stays {0,1}, item_type stays a small enum).
        pub use_map_icon_alert: u8,
        pub item_type: u8,
        pub material_key: u32,
        pub material_match_info: MaterialMatchKey,
        pub item_desc: LocalizableString<'a>,
        pub item_desc2: LocalizableString<'a>,
        pub equipable_level: u32,
        pub category_info: CategoryKey,
        pub knowledge_info: KnowledgeKey,
        pub knowledge_obtain_type: u8,
        pub destroy_effec_info: EffectKey,
        pub equip_passive_skill_list: CArray<PassiveSkillLevel>,
        pub use_immediately: u8,
        pub apply_max_stack_cap: u8,
        // 1.0.8: _extractMultiChangeInfo removed
        pub extract_additional_drop_set_info: u32,
        pub minimum_extract_enchant_level: u16,
        pub item_memo: CString<'a>,
        pub filter_type: CString<'a>,
        pub gimmick_info: GimmickInfoKey,
        pub gimmick_tag_list: CArray<CString<'a>>,
        pub max_drop_result_sub_item_count: u32,
        pub use_drop_set_target: u8,
        pub is_all_gimmick_sealable: u8,
        pub sealable_item_info_list: CArray<SealableItemInfo<'a>>,
        pub sealable_character_info_list: CArray<SealableItemInfo<'a>>,
        pub sealable_gimmick_info_list: CArray<SealableItemInfo<'a>>,
        pub sealable_gimmick_tag_list: CArray<SealableItemInfo<'a>>,
        pub sealable_tribe_info_list: CArray<SealableItemInfo<'a>>,
        pub sealable_money_info_list: CArray<ItemKey>,
        pub delete_by_gimmick_unlock: u8,
        pub gimmick_unlock_message_local_string_info: LocalStringInfoKey,
        pub can_disassemble: u8,
        pub transmutation_material_gimmick_list: CArray<GimmickInfoKey>,
        pub transmutation_material_item_list: CArray<ItemKey>,
        pub transmutation_material_item_group_list: CArray<ItemGroupKey>,
        pub is_register_trade_market: u8,
        pub multi_change_info_list: CArray<MultiChangeKey>,
        pub is_editor_usable: u8,
        pub discardable: u8,
        pub is_dyeable: u8,
        pub is_editable_grime: u8,
        pub is_destroy_when_broken: u8,
        // Restored 2026-05-06 per IDA decomp of sub_101885C38 in
        // CrimsonDesert_Steam: ItemInfo의 _isHousingOnly is read between
        // _isDestoryWhenBroken and _quickSlotIndex.
        pub is_housing_only: u8,
        // 1.0.8: new field between _isHousingOnly and _quickSlotIndex
        pub is_extract_able_item: u8,
        pub quick_slot_index: u8,
        pub reserve_slot_target_data_list: CArray<ReserveSlotTargetData>,
        pub item_tier: u8,
        pub is_important_item: u8,
        pub apply_drop_stat_type: u8,
        // 1.11: one new u8 read between _applyDropStatType and _dropDefaultData
        // (ItemInfo reader sub_101935168 @ a2+531, a 1-byte vtable read). Without
        // it the whole drop/prefab tail shifted by 1 — default_sub_item read the
        // wrong disc byte (0x00 disc-0 +u32 instead of 0x0f disc-15 None), and
        // prefab_data_list.count blew up at offset 348. Verified via IDA + the
        // PrefabData reader (sub_101969834, fields unchanged).
        pub apply_drop_stat_extra_111: u8,
        pub drop_default_data: DropDefaultData,
        pub prefab_data_list: CArray<PrefabData>,
        pub enchant_data_list: CArray<EnchantData>,
        pub gimmick_visual_prefab_data_list: CArray<GimmickVisualPrefabData>,
        pub price_list: CArray<ItemPriceInfo>,
        pub docking_child_data: COptional<DockingChildData<'a>>,
        pub inventory_change_data: COptional<InventoryChangeData>,
        pub unk_texture_path: CString<'a>,
        pub fixed_page_data_list: CArray<PageData<'a>>,
        pub dynamic_page_data_list: CArray<PageData<'a>>,
        pub inspect_data_list: CArray<InspectData<'a>>,
        pub inspect_action: InspectAction<'a>,
        pub default_sub_item: SubItem,
        // 24-byte struct in wire (3 × i64), not a single i64. See structs::Cooltime.
        pub cooltime: Cooltime,
        pub item_charge_type: u8,
        // Restored 2026-05-06 per IDA decomp: ItemInfo의 _usableAlertType is
        // read between _itemChargeType and _sharpnessData. The schema revert
        // had renamed this to `usable_alert` and placed it later in the
        // struct; both placement and name were wrong.
        pub usable_alert_type: u8,
        pub sharpness_data: ItemInfoSharpnessData,
        // 12-byte struct in wire (3 × u32), not single u32. See structs::MaxChargedUseableCount.
        pub max_charged_useable_count: MaxChargedUseableCount,
        // 1.0.8: wire u16 per element (IDA sub_1410F5E50 confirmed)
        pub hackable_character_group_info_list: CArray<u16>,
        // 1.0.8: wire u16 per element (was u32 ItemGroupKey in 1.05)
        pub item_group_info_list: CArray<u16>,
        pub discard_offset_y: f32,
        // Restored 2026-05-06 per IDA decomp: _discardAttachTerrain between
        // _discardOffsetY and _hideFromInventoryOnPopItem.
        pub discard_attach_terrain: u8,
        pub hide_from_inventory_on_pop_item: u8,
        pub is_shield_item: u8,
        pub is_tower_shield_item: u8,
        pub is_wild: u8,
        pub packed_item_info: ItemKey,
        pub unpacked_item_info: ItemKey,
        pub convert_item_info_by_drop_npc: ItemKey,
        // Restored 2026-05-06 per IDA decomp: _stageInfo + _patternDescriptionDataList
        // between _convertItemInfoByDropNPC and _lookDetailGameAdviceInfoWrapper.
        pub stage_info: u32,
        pub pattern_description_data_list: CArray<PatternDescriptionData<'a>>,
        pub look_detail_game_advice_info_wrapper: GameAdviceInfoKey,
        pub look_detail_mission_info: MissionKey,
        pub enable_alert_system_to_ui: u8,
        pub is_save_game_data_at_use_item: u8,
        pub is_logout_at_use_item: u8,
        pub shared_cool_time_group_name_hash: u32,
        pub item_bundle_data_list: CArray<ItemBundleData>,
        pub money_type_define: COptional<MoneyTypeDefine<'a>>,
        pub emoji_texture_id: CString<'a>,
        pub enable_equip_in_clone_actor: u8,
        pub is_blocked_store_sell: u8,
        pub is_preorder_item: u8,
        // Restored 2026-05-06 per IDA decomp: _isHasItemUseDataInventoryBuff
        // and _isPreservedOnExtract between _isPreorderItem and _respawnTimeSeconds.
        pub is_has_item_use_data_inventory_buff: u8,
        pub is_preserved_on_extract: u8,
        // 1.12: new _itemEffectInfo field between _isPreservedOnExtract and
        // _respawnTimeSeconds. Game reader sub_1013632AC reads a 4-byte EffectKey
        // (resolved to a u16 EffectInfo index at struct+990). Stored/round-tripped
        // as the raw u32 wire key.
        pub item_effect_info: u32,
        pub respawn_time_seconds: i64,
        // 1.12: NEW u32 inserted between respawn_time_seconds and max_endurance
        // (always 0 in vanilla). Byte-decisive: 1.11/1.12 iteminfo are byte-identical
        // except a +4 `00 00 00 00` at this exact field boundary (tracked field map:
        // respawn_time_seconds ends @622, max_endurance starts @622 — the 4 new bytes
        // land between them), once per record. Semantic unknown → u32 placeholder so
        // the record realigns and stays JSON-addressable.
        pub unk_u32_112: u32,
        pub max_endurance: u16,
        pub repair_data_list: CArray<RepairData>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Cross-platform fixture resolution. Tries env var first, then known paths.
    // Returns None (test SKIPs) if no fixture found anywhere.
    fn find_iteminfo() -> Option<Vec<u8>> {
        let candidates: &[&str] = &[
            "/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/iteminfo.pabgb",
            r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/iteminfo.pabgb",
            r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/iteminfo.pabgb",
        ];
        if let Ok(p) = std::env::var("DMM_PARSER_ITEMINFO_PATH") {
            if let Ok(d) = std::fs::read(&p) {
                return Some(d);
            }
        }
        for c in candidates {
            if let Ok(d) = std::fs::read(c) {
                return Some(d);
            }
        }
        None
    }

    macro_rules! load_or_skip {
        () => {
            match find_iteminfo() {
                Some(d) => d,
                None => {
                    eprintln!("SKIP: iteminfo_1.0.4.0.pabgb fixture not found");
                    return;
                }
            }
        };
    }

    #[test]
    fn test_parse_first_item() {
        let data = load_or_skip!();
        let mut offset = 0;
        let item = ItemInfo::read_from(&data, &mut offset).unwrap();
        assert_eq!(item.key, ItemKey(2200));
        assert_eq!(item.string_key.data, "Pyeonjeon_Arrow");
        // First item size is version-dependent (628 B on 1.11). Don't hardcode —
        // just assert the read consumed a plausible record and the next item
        // parses from there (covered by test_parse_second_item).
        assert!(offset > 0x100 && offset < data.len(), "implausible first-item size {:#x}", offset);
    }

    #[test]
    fn test_parse_second_item() {
        let data = load_or_skip!();
        // Derive item-1 offset by parsing item 0 (size is version-dependent).
        let mut offset = 0;
        let _first = ItemInfo::read_from(&data, &mut offset).unwrap();
        let item = ItemInfo::read_from(&data, &mut offset).unwrap();
        assert_ne!(item.key, ItemKey(0));
        println!(
            "Second item: key={}, name={}",
            item.key.0, item.string_key.data
        );
    }

    #[test]
    fn test_first_item_roundtrip() {
        let data = load_or_skip!();
        let mut offset = 0;
        let item = ItemInfo::read_from(&data, &mut offset).unwrap();
        let end = offset;

        let mut out = Vec::new();
        item.write_to(&mut out).unwrap();
        assert_eq!(out.len(), end, "written size mismatch");
        assert_eq!(&out[..], &data[..end], "roundtrip bytes mismatch");
    }
}
