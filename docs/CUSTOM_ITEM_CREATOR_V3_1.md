# Handover: Custom Item Creator (v3.1 Edition)

**Replaces:** `HANDOVER_CUSTOM_ITEM_CREATOR.md` (Benreuveni, 2026-04-17)
**Approach:** Field-level intents via Field JSON v3.1 + dmm-parser's typed `item_info` module
**Effective Date:** 2026-05-01
**License:** CDMTL v1.0

> **Why this rewrite:** Benreuveni's original implementation patches binary blobs and "echo keys" by hand. dmm-parser already exposes those as typed fields (`item_name: LocalizableString`, `item_desc: LocalizableString`). v3.1 intents can express the entire custom-item workflow as data — no binary patching, survives game updates, ships as a single `.field.json` file.

---

## 1. dmm-parser Coverage Map (What We Have vs What We Need)

| Format | Used For | Status |
|---|---|---|
| `iteminfo.pabgb` | Item definitions (key, name, stats, etc.) | ✅ Fully field-decoded — typed via `dmm_parser::item_info` module + `dispatch.rs` |
| `store_info.pabgb` | Vendor inventory / store listings | ✅ Typed in dmm-parser dispatch — Benreuveni's storeinfo failure can be retested via typed serializer |
| `drop_set_info.pabgb` | Item drop tables | ✅ Fully writable via dispatch |
| `buff_info.pabgb` | Buff/debuff effects (all 120 BuffData variants) | ✅ Fully writable |
| `equip_info.pabgb` / `equip_slot_info.pabgb` | Equipment configuration | ✅ Fully writable |
| `gimmick_info.pabgb` | Gameplay gimmicks | ✅ Recently expanded (alt_body 1536 bytes) |
| `paloc.pamt` | Item names/descriptions (localization) | ✅ **Added in P6-P8** — `parse_paloc_to_json` / `serialize_paloc_from_json` + Python bindings + dispatch integration. Field-level localization editing now possible via v3.1 intents. |
| `.dds` textures | Texture replacement | ⚠️ Out of scope for typed parsing; binary asset, handled by DMM's injection pipeline |
| `.wem` / `.bnk` audio | Audio replacement | ⚠️ Out of scope; binary asset, handled by DMM's injection pipeline |
| `.save` files | Save editor target (item swap into save) | ⚠️ Handled by DMM's `save_engine` module, not dmm-parser |
| `.papgt` (PackGroupTree) | Overlay registration | ✅ dmm-parser has this in `binary/papgt.rs` |
| `.pamt` (Pack Metadata) | Group metadata | ✅ dmm-parser has this in `binary/pamt.rs` |
| `.paz` archives | Container format | ✅ dmm-parser has this via `PackGroupBuilder` |

**Verified facts (2026-05-01):**
- 122 PABGB tables in `dispatch.rs` — every one supports both parse and serialize
- iteminfo additionally exposed via `pub mod item_info` with dedicated `parse_iteminfo_to_json` / `serialize_iteminfo_from_json`
- 308 tests pass; round-trip byte-perfect verified across all tables

**The gap:** Paloc localization. Everything else needed for the custom-item workflow is already typed and writable.

---

## 2. The Echo Key Mystery — Solved

Benreuveni's original handover described "echo keys" as critical magic markers:

> Echo key pattern (critical for game name lookup):
> ```
> byte 0x07 + u32 0x70 + u32 item_key  → name lookup
> byte 0x07 + u32 0x71 + u32 item_key  → description lookup
> ```

Looking at dmm-parser's `LocalizableString` type (`src/binary/types.rs:224`):

```rust
pub struct LocalizableString<'a> {
    pub category: u8,        // ← Benreuveni's "0x07 marker"
    pub index: u64,          // ← (item_key << 32) | 0x70 or 0x71
    pub default: CString,
}
```

And in `ItemInfo` (`src/item_info/item.rs`):

```rust
pub struct ItemInfo<'a> {
    pub key: ItemKey,
    pub item_name: LocalizableString<'a>,    // his "echo key 0x70" path
    pub item_desc: LocalizableString<'a>,    // his "echo key 0x71" path
    pub item_desc2: LocalizableString<'a>,
    // ... 60+ more typed fields
}
```

**Translation:** the "echo keys" are simply the `index` field of `LocalizableString`. dmm-parser exposes them as named, typed fields. No binary marker hunting required.

---

## 3. The v3.1 Approach — Custom Items as Data

### 3.1 Required New Intent Type: `clone_record`

