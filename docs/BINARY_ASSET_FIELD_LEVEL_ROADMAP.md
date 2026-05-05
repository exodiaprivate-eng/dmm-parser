# Binary Asset Field-Level Roadmap

Long-term plan for extending dmm-parser from "every pabgb table is field-level"
(current state) to "every binary asset format the game ships is field-level."

End goal: a v3.1 mod can express any change — texture pixels, audio samples,
mesh vertices, animation keyframes, item records — as a typed JSON intent
instead of opaque byte blobs. Mods become diffable, mergeable, and version-
portable.

## Scope

Inventory taken from a live PAZ-archive walk on 2026-05-04 (1.05.02 game).
**1,500,933 files across the install**, roughly **131 GB uncompressed**.

## Status legend

- **DONE** — dmm-parser exposes typed parse + serialize + field-level JSON
- **PARTIAL** — partial parser exists but not all fields decoded
- **STARTED** — RE notes captured, no working parser yet
- **PLANNED** — on the roadmap
- **DEFERRED** — low impact / niche / out of scope for foreseeable future

## Inventory: what the game ships

Sorted by file count (most numerous first). MB column is total uncompressed.

| Ext | Count | MB | Status | Spec | Notes |
|---|---:|---:|---|---|---|
| **paa** | 300,337 | 13,911 | PLANNED | proprietary | PA Animation — keyframe-per-bone clips |
| **dds** | 280,826 | 66,828 | PLANNED | public (Microsoft) | DirectDraw Surface texture, BC1/3/7 + mip chain |
| **wem** | 240,014 | 7,109 | PLANNED | public (Wwise) | Audiokinetic Wwise Encoded Media (RIFF/Vorbis variant) |
| **paa_metabin** | 152,214 | 236 | PLANNED | proprietary | PAA companion metadata |
| **padxil** | 87,170 | 3,004 | DEFERRED | partial (DXIL) | Compiled DirectX IL shader bytecode |
| **pami** | 66,896 | 331 | PLANNED | proprietary | PA Material Instance |
| **hkx** | 57,268 | 3,831 | PLANNED | public (Havok) | Havok physics/collision/animation graph |
| **pam** | 50,785 | 19,552 | PLANNED | proprietary | PA Mesh (raw geometry) |
| **prefab** | 46,050 | 278 | PLANNED | proprietary | Game-object prefab (component graph) |
| **meshinfo** | 34,715 | 208 | PLANNED | proprietary | Mesh-info index (companion to pam) |
| **pamlod** | 32,470 | 10,029 | PLANNED | proprietary | PA Mesh LOD chain |
| **palevel** | 19,867 | 2,867 | PLANNED | proprietary | Per-level data (terrain, spawns, geometry refs) |
| **pampg** | 15,286 | 953 | PLANNED | proprietary | PA Mesh Polygon Group |
| **levelinfo** | 14,420 | 64 | PLANNED | proprietary | Per-level metadata |
| **pac** | 12,784 | 6,860 | STARTED | proprietary | PA Skinned Mesh container (mesh + skeleton + materials) |
| **pac_xml** | 12,708 | 494 | DONE-ish | XML | Text form of pac (DMM already three-way merges these) |
| **binarygimmick** | 12,648 | 95 | PLANNED | proprietary | Interactable/gimmick scene data |
| **xml** | 7,378 | 275 | DONE | XML | Generic XML, three-way merged in DMM |
| **pae** | 5,995 | 278 | PLANNED | proprietary | PA Effect (visual effect graph) |
| **roadsector** | 5,698 | 963 | DEFERRED | proprietary | Road sector geometry |
| **app_xml** | 5,601 | 3 | DONE | XML | UI application XML |
| **paseq** | 4,659 | 236 | PARTIAL | proprietary | PA Sequence (cutscene/scripted action) — see project_paseq_parsing_roadmap |
| **paschedule** | 4,084 | 74 | PLANNED | proprietary | Time-of-day / NPC schedule |
| **paschedulepath** | 3,737 | 39 | PLANNED | proprietary | Path data for schedules |
| **pastage** | 3,320 | 25 | PLANNED | proprietary | Stage (zone instance) |
| **bnk** | 3,157 | 614 | PLANNED | public (Wwise) | Wwise SoundBank — event/sound-object hierarchy |
| **paseqc** | 2,932 | 35 | PARTIAL | proprietary | Compiled paseq |
| **prefabdata_xml** | 2,597 | 1 | DONE | XML | Prefab data XML |
| **road** | 1,694 | 10 | DEFERRED | proprietary | Road data (pre-sector) |
| **paccd** | 1,641 | 1 | PLANNED | proprietary | PA Collision Data |
| **motionblending** | 1,555 | 6 | PLANNED | proprietary | Animation blend tree |
| **pat** | 1,340 | 658 | PLANNED | proprietary | PA Texture (PA-side container, may wrap dds) |
| **mp4** | 695 | 3,119 | DEFERRED | public (MP4) | Cutscene videos — direct replacement only |
| **pbd** | 581 | 209 | PLANNED | proprietary | unknown (PA Body Data?) |
| **paem** | 556 | 11 | PLANNED | proprietary | PA Effect Mesh |
| **paac** | 551 | 80 | PLANNED | proprietary | PA Action Chart |
| **pabc** | 456 | 21 | PLANNED | proprietary | PA Behavior Chart |
| **material** | 389 | 1 | PLANNED | proprietary | Material descriptor |
| **save** | 327 | 8 | DONE | proprietary | Save game format (separate sub-system) |
| **pab** | 246 | 7 | PLANNED | proprietary | unknown (PA Block?) |
| **paatt** | 220 | 8 | DONE | proprietary | PA Attachment data — already parsed in dmm-parser |
| **css** | 175 | 3 | DONE-ish | CSS | UI stylesheet, three-way merged |
| **html** | 157 | 3 | DONE-ish | HTML | UI markup, three-way merged |
| **pabgb** | 122 | 114 | DONE | proprietary | **All 122 tables field-level via dispatch** |
| **pabgh** | 122 | 2 | DONE | proprietary | pabgb companion index |
| **paloc** | 14 | 234 | DONE | proprietary | Localization strings |
| _(remaining 30+ extensions)_ | | | DEFERRED | mixed | Long tail, mostly under 100 files each |

