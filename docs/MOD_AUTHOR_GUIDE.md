# Crimson Desert Mod-Author Guide

> **Audience.** People writing mods for Crimson Desert using the
> Field-JSON v3 / v3.1 manifest format. Goal: ship a mod that SWISS
> Stacker can load + verify, and that other mods can compose with.
>
> This is the user-facing entry point. For the binary formats themselves
> see `docs/BINARY_FORMATS.md#file-format-reference-formats`. For the Python API surface see `docs/api.md`.

---

## Contents

- [0. The big picture](#0-the-big-picture)
- [0.5. File extension reference](#05-file-extension-reference)
- [1. Picking a mod type](#1-picking-a-mod-type)
- [2. Data mods (table targets)](#2-data-mods-table-targets)
- [3. Texture mods (DDS asset targets)](#3-texture-mods-dds-asset-targets)
- [4. Audio mods (WEM / BNK asset targets)](#4-audio-mods-wem--bnk-asset-targets)
- [5. Paloc mods (localization)](#5-paloc-mods-localization)
- [6. Custom items](#6-custom-items)
- [7. Mixed mods](#7-mixed-mods)
- [8. The author workflow](#8-the-author-workflow)
- [9. Distribution](#9-distribution)
- [10. Common pitfalls](#10-common-pitfalls)
- [11. Where to look next](#11-where-to-look-next)
- [12. Sequencer / schedule / attack-info mods](#12-sequencer--schedule--attack-info-mods-tier-1-formats)
- [13. Format-internal gotchas (recent discoveries)](#13-format-internal-gotchas-recent-discoveries)

**Companion docs:**

- [`docs/BINARY_FORMATS.md#file-format-reference-formats`](BINARY_FORMATS.md#file-format-reference-formats) — every binary format reference in one place
- [`docs/api.md`](api.md) — Python API surface (classify_dds / parse_bnk / paloc / save / etc.)
- [`docs/CUSTOM_ITEM_CREATOR_V3_1.md`](CUSTOM_ITEM_CREATOR_V3_1.md) — end-to-end custom-item walkthrough
- [`samples/`](../samples/) — five runnable example mods (data, texture, audio, custom item, mixed)

---

## 0. Authoring mods against canonical Pearl Abyss field names (v3.1 surface)

**Last refreshed 2026-05-10 — added Session 28 + 1-min-loop findings.**

> **TL;DR for mod authors:** dmm-parser now ships a *canonical-name surface*
> alongside the snake_case names you've been using. Pass `shape="v3.1"` to
> `parse_table` / `serialize_table` to receive Pearl Abyss's real
> `_camelCase` field identifiers (e.g. `_cooltime`, `_skeletonName`,
> `_buffLevelList`). v3 (default) is unchanged — every existing mod still
> round-trips byte-perfect.

### Why v3.1 exists

The original v3 mod surface used dmm-parser's snake_case Rust struct field
names (`cooltime`, `skeleton_name`). Those names were derived from PA's
internal Korean error strings during the Tier 1 promotion arc — they're
real C++ identifiers with the convention flipped. Schema verification
across 109 of 122 *Info tables (commit `2724abe`) confirmed every shipped
mechanical translation matches Pearl Abyss's canonical name verbatim.

### Coverage at a glance

| Table category | v3.1 surface | Source |
|---|---|---|
| 109 *Info pabgb tables | ✅ schema-verified canonical names | NattKh's `pabgb_complete_schema.json` (Korean-error-string extraction) |
| 13 fallback *Info tables | 🟡 mechanical translation only | snake → camelCase by convention; not in NattKh schema (4 named tables + 9 zero-field tables) |
| 27 closures shipped | ✅ verified during 1-min loop | covers acronym-casing, plural divergences, PA-internal typos like `_questGroupkey` (lowercase k) |
| `iteminfo.pabgb` | 🟡 v3.1 NOT yet wired (separate `src/item_info/` module) | future-work item — track in `_BREAKDOWN_WORKPLAN.md` |
| Reflection-format files (`.prefab` etc.) | 📚 catalogued via pycrimson, not natively parsed | 231 PA reflection classes / 938 fields harvested into `docs/v3_1_reflection_schema.json` |

### Picking a shape

Both shapes round-trip identically. The difference is the field-name
surface you author against:

```python
# v3 (default) — what every existing mod uses
items = dmm_parser.parse_table("skill_info", pabgb, pabgh)
items[0]["cooltime"] = 30
items[0]["skeleton_name"] = "_kr_human_a_v01"

# v3.1 — canonical Pearl Abyss names
items = dmm_parser.parse_table("skill_info", pabgb, pabgh, shape="v3.1")
items[0]["_cooltime"] = 30
items[0]["_skeletonName"] = "_kr_human_a_v01"

# Either name set is accepted on input regardless of shape
raw = dmm_parser.serialize_table("skill_info", items, shape="v3.1")
```

### When to use v3.1

- **Prefer v3.1** if you're authoring new mods and want forward-stable
  field names that match Pearl Abyss's own internal naming. These are the
  identifiers you'd see in a (hypothetical) PaWorks editor.
- **Stay on v3 (default)** if you're maintaining an existing mod or have
  tooling that assumes snake_case. Nothing changes.

### Mixed-shape input is accepted

You can author with EITHER snake_case OR _camelCase keys (or mix them
within the same item) — the parser accepts both regardless of the
`shape` parameter. The `shape` only controls the EMIT side:

```python
import dmm_parser, json

# Mix of v3 (snake) and v3.1 (camel) in the same dict — both accepted
edits = [
    {
        "key": 12345,
        "_cooltime": 30,            # v3.1-style
        "skeleton_name": "...",     # v3-style
        "_isBlocked": 0,            # v3.1-style
    },
]

# Emit shape controls output:
v3_bytes = dmm_parser.serialize_table("skill_info", edits)              # snake_case keys
v31_bytes = dmm_parser.serialize_table("skill_info", edits, shape="v3.1")  # _camelCase keys

# Round-trip: parse(serialize(items)) == items at byte level
items = dmm_parser.parse_table("skill_info", v31_bytes, pabgh, shape="v3.1")
```

This means you can MIGRATE an existing mod from v3 → v3.1 by simply
re-serializing with `shape="v3.1"`. Or upgrade individual entries by
just changing the key names you write.

### Canonical-field reference

Every *Info table now has a **canonical-field catalog comment block** at the
top of its `info.rs` file (e.g. `src/tables/skill_info/info.rs`). The block
lists every Pearl Abyss canonical name for that table with ✅ (decoded by
dmm-parser) or ⏳ (in canonical schema but not yet exposed). Quick lookup
without leaving the source. See `docs/V3_1_DECODER_GAPS.md` for the
398-field decoder gap worklist (with per-table Win-IDA parser-fn pointers
when an implementer wants to close gaps).

### PA-internal typos (preserved as canonical)

Some Pearl Abyss field names have typos (lowercase k in "key", missing 'e'
in "Frequency", etc.). The v3.1 surface preserves them as-is per NattKh's
schema. If you hit one of these in your mod and want to verify it's
intentional, see `docs/V3_1_DECODER_GAPS.md` § "Auto-closure analysis":

- `_questGroupkey`, `_regionEnterknowledgeInfoList` — lowercase k
- `_fishSummonTimeFrquencyType` — missing 'e' (Frquency)
- `_radgollEquipTableGroupDataList` — radgoll vs ragdoll
- `_collectFilter_Dev`, `_wayPointDataList_deprecated` — mid-name underscores
- `_complteDescription` (multi_change_info) — missing 'e' (complete)
- `_overriedMaxHeight` (region_info) — missing 'd' (overridden/overrided)

### Residual v3.1 surface coverage (as of 2026-05-10, iter 144)

Of 126 dmm-parser tables, **109 are in NattKh's canonical schema**. After
the resumed-loop closure work (iters 70-143):

- **90 tables** — 100% canonical coverage (every `_camelCase` aliased,
  shape='v3.1' is a drop-in for snake_case).
- **19 tables** — still have ≥1 gap. Total residual gaps: **460**.

Progress this loop (iter 84 → 144): +4 fully-covered tables, -86 residual
gaps. 4 class-5 tables fully closed: global_game_event_group_info (iter 96),
level_gimmick_scene_object_info (iter 97), mission_info (iter 120, 7 closures),
tribe_info (iter 123, 25 closures). Plus deep-progress on faction_node_info
(0% → 90%, 11 closures) and field_info (8% → 79%, 17 closures).

The 19 remaining tables fall into three classes (class 3 closed iters 96-97):

1. **1-to-N wrap, pending alias-mechanism extension** (~10 tables).
   The current `FIELD_ALIASES_V3_1` mechanism only maps 1 snake → 1 camel.
   These tables have one canonical that wraps multiple unrolled rust fields
   (e.g. `_relationTypeList` ↔ `relation_type_list_0..6`,
   `_destroyedAiEvent` ↔ four `destroyed_ai_event_*` fields). Closure is
   purely cosmetic — wire bytes already round-trip identically. Resolution
   is to extend the alias mechanism to support `(snake, &[snake...])`
   1-to-N tuple-keyed entries (per `docs/V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md`).

2. **Real decoder work needed / sub-struct decomposition** (~6 tables, ~50 gaps).
   Notably `global_game_event_info` (3 gaps: `_eventDesc`, `_uiIconPath`,
   `_targetRegionInfoList`) where the current `execute_data` polymorphic
   wrapper absorbs 3 separate canonical reads as a single typed field.
   Plus `interaction_info` (28 gaps inside InteractionTailDecoded sub-struct),
   field_info (5 gaps inside FieldInfoComposite), faction_node_info residual
   (3 gaps inside big_composite_slots/de690_data sub-structs), action_point_info
   (2 gaps inside ActionPoint sub-struct). Closure requires decomposing
   sub-structs into top-level rust fields OR extending the alias mechanism
   to support nested field paths.

3. **Larger un-audited tables** (4 tables holding 411 of 460 gaps =
   89% of remaining): `gimmick_info` (153 gaps, Tier-1.5 typed-prefix +
   opaque blob), `character_info` (146 gaps, 8.7KB per-record reader),
   `gimmick_group_info` (45 gaps, interleaved wire layout per iter 118),
   `stage_info` (68 gaps, 3.5KB reader). Each needs its own audit pass
   per `docs/V3_1_REMAINING_GAPS_MASTER_PLAN.md`.

Day-to-day mods are unaffected. If your mod targets one of the 90 fully-
covered tables, both `shape='v3.1'` and the snake_case default round-trip
identically. The residual gaps only matter if you author against canonical
names for one of the 19 partially-covered tables — and even there, the
snake_case rust names still work as input.

---

## 0. The big picture

A Crimson Desert mod is a single **manifest** (`*.field.json`) that
describes a set of **targets** — the things the mod changes. Each
target is one of:

| Kind   | Touches                                                            |
|--------|--------------------------------------------------------------------|
| `table` | Game data (items, skills, NPCs, …) parsed from `pabgb` archives  |
| `asset` | Binary asset files: `.dds` textures, `.wem`/`.bnk` audio         |
| `paloc` | Localization strings (`*.paloc`)                                 |

A v3.1 manifest is just JSON with a list of targets:

```jsonc
{
  "format": 3,
  "format_minor": 1,
  "name": "Macduff Voice Pack",
  "author": "potter",
  "version": "1.0.0",

  "targets": [
    { "kind": "asset", "asset_type": "wem",
      "file": "voice/macduff_attack01.wem",
      "vpath": "0006/sound/windows/english(us)/3684722581.wem" },

    { "kind": "table", "table": "ItemInfo",
      "ops": [ { "key": "id_4242", "field": "max_count", "value": 99 } ] }
  ]
}
```

When SWISS Stacker loads this manifest it:

1. **Validates** every target (`dmm-mod-validate`)
2. **Diffs** it against every other enabled mod (`dmm-mod-diff`)
3. Compiles the deltas into the runtime archive overlay

---

## 0.5. File extension reference

Every Crimson Desert asset uses one of the file types below.

**Naming convention.** `pa*` extensions are all engine-internal — `pa`
stands for **Pearl Abyss** (confirmed by namespace symbols all over the
binary: `pa::StaticInfoWrapper<...>`, `pa::convertToStringList<...>`,
plus debug strings like `PearlAbyssEngine.Debug.PauseFrameIndex` and
`PearlAbyssEngine.Debug.PauseCountResourceBinding`). The non-`pa*`
formats (`.dds`, `.wem`, `.bnk`, `.css`, `.xml`) are
industry-standard containers Crimson uses unmodified at the byte level.

The tables below are split by category so you can jump to the section
relevant to your mod.

### 0.5.1 Infrastructure (overlays / archives / registries)

These wrap or register everything else. Touch only when shipping a mod.

| Ext | Stands for | What it actually is | Modding context |
|---|---|---|---|
| `.paz` | **P**earl **A**byss **Z**ip | ChaCha20-encrypted, LZ4-compressed asset archive — the shipping container for raw bytes | "The bag — vanilla in `0008/`, mods overlay via numbered PAZ folders (`0035`, `0036`, `dmmv3_*`)" |
| `.pamt` | **P**earl **A**byss **M**eta **T**able | Per-PAZ index — directory entries with file offsets, decompressed sizes, flags | "Sister file to a `.paz`; patch when you inject a file. **Header CRC at bytes 0–3**" |
| `.papgt` | **P**earl **A**byss **P**ack **G**roup **T**ree | Master registry of overlay groups — game iterates entries, last-claim wins | "`meta/0.papgt` is THE file. Adding `dmmv3_iteminfo` (etc.) here is what makes the game load your overlay. **Header CRC at bytes 4–7**" |
| `.paver` | **P**earl **A**byss **Ver**sion | Game build version stamp at `meta/0.paver` | "Used for vanilla-drift detection — never modify" |

### 0.5.2 Structured data tables (where most mod work happens)

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.pabgb` | **P**earl **A**byss **B**inary **G**roup **B**ody | Row data for a structured table | "The actual records — parse, mutate, write back" | Items (`iteminfo`), Characters/NPCs (`characterinfo`), World (`regioninfo`, `terrainregionautospawninfo`), Spawning (`spawningpoolautospawninfo`), Skills (`skillinfo`), Buffs (`buffinfo`), Conditions, Stage info, Drops, ~100 more |
| `.pabgh` | **P**earl **A**byss **B**inary **G**roup **H**eader | Companion key/offset index for a `.pabgb` | "Always paired with `.pabgb`. Offsets get rewritten when rows are added or reordered" | Same as paired `.pabgb` |

### 0.5.3 Cinematics & scripted gameplay

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.paseq` | **P**earl **A**byss **Seq**uencer | Cutscene / scripted-action timeline (uncompiled source) | "Walk-and-replace LP-strings — script expressions, trigger names, game event hooks" | Cinematics, scripted gameplay |
| `.paseqc` | **P**earl **A**byss **Seq**uencer **C**ompiled | Compiled `.paseq` — runtime-ready bytecode | "Same edit pattern as `.paseq`. Embedded class blocks expose the field directory" | Cinematics, scripted events |
| `.paseqh` | **P**earl **A**byss **Seq**uencer **H**eader | Sequencer stage header (`sequencerStageHeader.paseqh`) | "Master index used by the sequencer subsystem" | Cinematics |
| `.pastage` | **P**earl **A**byss **Stage** | Stage chart — timeline state-machine for boss encounters / scripted set-pieces | "Segments with start/end frames + animation targets. Tagged-field reflection" | Boss fights, scripted gameplay |

### 0.5.4 NPC behavior, AI, combat

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.paschedule` | **P**earl **A**byss **Schedule** | NPC time-of-day activity loop | "JSON-path edit `name` plus opaque body" | NPC behavior, world simulation |
| `.paschedulepath` | **P**earl **A**byss **Schedule Path** | Companion patrol / pathing data | "Sister to `.paschedule`. Don't orphan one without the other" | NPC navigation |
| `.paschedulectx` | **P**earl **A**byss **Schedule C**on**t**e**x**t | Schedule context (`scheduleContext.paschedulectx`) | "Shared schedule lookup state. Rarely modded" | NPC behavior |
| `.pai` | **P**earl **A**byss **AI** | AI charts (`aichart.pai`) and pathfinding tables (`PathFindTable.pai`) | "Behavior-tree / decision-graph data" | AI, NPC combat behavior |
| `.paatt` | **P**earl **A**byss **Att**ack | Per-weapon attack info — hitboxes, damage tables, frame events | "Frame events, damage scaling per weapon archetype. JSON-path workflow" | Combat (player + NPC weapons) |

### 0.5.5 Animation & motion

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.paa` | **P**earl **A**byss **A**nimation | Single animation clip (`Object/Animation/*.paa`) | "Standard motion clips — idle, attack, walk, etc." | Character / creature animations |
| `.paasmt` | **P**earl **A**byss **A**nimation **S**et **M**atching **T**able | Animation set lookup (`animationsetmatchingtable.paasmt`) | "Maps gameplay state → which `.paa` plays" | All animated characters |
| `.pampg` | **P**earl **A**byss **M**esh **P**roxy **G**roup | Proxy LOD mesh group | "Rarely modded — distance-LOD mesh container" | World / level optimization |
| `.pamlod` | **P**earl **A**byss **M**esh **LOD** | LOD chain for a mesh | "Distance LODs for character/world meshes" | World, characters |

### 0.5.6 Mesh / skeleton / geometry

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.pac` | **P**earl **A**byss **C**haracter (skinned mesh) | Skinned mesh — `Mesh + Bones + Bindings` | "If your model load errors with `Skinned Mesh(.pac)`, this is the file" | Characters, weapons, mounts |
| `.pac_xml` | **P**earl **A**byss **C**haracter **XML** | XML form of `.pac` (editor source) | "Source for re-exporting `.pac`" | Characters (authoring) |
| `.pab` | **P**earl **A**byss **B**ones / skeleton volume | Identity skeleton + volume data (`character/identityskeleton.pab`) | "Bone hierarchy + collision volumes" | Characters, ragdoll |
| `.pab.sockets.xml` | sockets metadata | Socket attach points on a `.pab` | "Defines where weapons/effects attach to the skeleton" | Characters (sockets) |
| `.pam` | **P**earl **A**byss **M**esh (static) | Static mesh (`object/unitsphere.pam`, `object/unitcube.pam`, world props) | "Non-animated geometry" | Props, environment |
| `.pami` | **P**earl **A**byss **M**esh **I**nfo | Mesh metadata sibling | "Pairs with `.pam` for runtime info" | Props, environment |
| `.pat` | **P**earl **A**byss **T**ree (vegetation) | Vegetation mesh / breakable tree (`tree/*.pat`, `*_breakable.pat`) | "Trees, foliage, breakable props" | World vegetation |
| `.pati` | **P**earl **A**byss **T**ree **I**nfo | Tree metadata sibling | "Pairs with `.pat`" | World vegetation |
| `.pampg` / `.pamhc` | **M**esh **P**roxy **G**roup / **M**odel **P**roperty **H**eader **C**ollection | Mesh property registries (`modelpropertyheadercollection.pamhc`) | "Engine-side property tables — rarely modded" | World, characters |
| `.pappt` | **P**earl **A**byss **P**art **P**refab **T**able | Part prefab table (`character/bin__/partprefabtable.pappt`) | "Hair / face / armor part library" | Character customization |
| `.padock` | **P**earl **A**byss **Dock**ing | Docking metadata for child gimmicks | "Defines how items attach to characters" | Equipment, items |
| `.patag` | **P**earl **A**byss **Tag** | Generic tag data (`tag.patag`) | "Engine tag lookup" | Various |
| `.pareflect` | **P**earl **A**byss **Reflect**ion | Reflection probe / cubemap metadata | "Lighting bake data" | World rendering |

### 0.5.7 Levels, world, particles

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.palevel` | **P**earl **A**byss **Level** | Level / sector data (`LevelData/<region>/*.palevel`) | "World streaming chunks" | World, regions |
| `.palevel_xml` | **P**earl **A**byss **Level XML** | Editor source for `.palevel` | "Authoring form of `.palevel`" | World (authoring) |
| `.paem` | **P**earl **A**byss **E**mitter | Particle / VFX emitter data (`*_EmitterData.paem`) | "Effect emitter definitions" | VFX, particles |

### 0.5.8 Scripts, shaders, debug

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.pacpp` | **P**earl **A**byss **C++** | Engine-side C++ script source (`PAActionChartEvent*.pacpp`, `WeatherSimulator.pacpp`) | "Compiled at build time. Modders rarely touch unless reverse-engineering behavior" | Game logic, weather, action charts |
| `.pacpph` | **P**earl **A**byss **C++** **H**eader | Header for `.pacpp` (`objectList.pacpph`) | "Symbol declarations" | Game logic |
| `.pacpp.o` | object file | Compiled `.pacpp` | "Build artifact, not source" | Game logic |
| `.padxil` | **P**earl **A**byss **DXIL** | Compiled DirectX shader bytecode | "Shader programs (vertex, pixel, compute)" | Rendering |
| `.pagputracer` | **GPU Tracer** | Per-frame GPU profiler trace | "Diagnostic / dev — view in `chrome://tracing`" | Profiling |

### 0.5.9 UI (controllers + styling)

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.css` | **C**ascading **S**tyle **S**heets | Standard CSS for in-game UI controllers (`UI/BaseControllerEditor.css`, `UI/widgets/BuiltInController.css`) | "Style the in-game UI. **Must use the overlay path** — direct PAZ injection breaks the encrypted layout" | UI (HUD, menus, widgets) |
| `.html` | Standard HTML | UI markup (some panels are rendered as HTML) | "Same overlay rule as CSS — use the file overlay system, not PAZ injection" | UI (HUD, menus) |

### 0.5.10 Localization, text, config

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.paloc` | **P**earl **A**byss **Loc**alization | Per-language string catalog (category + key + value) | "Translate / add UI text. Required for custom-item display names" | UI, dialogue, item names, all visible strings |
| `.xml` | Standard XML | Editor configs, dummy test data, prefab metadata, level XML form, savepreset (`GameData/SavePreset/xml/*.xml`) | "Editor-authoring sources. Some override files are XML (e.g. `OverrideObstacleList.xml`)" | Editor / build pipeline |

### 0.5.11 Audio

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.wem` | **W**wise **E**ncoded **M**edia | Per-clip compressed audio (Wwise format, unmodified) | "Voice lines, SFX. Size must match parent BNK's DIDX, else ship the BNK too" | Audio — voice, SFX, ambient |
| `.bnk` | Sound**B**a**nk** | Wwise multi-clip soundbank (DIDX + DATA + HIRC) | "Replace when changing clip sizes or reorganizing banks" | Audio — voice packs, SFX packs, music |
| `.pasound` | **P**earl **A**byss **Sound**banks-info | Wwise SoundbanksInfo metadata (`SoundbanksInfo_*.pasound`) | "Index of which BNK contains which clip — engine-side lookup" | Audio (BNK lookup) |

### 0.5.12 Textures

| Ext | Stands for | What it actually is | Modding context | Touches |
|---|---|---|---|---|
| `.dds` | **D**irect**D**raw **S**urface | Standard DDS texture (BC7 SRGB color, BC5 Linear normals) | "vpath path-prefix decides the 'last4' routing class. **Inject via PAZ** for color textures, **overlay** for UI surfaces" | UI icons, character textures, weapon skins, world textures, item icons |
| `.pathc` | **P**earl **A**byss **T**exture / Pat**h** **C**ache | Master texture / vpath index (`miscellaneous/textureheadercollection.pathc`) | "Every DDS lookup hits this first. Modify when injecting new texture vpaths" | Textures, UI, characters, world |

### 0.5.13 Mod-author formats (you author these, the game doesn't load them directly)

| Ext | What it is | Modding context |
|---|---|---|
| `.json` / `.field.json` | v3.1 mod manifest | "Your mod's table of contents — what to apply where" |
| `.zip` | Distribution package | "What `dmm-mod-pack` produces; what NexusMods uploads accept" |
| `.asi` | DLL injection plugin | "Third-party DLL hooks (Ultimate ASI Loader). Out of scope for v3.1 manifests but commonly distributed alongside data mods" |

### 0.5.14 Quick-reference: "my mod broke X"

Symptom-to-format map for triage:

| Category | Formats you'll touch |
|---|---|
| **Won't even register the overlay** | `.papgt` (header CRC), `.paz`, `.pamt` |
| **Item / equipment data** | `.pabgb` + `.pabgh` (`iteminfo.pabgb`) |
| **Character / NPC stats / behavior** | `.pabgb` (`characterinfo`), `.paschedule`, `.paschedulepath`, `.pai` |
| **World / region / spawning** | `.pabgb` (`regioninfo`, `spawningpoolautospawninfo`, `terrainregionautospawninfo`), `.palevel` |
| **Combat numbers (damage, hitboxes)** | `.pabgb` (`skillinfo`, `buffinfo`), `.paatt` |
| **Cinematics & scripted scenes** | `.paseq`, `.paseqc`, `.pastage`, `.paseqh` |
| **UI text** | `.paloc` |
| **UI styling / layout** | `.css`, `.html` (overlay path only) |
| **Item / UI icons** | `.dds` (PAZ injection), `.pathc` |
| **Character textures** | `.dds` (PAZ injection) |
| **Audio (voice / SFX / music)** | `.wem`, `.bnk`, `.pasound` |
| **Animations** | `.paa`, `.paasmt` |
| **Meshes (characters, props, vegetation)** | `.pac`, `.pab`, `.pam`, `.pat`, `.pampg` |
| **Particles / VFX** | `.paem` |
| **Reflections / lighting** | `.pareflect` |
| **Custom item attach points** | `.padock`, `.pappt` |

---

## 1. Picking a mod type

| You want to…                       | Mod type      | Section |
|------------------------------------|---------------|---------|
| Change item / skill / NPC numbers  | Data mod      | §2      |
| Replace an in-game texture         | Texture mod   | §3      |
| Replace voice / SFX                | Audio mod     | §4      |
| Translate / change UI strings      | Paloc mod     | §5      |
| Add a brand-new item               | Custom item   | §6      |
| Combine multiple of the above      | Mixed mod     | §7      |

Most useful mods are **mixed** — a custom-item mod usually ships a DDS
icon, a few paloc strings, and a row of `ItemInfo` data.

---

## 2. Data mods (table targets)

A table target is a list of operations against a parsed game table. The
authoritative table list is whatever `dmm_parser.is_supported_table`
returns — currently includes `ItemInfo`, `SkillInfo`, `BuffInfo`,
`CharacterInfo`, `StageInfo`, `GimmickInfo`, plus the localization
catalogs.

```jsonc
{
  "kind": "table",
  "table": "ItemInfo",
  "ops": [
    { "key": "id_4242", "field": "max_count",   "value": 99 },
    { "key": "id_4242", "field": "buy_price",   "value": 0  },
    { "key": "id_4242", "field": "description", "value": "A trusty blade." }
  ]
}
```

**What each op needs:**
- `key` — the stable row identifier the parser exposes. For
  ItemInfo this is the `id_<int>` form returned by
  `dmm_parser.parse_iteminfo_from_file`.
- `field` — the field name, exactly as it appears in the parsed JSON.
- `value` — the new value. Strings get written as PAMT BStrings, ints
  go straight through, optional fields can be set to `null` to clear.

**Pitfalls:**
- Whitespace inside string values is preserved verbatim — including
  trailing newlines. Strip them.
- Numeric overflow is not caught at parse time. Read the field's
  underlying type from the schema before writing huge values.

---

## 3. Texture mods (DDS asset targets)

> **For mod authors:** the dedicated guide
> [`TEXTURE_MOD_AUTHORING.md`](TEXTURE_MOD_AUTHORING.md) covers everything
> — five-minute quickstart, DDS format chart, vpath cheatsheet, DCC tool
> exports (Photoshop / GIMP / Substance Painter / `nvtt_export`),
> validation, packing, NexusMods page templates, recipes, and
> troubleshooting. The one-page
> [`BINARY_FORMATS.md#texture-vpath-cheatsheet`](BINARY_FORMATS.md#texture-vpath-cheatsheet) is the
> printable reference card. The summary below is the minimum to know
> textures exist as a v3.1 target type.

A DDS asset target replaces a texture file in the game's archive
overlay. The mounted vpath must match the in-game path exactly,
**including the Crimson "last4" overlay class** (see `docs/api.md`
→ DDS Textures).

```jsonc
{
  "kind": "asset",
  "asset_type": "dds",
  "file": "textures/sword_diffuse.dds",
  "vpath": "/character/texture/macduff/sword_diffuse.dds",
  "sha256": "...auto-filled by dmm-mod-pack..."
}
```

**The 60-second version of the rules:**
- Color textures (UI / armor / tattoo) → **BC7 SRGB** with mips
- Normal maps (`*_n.dds`) → **BC5 Linear** with mips
- vpath path-prefix decides the last4 (`/ui/*` → `0x1580`,
  `/character/texture/*_n.dds` → `0x0480`, etc.)
- `dmm-mod-pack` auto-fills sha256 + size + last4
- `dmm-mod-validate` catches malformed DDS before users see it

**Build pipeline:**
1. Author your texture in your DCC tool.
2. Export to DDS with the right format per
   [BINARY_FORMATS.md#texture-vpath-cheatsheet](BINARY_FORMATS.md#texture-vpath-cheatsheet).
3. Run `python -m dmm_parser.tools.validate <manifest>` to catch
   format / vpath issues.
4. Run `python -m dmm_parser.tools.pack <manifest>` to produce the
   distributable zip.

---

## 4. Audio mods (WEM / BNK asset targets)

Audio replacement uses Wwise's WEM (per-clip) and BNK (soundbank)
formats. The Crimson conventions are documented in
`references/wwise_notes.md`.

```jsonc
{
  "kind": "asset",
  "asset_type": "wem",
  "file": "voice/macduff_attack01.wem",
  "vpath": "0006/sound/windows/english(us)/3684722581.wem"
}
```

**Rules:**
- WEM file size must match what the original soundbank's DIDX entry
  expects, OR you must ship a replacement BNK with the new DIDX
  offsets. SWISS warns if the DIDX of the parent BNK and the new
  WEM size disagree.
- Voice clips for English go under
  `0006/sound/windows/english(us)/<id>.wem`. Other languages use
  the language tag in place of `english(us)` (e.g. `korean`,
  `japanese`).
- Common SFX use `soundcommon/windows/<id>.wem` / `.bnk`.
- `dmm_parser.infer_audio_vpath(path)` returns the Crimson class
  (`LocalizedVoiceClip`, `CommonSoundBank`, etc.) which SWISS uses
  to pick the right archive group.

**Building WEMs.** Use Wwise authoring (free for non-commercial) to
encode and export. The `validate_audio` checker covers format-tag,
sample-rate sanity, channel counts, and basic envelope structure;
real perceptual quality is on you.

---

## 5. Paloc mods (localization)

```jsonc
{
  "kind": "paloc",
  "language": "english(us)",
  "entries": [
    { "category": 0, "key": "STR_ITEM_4242_NAME", "value": "Trusty Sword" },
    { "category": 0, "key": "STR_ITEM_4242_DESC", "value": "Reliable, sharp." }
  ]
}
```

The `category` field matches the upstream paloc `u64` category. For
typical UI text it's `0`. Use `dmm_parser.parse_paloc_from_file` on a
vanilla file to discover the category convention for your strings.

---

## 6. Custom items

A "custom item" is a mod that **adds** a new row to `ItemInfo` rather
than editing an existing one. The right pattern:

1. Pick a free `id_<int>` (anything ≥ 100000 to avoid colliding with
   future game patches).
2. One **table** target inserting the row.
3. One **asset** target with the icon DDS.
4. One **paloc** target with the name + description strings.

See `docs/CUSTOM_ITEM_CREATOR_V3_1.md` for the full worked example
including the equipment / sockets fields.

---

## 7. Mixed mods

Just put multiple targets in `targets: [...]` — order doesn't matter,
SWISS sorts them by kind during apply. Hash + size on each asset
target lets the validator catch corruption before the mod hits the
runtime overlay.

```jsonc
{
  "format": 3, "format_minor": 1,
  "name": "Sword of Potter",
  "author": "potter",
  "version": "1.0.0",
  "targets": [
    { "kind": "table", "table": "ItemInfo",
      "ops": [{"key": "id_100001", "field": "name", "value": "Sword of Potter"}] },
    { "kind": "asset", "asset_type": "dds",
      "file": "icons/sword.dds", "vpath": "/ui/icon/sword.dds" },
    { "kind": "paloc",
      "entries": [{"category": 0, "key": "STR_ITEM_100001_DESC",
                   "value": "Forged in the loop."}] }
  ]
}
```

---

## 8. The author workflow

```
my_mod/
├── my_mod.field.json
├── icons/
│   └── sword.dds
└── voice/
    └── macduff_attack01.wem
```

```sh
# 1. Validate while iterating
python -m dmm_parser.tools.validate my_mod/my_mod.field.json --assets my_mod/

# 2. Inspect what your manifest will do
python -m dmm_parser.tools.inspect my_mod/my_mod.field.json

# 3. Pack for distribution (auto-fills sha256 + size + vpath)
python -m dmm_parser.tools.pack my_mod/my_mod.field.json --out my_mod-1.0.0.zip

# 4. Diff against another mod to check for conflicts
python -m dmm_parser.tools.diff my_mod-1.0.0.zip other_mod.zip
```

Exit codes: 0 = no fatals/errors; 1 = at least one finding requires
attention. SWISS Stacker uses these exit codes during its enable/disable
flow.

---

## 9. Distribution

- **NexusMods**: ship the `.zip` produced by `dmm-mod-pack`. SWISS
  Stacker reads `.zip` directly. Tag with the `crimson-desert-modding`
  category and link to the `dmm-parser` README so users know where the
  manifest format comes from.
- **CDMTL v1.0** (this repo's license): the parser tooling itself is
  source-available with a no-competing-implementation clause; mod
  manifests you author with it are yours to license however you like.

---

## 10. Common pitfalls

- **Forgot to update SHA-256.** Symptom: validator complains
  `asset_sha_mismatch` after you tweak a DDS. Fix: re-pack — `dmm-mod-pack`
  recomputes the hash if you didn't pre-set one.
- **Path case mismatch.** Crimson is case-insensitive at lookup but
  the SWISS Stacker overlay map is case-sensitive. Always use lowercase
  vpaths.
- **Missing `dwReserved2[3]` in DDS.** Symptom: texture loads but
  appears as a blank/pink fallback in-game. Fix: re-export keeping the
  Crimson reserved fields, or fix in Hex with `classify_dds` as guide.
- **WEM sample-rate outside 8k–96k.** Triggers the
  `wem_unusual_sample_rate` warning. Crimson will technically play it,
  but the resampler's pitch envelope was tuned around 44.1k/48k.
- **Custom item ID under 100k.** Will eventually collide with a game
  patch. Pick high.

---

## 11. Where to look next

- `docs/BINARY_FORMATS.md#file-format-reference-formats` — binary format reference (PAPGT, PAMT, PAZ, paloc,
  DDS, WEM, BNK, save) including:
  - §1.1 PAPGT header CRC offset gotcha (bytes 4–7, not 0–3).
  - §6.1 Tier-1 PABGB inventory — all 14+ tables promoted post-1.3.3.
  - §6.2 1.05.01 ItemInfo schema corrections (Cooltime,
    MaxChargedUseableCount, ItemIconData 5-field, restored fields).
  - §11/§12 canonical Tier status for the sequencer / attack family.
- `docs/api.md` — full Python API.
- `docs/CUSTOM_ITEM_CREATOR_V3_1.md` — end-to-end custom-item example.
- `docs/_archive/TIER1_PROMOTION_PROGRESS.md` — Sequencer / schedule / attack
  format notes and field directories.
- `docs/BINARY_FORMATS.md#paatt-basedata-field-layout` — `.paatt` AttackInfo field reference.
- `references/*.hexpat` — ImHex pattern files for binary exploration.

---

## 12. Sequencer / schedule / attack-info mods

Six formats drive cutscenes, NPC behavior, and combat. All six
round-trip byte-perfect on every vanilla sample. Two
(`.paseq`, `.pastage`) shipped Tier-1 field-level decode; the
remaining four are Tier 1.5 (envelope decoded, body opaque) until the
promotion work in `docs/BINARY_FORMATS.md#file-format-reference-formats` §12 finishes.

| Extension | What it is | Vanilla samples | Tier | Editing pattern |
|---|---|---|---|---|
| `.paseq` | Sequencer (cutscene/scripted action) | 4,659 | **1** | JSON path (preferred) or walk + replace |
| `.pastage` | Stage-chart binary (timeline state machine) | 3,320 | **1** | JSON path (preferred) or walk + replace |
| `.paseqc` | Compiled sequencer chart | 2,932 | 1.5 | walk + replace |
| `.paschedule` | NPC time-of-day / activity schedule | 4,084 | 1.5 | JSON path (envelope only) |
| `.paschedulepath` | Companion path data for a `.paschedule` | 3,737 | 1.5 | JSON path (envelope only) |
| `.paatt` | Per-weapon attack info (hitboxes, damage, frame events) | 220 | **1** | JSON path (full field-level via `paatt_decode_base_data`) |

### 12.1 Two editing patterns

The 6 formats split into two groups based on tier + byte layout:

**Tier 1** (`.paseq`, `.pastage`) — full field-level JSON path. Parse,
edit any named field, reserialize. Walk + replace still works as a
fallback for surgical string edits, but the field-level workflow is
preferred for any non-trivial change. See §12.5 for the typed
PyO3 entry points.

**Tier 1.5 walk + replace** (`.paseqc`): strings are stored as
`u32 length + bytes`. Use the walk + replace primitive — find any
string, edit it by file offset:

```python
import dmm_parser

with open("cd_seq_ui_appear_all.paseq", "rb") as f:
    data = f.read()

# Find every length-prefixed string
strings = dmm_parser.walk_lp_strings(data)
for s in strings[:5]:
    print(f"  0x{s['file_offset']:04x}  {s['value']!r}")

# Replace one — length-flexible (file size adjusts)
modified = dmm_parser.replace_cstring_at(
    data, strings[0]['file_offset'],
    new_value="my_renamed_value",
    expected_value=strings[0]['value'],  # safety check
)

with open("cd_seq_ui_appear_all_modded.paseq", "wb") as f:
    f.write(modified)
```

**Structured-header formats** (`.paschedule`, `.paschedulepath`,
`.paatt`): use the **JSON path** — parse, edit named fields,
reserialize:

```python
import dmm_parser
import base64

with open("npc_schedule.paschedule", "rb") as f:
    data = f.read()

parsed = dmm_parser.parse_paschedule_bytes(data)
# parsed is a dict: {"version", "hash", "flag", "hash_repeated",
#                    "reserved_b64", "name", "opaque_body_b64"}
parsed["name"] = "my_renamed_schedule"
modified = dmm_parser.serialize_paschedule(parsed)
```

For `.paatt`:

```python
parsed = dmm_parser.parse_paatt_bytes(data)
# parsed["string_table"], parsed["effect_name_table"], etc. are lists
parsed["effect_name_table"][0] = "MyCustomEffect"

# Per-AttackInfo body is a versioned blob — decode it into named fields,
# edit, and re-encode. Versions 0/1/2/3 are all field-decoded; version 4
# (and any future version) round-trips opaquely via base_data_b64.
info = parsed["infos"][0]
raw  = base64.b64decode(info["base_data_b64"])
fields = dmm_parser.paatt_decode_base_data(info["version"], raw)

fields["physic_impulse_power"] = 2.5      # double knockback
fields["repeat_count"]         = 3        # extra hit per swing
# V2 (throw) extras: fields["projectile_key"], fields["frame_time"], …
# V3 (release-catch) extras: fields["release_angle_rad"], fields["frame_time"], …

info["base_data_b64"] = base64.b64encode(
    dmm_parser.paatt_encode_base_data(info["version"], fields)
).decode()

modified = dmm_parser.serialize_paatt(parsed)
```

See `docs/api.md` → "**.paatt — typed AttackInfo BaseData**" for the
per-version dict shape and the most-commonly-edited field reference, and
`docs/BINARY_FORMATS.md#paatt-basedata-field-layout` for the full per-byte layout including
every `_unkXXXX` field still pending IDA-resolved C++ names.

### 12.2 Discovering what's in a sequencer file

For `.paseq` / `.paseqc`, the typed reader can enumerate the file's
embedded class hierarchy and field directory:

```python
import dmm_parser

with open("cd_seq_spawn_doc_animal_fish_jump_00.paseqc", "rb") as f:
    data = f.read()

# What class blocks does this file contain?
blocks = dmm_parser.parse_paseqc_all_class_blocks(data)
for block in blocks[:3]:
    print(f"\nClass: {block['class_name']}")
    for f in block['fields'][:5]:
        print(f"  {f['field_name']:30}  {f['type_name']}")
    if len(block['fields']) > 5:
        print(f"  ... {len(block['fields']) - 5} more")

# What value strings (script expressions, asset paths) does it embed?
strings = dmm_parser.paseqc_value_section_strings(data)
for s in strings[:10]:
    print(f"  0x{s['file_offset']:04x}  {s['value']!r}")
```

This surfaces things like:
- Script expressions: `Timeline.condition_timelineEnd()`,
  `Player.condition_enterTrigger(Trigger_00)`
- Trigger names: `WAIT_Trigger`, `SCENE_1`, `Trigger_00`
- Game event hooks: `OnSequencerBlindWait`, `OnSequencerBattleStart_Chase`

Edit them to change cutscene flow without rebuilding the whole file.

### 12.3 Validation

After every edit, re-parse to confirm:

```python
parsed_again = dmm_parser.parse_paseqc_bytes(modified)
# If this raises, your edit broke the structure — undo and try again
```

The full Tier 1 round-trip validator
(`examples/tier1_full_roundtrip.rs`) covers all 18,952 vanilla samples
across both direct and JSON paths at 100% byte-perfect — drop it into
your CI to catch regressions.

### 12.4 CLI: `rename_string`

A small command-line tool ships for the most common edit — renaming
a single string value:

```sh
# List every editable string in a file
python -m dmm_parser.tools.rename_string my_seq.paseq --list

# Rename one (writes back in place by default, or use --out)
python -m dmm_parser.tools.rename_string my_seq.paseq \
    "old_value" "new_value" --out my_seq_modded.paseq
```

The tool wraps `walk_lp_strings` + `replace_cstring_at`. It refuses
to write if `old_value` isn't found, and warns when multiple
occurrences exist (replaces only the first).

For structured-header formats (`.paschedule`, `.paatt`), this CLI
isn't applicable — use the JSON-path workflow above instead.

### 12.5 Reference: Tier 1 PyO3 functions

| Function | Purpose |
|---|---|
| `parse_pastage_bytes(b)` / `serialize_pastage(d)` | `.pastage` round-trip |
| `parse_paseq_bytes(b)` / `serialize_paseq(d)` | `.paseq` round-trip |
| `parse_paseqc_bytes(b)` / `serialize_paseqc(d)` | `.paseqc` round-trip |
| `parse_paschedule_bytes(b)` / `serialize_paschedule(d)` | `.paschedule` round-trip |
| `parse_paschedulepath_bytes(b)` / `serialize_paschedulepath(d)` | `.paschedulepath` round-trip |
| `parse_paatt_bytes(b)` / `serialize_paatt(d)` | `.paatt` round-trip |
| `parse_paseq_field_directory(b)` | Outer-class fields from `.paseq` |
| `parse_paseqc_field_directory(b)` | Outer-class fields from `.paseqc` |
| `parse_paseq_all_class_blocks(b)` | All schema class blocks (`.paseq`) |
| `parse_paseqc_all_class_blocks(b)` | All schema class blocks (`.paseqc`) |
| `paseq_value_section_offset(b)` | Byte offset where values start |
| `paseq_value_section(b)` | Raw value-section bytes |
| `paseq_value_section_strings(b)` | LP-strings only inside values |
| `paseqc_value_section_offset(b)` / `_section(b)` / `_strings(b)` | sister accessors |
| `walk_lp_strings(data)` | Generic LP-string walker, any byte slice |
| `replace_cstring_at(data, offset, new_value, expected_value=None)` | Generic length-flexible string edit |
- `samples/` — runnable example mods (each its own README).

---

## 13. Format-internal gotchas (recent discoveries)

Stuff caught while bringing the parser up to 1.05.01 that's worth
knowing if you're hand-rolling intents or hex-editing PA files.

### 13.1 Wrapper readers in `iteminfo.pabgb` — `cooltime` and `max_charged_useable_count`

The engine reader for `ItemInfo._cooltime` is `sub_101886C44`. It looks
like a single field on the wire but actually invokes the i64 reader
**three times** at memory offsets 0 / 8 / 16 — so the wire layout for
`cooltime` is **24 bytes (3 × i64)**, not 8.

Same shape for `ItemInfo._maxChargedUseableCount` (reader
`sub_101886C94`): three u32 reads → **12 bytes (3 × u32)** on the wire,
not 4.

What this means for mod authors:

- **In v3.1 intents written against the new schema**, both fields are
  objects:

  ```json
  {
    "key": "id_2200",
    "field": "cooltime",
    "value": { "a": 30, "b": 0, "c": 0 }
  }
  ```

- **Legacy intents** that stored these as single numbers
  (`"value": 30`) still work. The parser accepts both forms and
  promotes a number to `{a: n, b: 0, c: 0}`. Your old SuperMod-style
  intents do NOT need to be rewritten.

- The `b` and `c` fields are non-zero for **all 6,236** items in
  the case of `max_charged_useable_count` (and 659 items for
  `cooltime`). If you're authoring a fresh intent for one of those
  rows, parse the vanilla value first — don't drop b/c by accident.

The previous 1-field schema overran 16 bytes per item, cascading into
bogus CArray counts (e.g. 131,072 `stat_list_static` entries) and
crashing parse on item 1. Anyone re-implementing the parser should
check for this wrapper-reader pattern across other tables — there
may be more.

### 13.2 PAPGT header CRC lives at bytes 4–7

`meta/0.papgt` layout: bytes 0–3 are the platform magic, bytes 4–7
are the integrity CRC over `papgt[12..]`. A common mistake (we shipped
this bug in pre-release.11) is to write the recomputed hash to bytes
0–3 instead. That clobbers the magic AND leaves the real hash field
stale — next mount fails parse with `Checksum mismatch`.

If you're writing a tool that mutates PAPGT directly:

```rust
// CORRECT
let crc = hashlittle(&papgt[12..], INTEGRITY_SEED);
papgt[4..8].copy_from_slice(&crc.to_le_bytes());

// WRONG — clobbers platform_magic, leaves checksum stale
papgt[0..4].copy_from_slice(&crc.to_le_bytes());
```

If you encounter a PAPGT with mismatched CRC on disk, the body is
usually intact — recompute hashlittle over `[12..]`, write to `[4..8]`,
and the file parses again. dmm-parser's `PackGroupTreeMeta::to_bytes`
and DMM's `build_papgt_with_overlay_named` both handle this correctly.

### 13.3 PAMT header CRC is at bytes 0–3 (different from PAPGT!)

PAMT uses the opposite layout: bytes 0–3 are the integrity CRC,
PazInfo CRC sits at bytes 16–19. Don't copy-paste between PAMT and
PAPGT writers without checking.

```rust
// PAMT header CRC update
let crc = hashlittle(&pamt[12..], INTEGRITY_SEED);
pamt[0..4].copy_from_slice(&crc.to_le_bytes());
```

### 13.4 Field-order matters for round-trip on schema-rev'd structs

`ItemIconData` is a 5-field struct. The wire reader (sub_101884D3C)
reads them in the order
`iconPath / highlightIconPath / checkExistSealedData / gimmickStateList / checkUsable` —
NOT the order they appear in the in-memory C++ struct (which has
`checkUsable` between the two icons in some Win builds). If your
parser reads the in-memory order, total byte count happens to come out
right but per-byte round-trip fails. Always order fields by the wire
read sequence (= IDA decomp call order), not the memory layout.

### 13.5 Mac-binary symbols are the ground truth for type widths

When in doubt about whether a key is u16 or u32 wire, decompile the
reader function. Look at its inner vtable call — the third argument
is the byte width:

```c
// 4-byte read (e.g. ItemKey, EquipTypeKey, MultiChangeKey)
(*((__int64(__fastcall**)(__int64,int*,__int64))(*(_QWORD*)a1+16LL)))(a1,&v4,4LL);

// 2-byte read (e.g. CraftToolKey, InventoryKey, CategoryKey)
(*((__int64(__fastcall**)(__int64,short*,__int64))(*(_QWORD*)a1+16LL)))(a1,&v4,2LL);
```

The `pa::StaticInfoWrapper<Key, Info, Manager, unsigned short>` template
parameter at the end is the **memory** type after hash resolution —
NOT the wire type. Don't confuse them. Memory width is what the
in-game C++ struct holds; wire width is what's stored in the
`.pabgb` file.

---

- `samples/` — runnable example mods (each its own README).

## §13: Havok-Layer Files (Tier 1, iter 3-15 of repair loop)

Crimson Desert ships a family of PA-engine wrappers around Havok 2024.2
content. As of the iter-15 ship, **all 12** original Havok-layer
extensions are Tier 1 with byte-perfect round-trip and Python bindings
exposed via `dmm_parser`:

| Ext | Module | Mod-relevant fields |
|---|---|---|
| `.pami` | StaticMeshInstance XML | `mesh_paths`, `version`, `xml_body` |
| `.pab`/`.paa`/`.pam`/`.pabc`/`.pabv`/`.pac` | PAR family | `ext_classification`, `version_hex`, `body_b64` |
| `.motionblending` | Animation blend tree | 15 named fields × 2 type tags (`staticstringA`, `bool`) |
| `.pamlod` | Mesh LOD descriptor | `lod_count`, `lod_distance`, `texture_paths` |
| `.paasmt` | Animation Set Matching | `record_pairs[].model_path` ↔ `animset_xml_path` |
| `.paccd` | Customization Data | `format_version=14`, `no_override_byte_count` |
| `.hkx` | Havok native | `sdk_version` (always `20240200`), TAG0 sections |

See `docs/api.md` → "Havok-Layer Formats (Tier 1)" for full JSON-shape
documentation and round-trip discipline. Each format's parse function
is exposed as `dmm_parser.parse_<format>_bytes(data) -> dict`.

**What's not done yet (queued for future iters):**
- Typed value decode for `.motionblending` records (`staticstringA` array
  body layout, `bool` payload — needs IDA RE)
- Per-slider semantic mapping for `.paccd` (which byte = which slider)
- Havok class-registry decode for `.hkx` (the `hkClass` family lives
  inside `CrimsonDesert.exe`)
- Partial-compression-with-size-differential format used by ~17 .pam
  and ~6 .pac files (also IDA RE blocker)
