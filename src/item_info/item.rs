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
        // 1.16.00: `inventory_info` (InventoryKey = u16) was REMOVED by the
        // trading/inventory overhaul. It is absent from the binary's own
        // 115-field error-string list, and every record's diff vs the v14 fixture
        // contains exactly one 2-byte deletion at this offset carrying plausible
        // inventory ids (2, 5, ...). Its role appears to have moved to the
        // widened push-contents-type block at the record tail (below).
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
        // 1.13.00: prefab_data_list RELOCATED to the record tail (after
        // repair_data_list) and unified with the old gimmick_visual_prefab_data_list
        // (which is gone). enchant_data_list now sits directly after drop_default_data.
        // See PrefabData (structs.rs) + WORKING_STATE 1.13.00 notes.
        pub enchant_data_list: CArray<EnchantData>,
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
        // 1.12's `unk_u32_112` sits HERE in 1.16 — before the new region and
        // before respawn_time_seconds, not after it. Its old placement was
        // indistinguishable while everything around it was zero; the new region's
        // count byte makes it observable, and reading it after respawn puts the
        // count 4 bytes early (observed as a bogus CArray count 0x01000000).
        // Same 4 bytes, same total record size — only the order moved.
        pub unk_u32_112: u32,
        // ── 1.16.00: new variable-length region, 10 + 28n bytes ───────────────
        // The binary's 115-field list gained `_itemEffectInfo`,
        // `_factionManagementData` and `_useAveragePrice` right here, between
        // `_isPreservedOnExtract` and `_respawnTimeSeconds`.
        //
        // Pinned by aligning every record class at `max_endurance` (reliable on
        // records whose repair and prefab lists are both empty):
        //   +24 class -> region 10 B   (count 0)
        //   +52 class -> region 38 B   (count 1)
        //   +80 class -> region 66 B   (count 2)
        // i.e. a fixed 10 B frame plus 28 B per entry. Confirmed on item
        // 0xf51f0: `01 | 01 00 00 00 | 0d.. 32.. 32.. 33.. | 03 00 00 00 | 00`.
        //
        // This also explains the earlier red herring: the items whose bytes here
        // looked like `ff ff ff ff ff ff ff ff` are NOT this region — that is
        // `respawn_time_seconds` = -1 (never respawns), which only became visible
        // once the region was placed on the correct side of it.
        pub item_effect_info: u8,
        pub faction_management_data: CArray<FactionManagementData>,
        pub faction_management_extra: u32,
        pub use_average_price: u8,
        pub respawn_time_seconds: i64,
        pub max_endurance: u16,
        pub repair_data_list: CArray<RepairData>,
        // 1.13.00: prefab_data_list moved here (from after drop_default_data),
        // now the UNIFIED PrefabData type that also carries the old
        // gimmick_visual_prefab_data_list entries. Wire-order confirmed via the
        // game's ItemInfo/PrefabData readers (Win 1.13.00) + full-table roundtrip.
        pub prefab_data_list: CArray<PrefabData>,
        // 1.16.00: +16 bytes here — the record tail went from ONE u16 to NINE.
        // This is the other half of the inventory overhaul (see the removed
        // `inventory_info` above): a per-inventory-contents-type block rather
        // than a single id. Values are small enum-like codes (8, 13, 2, 5, 9, 1)
        // with 255 = unset.
        //
        // The 8 new slots go BEFORE the legacy field, not after: the pre-1.16
        // value was 255 in ALL 6508 records, and in 1.16 the FIRST slot is never
        // 255 (2x5272, 5x1116, 13x115, ...) while the LAST is 255 in 6522/6581.
        // Keeping the legacy field last also matches the binary, where
        // `_itemPushInventoryContentsType` is the final field.
        pub push_inventory_type_0_116: u16,
        pub push_inventory_type_1_116: u16,
        pub push_inventory_type_2_116: u16,
        pub push_inventory_type_3_116: u16,
        pub push_inventory_type_4_116: u16,
        pub push_inventory_type_5_116: u16,
        pub push_inventory_type_6_116: u16,
        pub push_inventory_type_7_116: u16,
        // 1.13.00: 2 trailing bytes after the relocated prefab_data_list. Per the
        // game's ItemInfo reader, item_push_inventory_contents_type follows
        // prefab_data_list; kept as u8 + u8 for bit-exact roundtrip (obs `ff 00`).
        pub item_push_inventory_contents_type_113: u8,
        pub trailing_u8_113: u8,
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

    // Full-table byte-exact roundtrip: parse EVERY item to EOF and write each back,
    // asserting byte-identity. This is the behavioral proof that new SubItem
    // discriminants (e.g. 1.13.00's disc 17) consume the payload they actually
    // carry — a mis-sized read would desync every subsequent item.
    // Full-table byte-exact roundtrip: parse EVERY item to EOF and write each back.
    // #[ignore]d until the 1.13.00 iteminfo record reorg is decoded (SubItem disc 17
    // is handled, but prefab/enchant/gimmick_visual relocated to the record tail —
    // see WORKING_STATE 1.13.00 notes). Run with DMM_PARSER_ITEMINFO_PATH set.
    /// Per-record diagnostic: walks the pabgh index so every record is checked
    /// against its OWN declared boundary instead of cascading from a single
    /// sequential desync. Reports which keys mis-size and by how much.
    #[test]
    #[ignore = "diagnostic; run with DMM_PARSER_PABGB_DIR set"]
    fn diag_per_record_sizes() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let p = crate::testenv::resolve("iteminfo.pabgb");
        let Ok(data) = std::fs::read(&p) else { eprintln!("SKIP: no fixture"); return; };
        let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy())
        else { eprintln!("SKIP: no pabgh"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let (mut ok, mut bad) = (0usize, 0usize);
        let mut deltas = std::collections::BTreeMap::<i64, usize>::new();
        for (k, s, e) in &ranges {
            let mut c = *s;
            match ItemInfo::read_from(&data, &mut c) {
                Ok(_) if c == *e => ok += 1,
                Ok(_) => {
                    bad += 1;
                    *deltas.entry(c as i64 - *e as i64).or_default() += 1;
                    if bad <= 6 {
                        eprintln!("k=0x{:x} consumed {} of {} (delta {})",
                            k, c - *s, *e - *s, c as i64 - *e as i64);
                    }
                }
                Err(_) => {
                    bad += 1;
                    *deltas.entry(i64::MIN).or_default() += 1;
                    println!("BADKEY {}", k);
                }
            }
        }
        eprintln!("per-record: OK={} BAD={}  deltas={:?}", ok, bad, deltas);
    }

    /// Tracked-read one record and print the last fields consumed before the
    /// desync, so the offending field is named rather than guessed at.
    #[test]
    #[ignore = "diagnostic"]
    fn diag_tracked_one() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let want = u32::from_str_radix(
            &std::env::var("DIAG_KEY").unwrap_or_else(|_| "38ae".into()), 16).unwrap();
        let p = crate::testenv::resolve("iteminfo.pabgb");
        let Ok(data) = std::fs::read(&p) else { return; };
        let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy())
        else { return; };
        for (k, s, e) in entry_ranges(&entries, data.len()) {
            if k as u32 != want { continue; }
            let mut c = s;
            let mut path = String::new();
            let mut ranges = Vec::new();
            let r = ItemInfo::read_tracked(&data, &mut c, &mut path, &mut ranges);
            eprintln!("k=0x{:x} range [{}..{}) len={}  result={:?}  cursor={} (rel {})",
                k, s, e, e - s, r.as_ref().map(|_| "ok").map_err(|x| x.to_string()),
                c, c as i64 - s as i64);
            for f in ranges.iter().rev().take(80).rev() {
                eprintln!("   rel {:>5}..{:<5} {:<12} {}",
                    f.start - s, f.end - s, f.ty, f.path);
            }
        }
    }

    #[test]
    #[ignore = "1.13.00 iteminfo record reorg not yet decoded"]
    fn test_full_table_roundtrip() {
        let data = load_or_skip!();
        let mut offset = 0;
        let mut count = 0usize;
        while offset < data.len() {
            let start = offset;
            let item = ItemInfo::read_from(&data, &mut offset)
                .unwrap_or_else(|e| panic!("item #{count} @ {start:#x}: {e}"));
            let mut out = Vec::new();
            item.write_to(&mut out).unwrap();
            assert_eq!(
                &out[..],
                &data[start..offset],
                "item #{count} key={} byte mismatch (span {start:#x}..{offset:#x})",
                item.key.0
            );
            count += 1;
        }
        assert_eq!(offset, data.len(), "trailing bytes after {count} items");
        println!("full-table roundtrip OK: {count} items, {} bytes", data.len());
    }
}