## Per-format field-level vision

### Tier 1 — Public specs, high user impact

#### `.dds` (DirectDraw Surface, 280,826 files, 67 GB)
- **Spec**: Microsoft DDS spec (DXGI_FORMAT_*) — fully public
- **Field-level unlock**: width, height, format, mip count, per-mip pixel data,
  cubemap faces. Power-user: per-channel pixel access for shader-style effects.
- **User benefit**: change format (e.g. BC1 → BC7 for higher quality), edit mip
  chain, decode/encode programmatically without external DDS tools.
- **Effort**: low. Many open-source Rust crates exist (`dds-rs`, `image_dds`).
  Wrap one + emit JSON dict per surface.
- **Mod use cases**: texture mods that want partial overwrite (e.g. just edit
  the diffuse channel, leave normal map alone).

#### `.wem` (Wwise Encoded Media, 240,014 files, 7 GB)
- **Spec**: Audiokinetic Wwise SDK + open-source `vgmstream`/`wwiser`
- **Field-level unlock**: sample rate, channel count, codec, loop points, raw
  PCM sample data after decode.
- **User benefit**: pitch shift, time stretch, channel rebalance, decode to
  WAV for editing then re-encode.
- **Effort**: moderate. WEM is RIFF + custom Vorbis. vgmstream has a
  reference implementation we can port or wrap via FFI.
- **Mod use cases**: voice mods that adjust pitch/timing without re-recording,
  "louder/quieter NPC" mods, audio cleanup mods.

#### `.bnk` (Wwise SoundBank, 3,157 files, 614 MB)
- **Spec**: public Wwise SDK
- **Structure**: chunked container (BKHD, DIDX, DATA, HIRC, STMG, ENVS, PLAT, INIT)
- **Field-level unlock**: per-chunk parsing. Especially HIRC (event hierarchy):
  Event ID → Action → Sound Object refs. Edit volume defaults, RTPC curves,
  swap which WEM plays for which event.
- **User benefit**: rewire game audio events without touching .wem files.
- **Effort**: moderate-high. HIRC has 30+ object types (Sound, RandomContainer,
  SwitchContainer, MusicSegment, etc.). `wwiser` Python tool already
  enumerates them; port the schema.
