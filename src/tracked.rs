// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
//
// Tracked-parse dispatcher.
//
// `parse_table_to_json` returns Vec<serde_json::Value> per record. The
// tracked variant also returns per-leaf-field byte ranges so consumers
// can map an absolute byte offset back to a structured field path
// (entry name + dotted path inside the record).
//
// This is the primitive the dmm-converter v2-byte-patch path needs:
// given a v2 patch's `(game_file, offset, original_bytes, patched_bytes)`,
// the converter walks the tracked records to find the owning record
// and the owning leaf field, decodes the patched bytes as that field's
// type, and emits a v3 intent.
//
// The macro `py_binary_struct!` already emits `BinaryReadTracked` for
// every typed table that uses it. Tier 1.5 tables (typed-prefix +
// opaque tail) only get tracked ranges for the prefix; the tail is
// reported as a single opaque range.
//
// Only the tables explicitly wired below produce tracked records.
// Adding a new table is one match arm; the schema work is already in
// the per-table mod.

use crate::binary::{BinaryReadTracked, FieldRange};
use serde_json::Value;
use std::io;

/// One record parsed in tracked mode. Carries the byte span of the
/// record in the source body, the per-leaf field ranges (absolute
/// offsets within the body, NOT record-relative), the record's JSON
/// dict, and convenience accessors for the canonical key/string_key.
///
/// `ranges` start/end are ABSOLUTE byte offsets in the input body so
/// callers can map directly from a v2 patch's `offset` field without
/// having to track the per-record base themselves.
#[derive(Debug, Clone)]
pub struct TrackedRecord {
    pub record_start: usize,
    pub record_end: usize,
    pub json: Value,
    /// Per-leaf field ranges with offsets RELATIVE TO THE RECORD START.
    /// Caller adds `record_start` to convert to absolute body offset.
    /// We keep them relative because the typed `read_tracked` impl
    /// always starts at offset 0 for each call.
    pub ranges: Vec<FieldRange>,
}

impl TrackedRecord {
    /// Numeric record key extracted from `json`. Tables wrap key as
    /// either `key: <int>` or `key: {"value": <int>}` (typed wrappers
    /// like ItemKey). Accept both.
    pub fn key(&self) -> Option<u64> {
        let key = self.json.get("key")?;
        if let Some(n) = key.as_u64() {
            return Some(n);
        }
        key.get("value").and_then(|v| v.as_u64())
    }

    /// Record's `string_key` field. Most tables expose this; a few
    /// don't and return None.
    pub fn string_key(&self) -> Option<&str> {
        self.json.get("string_key").and_then(|v| v.as_str())
    }

    /// Find the leaf field whose absolute byte range contains `offset`.
    /// Returns the field's relative range (start/end within the record)
    /// plus its leaf path string. None if offset is in the record's
    /// opaque tail or outside every tracked leaf.
    pub fn find_owning_field(&self, body_offset: usize) -> Option<&FieldRange> {
        let rel = body_offset.checked_sub(self.record_start)?;
        self.ranges.iter().find(|r| rel >= r.start && rel < r.end)
    }
}

