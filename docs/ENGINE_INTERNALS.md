# Engine Internals & Asset Research

Research notes on Crimson Desert's runtime engine + non-pabgb asset
formats. Different from `BINARY_FORMATS.md` (which is a byte-layout
reference) — this doc captures *how the game's runtime works* and what
we've learned from binary analysis about classes / pipelines / asset
types that aren't yet field-level parsed.

> **Last consolidated:** 2026-05-10. Merged from
> `engine_havok_skinning.md`, `effectdata-format.md`,
> `BINARY_ASSET_FIELD_LEVEL_ROADMAP.md`,
> `WORKBENCH_FILE_FORMAT_V3_HANDOFF.md`. The original four docs are
> deleted; their content lives in the sections below verbatim.

## Contents

- [Havok Integration & Skinning](#havok-integration--skinning)
- [EffectData Format](#effectdata-format)
- [Binary Asset Field-Level Roadmap](#binary-asset-field-level-roadmap)
- [Workbench File Format Handoff](#workbench-file-format-handoff)

---

## Havok Integration & Skinning

# Crimson Desert Engine — Havok Integration Research

Research notes from binary analysis of `CrimsonDesert.exe` (Win retail build).
Triggered by community question about whether the engine uses Havok rigid
bodies for skin proxies.

**Status:** Research notes only. The Havok layer itself (mesh/skel/anim binary
content) remains Layer 5 — not yet field-level parsed in dmm-parser. The
**Pearl Abyss-side metadata wrappers** that reference these Havok assets ARE
now reachable via the pycrimson reflection workflow shipped in Session 28
(commit `6273c7f`); see "Updates from Session 28" below.

**Last refresh:** 2026-05-10.

## TL;DR

The engine is built on **Havok 2024.2 SDK**, fully statically linked.
- Vertex weights / skinning live in **`hkaMeshBinding` + `hkxMesh::Section`**
  (the canonical Havok skinned-mesh classes).
- Rigid bodies / ragdoll live in a **parallel `hknpRagdoll` hierarchy** that
  shares the same `hkaSkeleton` as the skinned mesh.
- During normal animation: rigid bodies are keyframe-driven by the same pose
  that skins the mesh (effectively skin-following physics proxies).
- During physics events (hits, death, etc): `hknpRagdollStatePoweredDynamicAnimation`
  blends physics into the bones; the skinned mesh follows because it's still
  bound to the skeleton, which is now driven by physics.

## Confirmed via IDA Pro string + RTTI scan

### Build path (proves Havok 2024.2 statically linked)

```
d:\bs\mainline\code\trunk\External\Havok_2024_2\Public\HavokSDK\
    Common\Base\Container\RelArray\hkRelArray.inl
    Common\Base\Container\Array\hkArray.inl
    Common\Base\Reflect\Impl\hkBuiltinImpl.cpp
    Common\Base\Reflect\Impl\hkHavokImpl.cpp
    Common\Base\Thread\Pool\hkCpuThreadPool.cpp
    Common\Base\Memory\Debug\hkMemorySnapshot.cpp
    Common\Visualize\hkVisualDebugger.cpp
    Platform\Windows\Common\Base\hkWin32FileSystem.cxx
    ...
```

Plus thread names: `Havok Worker`, `Havok Async`.

### Animation module (`hka*`) — drives skinning

| Class | Purpose |
|---|---|
| `hkaSkeleton` | Bone hierarchy |
| `hkaBone` | Single bone |
| `hkaSkeletonMapperData` | Skeleton remapping |
| `hkaAnimationBinding` | Animation → skeleton binding |
| `hkaBoneAttachment` | Attachment point on a bone |
| **`hkaMeshBinding`** | **Skinned mesh ↔ skeleton binding (vertex weights)** |
| `hkaAnimation` / `hkaQuantizedAnimation` | Animation data |
| `hkaAnimationContainer` | Top-level container |
| `hkaFootstepAnalysisInfo` | Locomotion metadata |

**`hkaMeshBinding` is the canonical vertex-skinning class.** Its known field
layout (per Havok 2024.2 SDK + confirmed by string `boneFromSkinMeshTransforms`
in the exe):

```cpp
class hkaMeshBinding {
    hkRefPtr<hkxMesh>          m_mesh;
    hkStringPtr                m_originalSkeletonName;
    hkRefPtr<hkaSkeleton>      m_skeleton;
    hkArray<Mapping>           m_boneFromSkinMeshMappings;
    hkArray<hkInt16>           m_boneFromSkinMeshMappings;  // per-section indices
    hkArray<hkMatrix4>         m_boneFromSkinMeshTransforms;  // bind pose
};

class hkxMesh {
    hkArray<hkRefPtr<hkxMeshSection>> m_sections;
    hkArray<UserChannelInfo>          m_userChannelInfos;
};

class hkxMeshSection {
    hkRefPtr<hkxVertexBuffer>  m_vertexBuffer;  // pos + normal + uv + bone_weights + bone_indices
    hkRefPtr<hkxIndexBuffer>   m_indexBuffer;   // triangles
    hkRefPtr<hkxMaterial>      m_material;
    // ...
};
```

Vertex weights live inside `hkxVertexBuffer` as per-vertex `(weight×4,
boneIndex×4)` pairs (standard 4-bone skinning).

### Physics module (`hknp*`) — modern HKNP API

| Class | Purpose |
|---|---|
| `hknpBody` | Single rigid body (replaces legacy `hkpRigidBody`) |
| `hknpBodyCinfo` | Body construction info |
| `hknpBodyQuality` | Quality / LOD presets |
| `hknpCompoundShape` | Compound collision shape |
| `hknpMeshShape` | Mesh-based collision (read-only triangle mesh) |
| `hknpCharacterProxy` | Character physics proxy (movement-controller-style) |
| `hknpCharacterProxyManager` | Manages character proxies |
| **`hknpRagdoll`** | **Hierarchy of rigid bodies matching a skeleton** |
| **`hknpRagdollController`** | **Drives the rigid body hierarchy** |
| **`hknpRagdollKeyFrameHierarchyController`** | **Keyframes drive bodies (animation-following)** |
| **`hknpRagdollStatePoweredDynamicAnimation`** | **Animation-driven physics state** |
| `hkaRagdollInstance` | Animation-side ragdoll instance (binds anim ↔ physics) |

Multiple version-migration suffixes (`_0_to_1`, `_1_to_2`, etc.) confirm
HKNP is the actively-used API, with serialized data going through
version-upgrade paths.

### Cloth module (`hcl*`/`hct*`)

| Class | Purpose |
|---|---|
| `hclSimClothData` | Simulated cloth state |
| `hclClothContainer` | Top-level container |
| `hclClothState` / `hclClothStateBuffer` | Per-frame cloth state |
| `hclSimClothPose` | Pose at frame |
| `hctClothSetup20151` | Cloth tool import (Havok Cloth Tool) |

So the engine ALSO has Havok Cloth in use — relevant to cape/hair/garment
deformation that follows skeleton motion.

## Skin-proxy / vertex-weight pipeline as understood from binary

```
┌─────────────────────────────────────────────────────────────────┐
│                       hkaSkeleton (bones)                        │
│       Single source of truth for bone transforms at runtime.     │
└─────────────────────────────┬───────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
    ┌──────────────────┐ ┌──────────┐ ┌────────────────┐
    │  hkaMeshBinding  │ │ hknpRag- │ │ hclSimCloth   │
    │  + hkxMesh       │ │  doll    │ │  Data         │
    │  (vertex weights │ │ (rigid   │ │  (cloth      │
    │   GPU-skinned to │ │  bodies  │ │   proxies)   │
    │   bones)         │ │  matching│ │              │
    └──────────────────┘ │  bones)  │ └────────────────┘
                         └──────────┘
                              │
                              ▼
                   hknpRagdollKeyFrameHierarchyController
                   (animation pose → rigid body transforms)
                              │
                              ▼
                   On hit/death event:
                   hknpRagdollStatePoweredDynamicAnimation
                   (physics blends in, drives skeleton,
                    skin follows because still bound to it)
```

## Implications for mod work

### What "import/export character stuff kinda works"

The skinned mesh format is standard Havok (`hkxMesh` + `hkaMeshBinding` +
`hkaSkeleton`). Any DCC plugin that ingests Havok packfiles (`.hkt`, `.hkx`,
or the binary serialization of the same classes via `hkSerializeUtil`) will
recognize the bone hierarchy + vertex weights. That's why partial roundtrip
works: it's the canonical Havok class set.

The thing that probably feels janky:
- The PAZ archive layer (game-specific encryption + container format)
  wraps the Havok data and isn't a standard Havok packfile.
- The ragdoll/physics layer (`hknpRagdoll` + constraints) is HKNP, which
  is newer and has fewer DCC plugins than the legacy `hkp*` API.
- Cloth setup (`hctClothSetup20151`) is the 2015.1 cloth tool format
  embedded in 2024.2 — older toolchains might not match.

### Path forward for full mesh/skel mod authoring

There are now two layers to consider, with very different parsing paths:

**Layer A — PA reflection metadata (NOW REACHABLE via pycrimson, Session 28).**
The wrapper objects that say "this character uses these meshes / skeletons /
sockets" live in `.prefab` and `.meshinfo` files which go through PA's
reflection serializer. pycrimson parses these directly and emits canonical
PA class+field names. See "Updates from Session 28" below.

**Layer B — Raw Havok binary (still Layer 5, no parser).** The actual
`.pac` / `.pacc` / Havok-tagfile content (hkaSkeleton, hkaMeshBinding,
hkxMesh, hknpRagdoll, hclSimClothData) sits inside files referenced by
the Layer A metadata. To parse those:

1. Identify a referenced asset path in a `.prefab` (e.g. via pycrimson —
   `_skinnedMeshFile._path` resolves to a `.pac` filename).
2. Use pycrimson `extract-pack-files` to pull the `.pac` from the PAZ.
3. Detect Havok packfile signature (`\x57\xE0\xE0\x57\x10\xC0\xC0\x10` for
   binary tagfiles or `<hktagfile` for XML).
4. Use Havok `hkSerializeUtil::load()`-equivalent reader (or roll our own,
   since the schemas are stable per SDK version 2024.2).
5. Walk the loaded `hkRootLevelContainer` for `hkaAnimationContainer`,
   `hkaMeshBinding`, `hknpRagdoll`, `hclClothContainer`.
6. Emit JSON v3.1 schema with field-level intents per Havok class.

### Quick wins available now

- Search the exe for the function that ingests skel+mesh from PAZ to
  identify the loader entry point and which PAZ asset type it consumes.
  (Win-binary IDA recipe from Session 27 applies — bare property literals
  in `.rdata` + xrefs to setter functions.)
- Cross-reference `hkaMeshBinding::staticClass()` to find serializer
  registrations.
- Identify which PAZ class hash maps to skel/mesh assets via PAPGT
  type-table.
- **Run pycrimson on a representative `.meshinfo` file** to recover the
  full PA-side wrapping schema (class names + canonical fields) without
  any IDA work.

Tracked under future work: this isn't a Tier 1 priority since current
mod authoring (DDS texture replacement + pabgb table editing) doesn't
touch the skin pipeline. Document captured here so when the time comes,
we have the receipts.

## Updates from Session 28 (2026-05-10)

The pycrimson reflection workflow lands a partial answer for Layer A.

### What pycrimson covers

LukeFZ/pycrimson parses Pearl Abyss reflection-format files. Per its
`file_format_notes.md`, the relevant ones for the skin pipeline are:

| Extension | Status | Skin-pipeline relevance |
|---|---|---|
| `.prefab` | ✅ reflection | Top-level scene wrapper; references `_skinnedMeshFile`, `_skeletonFileName`, `_socketFileName` paths |
| `.meshinfo` | ✅ reflection | PA-side mesh metadata wrapper |
| `.pae` / `.paem` | ✅ reflection (parc) | PA effect/animation envelope |
| `.parg` / `.pasg` | ✅ reflection | PA animation graph / state graph |
| `.palevel` | ✅ reflection (parc) | PA level descriptor; references prefabs + scene objects |
| `.paseq` / `.paseqc` | ✅ reflection (custom header for paseq) | Sequence data |
| `.paatt` | ❌ NOT reflection | AttackInfo wrapper — uses custom binary container, no canonical names recoverable via pycrimson |
| `.pac` | ❌ NOT reflection | The Havok packfile itself — needs Layer B work above |

Workflow doc: `docs/V3_1_PYCRIMSON_WORKFLOW.md`. Pipeline:
`extract-pack-files` → `parse-serialized-file` →
`scripts/harvest_reflection_schema.py` → `docs/v3_1_reflection_schema.json`.

### Sample harvest result (commit `6273c7f`)

Parsing 502 character `.prefab` files yielded these PA-side wrapper
classes with canonical fields:

| Class | Fields |
|---|---|
| `SceneObject` | `_childSceneObjects`, `_components` |
| `SkinnedMeshComponent` | `_boneOffsetTag`, `_modelPropertyIndex`, `_shrinkMaskDistance`, `_shrinkTag`, `_skeletonFileName`, `_skinnedMeshFile`, `_socketFileName` |
| `ResourceReferencePath_SkinnedMesh` | `_path` |
| `ResourceReferencePath_CharacterSkeleton` | (no fields — pointer-only) |

Important: these are the **PA-side wrappers**. The actual Havok mesh +
skeleton bytes still need a Havok deserializer (Layer B).
`SkinnedMeshComponent._skinnedMeshFile._path` resolves to a `.pac` /
`.pacc` filename inside the PAZ archive — the file pointed at is the
raw Havok packfile.

### Concrete unblocked work for the skin pipeline

1. Run pycrimson against a representative `character/bin__/prefab/...`
   sample to enumerate every `*Component` class PA wraps around Havok
   data. Build the inverse of the table above for diff coverage.
2. Resolve the `_skinnedMeshFile._path` for one specific character (say
   `cd_phm_00_hand_0001.prefab`) and trace it to the `.pac` it points at.
3. Confirm the `.pac` is a Havok binary tagfile (sig `57 E0 E0 57…`).
4. Once confirmed, Layer B work (Havok deserializer) becomes scoped: a
   single SDK-2024.2-compatible reader for the standard Havok schemas.

### What remains blocked

`.paatt` (AttackInfo wrapper), `.pac` (Havok binary). The first is a
custom container with no reflection metadata — see
`docs/T0_AUDIT_TRACKING.md` Session 28 entries for the structural blocker.
The second is genuine Havok format and would need a Havok-SDK-aware
deserializer the way meshcoder, anim-coder, etc. work in DCC plugins.

### Cross-references

- `docs/V3_1_PYCRIMSON_WORKFLOW.md` — pipeline + sample harvest setup
- `docs/V3_1_SCHEMA_VERIFICATION.md` — per-table coverage of the v3.1
  surface (pabgb tables, separate from the reflection workflow)
- `docs/V3_1_DECODER_GAPS.md` — schema fields not yet decoded
- `docs/T0_AUDIT_TRACKING.md` — Session 28 iter notes including the
  `.paatt` structural blocker
- `_research_cache/pycrimson/` — cloned tool
- `_research_cache/extracted_reflection/` — extracted .prefab samples
- `_research_cache/parsed_reflection/` — pycrimson JSON dumps

## Source

All findings pulled via IDA Pro MCP (ida-pro-mcp 1.4.0) string + global
filter scans against CrimsonDesert.exe (retail Windows build).

---

## EffectData Format

# EffectData Binary Format — effectinfo.pabgb

Empirically derived from effectinfo.pabgb dumps via Python hex analysis
(`tools/analyze_effectinfo[1-25].py`). Documents the **outer blob structure**
as seen from the wire; the IDA-derived field-level decoder for the inner
EffectDataElement record lives in `src/binary/variants/effect_data.rs` and
uses a different (non-empirical) analysis layer.

Field names are inferred from context; true names are unknown without IDA
symbol access.

> **Research artifact — partially reconciled with IDA analysis.**
> The empirical blob layout (fixed\_prefix, sub\_elements, inner\_map entries at
> 364 bytes each, real MeshEffectData at 50 bytes each) has been cross-checked
> against `effect_data.rs` / `effect_info/info.rs`. Key corrections applied:
> FP=300 (not 299), all blob sizes +1, 364-byte chunks = `inner_map` entries
> (not "MeshEffectData"), real MeshEffectData = 50 bytes per IDA `sub_1410DBD90`.
> Use the IDA-derived code as ground truth for parsing; use this doc for pattern
> observations and offset landmarks within a single element's bytes.

---

## Version History

Three inner sizes changed across patch versions:

| constant         | pre-4-11 | 4-11  | 4-23 / 4-24 |
|------------------|----------|-------|-------------|
| `fixed_prefix`   | 300      | 287   | 300         |
| `sub_element`    | 316      | 303   | 316         |
| `inner_map_elem` | 364      | 351   | 364         |
| baseline blob    | 324      | 311   | 324         |

The 4-11 patch shrank all three constants; the 4-23 patch reverted them.
The outer container layout is unchanged across all versions.

`fixed_prefix` (FP) = 300 bytes: `byte_a`(1) + `lookup_b`(4) + `EffectDataCoreBlock`(254) +
`lookups_c`(24) + `fields_d`(16) + `byte_e`(1). The last field `byte_e` is always 0 in
vanilla, which caused early empirical analysis (and Snow's doc) to count FP as 299 by
absorbing `byte_e` into the "15 trailing zeros" at the end of Region 5.

**Diff (4-11 → 4-24):** 12 zero bytes inserted at blob offset 172
(= fixed_prefix offset 168) for entries originally 312 bytes; 13 bytes
for entries originally 311 bytes. In both cases the -π/2 constant float
(`db 0f c9 bf`) moved from blob[172] to blob[184]. All common keys between
the two dumps diverge at exactly blob offset 172 (confirmed across 2035/2036
common entries).

---

## Outer pabgb/pabgh Container

`effectinfo.pabgb` uses the standard indexed blob format:

- **`.pabgh`**: u16 entry_count, then entry_count × (u32 key, u32 offset)
- **`.pabgb`**: concatenated entries; each entry spans `[offset .. next_offset)`

Each raw entry starts with:

```
u32_le   key
u32_le   string_len
u8[string_len]  string_key
u8       null          (CString null terminator)
u8       is_blocked
[blob]
u8       has_equip_type    ┐
u8       has_preset        ┤ TAIL_SIZE = 3 bytes, stripped before blob parsing
u8       target_color_lerp_type ┘
```

The "blob" extracted after stripping the outer wrapper and the 3-byte tail
is what the layout below describes.

---

## Blob Layout

```
Offset  Size    Field
──────────────────────────────────────────────────────────────
0       4       CArray<EffectDataElement> count — always 1 in vanilla
4       FP      fixed_prefix  (= EffectDataElement fixed fields; see § Fixed Prefix)
4+FP    4       named_item_count   u32_le  (0 for ~95% of entries)
8+FP    var     string_pairs       named_item_count × StringPair
X       var     struct_section     u32_le count + count × 144-byte NamedItemStruct
Y       var     sub_elements       K × SUB bytes  (K implicit: (mc_off − Y) ÷ SUB)
mc_off  4       inner_map_count    u32_le  (IDA: CArray<{u32 key, EffectDataInner}> count)
mc_off+4  n×364   inner_map_elems  n × 364-byte inner_map entry (key + EffectDataInner)
end−8   8       {0,0,0,0,0,0,0,0}  trailing zeros (= inner_map_count=0 + mesh_count=0
                                    when no mesh; see IDA note below)
```

> **IDA-reality note:** Per `info.rs`, the actual blob wire format is
> `[effect_count(4)][EffectDataElement(var)][mesh_count(4)][m×50 MeshEffectData]`.
> The empirical "8 trailing zeros" = the inner_map CArray count (last 4 bytes of
> EffectDataElement, = 0 when n=0) plus the outer mesh_count (= 0 when m=0). For
> entries with m>0 real mesh elements the trailing section is
> `[inner_map_count=0][mesh_count=m][m×50 bytes]` — not all zeros.
>
> **Naming note:** Early empirical analysis (and Snow's external doc) called the 364-byte
> chunks "MeshEffectData." Per IDA they are `inner_map` entries: `u32 key +
> EffectDataInner`. The 364-byte wire size holds when all of EffectDataInner's embedded
> CArrays are empty. Real `MeshEffectData` per IDA `sub_1410DBD90` is 50 bytes:
> `u8 + 8×u32 + u8 + 4×u32` (lookups).

Where per-version constants are:

| symbol   | 4-11 | 4-23/4-24 |
|----------|------|-----------|
| FP       | 287  | 300       |
| SUB      | 303  | 316       |
| MESH     | 351  | 364       |

And:

```
X      = (8+FP) + Σ(4 + len_i)  for i in 0..named_item_count
           ↑ 4 bytes = the length-prefix for each string pair
           ↑ len_i  = byte length of the i-th string (no null terminator)

Y      = X + 4 + named_item_count × 144
           ↑ 4 bytes = the struct_section count prefix
           ↑ struct count always equals named_item_count

mc_off = Y + K × SUB   (solve from blob_size: see mc_off detection below)
```

**mc_off detection** — two-step: (1) iterate candidate `n` (inner_map count) from
largest to smallest; for each, compute `mc_off = blob_size − 8 − n×364`; check
that `u32_le(blob, mc_off) == n`. (2) verify `(mc_off − Y) % SUB == 0`.
The divisibility check is required to avoid false positives: when inner_map data
happens to be zero at a candidate mc_off, step (1) alone gives wrong n.
For entries with real mesh (m>0) the blob is longer by `m×50` — `find_mesh_split`
in `info.rs` locates the mesh boundary first, then mc_off detection runs on the
effect-data sub-slice.

---

## Size Examples

### 2026-4-11 pabgb (2039 entries, FP=287, SUB=303, MESH=351)

General formula: `blob_size = 311 + named_items_extra + K×303 + inner_map×351 + mesh×50`

where `named_items_extra = Σ(4+len_i) + named_item_count×144`.

| blob_size | named_item_count | K (×303) | inner_map (×351) | mesh (×50) | notes |
|-----------|-----------------|----------|-----------------|------------|-------|
| 311       | 0               | 0        | 0               | 0          | baseline (1935 entries) |
| 463       | 1 ("leaf", 4)   | 0        | 0               | 0          | |
| 464       | 1 ("dist1", 5)  | 0        | 0               | 0          | |
| 465       | 1 ("smoke1", 6) | 0        | 0               | 0          | |
| 615       | 2               | 0        | 0               | 0          | |
| 616       | 2               | 0        | 0               | 0          | |
| 614       | 0               | 1        | 0               | 0          | |
| 917       | 0               | 2        | 0               | 0          | |
| 662       | 0               | 0        | 1               | 0          | |
| 1013      | 0               | 0        | 2               | 0          | |
| 1364      | 0               | 0        | 3               | 0          | |
| 1715      | 0               | 0        | 4               | 0          | |

### 2026-4-24 pabgb (2057 entries, FP=300, SUB=316, MESH=364)

General formula: `blob_size = 324 + named_items_extra + K×316 + inner_map×364 + mesh×50`

| blob_size | named_item_count | K (×316) | inner_map (×364) | mesh (×50) | notes |
|-----------|-----------------|----------|-----------------|------------|-------|
| 324       | 0               | 0        | 0               | 0          | baseline (1952 entries) |
| 476       | 1 ("leaf", 4)   | 0        | 0               | 0          | |
| 477       | 1 ("dist1", 5)  | 0        | 0               | 0          | |
| 478       | 1 ("smoke1", 6) | 0        | 0               | 0          | |
| 628       | 2               | 0        | 0               | 0          | |
| 629       | 2               | 0        | 0               | 0          | |
| 374       | 0               | 0        | 0               | 1          | one real MeshEffectData (50 bytes) |
| 640       | 0               | 1        | 0               | 0          | |
| 956       | 0               | 2        | 0               | 0          | |
| 688       | 0               | 0        | 1               | 0          | |
| 1052      | 0               | 0        | 2               | 0          | 26 entries |
| 1416      | 0               | 0        | 3               | 0          | |
| 1780      | 0               | 0        | 4               | 0          | |
| 2144      | 0               | 0        | 5               | 0          | |
| 2508      | 0               | 0        | 6               | 0          | |
| 2872      | 0               | 0        | 7               | 0          | 19 entries |
| 6148      | 0               | 0        | 16              | 0          | max observed |
| 832       | 0               | 0        | 1†              | 0          | irregular — bone-name inner_map body (+144 bytes) |
| 2536      | 0               | 7        | 0               | 0          | confirmed: 312 + 7×316 + 12 = 2536 (`Weapon_Fire_ing`) |
| 1788      | 0               | —        | —               | —          | irregular — split-reference (K=5, see Type C below) |
| 2152      | 0               | —        | —               | —          | irregular — split-reference (K=6, see Type C below) |

---

## StringPair

```
u32_le  len          (byte length of name, no null terminator in stream)
u8[len] name         (ASCII, e.g. "leaf", "core", "sub", "par1", "vector1")
```

All string pairs for an entry are stored consecutively (no structs between them).

---

## NamedItemStruct (144 bytes)

Immediately follows the struct_section count u32 (which equals named_item_count).
One struct per named item, same order as the string pairs. The struct_section
header is a single `u32=N` count (4 bytes only). The 12 zero bytes that appear
to follow the count in most entries are the first struct's colour field
(struct[0..12] = f32[3] default (0,0,0)) — not extra header padding. Blob size
examples confirm the 4-byte interpretation: a 1-named-item 475-byte blob gives
4+299+4+(4+4)+( **4** +144)+0+4+0+8 = 475, whereas a 16-byte header gives 487.

Internal layout from systematic byte scan across all 27 named item structs
(27 entries across 4-24 475–933 blobs):

| offset (within struct) | size | observation |
|------------------------|------|-------------|
| 0..12                  | 12   | f32[3]: **named-item colour** — independent of prefix color1/color2; default (0,0,0); e.g. (0.784, 0.392, 0.078) for one `leaf` component |
| 12..24                 | 12   | f32[3]: **named-item secondary colour** — default (0,0,0); not necessarily equal to prefix color2 |
| 24..36                 | 12   | f32[3]: mirrors prefix[40:52] — all three = 0.0f or all three = 0.05f (`cd cc 4c 3d`); only set when prefix[40:52] is set |
| 36..84                 | 48   | zeros (`vec_d`–`vec_g` all zero in all 27 vanilla entries) |
| 84..88                 | 4    | f32 `field_84`: per-stage intensity multiplier — same role as prefix[88:92]; 0.0 default, 0.3 for one `dist1` entry |
| 88..92                 | 4    | f32 `field_88`: 0.0 (`smoke1`), 0.3 (`smoke1` variant), or 1.0 (most components) |
| 92..96                 | 4    | u32 `field_92`: **{0, 2, 30}** — nonzero **only** for `par1` components (2 of 27 structs; values 2 and 30); likely particle emitter burst-count |
| 96..100                | 4    | always 0 |
| 100..104               | 4    | f32 = 1.0 (constant; 0.0 for `vector1`/`vector2` components) |
| 104..108               | 4    | f32 = 1.0 (constant; 0.0 for `vector1`/`vector2` components) |
| 108..112               | 4    | f32 = **−1.0** (constant sentinel — same role as prefix[112:116]) |
| 112..116               | 4    | f32 = 0.0 (constant) |
| 116..120               | 4    | f32 = 1.0 (constant) |
| 120..124               | 4    | f32 = 1.0 (constant) |
| 124..128               | 4    | f32 = 0.0 (constant) |
| 128..132               | 4    | f32 = 1.0 (constant) |
| 132..136               | 4    | f32 = 1.0 (constant) |
| 136..138               | 2    | `0a 05` (constant type marker — same as prefix[140:142]) |
| 138..140               | 2    | u16 bitmask flags (same role as prefix[142]); values: 0x0000, 0x0001, 0x0004, 0x0009, 0x0010, 0x0020, 0x0100, 0x0102 |
| 140..144               | 4    | zeros |

Total size confirmed: 144 bytes across all 27 entries.

**Field alignment:** NamedItemStruct IS a standalone D3Block (144 bytes). Within
EffectDataElement, the same D3Block sits at `core_block[0..144]`, which is at
prefix offset 4 (after `lookup_b`). So `struct[X] ≡ prefix[X+4]` for every
D3Block field. E.g. `field_92` at struct[92:96] ≡ prefix[96:100];
`byte_136/byte_137` type-marker at struct[136:138] ≡ prefix[140:142]. No TRS
or hash/ID region (the NamedItemStruct is a D3Block only, not a CoreBlock).

**EffectDataD3Block semantic labels** (Rust field name ↔ semantic meaning from cross-analysis):

| Rust field   | struct offset | semantic                                                  |
|-------------|--------------|-----------------------------------------------------------|
| `vec_a`     | 0..12        | named-item colour (RGB f32[3])                            |
| `vec_b`     | 12..24       | named-item secondary colour (RGB f32[3])                  |
| `vec_c`     | 24..36       | 0.0 or 0.05f triplet (mirrors prefix[40:52])              |
| `vec_d`–`vec_g` | 36..84  | all zero in vanilla (D3Block color/scale fields)          |
| `field_84`  | 84..88       | per-stage intensity multiplier — f32; 0.0 default; same role as prefix[88:92] |
| `field_88`  | 88..92       | type-dependent scale: 0.0, 0.3, or 1.0 across component types |
| `field_92`  | 92..96       | particle emitter parameter — u32 {0, 2, 30}; nonzero only for `par1` components |
| `byte_136`  | 136..137     | type marker byte 1 = 0x0a (same as prefix[140])          |
| `byte_137`  | 137..138     | type marker byte 2 = 0x05 (same as prefix[141])          |
| `word_138`  | 138..140     | bitmask flags — u16 (same role as prefix[142:144])        |

`field_96`–`field_132`, `vec4_a`, and `field_140` are IDA-derived anonymous names;
semantics follow the mirror relationship `struct[X] ≡ prefix[X+4]` (see above).

---

## Sub-Element (303 bytes in 4-11, 316 bytes in 4-24)

Present when `(mc_off − Y) > 0`. Count K is implicit (no count field stored).
Mapped from the 4-24 640-blob (K=1), sub-element at blob[312..628].

**Header (bytes 0..92 — all zeros except the header packet):**

| offset (within sub-element) | size | observation |
|-----------------------------|------|-------------|
| 0..8                        | 8    | zeros |
| 8                           | 1    | `0x01` (constant — version or type byte) |
| 9..13                       | 4    | **sub-element type ID** — 4 bytes identifying the sub-element class; shared across all blobs of the same class (e.g. `57 04 06 24` for 5 different 639-blobs, `79 1c a5 9a` for two 955-blobs and one sub within a 2535-blob). Not per-blob-unique. |
| 13..92                      | 79   | zeros |

**Body (bytes 9..316):**

The inner EffectData record begins at sub[9]. Its fields mirror
fixed_prefix[0..] with a +9 byte offset: sub[9+X] corresponds to prefix[X].
Byte-match sub[9+X] vs prefix[X] = 272/299 identical across available entries
(variable fields like TRS and IDs naturally differ).

| sub offset    | prefix equiv  | description |
|---------------|---------------|-------------|
| 9+92 = 101    | prefix[92]    | inner sub-struct float cluster begins |
| 9+140 = 149   | prefix[140]   | `0a 05` type marker |
| 9+200 = 209   | prefix[200]   | position XYZ (real values, e.g. (−0.020, 0, −0.237)) |
| 9+212 = 221   | prefix[212]   | scale XYZ (e.g. (0.5, 1, 1)) |
| 9+224 = 233   | prefix[224]   | rotation XYZ radians |

Sub-element TRS differs from the parent blob's TRS — each sub-element has
its own transform relative to the parent effect origin.

---

## InnerMapElement (351 bytes in 4-11, 364 bytes in 4-24)

> **Naming:** Snow's doc and early empirical analysis called these "MeshEffectData." Per IDA
> they are `inner_map` entries: `u32 key + EffectDataInner`. The 364-byte wire size applies
> when all of EffectDataInner's embedded CArrays are empty. This section retains the
> landmark offsets from empirical analysis; see `effect_data.rs` for the field-typed
> IDA-derived decoder.

Location: immediately after `inner_map_count` u32 at `mc_off + 4`.
In 4-24, confirmed up to n=16 (6148-byte blob).

**Activity flag:** mesh[0] = u8, either 0x01 (active) or 0x00 (null slot).
Only mesh[0] carries a full slot directory; trailing null slots carry only a
truncated self-reference and their own hash.

**Slot directory (mesh[0] only, variable size 20..68 bytes depending on M):**

mesh[0] encodes a linked "slot directory" covering all M active mesh slots.
The directory is `M×8 + 12` bytes (20 bytes minimum for M=1):

```
[0:4]         = 1 (active flag — slot 0)
[4:8]         = hash_A (slot 0's own hash/ID)

For k = 1 .. M-1:               ← (M-1) additional slots; empty range for M=1
  [8k:8k+4]   = 1 (active flag for slot k)
  [8k+4:8k+8] = hash_k (slot k's hash/ID)

[8M:8M+4]     = M  (total slot count)
[8M+4:8M+8]   = hash_A (repeated)
[8M+8:8M+12]  = hash_A (repeated again)
[8M+12 : 80]  = zeros (for standard entries; some complex entries override)
```

Concrete examples (from 70-sample scan — all M values confirmed):

| M | directory bytes | M field at | hash_A terminator at |
|---|-----------------|------------|----------------------|
| 1 | 20  | [8:12]  | [12:20] |
| 2 | 28  | [16:20] | [20:28] |
| 3 | 36  | [24:28] | [28:36] |
| 4 | 44  | [32:36] | [36:44] |
| 5 | 52  | [40:44] | [44:52] |
| 6 | 60  | [48:52] | [52:60] |
| 7 | 68  | [56:60] | [60:68] |

For M=1: no additional pairs (k range 1..0 is empty); directory = 20 bytes.
For M≥2: each pair k=1..M-1 references one of the other active mesh slots.

**Null/trailing mesh slots** contain only a compact back-reference (no directory):
zeros at [0:4] (inactive), hash at [4:8], zeros elsewhere in [0:80].

**Remaining mesh[0..108] fields:**

| mesh offset | size | description |
|-------------|------|-------------|
| 0..8M+12    | var  | slot directory (see above) |
| 8M+12..80   | var  | zeros for standard entries; one observed entry has f32=4.0 and RGB at [44:56] |
| 80..104     | 24   | zeros (confirmed across all 70 active mesh[0] samples) |
| 104..108    | 4    | f32: 0.0 or ~1.4 (1 of 70 active samples) |

**Shared inner sub-struct (mesh[M×8+100..364]):**

Starts at mesh[M×8+100] and mirrors fixed_prefix[92..300] (208 bytes) with the
alignment `mesh[M×8+100+X] ≅ prefix[92+X]`. The `0a 05` marker therefore lands
at mesh[M×8+148] for every M (confirmed for all M=1..16).

Landmarks below use M=1 base offsets (sub-struct start = mesh[108]); for M>1
add M×8 to each mesh offset:

| mesh offset (M=1) | prefix equiv     | landmark |
|-------------------|------------------|----------|
| 108               | prefix[92]       | float cluster start (f32 ≈ 1.0) |
| 156               | prefix[140]      | `0a 05` type marker |
| 196               | prefix[180]      | −π/2 constant |
| 216..252          | prefix[200..236] | TRS (position, scale, rotation) |
| 252               | prefix[236]      | constant `0x00000001` |
| 268               | prefix[252]      | constant `0x01000005` |
| 274..298          | prefix[258..282] | lookups_c (6×u32, null sentinel `0xeac5e173`) |
| 298..315          | prefix[282..299] | fields_d (16 bytes) + byte_e |

**Sub-struct boundary and mesh[316..364] content:**

The sub-struct occupies mesh[M×8+100 .. M×8+308]. For M=7 the sub-struct
exactly fills the mesh to byte 364. For M≤6 the sub-struct ends before 364
and the remainder is zeros. For M≥8 the sub-struct is truncated at byte 364
and the tail of Region 5 / Region 4 is absent from the mesh.

| M  | sub-struct end | mesh[316:364] content |
|----|----------------|-----------------------|
| 1  | mesh[316]      | all zeros (sub-struct ends at 316) |
| 2  | mesh[324]      | [316:324] = last 8 bytes of fields_d/byte_e (zeros); [324:364] zeros |
| 3  | mesh[332]      | [316:332] = last 16 bytes of fields_d region (zeros); [332:364] zeros |
| 4  | mesh[340]      | [316:340] = lookups_c[4..5] + fields_d; [340:364] zeros |
| 5  | mesh[348]      | [316:348] = lookups_c[3..5] + fields_d; [348:364] zeros |
| 6  | mesh[356]      | [316:356] = prefix[252..292] (`0x01000005` + lc[0..4]); [356:364] zeros |
| 7  | mesh[364]      | [316:364] = prefix[252..300] (`0x01000005` + lc[] + fields_d + byte_e) |
| 8  | mesh[372]†     | [316:364] = prefix[244..292] (Region 5 flags + lookups_c partial) |
| 16 | mesh[436]†     | [316:364] = prefix[180..228] (−π/2 + Region 3 zeros + TRS partial) |

† Truncated at mesh boundary; prefix bytes past the cutoff are absent.

---

## Fixed Prefix (blob[4 .. 4+FP])

Full field-level map from systematic byte and 4-byte-window scans across all
1952 baseline blobs in the 4-24 dump (FP=300, prefix offset = blob offset − 4).

### Region 1 — Colour parameters (prefix[0..92])

| prefix offset | size | type    | description |
|---------------|------|---------|-------------|
| 0..4          | 4    | zero    | always zero |
| 4..16         | 12   | f32[3]  | **grayscale tint** (all three always equal): default (0,0,0); 5 entries set value to 0.3/0.5/0.6/0.85. Mutually exclusive with color1/color2 — entries use one or the other. |
| 16..28        | 12   | f32[3]  | **color1** (start color, RGB normalised 0..1): default (0,0,0); 47 entries non-zero |
| 28..40        | 12   | f32[3]  | **color2** (end color, RGB normalised 0..1): default (0,0,0); 22 entries non-zero. When both color1 and color2 are non-zero they are usually equal (constant color). |
| 40..52        | 12   | f32[3]  | 3 floats, each 0.0 or 0.05f (`cd cc 4c 3d`); only 2 of 1952 entries use this |
| 52..88        | 36   | zero    | always zero |
| 88..92        | 4    | f32     | **per-stage intensity multiplier** (IDA: `d3.field_84`; Rust: `EffectDataD3Block.field_84`, typed u32 but semantically f32): default 0.0 (99.3% of entries). All 15 non-zero entries are `_switch_` or ribbon effects. Wolf-claw switch series: _01=0.3, _02=0.5, _03=1.0 — a 3-stage opacity ramp (30%→50%→100%). Ribbon entries: damian variants = 1.0, com (common player) variants = 1.5. Encodes per-switch-state brightness; 0.0 = inactive/no override. |

### Region 2 — Inner sub-struct (prefix[92..145])

This block mirrors the same inner structure found in NamedItemStruct[88..141]
and in sub-elements at sub[92..145]. Offset −4 shift in NamedItemStruct.

| prefix offset | size | type    | description |
|---------------|------|---------|-------------|
| 92..96        | 4    | f32     | ~99.7% = 1.0; 5 unique values, range 0.02..1.0 |
| 96..104       | 8    | f32[2]  | 0.0, 0.0 (constant) |
| 104..108      | 4    | f32     | ~98% = 1.0; 13 unique values, range 0.3..3.0 |
| 108..112      | 4    | f32     | ~98% = 1.0; 16 unique values, range 0.0..4.0 |
| 112..116      | 4    | f32     | default **−1.0** (sentinel); 6 unique values, can be positive |
| 116..120      | 4    | f32     | 0.0 (constant) |
| 120..128      | 8    | f32[2]  | 1.0, 1.0 (constant) |
| 128..132      | 4    | f32     | 0.0 (constant) |
| 132..140      | 8    | f32[2]  | 1.0, 1.0 (constant) |
| 140..142      | 2    | u8[2]   | `0x0a 0x05` — **constant type marker** |
| 142..143      | 1    | u8      | bitmask flags: 96% zero; nonzero values are powers of 2 {2,4,6,8,16,32,48} |
| 143..144      | 1    | u8      | bool: 0 (1950 entries) or 1 (2 entries) |
| 144..145      | 1    | u8      | enum 0..5: 73% zero, then 1(23%), 2(2%), 3(1%), 4(0.5%), 5(0.2%) |

### Region 3 — Zero padding (prefix[145..200])

All 55 bytes are constant zero in every entry. This includes the 12 zero
bytes inserted at prefix[168:180] in the 4-24 patch (absent in 4-11 where
the `−π/2` constant began at prefix[168]).

Exception embedded within the zero run:

| prefix offset | size | type | description |
|---------------|------|------|-------------|
| 180..184      | 4    | f32  | `db 0f c9 bf` = **−π/2 ≈ −1.5708** (constant) |

### Region 4 — Transform (prefix[200..236])

TRS (translation, scale, rotation) transform for this effect element,
confirmed from non-trivial entries (e.g., fire-effect entries with realistic
position, rotated turret effects with π/2 angles, etc.).

| prefix offset | size | type   | description |
|---------------|------|--------|-------------|
| 200..212      | 12   | f32[3] | **position** XYZ offset — default (0,0,0); range ~±40 |
| 212..224      | 12   | f32[3] | **scale** XYZ — default (1,1,1); **never zero**; range −1..10 |
| 224..236      | 12   | f32[3] | **rotation** XYZ in radians — default (0,0,0); range ±π |

Sample non-trivial entries:
- `pos=(0,0,0.75)  scale=(1,1,1)  rot=(0, π/2, 0)` — vertical offset, 90° yaw
- `pos=(−0.36,0,0) scale=(1.5,0.7,1.5)` — lateral shift, non-uniform scale
- `pos=(0,0,0.035) scale=(2.5,2.5,2.5)` — vertical offset, uniform upscale
- `scale=(0.05,0.05,0.01)` — tiny uniform scale

### Region 5 — Flags and IDs (prefix[236..299])

| prefix offset | size | type | description |
|---------------|------|------|-------------|
| 236..240      | 4    | u32  | constant `0x00000001` (single `0x01` byte) |
| 240..244      | 4    | —    | mostly zero; 2 of 1952 entries non-zero |
| 244..248      | 4    | u8[4]| `{0x00, 0x01, 0x00, 0x00}` for most entries (byte 245 = 1) |
| 248..252      | 4    | u8[4]| `{0x01, 0x00, 0x00, X}` where X ∈ {0,1,2,3,5} — byte 251 is an enum |
| 252..256      | 4    | u32  | constant `0x01000005` (bytes: `05 00 00 01`) |
| 256..258      | 2    | u8[2] | `0x00 0x00` — EffectDataCoreBlock byte_252/byte_253 (last two bytes of CoreBlock, always zero) |
| 258..282      | 24   | u32[6]| **`lookups_c[0..6]`** (Rust: `EffectDataElement.lookups_c`; IDA: `read_u32_lookup_DA30`): 6 × u32 effect hash. Null sentinel = `0xeac5e173` (bytes `73 e1 c5 ea`). See table below. |
| 282..298      | 16   | u32[4]| **`fields_d[0..4]`** (Rust: `EffectDataElement.fields_d`): all zero in vanilla |
| 298..299      | 1    | u8    | **`byte_e`** — always 0 in vanilla; IDA reads as named field (`EffectDataElement.byte_e`), making FP=300 not 299 |

**lookups_c detail (prefix[258..282], 6 × u32, null sentinel `0xeac5e173`):**

| slot  | prefix offset | unique values | null %  | notes |
|-------|---------------|---------------|---------|-------|
| lc[0] | 258..262      | ~250 groups   | 0%      | **effect group hash** — groups L/R mirror pairs, same-character body-part variants, and same-weapon-type variants; 1521 unique low-u16 / 1741 unique high-u16 across 2057 entries |
| lc[1] | 262..266      | 118 distinct  | ~50%    | secondary hash; role unknown |
| lc[2] | 266..270      | 2 distinct    | ~99%    | nearly always null |
| lc[3] | 270..274      | 3 distinct    | ~99%    | nearly always null |
| lc[4] | 274..278      | 22 distinct   | ~96%    | null except ~4%; non-null correlates with sub-element effects |
| lc[5] | 278..282      | 1 (null only) | 100%    | always null |

---

## Coverage

| dump      | entries | parsed | failures | failure sizes |
|-----------|---------|--------|----------|---------------|
| 2026-4-11 | 2039    | ~2035  | ~5       | 361×2, 806×1, 1356×2, 1723×1, 2074×1 (sizes +1 from original; not re-classified) |
| 2026-4-24 | 2057    | 2054   | 3        | 832×1 (TypeA), 1788×1 (TypeC), 2152×1 (TypeC) |

4-24 failure sizes are exactly 13 more than their 4-11 counterparts. Types B, D, and E
are standard after the FP=300 correction:
- Type B (374-byte) = standard mesh=1 (one 50-byte MeshEffectData)
- Type D (2536-byte) = standard K=7 sub-elements
- Type E (1416-byte) = standard inner_map=3

4-11 failure classification not re-verified after FP correction; some may also resolve.

---

## Irregular Blobs

The 3 irregular entries in 4-24 (after reconciliation; Types B, D, and E are now
standard). Byte offsets below use FP=300 boundaries:

### Type A — Bone-name inner_map body (832-byte blob)

One entry (`pafx_mc_rotationbash_lightning_gain_001a_switch_01`) has inner_map=1
but the entry body is variable-length because it embeds a bone name list and bone
weight array. The outer layout is identical to a standard inner_map blob
(blob[304:312]=8 zeros, blob[312:316]=inner_map_count=1, body, 8 trailing zeros),
but the body is 508 bytes instead of the standard 364.

The bone name list begins at **mesh offset 298**:

```
                                        ... (298 bytes standard mesh data)
07 00 00 00             ← bone_count = 7
0b 00 00 00  42 69 70 30 31 20 53 70 69 6e 65          "Bip01 Spine" (11)
0c 00 00 00  42 69 70 30 31 20 53 70 69 6e 65 31       "Bip01 Spine1" (12)
0c 00 00 00  42 69 70 30 31 20 53 70 69 6e 65 32       "Bip01 Spine2" (12)
10 00 00 00  42 69 70 30 31 20 52 20 43 6c 61 76 69 63 6c 65  "Bip01 R Clavicle" (16)
10 00 00 00  42 69 70 30 31 20 52 20 55 70 70 65 72 41 72 6d  "Bip01 R UpperArm" (16)
0d 00 00 00  42 69 70 30 31 20 52 20 45 6c 62 6f 77          "Bip01 R Elbow" (13)
0c 00 00 00  42 69 70 30 31 20 52 20 48 61 6e 64             "Bip01 R Hand" (12)
06 00 00 00             ← weight_count = 6
89 88 08 3e  ×6         ← 6 × f32 ≈ 0.1333 (bone weights per bone attachment?)
[58 trailing zeros]
```

Total body size: 508 bytes (298 standard + 4 count + 120 names + 4 count + 24 weights + 58 zeros).

### Type C — Split-reference mesh (1787, 2151-byte blobs)

Two entries use a "split header" format where K mesh headers are stored
separately from K−1 mesh bodies:

```
blob[304:312]         8 zeros (standard — named_item_count=0, struct_count=0)
blob[312:316]         K  (u32 reference count — NOT inner_map count)
blob[316:316+K×8]     K reference entries, each = (u32=1, u32=hash)
blob[316+K×8:end-8]   M = K−1  bodies, each 356 bytes
blob[end-8:end]       8 trailing zeros
```

Size formula: `316 + K×8 + (K−1)×356 + 8 = 364×K − 32`

| blob size | K | M=K-1 | entry name |
|-----------|---|-------|------------|
| 1788      | 5 | 4     | `pafx_Swim_Foot_Warmachine` |
| 2152      | 6 | 5     | `fx_smokeshell_out` |

The last two reference entries always share the same hash (a back-reference or
deduplication marker). Body layout (each 356 bytes):

| body offset | size | description |
|-------------|------|-------------|
| 0..4        | 4    | M (total body count) for body[0]; 0 for body[i>0] |
| 4..8        | 4    | hash — equals ref[i].hash for the corresponding reference entry |
| 8..12       | 4    | same hash repeated |
| 12..100     | 88   | zeros |
| 100..308    | 208  | **inner sub-struct** (≅ prefix[92..300]) — same layout as InnerMapElement[108..316]; `body[100+X] ≅ prefix[92+X]`; all landmarks confirmed |
| 308..356    | 48   | tail — all zeros (confirmed across all 9 bodies: 4×pafx_Swim + 5×fx_smokeshell_out) |

Key landmarks within the inner sub-struct (all confirmed):

| body offset | prefix equiv    | landmark |
|-------------|-----------------|----------|
| 100         | prefix[92]      | float cluster start (f32 = 1.0) |
| 148:150     | prefix[140:142] | `0a 05` type marker |
| 188:192     | prefix[180:184] | −π/2 constant |
| 208:244     | prefix[200:236] | TRS — per-body position/scale/rotation (e.g. pafx_Swim body 0: pos=(−0.36, 0.7, −0.4), scale=(1.8, 1.8, 1.8)) |
| 244:248     | prefix[236:240] | constant `0x00000001` |
| 260:264     | prefix[252:256] | constant `0x01000005` |
| 266:290     | prefix[258:282] | lookups_c (6×u32, null sentinel `0xeac5e173`) |

The body's inner sub-struct uses the same layout as InnerMapElement[108..316], with
the base offset 8 bytes earlier in the body's own coordinate space.

### Type D — Extended sub-element blob (2536-byte blob)

> **Reconciled:** this is a standard blob with K=7 sub-elements; not irregular.

One entry (`Weapon_Fire_ing`) has K=7 sub-elements, fitting the standard
sub-element formula `312 + K×316 + 12`:

```
312 + 7×316 + 12 = 2536  ✓
```

Each sub-element starts at blob[312 + i×316] with the standard header
`00 00 00 00 00 00 00 00 01` (8 zeros + 0x01). The trailing 12 zeros are
also standard. This blob fits the general formula — it was previously
miscounted because K=7 is larger than the K≤2 samples used to calibrate.

### Type E — Reconciled: standard inner_map=3 (1416-byte blobs)

> **Reconciled:** these entries are no longer classified as irregular. With FP=300
> the correct size for inner_map=3 is 324 + 3×364 = 1416, not 1407. The earlier
> "keyed-entry" analysis was derived from a wrong byte boundary (byte_e missed).
> Both entries fit the general formula with inner_map_count=3.

Two entries (`cdfx_mc_onguard_shield_fxpreset_01` and
`cdfx_mc_onguard_shield_fxpreset_01_applyAnimationSpeed`) — blob_size 1416 =
baseline(324) + 3×364(inner_map). Standard layout; no special handling required.

---

## Next Steps

1. ~~**prefix[88:92]**~~ **Resolved**: per-stage intensity multiplier (`d3.field_84`, f32).
   All 15 non-zero entries are `_switch_`/ribbon effects; wolf-claw switch stages step
   0.3→0.5→1.0 (_01/_02/_03). IDA types field as u32; actual wire values are clean f32.
   prefix[40:52] confirmed to mirror NamedItemStruct struct[24:36] (0.05f triplet).

2. ~~**Identify prefix[256:264] IDs**~~ **Resolved**: prefix[256:258] = CoreBlock byte_252/byte_253
   (always zero); prefix[258:282] = `lookups_c[0..6]` — 6 × u32 effect hashes.
   lc[0] is an **effect group hash** grouping visual variants (L/R mirrors, body parts,
   weapon-type variants); ~250 groups across 2057 entries. lc[1..5] are secondary hashes
   with decreasing cardinality; lc[5] is always null. Null sentinel = `0xeac5e173`. Earlier
   "unique per-entry ID" interpretation was an artefact of reading lc[0] as two u16s in
   isolation.

3. ~~**inner_map slot directory hashes**~~ **Resolved**: `InnerMapEntry.key` (the u32 map
   key stored at `mc_off+4 + n×364`) is a **Crimson Desert Jenkins hashlittle2 hash of a
   skeleton bone name** (init = `length + 0xDEBA1DCD`, returns `c`). The same value is
   redundantly stored as `EffectDataInner.field_0` (first u32 of the inner body). 9 of 168
   unique keys were cross-verified against `stringinfo.pabgb`: all decode to Biped
   skeleton bones (`Bip_Sphere_01`, `Bip_Spine_03/04/05`, `Bip_Spin_01..05` for the Kutum
   boss entry). The remaining 159 unique keys are almost certainly additional bone names
   not present in `stringinfo`. Confirmed NOT blob keys within effectinfo.pabgb (0 hits);
   the "different archive" hypothesis was wrong — these are inline skeleton attachments.

4. ~~**NamedItemStruct struct[80:84]**~~ **Resolved**: the field was at the wrong offset
   in the doc. Actual position is **struct[92:96]** (`field_92`); doc table rows 72:76
   through 128:144 were all wrong (cascading offset error). Corrected in full. `field_92`
   = u32 ∈ {0, 2, 30}, nonzero for exactly 2 of 27 par1 particle components
   (`fx_Soul_spear_On_Lv3` → 2, `fx_ancient_aura_a_aura1_custom1` → 30); likely a
   particle burst-count. Adjacent fixes: `field_84` = intensity multiplier (not zeros);
   `field_88` = type-dependent scale (not constant 1.0); byte_136/137/word_138 roles
   corrected (marker bytes are byte_136=0x0a, byte_137=0x05; word_138=bitmask, not
   the other way around); float-cluster alignment note corrected to `struct[X] ≡
   prefix[X+4]`.

5. ~~**Type C body remainder (body[152:356])**~~ **Resolved**: full mapping confirmed.
   body[100..308] = inner sub-struct ≅ prefix[92..300] (same structure as
   InnerMapElement[108..316]); body[308..356] = 48-byte tail, all zeros (verified
   across all 9 bodies: 4×pafx_Swim_Foot_Warmachine + 5×fx_smokeshell_out). All key
   landmarks confirmed: body[148:150]=`0a 05`, body[188:192]=−π/2, body[208:244]=TRS
   with per-body real values, body[244:248]=`0x01`, body[260:264]=`0x01000005`,
   body[266:290]=lookups_c with null sentinels. The 48-byte tail is entirely zeros —
   no hidden fields beyond the prefix-equivalent range.

6. ~~**InnerMapElement inner sub-struct tail (mesh[316..364], 48 bytes)**~~ **Resolved**:
   the sub-struct does not start at a fixed mesh[108]. It starts at mesh[M×8+100],
   shifting 8 bytes per active slot. The `0a 05` marker is at mesh[M×8+148] for every
   M=1..16. Sub-struct end = mesh[M×8+308]; for M=7 this is exactly mesh[364] (no
   tail). mesh[316:364] is NOT padding: for M≤6 it is the trailing fields_d/byte_e
   region of the sub-struct followed by zeros; for M≥7 it is an interior slice of
   Region 5 / Region 4 data (lookups_c, TRS, or −π/2 depending on M). For M≥8 the
   sub-struct is truncated at the 364-byte mesh boundary. Slot directory formula
   also corrected: M×8+12 bytes (not max(M,2)×8+12); M=1 directory = 20 bytes with
   no self-reference pair.

---

## Binary Asset Field-Level Roadmap

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

---

## Workbench File Format Handoff

# Workbench → DMM Field-JSON v3 Handoff: File-Format Tables

**Audience:** NattKh / mod-workbench maintainer
**Author:** RicePaddySoftware (DMM)
**Status:** dmm-parser Phase 1+2 + DMM Phase 3 complete (pre-release.7).
Workbench-side v3-intent export is the missing piece — that's the ask
in this doc.
**Date:** 2026-05-09

---

## What just landed in DMM (pre-release.7)

DMM now applies field-JSON-v3 intents end-to-end against the four file-format tables Workbench currently exports as PAZ-overlay folders:

- `paac` (action chart — `commonactioninfo.paac`, `*_upper.paac`)
- `paatt` (projectile attribute — `actionchart/projectileinfo*.paatt`)
- `pamhc` (`miscellaneous/modelpropertyheadercollection.pamhc`)
- `pappt` (`character/bin__/partprefabtable.pappt`)

The pipeline:

1. Mod manifest declares a target whose key is the file's name
   (`partprefabtable.pappt`) or vfs path
   (`character/bin__/partprefabtable.pappt`).
2. DMM detects the file-format extension, finds the live file in any
   vanilla group via `find_file_in_game`, extracts via `extract_from_paz`.
3. Bytes flow through `dmm_parser::dispatch::parse_table_to_json` →
   intent apply → `serialize_table_from_json`.
4. Modified bytes pack into a fresh overlay group:
   - `dmmv3_pappt` / `dmmv3_pamhc` (singletons)
   - `dmmv3_paac_<stem>` / `dmmv3_paatt_<stem>` (per-file)
5. Group registers in PAPGT, `.dmm_owned` marker dropped, mount log
   carries `[V3_FILE]` lines for the apply.
6. Unmount cleans up automatically (`is_dmm_owned_group` already
   recognises `dmmv3_*` prefixes).

Library code paths (all public):

- `dmm_parser::dispatch::parse_table_to_json("pappt", bytes, None)` →
  `Vec<Value>`
- `dmm_parser::dispatch::serialize_table_from_json("pappt", &items)` →
  `Vec<u8>`
- `dmm_parser::dispatch::is_file_format_table("pappt")` → `bool`
- DMM-internal: `apply_v3_to_file_format_body`,
  `install_v3_file_format_overlay`

---

## JSON shape per format

Each file-format table parses to a 1-element `Vec<Value>` where the
single value carries the entire file shape plus synthetic
`key: 0` / `string_key: ""` so the v3 intent dispatcher's
`find_record_index` can resolve it.

### pappt

```json
{
  "key": 0,
  "string_key": "",
  "header": [222, 173, 190, 239, 0, 1, 2, 3],
  "primary": [
    {
      "key_a": "Kliff",
      "key_b": "hair",
      "key_c": "src/kliff_hair.pmod",
      "asset_id": "kliff_hair_default",
      "flag": 1,
      "children": [
        { "sub_key": "kliff_hair_long", "sub_flag": 2 }
      ]
    }
  ],
  "secondary": [
    { "alias_a": "old_kliff_hair", "alias_b": "kliff_hair_default" }
  ]
}
```

Field paths v3 intents address:
- `primary[N].key_a` / `key_b` / `key_c` / `asset_id` / `flag`
- `primary[N].children[M].sub_key` / `sub_flag`
- `secondary[N].alias_a` / `alias_b`
- `header[N]` (one byte of the 8-byte opaque header)

### pamhc

```json
{
  "key": 0,
  "string_key": "",
  "header": [202, 254, 186, 190, 1, 2, 3, 4],
  "section_a": [16909060, 84281096, 151587081],
  "section_b": [176, 177, 178],
  "section_c": [],
  "section_d": [],
  "section_e": [224, 225, 226]
}
```

Field paths:
- `header[N]`
- `section_a[N]` (u32 entries)
- `section_b[N]` / `section_c[N]` / `section_d[N]` / `section_e[N]`
  (opaque bytes — element schemas not decoded)

### paatt

```json
{
  "key": 0,
  "string_key": "",
  "entry_count": 209,
  "hash_marker": 1160449792,
  "body": [...]
}
```

Field paths:
- `entry_count` / `hash_marker`
- `body[N]` (raw byte at offset N — physics fields like
  `projectileRadius` are anchor-detected; Workbench computes the
  byte offset and emits a `body[OFFSET]` intent)

### paac

```json
{
  "key": 0,
  "string_key": "",
  "format": "action_chart_v1",
  "size": 12345,
  "header_node_count": 703,
  "header_speed": 1.3333,
  "state_count": 35,
  "transition_count": 371,
  "condition_record_count": 50,
  "raw": [...]
}
```

`format` / `size` / `header_*` / `*_count` are read-only derived
views. The writable field is `raw[N]` — Workbench computes byte
offsets from its parsed view and emits `raw[OFFSET]` intents.

---

## Sample v3 intent file

`examples/v3_file_format_samples/sample_pappt.field.json`:

```json
{
  "format": 3,
  "modinfo": {
    "title": "Sample pappt mod (test fixture)",
    "version": "1.0.0",
    "author": "DMM Phase 3 test",
    "description": "Edits primary[0].flag in partprefabtable.pappt",
    "note": ""
  },
  "targets": [
    {
      "file": "partprefabtable.pappt",
      "intents": [
        {
          "entry": "",
          "key": 0,
          "field": "primary[0].flag",
          "op": "set",
          "new": 1
        }
      ]
    }
  ]
}
```

`entry` and `key` are the canonical synthetic-record identifiers that
the JSON layer emits — file-format records don't have real string keys
or numeric keys, so v3 intents target the implicit single record by
key=0.

The `file` field accepts either:
- File name only: `"partprefabtable.pappt"` — DMM walks every vanilla
  group PAMT to locate it.
- Full vfs path: `"character/bin__/partprefabtable.pappt"` —
  unambiguous, faster lookup.

For paac/paatt where multiple files share the table (`fist_upper.paac`
vs `pistol_upper.paac` vs `sword_upper.paac`), the file name is what
disambiguates and what determines the overlay group stem
(`dmmv3_paac_fist_upper`).

---

## The ask: Workbench v3-intent export

Mod-workbench currently exports file-format mods via dedicated PAZ-overlay deploy paths (`paac_editor::deploy_paac_overlay`, etc.). Each
deploy writes directly to a numbered PAZ overlay group on disk
(`0066/`, `0067/`, ...) — bypassing any user-visible mod folder.

**Request:** add a parallel "Export as Field-JSON v3" button to each
of the four editors that emits the JSON shape above. Concretely, in
`mod-workbench/src/mod_io.rs::export_dmm_v3` (or a sibling function):

1. After the user finishes editing, diff the modified PaacFile /
   PaattFile / PamhcFile / PapptFile against the original.
2. For each scalar leaf that differs, emit one `{ entry: "", key: 0,
   field: <path>, op: "set", new: <value> }` intent.
3. Write the standard v3 envelope (`format: 3`, `modinfo`, `targets`)
   with one target whose `file` is the file's basename.

Field-path generation can use the same `flatten_leaves` helper that
already powers pabgb v3 export — the JSON shapes round-trip the same
way, so dot/bracket notation works unchanged.

For paac specifically, the simplest first-cut export is "diff `raw`
byte-by-byte and emit `field: "raw[N]"` for every changed offset" —
that mirrors how `patch_float` / `patch_transition` already mutate
the parser. Per-state-machine semantic exports can come later once
the JSON shape design is settled on both sides.

---

## Verification

DMM-side verification per phase (all green as of pre-release.7):

| Format | dmm-parser tests | DMM build |
|--------|------------------|-----------|
| pappt  | 7/7 pass         | clean     |
| pamhc  | 8/8 pass         | clean     |
| paatt  | 6/6 pass         | clean     |
| paac   | 9/9 pass         | clean     |

Mount-time end-to-end verification still requires:
- A live game install with one of these files in vanilla state
- A v3 mod targeting that file dropped into DMM's mods directory
- `mount_log.txt` after mount contains `[V3_FILE] <target> → group dmmv3_<table>` line

Sample mod at `examples/v3_file_format_samples/sample_pappt.field.json`
is the easiest one to test — the edit is benign (flag byte change with
no observable in-game effect) and exercises the full dispatch path.

---

## Contact

DMM bugs, JSON shape questions, or coordination on the export format:
exodiaprivate@gmail.com / `exodiaprivate-eng/DMM-BETA` issues.

dmm-parser bugs or schema questions:
`exodiaprivate-eng/dmm-parser` issues.

Both repos are under CDMTL v1.0; mod-workbench is part of the
Authorized Software Suite under §1(g) so straight ports of these
shapes back into Workbench are explicitly allowed.