- **Mod use cases**: "Kliff says line A in context B" rewiring, audio event
  defaults, RTPC tweaks.

#### `.hkx` (Havok, 57,268 files, 3.8 GB)
- **Spec**: public Havok format (well-documented in Havok SDK)
- **Field-level unlock**: skeletons, animation tracks, ragdoll constraints,
  collision geometry
- **User benefit**: animation mods that adjust pose/timing without ripping
  Havok tooling
- **Effort**: high. Havok format is extensive; not all fields needed for
  modding. Start with skeleton + animation tracks subset.
- **Mod use cases**: animation retargeting, ragdoll tuning.

### Tier 2 — Proprietary, high user impact, achievable

#### `.pac` (PA Skinned Mesh, 12,784 files, 6.9 GB)
- **Spec**: proprietary, requires RE
- **Field-level unlock**: vertex buffer (positions, normals, UVs, weights,
  indices), bone hierarchy, material refs, LOD chain refs (likely → pamlod),
  skeletal volume refs
- **RE approach**: trace `pa::ResourceType::SkinnedMesh` loader (saw error
  string "ResourceType은 SkinnedMesh(.pac)이어야 합니다" at 0x1073e26ee).
  The loader function is one xref away.
- **Effort**: high. Custom mesh format + skeleton + material binding.
  ~1-2 weeks of decomp + iteration.
- **Mod use cases**: model swaps with field-level access (e.g. swap just the
  weapon prefab without touching the body), vertex deformation mods.

#### `.pam` (PA Mesh raw geometry, 50,785 files, 19.5 GB)
- **Spec**: proprietary, RE-needed
- **Field-level unlock**: same as pac but without the skinned/animation layer
- **Effort**: high (same family as pac, can share parser code)

#### `.pamlod` (PA Mesh LOD chain, 32,470 files, 10 GB)
- **Spec**: proprietary
- **Field-level unlock**: per-LOD distance thresholds + mesh refs
- **Effort**: medium (smaller than pam itself; mostly indirection)

#### `.pae` (PA Effect, 5,995 files, 278 MB)
- **Spec**: proprietary
- **Field-level unlock**: emitter graph, lifetime curves, shader refs
- **Effort**: medium-high

#### `.prefab` (46,050 files, 278 MB)
- **Spec**: proprietary
- **Field-level unlock**: component graph (reflection-based per the existing
  `ReflectObject` infrastructure already touched by ConditionInfo work)
- **Effort**: medium. Likely shares the `ReflectObject` family decoder
  pattern dmm-parser already uses.
- **Mod use cases**: edit any in-world object's component values without
  shipping a whole new prefab file.

### Tier 3 — Animation, materials, level data

#### `.paa` + `.paa_metabin` (300k + 152k files, 14 GB total)
- **Spec**: proprietary
- **Field-level unlock**: animation clips (keyframes per bone). Metabin =
  index/header.
- **Effort**: high. Animation formats are dense; full RE is multi-week.
- **Mod use cases**: animation tweaks (timing, blend curves)

#### `.pami` (Material Instance, 66,896 files, 331 MB)
- **Spec**: proprietary
- **Field-level unlock**: material slot bindings, texture refs, parameter
  overrides
- **Effort**: medium. Likely a small reflection-based struct.

#### `.binarygimmick` (12,648 files, 95 MB)
- **Spec**: proprietary
- **Field-level unlock**: per-gimmick component data
- **Effort**: medium (related to the binary_gimmick.pabgb table dmm-parser
  already parses — likely the same record layout in standalone form)

#### `.palevel` / `.pastage` / `.levelinfo` (~37k files, 3 GB)
- **Spec**: proprietary
- **Field-level unlock**: terrain heightmaps, spawn definitions, geometry refs
- **Effort**: high. Level data is large + complex.

### Tier 4 — Specialized / niche

| Ext | Notes |
|---|---|
| `.padxil` | Compiled DXIL shader. Standard format but rarely modded. |
| `.paac` / `.pabc` | Action chart / behavior chart. Reflection-based. Medium effort. |
| `.paschedule*` | NPC time-of-day schedules. Medium effort. |
| `.paseq*` | Cutscene/sequence (already PARTIAL — see paseq_parsing_roadmap). |
| `.pat` | PA Texture wrapper. Likely just metadata around a .dds payload. |
| `.paccd` | PA Collision Data. Small, likely simple struct. |
| `.motionblending` | Animation blend tree. Moderate. |
| `.roadsector` / `.road` | Road geometry. Niche. |
| `.mp4` | Cutscene video. Public format but no field-level mod use case. |
| `.paatt` | DONE in dmm-parser. |