/// Dispatch to the right per-table tracked parser. `table_name` is the
/// same identifier `parse_table_to_json` consumes (e.g. "iteminfo",
/// "faction_node_info"). Returns the same record set in tracked form.
///
/// Tables not wired here return `InvalidInput`. To add support: pick a
/// match arm, pick the `tracked_p!` (pabgh-bounded) or `tracked_s!`
/// (sequential) macro, point it at the per-table struct. The schema
/// work in the parser is already done.
pub fn parse_table_tracked(
    table_name: &str,
    pabgb: &[u8],
    pabgh: Option<&[u8]>,
) -> io::Result<Vec<TrackedRecord>> {
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets_from_bytes};

    // pabgh-bounded macro: walk pabgh entries, read each record with
    // tracked = true. Skips records whose typed parser doesn't accept
    // the bytes (errors out as InvalidData up the chain — same policy
    // as `parse_table_to_json::p!`).
    // pabgh-bounded macro: walks pabgh entries, calls each table's
    // BinaryReadTracked impl (emitted by `py_binary_struct!`). Tables
    // that have a typed-prefix-plus-tail shape use the alternate
    // `tracked_p_typed!` macro below which appends an opaque-tail
    // range. For Tier 1 tables, read_tracked consumes exactly the
    // entry's bytes and we add a size-check post-call.
    macro_rules! tracked_p {
        ($ty:path) => {{
            let ph = pabgh.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                format!("table '{}' requires a pabgh file", table_name)))?;
            let entries = load_pabgh_offsets_from_bytes(ph).ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, "pabgh parse failed"))?;
            let ranges = entry_ranges(&entries, pabgb.len());
            let mut out = Vec::with_capacity(ranges.len());
            for (_key, start, end) in ranges {
                let mut offset = start;
                let mut path = String::new();
                let mut leaf_ranges: Vec<FieldRange> = Vec::new();
                let item = <$ty as BinaryReadTracked>::read_tracked(
                    pabgb, &mut offset, &mut path, &mut leaf_ranges,
                )?;
                // Tier 1 tables consume the entire entry. If the
                // typed parser under-consumed (Tier 1.5 prefix +
                // opaque tail) record the residual as a `_tail_blob`
                // leaf so the converter still sees those bytes and
                // can fall back to file_replacement on offsets that
                // land in them.
                if offset < end {
                    leaf_ranges.push(FieldRange {
                        path: "_tail_blob".to_string(),
                        start: offset,
                        end,
                        ty: "opaque_tail",
                    });
                    offset = end;
                }
                // ranges are absolute. Re-normalise to record-relative.
                for r in &mut leaf_ranges {
                    if r.start >= start { r.start -= start; }
                    if r.end >= start { r.end -= start; }
                }
                let json = Value::Object(item.to_json_dict());
                out.push(TrackedRecord {
                    record_start: start,
                    record_end: end,
                    json,
                    ranges: leaf_ranges,
                });
            }
            out
        }};
    }

    // pabgh-bounded macro for Tier 1.5 tables (typed prefix + opaque
    // tail). These declare via `pabgh_typed_blob_table!`, which emits
    // the inherent method `read_tracked_with_size(data, offset,
    // entry_size, path, ranges)` instead of a `BinaryReadTracked` trait
    // impl. The trait can't represent the tail boundary (it has no
    // `entry_size` parameter), so we call the inherent method directly.
    // The method registers its own `_tail_blob` range for the opaque
    // tail, so we don't add one here.
    macro_rules! tracked_p_typed {
        ($ty:path) => {{
            let ph = pabgh.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                format!("table '{}' requires a pabgh file", table_name)))?;
            let entries = load_pabgh_offsets_from_bytes(ph).ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, "pabgh parse failed"))?;
            let ranges = entry_ranges(&entries, pabgb.len());
            let mut out = Vec::with_capacity(ranges.len());
            for (_key, start, end) in ranges {
                let mut offset = start;
                let entry_size = end - start;
                let mut path = String::new();
                let mut leaf_ranges: Vec<FieldRange> = Vec::new();
                let item = <$ty>::read_tracked_with_size(
                    pabgb, &mut offset, entry_size, &mut path, &mut leaf_ranges,
                )?;
                // ranges are absolute. Re-normalise to record-relative.
                for r in &mut leaf_ranges {
                    if r.start >= start { r.start -= start; }
                    if r.end >= start { r.end -= start; }
                }
                let json = Value::Object(item.to_json_dict());
                out.push(TrackedRecord {
                    record_start: start,
                    record_end: end,
                    json,
                    ranges: leaf_ranges,
                });
            }
            out
        }};
    }

    // Sequential macro: no pabgh, just walk the body. Used by tables
    // whose records are self-delimiting (iteminfo, skill).
    macro_rules! tracked_s {
        ($ty:path) => {{
            let mut offset = 0usize;
            let mut out: Vec<TrackedRecord> = Vec::new();
            while offset < pabgb.len() {
                let start = offset;
                let mut path = String::new();
                let mut leaf_ranges: Vec<FieldRange> = Vec::new();
                let item = <$ty as BinaryReadTracked>::read_tracked(
                    pabgb, &mut offset, &mut path, &mut leaf_ranges,
                )?;
                let end = offset;
                for r in &mut leaf_ranges {
                    if r.start >= start { r.start -= start; }
                    if r.end >= start { r.end -= start; }
                }
                let json = Value::Object(item.to_json_dict());
                out.push(TrackedRecord {
                    record_start: start,
                    record_end: end,
                    json,
                    ranges: leaf_ranges,
                });
            }
            out
        }};
    }

    Ok(match table_name {
        // Iteminfo is sequential.
        "iteminfo" | "iteminfo.pabgb" => {
            tracked_s!(crate::item_info::ItemInfo)
        }

        // Tables wired as of 2026-05-11. Each compiles because every
        // nested helper type the table reaches has a BinaryReadTracked
        // impl — directly via the macro or via hand-rolled impls that
        // landed during the v3.1 closure-loop work in dmm-parser
        // (iters 100-160). Re-probe periodically as more closures ship.
        "faction_node_info" => tracked_p_typed!(crate::tables::faction_node_info::FactionNodeInfo),
        "equip_info" => tracked_p!(crate::tables::equip_info::EquipInfo),
        "faction_info" => tracked_p!(crate::tables::faction_info::FactionInfo),
        "gimmick_group_info" => tracked_p!(crate::tables::gimmick_group_info::GimmickGroupInfo),
        "elemental_material_info" => tracked_p!(crate::tables::elemental_material_info::ElementalMaterialInfo),
        "special_mode_info" => tracked_p!(crate::tables::special_mode_info::SpecialModeInfo),
        "knowledge_info" => tracked_p!(crate::tables::knowledge_info::KnowledgeInfo),
        "mission_info" => tracked_p!(crate::tables::mission_info::MissionInfo),
        "multi_change_info" => tracked_p!(crate::tables::multi_change_info::MultiChangeInfo),
        "tribe_info" => tracked_p!(crate::tables::tribe_info::TribeInfo),
        // field_info: 1.0.8 format restructure, now blob mode — no tracked support
        // "field_info" => tracked_p!(crate::tables::field_info::FieldInfo),

        // Tier 1.5 typed-blob tables. Use tracked_p_typed! because
        // their generated code is read_tracked_with_size, not the
        // BinaryReadTracked trait. The macro auto-registers the
        // opaque tail as a `_tail_blob` range.
        "character_info" => tracked_p_typed!(crate::tables::character_info::CharacterInfo),
        "character_change_info" => tracked_p_typed!(crate::tables::character_change_info::CharacterChangeInfo),
        // inventory_info: converted to blob mode — no tracked support
        // "inventory_info" => tracked_p_typed!(crate::tables::inventory_info::InventoryInfo),
        "buff_info" => tracked_p_typed!(crate::tables::buff_info::BuffInfo),
        "condition_info" => tracked_p_typed!(crate::tables::condition_info::ConditionInfo),
        "drop_set_info" => tracked_p_typed!(crate::tables::drop_set_info::DropSetInfo),
        "effect_info" => tracked_p_typed!(crate::tables::effect_info::EffectInfo),
        "gimmick_info" => tracked_p_typed!(crate::tables::gimmick_info::GimmickInfo),
        "interaction_info" => tracked_p_typed!(crate::tables::interaction_info::InteractionInfo),
        "store_info" => tracked_p_typed!(crate::tables::store_info::StoreInfo),
        "faction_node_spawn_info" => tracked_p_typed!(crate::tables::faction_node_spawn_info::FactionNodeSpawnInfo),
        "quest_info" => tracked_p_typed!(crate::tables::quest_info::QuestInfo),
        "item_use_info" => tracked_p_typed!(crate::tables::item_use_info::ItemUseInfo),
        "ai_dialog_string_info" => tracked_p_typed!(crate::tables::ai_dialog_string_info::AIDialogStringInfo),
        "frame_event_attr_group_info" => tracked_p_typed!(crate::tables::frame_event_attr_group_info::FrameEventAttrGroupInfo),
        "stage_info" => tracked_p_typed!(crate::tables::stage_info::StageInfo),

        // Tables still blocked on BinaryReadTracked propagation through
        // nested helper types. Each compile error names the offending
        // helper struct; the fix is a hand-rolled BinaryReadTracked
        // impl matching the pattern in
        // src/binary/variants/gimmick_interaction_override.rs:215+.
        //
        // Blocked: character_info, character_change_info, inventory_info,
        // buff_info, condition_info, drop_set_info, effect_info,
        // gimmick_info, interaction_info, store_info, faction_node_spawn_info,
        // quest_info, item_use_info, ai_dialog_string_info,
        // frame_event_attr_group_info, stage_info.
        //
        // Hybrid converter (parse_table_to_json + record-diff) handles
        // these mods without tracked support — only v2-byte-patch
        // conversion is blocked.

        // ── Tables blocked on missing BinaryReadTracked impls ──────
        //
        // The following are wired in dispatch.rs::parse_table_to_json
        // but their tracked variant errors out at compile time because
        // one or more nested struct types (custom CArray subtypes
        // mostly) lack a BinaryReadTracked impl. py_binary_struct!
        // emits the impl on the OUTER struct only when every inner
        // field type already has it — and the polymorphic-variant
        // helpers (Conditional92, GimmickInteractionOverrideCArray,
        // etc.) currently impl BinaryRead alone.
        //
        // To unblock each table: add a BinaryReadTracked impl to the
        // helper types in that table's mod. The impl is mechanical —
        // wraps each read_from in push_path/pop_path and forwards to
        // BinaryReadTracked::read_tracked on the inner type. See
        // tables/character_info/info.rs:378 (Conditional92) for the
        // shape — that struct's BinaryReadTracked is already
        // hand-written, so use it as the template.
        //
        // Tables waiting for trait impls (sorted by mod-corpus
        // priority, highest first):
        //
        //   inventory_info        — BagSpace, Storage 2000, More_Inventory_JSON
        //   character_info        — UniEquip, Crewny23 Runtime Packages
        //   character_change_info — FemaleAnimations
        //   faction_node_info     — Let_Me_Repeat (dispatch periods)
        //   condition_info        — buff/condition byte-patches
        //   buff_info             — buff value mods
        //   effect_info           — visual effect mods
        //   equip_info            — gear stat mods
        //   faction_info          — faction reputation mods
        //   gimmick_info          — interactable behaviour mods
        //   gimmick_group_info    — gimmick set mods
        //   interaction_info      — context-action mods
        //   store_info            — vendor inventory mods
        //   elemental_material_info — refining mods
        //   special_mode_info     — game-mode mods
        //   faction_node_spawn_info — spawn density mods
        //   knowledge_info        — collectible mods
        //   mission_info          — quest mods
        //   quest_info            — quest dialog mods
        //   item_use_info         — consumable behaviour mods
        //   multi_change_info     — transmute recipe mods
        //   ai_dialog_string_info — NPC dialogue mods
        //   frame_event_attr_group_info — animation event mods
        //
        // All these tables CAN be converted via the hybrid path
        // (whole-record diff via parse_table_to_json — works today)
        // but NOT via the v2-byte-patch path until BinaryReadTracked
        // is propagated through their helper types.
        other => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                format!(
                    "no tracked parser wired for table '{}' (only iteminfo today; \
                     other tables blocked on BinaryReadTracked propagation through \
                     nested helper types — see tracked.rs comments)",
                    other,
                )));
        }
    })
}

/// True if `parse_table_tracked` will succeed for this table name.
/// Useful for the converter to decide between v2→v3 intent emission
/// (only when this returns true) and v2-fallback (everything else).
pub fn is_tracked_table(table_name: &str) -> bool {
    matches!(
        table_name,
        "iteminfo"
            | "iteminfo.pabgb"
            | "faction_node_info"
            | "equip_info"
            | "faction_info"
            | "gimmick_group_info"
            | "elemental_material_info"
            | "special_mode_info"
            | "knowledge_info"
            | "mission_info"
            | "multi_change_info"
            | "tribe_info"
            | "field_info"
            | "character_info"
            | "character_change_info"
            // inventory_info: converted to blob mode — no tracked support
            | "buff_info"
            | "condition_info"
            | "drop_set_info"
            | "effect_info"
            | "gimmick_info"
            | "interaction_info"
            | "store_info"
            | "faction_node_spawn_info"
            | "quest_info"
            | "item_use_info"
            | "ai_dialog_string_info"
            | "frame_event_attr_group_info"
            | "stage_info"
    )
}