v3.1 currently supports field-level **patches** to existing records. To create custom items it needs ONE new intent type that clones an existing record under a new key, then patches the clone.

**Schema extension** (additive — does NOT bump format_minor; v3.1 still says `format_minor: 1`):

```json
{
  "op": "clone_record",
  "source_key": 12345,
  "new_key": 999001,
  "patches": [
    { "path": "field.name", "new": <value> },
    ...
  ]
}
```

Plus optional siblings for symmetry:

| Intent op | Purpose |
|---|---|
| `clone_record` | Duplicate existing record under new key, then patch |
| `new_record` | Create record from scratch (advanced) — caller provides full field tree |
| `delete_record` | Remove a record by key (rare but useful) |
| (default) | Field-level patch on existing record (current v3.1 behavior) |

### 3.2 Sample Custom Item Mod

```json
{
  "format": 3,
  "format_minor": 1,
  "modinfo": {
    "title": "999K Damage Sword",
    "author": "RicePaddySoftware",
    "version": "1.0",
    "description": "Custom OP weapon cloned from Bale's Sword",
    "category": "custom_item"
  },
  "targets": [
    {
      "target": "iteminfo.pabgb",
      "intents": [
        {
          "op": "clone_record",
          "source_key": 12345,
          "new_key": 999001,
          "patches": [
            { "path": "item_name.default", "new": "999K Damage Sword" },
            { "path": "item_name.index",   "new": 4290772592 },
            { "path": "item_desc.default", "new": "Hits like a truck" },
            { "path": "item_desc.index",   "new": 4290772593 },
            { "path": "enchant_data_list[0].enchant_stat_data.stat_list_static[0].value", "new": 999000 },
            { "path": "enchant_data_list[0].enchant_stat_data.stat_list_static[1].value", "new": 50000 }
          ]
        }
      ]
    }
  ]
}
```