## Suggested phasing

### Phase A — Public-spec wins (lowest risk, highest immediate impact)
1. `.dds` field-level (port/wrap an existing crate)
2. `.wem` field-level (port vgmstream pieces)
3. `.bnk` field-level (HIRC chunk parsing)

These three cover ~537,000 files (37% of the asset library) and the entire
texture + audio modding surface. Public specs mean RE risk is near zero.

### Phase B — Pearl Abyss core formats
4. `.pac` field-level (skinned mesh) — flagship win for character/weapon mods
5. `.pam` (reuse pac infrastructure)
6. `.pamlod` (reuse pac infrastructure)
7. `.prefab` (likely ReflectObject-based, reuses existing reflection code)
8. `.pami` (material instance)
9. `.pae` (effect)

These cover model + material + effect mods with field-level granularity.

### Phase C — Animation
10. `.paa` + `.paa_metabin` (animation clips)
11. `.motionblending`
12. `.hkx` (Havok subset for skeleton + tracks)

### Phase D — Level/world data
13. `.palevel` / `.pastage` / `.levelinfo`
14. `.paseq` / `.paseqc` (continue existing PARTIAL work)
15. `.paschedule*`

### Phase E — Long tail
16. Everything else as user demand arises.

## Pre-requisites the user already has

- **PAZ archive parsing**: dmm-parser handles read + write
- **PAMT/PAPGT registration**: dmm-parser handles
- **Path resolution**: dmm-parser handles
- **122 pabgb tables field-level**: dmm-parser dispatches all of them
- **Paloc field-level**: dmm-parser handles
- **`paatt`**: parsed
- **Reflection infrastructure** (used for ConditionInfo polymorphic dispatch):
  applicable to .prefab and similar component-graph formats

The asset-format work doesn't replace any of this; it extends dmm-parser to
also expose the contents of leaf binary assets.

## How the v3.1 spec extends to cover this

When a format graduates to field-level, v3.1 mods can target it the same way
they target pabgb tables today:

```json
{
  "format": 3, "format_minor": 1,
  "modinfo": { ... },
  "targets": [
    {
      "file": "ui/icon/item/sword_001.dds",
      "intents": [
        { "field": "header.format", "op": "set", "new": "BC7_UNORM" },
        { "field": "mips[0].width",  "op": "set", "new": 512 }
      ]
    },
    {
      "file": "sound/windows/media/english(us)/1006515747.wem",
      "intents": [
        { "field": "header.sample_rate", "op": "set", "new": 44100 }
      ]
    }
  ]
}
```

Same shape as today's pabgb intents. DMM dispatches on file extension to the
right typed parser.

## Open questions / unknowns

1. **Is the `.pac` skeleton Havok-based?** If yes, .hkx parsing covers most of
   .pac's animation rig. If no, separate effort.
2. **Does the game accept arbitrary new asset paths** (e.g. add a brand new
   texture at a path that wasn't in vanilla)? Need to confirm via PAPGT
   registration test before scoping "add new asset" intents.
3. **`.paacdesc` / `.pamhc` / `.pappt` / `.paasmt`** — only 1 file each. May
   be debug/dev artifacts. Worth investigating once for completeness.
4. **`.paa` vs `.paa_metabin` ratio** (300k vs 152k) — not 1:1, suggesting
   not every animation has metabin companion. Need to understand the binding.
5. **`.pabd` vs `.pbd`** — naming similarity, may be related families.

## What this doc does NOT replace

The work to actually IMPLEMENT each format's parser. This roadmap captures
**what to build** and **why**. The HOW per format requires real
RE/decompile work (or SDK reading for public formats) at implementation
time. Estimate per-format effort: low (DDS/known specs) to high (PAC/PA
custom formats) — see per-format sections.

## Tracking

This doc is the source of truth for "where are we on field-level binary
assets." Update Status column as each format moves through PLANNED →
STARTED → PARTIAL → DONE.
