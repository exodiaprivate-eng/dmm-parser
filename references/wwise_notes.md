# Wwise Audio Format Notes — Crimson Desert (Phase A0)

**Date:** 2026-05-01
**Source:** Real-sample inspection (DMM backup folder + SWISS mod folder); reference reading of DMM's `commands.rs` (NOT modifying DMM).
**Status:** A0 Recon complete — ready for A1/A2 (hexpat) and A3 (Rust skeleton).

This document captures the Audiokinetic Wwise WEM (audio) and BNK (SoundBank) formats as Crimson Desert ships them, plus the Crimson-specific quirks dmm-parser needs to honor for SWISS Stacker validation of audio mods.

---

## 1. WEM File Format (Audio Asset)

### Container

WEM = "Wwise Encoded Media". Files use the standard RIFF-WAVE wrapper:

```
+0x00  "RIFF" (4)
+0x04  riff_size: u32 LE          file_size - 8
+0x08  "WAVE" (4)
+0x0C  chunks[]                   sequence of 8-byte header (id + size) + payload
```

### Chunks (typical Wwise WEM)

Verified against two real samples (`1045272379.wem` 433 KB, `113958244.wem` 62 MB):

| Chunk ID | Required | Size | Purpose |
|---|---|---|---|
| `fmt ` | yes | 24-66 bytes | WAVEFORMATEX or extended Wwise format |
| `hash` | yes (Wwise) | 16 bytes | Wwise-specific content hash |
| `junk` | optional | 12 bytes | Padding / version-specific filler |
| `data` | yes | varies | Encoded audio payload |

The `hash` chunk is **Wwise-specific** — not part of the standard RIFF spec. Its presence is a strong fingerprint that this is a Wwise WEM rather than a raw WAVE.

### `fmt` Chunk Payload (first 16 bytes)

Standard WAVEFORMATEX layout:

```
+0x00  format_tag: u16          0xFFFE = WaveformatExtensible (PCM-style)
                                 0xFFFF = Wwise Vorbis (most common for game audio)
                                 (other Wwise codec IDs exist)
+0x02  channels: u16             1 = mono, 2 = stereo
+0x04  sample_rate: u32          44100, 48000 typical
+0x08  byte_rate: u32            (sometimes 0 for compressed formats)
+0x0C  block_align: u16          0 for compressed
+0x0E  bits_per_sample: u16      0 for compressed; 16 for PCM
+0x10  cb_size: u16              extension byte count (varies)
+0x12  ... Wwise-specific extension fields ...
```

When `format_tag` is `0xFFFE`, the next 22+ bytes are the WAVEFORMATEXTENSIBLE / Wwise-extended fields. We don't need to decode the audio to validate it — just verify the wrapper is sane and `data` chunk is reachable.

### Validation hooks (for A7)

- File must start with `RIFF`
- 12-byte minimum header (RIFF + size + WAVE)
- `WAVE` ID at offset 8
- `fmt ` chunk must come first (Wwise convention)
- At least one `data` chunk required
- `format_tag` recognized (`0xFFFE` or `0xFFFF` for Crimson)
- `sample_rate` in sane range (8000-96000)
- `channels` in sane range (1-8)

---

## 2. BNK File Format (SoundBank)

### Container

BNK = "Wwise SoundBank". **Not** RIFF-wrapped — sections appear back-to-back at offset 0.

Verified against `2498340951.bnk` (135 MB):

```
+0x00  section[0]: BKHD          required, always first
+...   section[1]: DIDX          optional, lists embedded WEMs
+...   section[2]: DATA          optional, raw WEM payload referenced by DIDX
+...   section[3]: HIRC          hierarchy (events, sound objects, actions)
+...   section[4]: STID          referenced bank IDs
```

Each section has the same envelope:

```
+0x00  section_id: u8[4]         "BKHD" / "DIDX" / "DATA" / "HIRC" / "STID"
+0x04  section_size: u32         payload size in bytes (NOT including this header)
+0x08  payload: u8[section_size]
```

### BKHD (Bank Header) — section_size 52 bytes (verified)

```
+0x00  bank_version: u32          150 = Crimson-era Wwise build
+0x04  bank_id: u32               unique bank ID (matches the file's numeric name)
+0x08  ... language ID, project ID, padding (40 bytes) ...
```

### DIDX (Data Index) — embedded WEM lookup table

Payload is `N × 12-byte entries`:

```
struct DidxEntry {
    u32 wem_id;       // Wwise-hashed WEM identifier
    u32 wem_offset;   // offset INTO the DATA section
    u32 wem_size;     // bytes
}
```

DIDX entry count = `section_size / 12`. The verified sample had 3 embedded WEMs.

### DATA section — raw WEM payload

Referenced by DIDX entries. Each WEM's bytes start at `bnk_data_offset + entry.wem_offset` where `bnk_data_offset` is the absolute file offset of the DATA section's payload.

### HIRC (Hierarchy) — event/sound graph

The most complex section. Defines:
- Events (e.g. "Play_VO_Macduff_Attack01")
- Sound objects with codec parameters
- Actions (play, stop, set RTPC)
- Mappings from WEM IDs → playback parameters

dmm-parser doesn't need to fully parse HIRC for v3.1 validation — knowing the section exists and its size is enough. Modders typically don't author BNK by hand; they generate it via the Wwise authoring tool.

### STID (Sound TID — referenced banks)