**Paloc index formula** (preserved from Benreuveni's research):
- `item_name.index = (new_key << 32) | 0x70` → `(999001 << 32) | 0x70 = 4290772592`
- `item_desc.index = (new_key << 32) | 0x71` → `(999001 << 32) | 0x71 = 4290772593`

SWISS Stacker's UI computes these automatically when the user picks a `new_key`.

### 3.3 What This Replaces from Benreuveni's Pipeline

| Benreuveni's Step | v3.1 Equivalent |
|---|---|
| `clone_item(rust_items, donor_key, new_key, new_name)` | `op: "clone_record"` intent |
| Patch echo keys (0x70/0x71 markers in binary) | `path: "item_name.index"` and `path: "item_desc.index"` patches |
| `find_next_free_key(start=999001)` | SWISS UI helper that scans parsed iteminfo, finds max key, +1 |
| `compute_paloc_ids(item_key)` | SWISS UI computes `(key << 32) \| 0x70` for the JSON output |
| `build_modded_iteminfo(vanilla_body, vanilla_head, new_items)` | DMM's apply path: parse → mutate → serialize via dmm-parser |
| Stat modification via `enchant_data_list[0]...` | `path: "enchant_data_list[0].enchant_stat_data.stat_list_static[N].value"` patch |
| Deploy as 0058 overlay folder | Deploy as `dmmv3` overlay (DMM's standard v3.1 slot) |

---

## 4. DMM Implementation Plan

### 4.1 Extend `field_json_v3.rs` Intent Schema

**File:** `dmm-api-test/src-tauri/src/iteminfo/field_json_v3.rs`

Add to the intent enum:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op")]
pub enum IntentOp {
    // Existing — field-level patch on existing record (current v3.1)
    #[serde(rename = "patch")]
    Patch { /* existing intent fields */ },

    // NEW — clone existing record under new key, then patch
    #[serde(rename = "clone_record")]
    CloneRecord {
        source_key: u32,
        new_key: u32,
        patches: Vec<Patch>,
    },

    // NEW — create record from scratch
    #[serde(rename = "new_record")]
    NewRecord {
        new_key: u32,
        template: serde_json::Value,   // full field tree
    },

    // NEW — remove record by key
    #[serde(rename = "delete_record")]
    DeleteRecord {
        key: u32,
    },
}
```

**Backward compat:** intents without an explicit `op` field default to `"patch"` (current v3.1 behavior).

### 4.2 Extend `apply_v3_to_typed_table_body` Dispatcher

```rust
pub fn apply_v3_to_typed_table_body(
    body: &[u8],
    pabgh: Option<&[u8]>,
    table_name: &str,
    v3_mods: &[(String, Vec<Intent>)],
) -> Result<(Vec<u8>, Vec<(String, V3ApplyReport)>), String> {
    // Parse the table to JSON
    let mut records: Vec<serde_json::Value> = dmm_parser::parse_table_to_json(table_name, body, pabgh)?;

    // Index by key for fast lookup
    let mut by_key: HashMap<u32, usize> = records.iter().enumerate()
        .filter_map(|(i, r)| r.get("key").and_then(|k| k.as_u64()).map(|k| (k as u32, i)))
        .collect();

    for (mod_id, intents) in v3_mods {
        for intent in intents {
            match &intent.op {
                IntentOp::Patch { .. } => apply_patch_intent(&mut records, &by_key, intent)?,

                IntentOp::CloneRecord { source_key, new_key, patches } => {
                    let src_idx = by_key.get(source_key)
                        .ok_or_else(|| format!("clone_record: source_key {} not found", source_key))?;
                    let mut clone = records[*src_idx].clone();
                    set_field(&mut clone, "key", json!(*new_key))?;
                    for p in patches {
                        apply_patch_to_value(&mut clone, p)?;
                    }
                    records.push(clone);
                    by_key.insert(*new_key, records.len() - 1);
                }

                IntentOp::NewRecord { new_key, template } => {
                    let mut new = template.clone();
                    set_field(&mut new, "key", json!(*new_key))?;
                    records.push(new);
                    by_key.insert(*new_key, records.len() - 1);
                }

                IntentOp::DeleteRecord { key } => {
                    if let Some(idx) = by_key.remove(key) {
                        records.remove(idx);
                        // rebuild by_key index after removal
                        by_key = records.iter().enumerate()
                            .filter_map(|(i, r)| r.get("key").and_then(|k| k.as_u64()).map(|k| (k as u32, i)))
                            .collect();
                    }
                }
            }
        }
    }

    // Serialize back
    let new_body = dmm_parser::serialize_table_from_json(table_name, &records)?;
    Ok((new_body, vec![/* reports */]))
}
```

### 4.3 PABGH Index Rebuild

For pabgh-bounded tables (iteminfo, etc.), adding/removing records changes the entry count. DMM already has `build_pabgh_for_modified_body` for this; ensure `apply_v3_for_target` calls it after `clone_record`/`new_record`/`delete_record` operations on pabgh-bounded tables.

### 4.4 Tests

- Unit: clone_record produces a new record with new key, source unchanged
- Unit: clone_record + patches applies all patches to the clone
- Unit: new_record adds record from template
- Unit: delete_record removes record and rebuilds index
- Integration: round-trip a clone-record mod (parse → clone → serialize → re-parse → verify)
- Integration: pabgh entry count matches records.len() after mutations

### 4.5 Estimated Effort

~150-200 lines of new code in `field_json_v3.rs`, plus ~5 unit tests. Half a day to a day of focused work.

---

## 5. SWISS Implementation Plan

### 5.1 New "Custom Item" Tab or Dialog

**File:** `CrimsonGameMods/gui/dialogs/item_creator_dialog.py` (new) — or extend the existing Stacker tab with a "Create Item" button.

**UI matches Benreuveni's mockup** (preserved from original handover §2):

```
+--[ Create Custom Item ]--------------------------------------------+
|  [Donor Item: _________________ v]  [Icon 96x96]                   |
|  +--- LIVE PREVIEW ---+  +--- EDIT FORM ----------------+         |
|  | Bale's Sword       |  | Name: [________________]      |         |
|  | Legendary | Weapon |  | Description: [__________]    |         |
|  | Attack 500         |  | Key: [999001] (auto-next)    |         |
|  | Defense 200        |  | Attack: [__500__]            |         |
|  | ...                |  | Defense: [__200__]           |         |
|  +--------------------+  +------------------------------+         |
|  [Create Item]                                  [Cancel]           |
+--------------------------------------------------------------------+
```

### 5.2 Replace `item_creator.py` Binary Logic With v3.1 Generator

Instead of Benreuveni's `clone_item()` that copies binary blobs, build a v3.1 intent:

```python
def build_v3_clone_intent(donor_key: int, new_key: int, name: str, description: str,
                          stats: dict[str, float]) -> dict:
    """Produce a v3.1 clone_record intent with patches for name/desc/stats."""
    name_id = (new_key << 32) | 0x70
    desc_id = (new_key << 32) | 0x71

    patches = [
        {"path": "item_name.default", "new": name},
        {"path": "item_name.index", "new": name_id},
        {"path": "item_desc.default", "new": description},
        {"path": "item_desc.index", "new": desc_id},
    ]

    # Map UI stat fields to enchant_data_list paths
    for i, (stat_field, value) in enumerate(stats.items()):
        patches.append({
            "path": f"enchant_data_list[0].enchant_stat_data.stat_list_static[{i}].value",
            "new": value,
        })

    return {
        "op": "clone_record",
        "source_key": donor_key,
        "new_key": new_key,
        "patches": patches,
    }


def export_custom_item_v3_1(items: list[dict], output_path: Path):
    """Write a v3.1 .field.json containing clone_record intents for each item."""
    doc = {
        "format": 3,
        "format_minor": 1,
        "modinfo": {
            "title": items[0].get("title", "Custom Items"),
            "author": "CrimsonGameMods Custom Item Creator",
            "version": "1.0",
            "category": "custom_item",
        },
        "targets": [
            {
                "target": "iteminfo.pabgb",
                "intents": [build_v3_clone_intent(**i) for i in items],
            }
        ],
    }
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(doc, f, indent=2, ensure_ascii=False)
```

### 5.3 Auto-Key Assignment

```python
def find_next_free_key(parsed_items: list[dict], start: int = 999001) -> int:
    """Find the smallest key >= start that doesn't conflict with any existing item."""
    used = {i["key"] for i in parsed_items}
    k = start
    while k in used:
        k += 1
    return k
```

Uses dmm-parser's parse output directly; no binary scanning needed.

### 5.4 Live Preview

The preview card pulls fields from the donor's parsed item, applies the user's pending patches in-memory, and renders. Same logic as `_buff_preview_item()` in `buffs_v319.py:3951`. No binary involved.

### 5.5 Estimated Effort

~300-500 lines (UI + dialog + v3.1 generator). 1-2 days.

---

## 6. The Paloc Gap — Three Options

The custom item won't display its custom name in-game without a paloc entry. Three paths:

### Option A — Add paloc parser to dmm-parser (recommended, ~1-2 days)

Benreuveni's `lib/paloc.py` is 118 lines and documents the format:
- 8-byte marker `07 00 00 00 00 00 00 00`
- u32 `key_len`
- key UTF-8 bytes
- u32 `value_len`
- value UTF-8 bytes
- (optionally encrypted at flags=0x0032 with ChaCha20+LZ4)

Port to Rust as `dmm_parser::paloc::*` module. Once typed, paloc becomes another v3.1 target:

```json
{
  "target": "paloc.pamt",
  "intents": [
    { "op": "set_localization", "key": 4290772592, "lang": "en", "value": "999K Damage Sword" },
    { "op": "set_localization", "key": 4290772593, "lang": "en", "value": "Hits like a truck" }
  ]
}
```

DMM's apply path mutates paloc the same way it mutates iteminfo. End user gets the custom name without manual paloc patching.

### Option B — Sidecar paloc patcher (interim, ~half day)

SWISS exports the v3.1 `.field.json` AND a `paloc_patches.json` sidecar. DMM detects the sidecar and runs its existing paloc patching tools. Less elegant but ships sooner.

### Option C — Embed paloc strings in the v3.1 mod (~1 day)

Define a v3.1 intent type `paloc_string` that DMM resolves at apply time using existing paloc tooling:

```json
{
  "target": "paloc.pamt",
  "intents": [
    { "op": "paloc_string", "id": 4290772592, "lang": "en", "value": "999K Damage Sword" }
  ]
}
```

Internally DMM maps this to its existing paloc patcher. Cleaner UX than B, simpler than A.

**Recommendation:** Start with C (works immediately, single .field.json file), then promote to A once paloc parser lands in dmm-parser.

---

## 7. Save Editor Coordination

Benreuveni's pipeline required users to:
1. Mount the iteminfo overlay
2. Open the save editor → Repurchase tab
3. Manually swap a junk item to the new key
4. Re-launch the game and buy back from the vendor

The "swap" step can be automated via DMM's `save_engine` module. v3.1 multi-target lets the same mod include a save patch:

```json
{
  "targets": [
    { "target": "iteminfo.pabgb", "intents": [{ "op": "clone_record", ... }] },
    {
      "target": "save",
      "intents": [
        { "op": "swap_item_in_save", "from_key": 12345, "to_key": 999001 }
      ]
    }
  ]
}
```

DMM's save_engine applicator handles the save mutation. User mounts the mod, launches the game, custom item is in their inventory.

**Caveat:** save mutation is destructive — DMM should require explicit user confirmation before applying save-target intents, with auto-backup of the save file first.

---

## 8. Storeinfo Re-Test (Benreuveni's Failed Path)

His handover reported:

> Store injection via storeinfo overlay (0060): Adding new key to storeinfo didn't show up in vendor. Likely a storeinfo binary format issue with our add_item() insertion.

dmm-parser has `store_info` typed and writable in dispatch. Re-test using v3.1:

```json
{
  "target": "store_info.pabgb",
  "intents": [
    {
      "op": "patch",
      "target": { "type": "field", "key": <vendor_id>, "path": "items[]" },
      "append": { "item_key": 999001, "price": 1000, "stock": -1 }
    }
  ]
}
```

(Schema for "append to array" intent doesn't exist yet — would need a small extension. Simpler interim: use `clone_record` on the vendor's StoreInfo entry, patch its items array.)

If this works, vendors can sell custom items WITHOUT needing the save-editor swap workflow.

---

## 9. Migration Path From Benreuveni's Tool

Existing users of `crimson-desert-add-item-main`:

1. Old tool's compiled overlays (folder mods with `0058/0.paz`) continue to mount via DMM's legacy overlay path. No breakage.
2. New custom items should be authored via SWISS Stacker's v3.1 workflow.
3. Old items can be re-exported from existing overlays into v3.1 form by:
   - Loading the overlay's iteminfo via dmm-parser
   - Diffing against vanilla iteminfo
   - Generating `clone_record` intents for any items with key >= 999001
   - Writing the v3.1 .field.json

A one-shot migration script (`migrate_custom_items_to_v3_1.py`) would automate this.

---

## 10. File Locations

| File | Role |
|---|---|
| `dmm-api-test/src-tauri/src/iteminfo/field_json_v3.rs` | Add `clone_record` / `new_record` / `delete_record` intent types + apply logic |
| `CrimsonGameMods/gui/dialogs/item_creator_dialog.py` | New dialog UI |
| `CrimsonGameMods/item_creator_v3.py` | v3.1 intent generator (replaces Benreuveni's binary `item_creator.py`) |
| `CrimsonGameMods/migrate_custom_items_to_v3_1.py` | Optional one-shot migration script |
| `dmm-parser/src/paloc/` (NEW, optional Phase 2) | Paloc parser if pursuing Option A |

---

## 11. Estimated Total Effort

| Phase | Work | Time |
|---|---|---|
| 1 | Add `clone_record` intent to v3.1 spec + DMM apply logic | 1 day |
| 2 | SWISS dialog + v3.1 generator | 1-2 days |
| 3 | Paloc handling (Option C — embedded intents, sidecar tooling) | 1 day |
| 4 | Storeinfo re-test using typed serializer | half day |
| 5 | End-to-end testing (donor → custom item → in-game verify) | 1 day |
| 6 | Optional: paloc parser in dmm-parser (Option A) | 1-2 days |
| 7 | Optional: save editor coordination via v3.1 save target | 1 day |
| **Core (1-5)** | **Replaces Benreuveni's tool with cleaner v3.1 flow** | **~5 days** |

---

## 12. Open Questions

1. Should `clone_record` intents support cross-table cloning (e.g., clone iteminfo entry AND its drop_set entries)? Probably v3.2 territory.
2. Save editor confirmation UX — single dialog "this mod modifies your save, continue?" or per-mod opt-in?
3. Should custom items have a discoverable manifest (e.g., `custom_items.json` listing all custom keys) so the save editor can browse them?
4. Storeinfo schema — does the typed serializer match the game's expected format? Re-test confirms or denies.
5. Trademark on "DMM Custom Item Creator" — register or skip?

---

## 13. References

- **Original handover:** `C:/Users/corin/Desktop/ZIPS/HANDOVER_CUSTOM_ITEM_CREATOR.md` (Benreuveni, 2026-04-17)
- **dmm-parser ItemInfo struct:** `src/item_info/item.rs`
- **dmm-parser LocalizableString:** `src/binary/types.rs:224`
- **dmm-parser dispatch (122 tables):** `src/dispatch.rs`
- **DMM v3 apply path:** `dmm-api-test/src-tauri/src/iteminfo/field_json_v3.rs`
- **SWISS Stacker:** `CrimsonGameMods/gui/tabs/stacker.py`
- **Field JSON v3.1 Spec:** `FIELD_JSON_V3_1_SPEC.md` (in CrimsonGameMods repo)
- **Benreuveni's source:** `crimson-desert-add-item-main/lib/iteminfo.py`, `lib/paloc.py`, `lib/stats.py`

---

*End of Custom Item Creator v3.1 Handover.*