```
+0x00  type: u32                  hash type (1 = bank ID list)
+0x04  num_banks: u32
+0x08  banks[N]: { u32 bank_id; u32 name_len; char name[name_len]; } ...
```

### Validation hooks (for A7)

- File must start with `BKHD`
- BKHD section_size >= 8 (must hold version + bank_id)
- Bank version recognized (150 for Crimson)
- DIDX/DATA presence is consistent (DIDX without DATA = malformed)
- DIDX wem_offset + wem_size <= DATA payload size
- HIRC is optional but if present, payload must be parseable section-by-section
- Total file size matches sum of section envelopes

---

## 3. Crimson-Specific Mounting Behavior

From DMM's `commands.rs:11654-11665`:

> WEM audio needs direct PAZ injection — Wwise reads from base PAZ groups.
>
> BNK soundbanks also need direct PAZ injection. Wwise loads banks from their native PAZ group, not from the overlay. For voice replacement mods, the BNK maps events to WEM IDs — if the BNK is only in the overlay, Wwise may read the vanilla BNK from the original group and never see our event remapping, leaving action sounds (which are often mapped through these same BNKs) playing the vanilla voice.

Implications for v3.1:
- WEM/BNK assets in a v3.1 mod target the **base PAZ group**, not an overlay group. The asset target's vpath must reflect the actual game-internal location.
- The dispatch on file extension already separates WEM/BNK from DDS in the X0 spec. dmm-parser's classifier just needs to confirm the file is a valid WEM/BNK.

---

## 4. Crimson Wwise Path Conventions

Observed in real mods + game data:

| Path pattern | Purpose | Group |
|---|---|---|
| `0006/sound/windows/<lang>/<id>.bnk` | Localized voice banks | 0006 |
| `0006/sound/windows/<lang>/<id>.wem` | Localized voice clips | 0006 |
| `soundcommon/windows/<id>.bnk` | Common sound effects | soundcommon |
| `soundcommon/windows/<id>.wem` | Common sound clips | soundcommon |
| `0014/sound/banks/<id>.bnk` | (legacy DMM convention) | 0014 |
| `0014/sound/character/<name>/voice_<event>.wem` | (legacy DMM convention) | 0014 |

Languages observed: `english(us)`, `korean`, etc.

The numeric file IDs are Wwise-hashed names (FNV-1A 32-bit). Tools like `wwise-utilities` can resolve them to original event names.

For dmm-parser's `infer_audio_vpath` (Phase A6), the path-prefix table is similar to DDS but with a different structure:

| Vpath prefix | Asset class |
|---|---|
| `0006/sound/windows/*.bnk` | Localized voice bank |
| `0006/sound/windows/*.wem` | Localized voice clip |
| `soundcommon/windows/*.bnk` | Common sound bank |
| `soundcommon/windows/*.wem` | Common sound clip |
| (other) | Unknown — bundle but flag for review |

---

## 5. Sample Files for Testing

| Sample | Size | Format details |
|---|---|---|
| `1045272379.wem` | 433 KB | format_tag=0xFFFF (Wwise Vorbis), 2ch, 48kHz |
| `113958244.wem` | 62 MB | format_tag=0xFFFE (PCM extensible), 2ch, 44.1kHz, 16bit |
| `2498340951.bnk` | 135 MB | bank_version=150, bank_id=2498340951, 3 embedded WEMs |
| `kliff_female_voice_english/0006/sound/windows/english(us)/3684722581.bnk` | (modder asset) | voice replacement bank |
| `kliff_female_voice_english/0006/sound/windows/english(us)/694511365.bnk` | (modder asset) | voice replacement bank |

Sample paths are recorded for the integration tests in A4/A5.

---

## 6. dmm-parser Implementation Targets (A1-A9)

| Phase | Deliverable |
|---|---|
| A1 | `references/wem.hexpat` — RIFF wrapper + Wwise chunks |
| A2 | `references/bnk.hexpat` — sectioned format |
| A3 | `src/audio/{mod,wem,bnk}.rs` — module skeleton |
| A4 | `classify_wem(bytes) -> WemMetadata` — RIFF parse + format tag + sample rate |
| A5 | `parse_bnk(bytes) -> BnkBank` — section walker + DIDX entries + bank ID |
| A6 | `infer_audio_vpath(path)` — path-prefix table |
| A7 | `validate_audio(bytes)` — RIFF/BNK structure + sane format ranges |
| A8 | Python bindings: `classify_wem`, `parse_bnk`, `infer_audio_vpath` |
| A9 | Tests + `docs/api.md` Audio section + sample-file integration tests |

---

## 7. Open Questions

1. **Wwise Vorbis decoder**: dmm-parser's scope is metadata only — we don't decode Vorbis. Modders need an external tool (vgmstream / Wwise authoring tool) to produce valid WEMs.
2. **Bank version compatibility**: Crimson uses bank_version=150. If the game updates to a newer Wwise build, the section layout may shift slightly. Detect via BKHD version + warn if unrecognized.
3. **HIRC granularity**: do mod authors ever need to edit individual events inside an HIRC section? For v3.1, we say no — modders replace whole BNK files. If finer-grained editing becomes a requirement, that's a future phase.
4. **Localized voice replacement**: the SWISS-style mod path `0006/sound/windows/english(us)/<id>.bnk` works because the language code is part of the directory. Need to confirm the path-prefix table handles all the language codes the game ships.

---

*End of A0 notes. Ready for A1 (WEM hexpat) + A2 (BNK hexpat).*
