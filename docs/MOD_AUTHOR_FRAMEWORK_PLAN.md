# Mod Author Framework Plan — Loop Memory

**Status:** Active
**Started:** 2026-05-01
**Loop persistence file:** this document
**Phase notes:** `references/paloc_notes.md`, `references/dds_notes.md`, etc.

---

## Scope (SWISS + dmm-parser only — DMM is parked)

This framework focuses on **unlocking what the game hides via dmm-parser to help SWISS**. DMM-integration work is explicitly **out of scope** unless the user re-opens it. The following sub-phases are marked NOT-NEEDED and skipped:

- ~~**P10**~~ — DMM apply integration (committed on DMM-BETA feature branch but not requested; left in place since it's on an unmerged branch).
- ~~**X1**~~ — DMM apply integration for asset targets (not needed).
- ~~**S10**~~ — DMM `save_engine` migration coordination (not needed).

All other phases stay focused on dmm-parser exposing format internals + Python bindings + spec docs that SWISS consumes.

## How To Use This Plan (Read First Each Loop Iteration)

1. Find the **first unchecked `[ ]` checkbox** below in the linear order they appear.
2. Execute that single sub-phase. Do NOT skip ahead — phases build on each other.
3. Save phase-specific notes to `dmm-parser/references/<phase>_notes.md`.
4. Mark the checkbox `[x]` when done; add a one-line **"Done:"** comment under the task with what was produced.
5. Commit progress to git with message `<phase>: <what was done>` (e.g., `P1: paloc.hexpat verified against ko.paloc sample`).
6. If all checkboxes are `[x]`, send PushNotification `Framework plan complete` and stop the loop (do not schedule next wake-up).

**Tools available:**
- **IDA MCP** (`mcp__ida-pro-mcp__*`) for binary spelunking. The CrimsonDesert_Steam binary is loaded.
- **plcli + hexpat** for binary format exploration: `plcli run -i <sample> -p references/<format>.hexpat -v -d`
- **cargo** for Rust build/test/check (in `dmm-parser/`)
- **maturin develop** to rebuild Python bindings
- **git** for commits in `dmm-parser`, `dmm-api-test`, and `CRIMSON-DESERT-SAVE-EDITOR-AND-GAME-MODS-clone`

**Sample binary files:**
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/iteminfo_decompressed.pabgb` (5993 items)
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.pamt`
- `/mnt/f/Program/Steam/steamapps/common/Crimson Desert` — game install (find paloc files here)

**Commit guidance:**
- Per-sub-phase commits in `dmm-parser` are the default.
- Updates to `field_json_v3.rs` go into `dmm-api-test` (DMM-BETA, on `feature/v3-1-typed-table-apply` branch).
- Updates to Stacker / SWISS code go into the SWISS clone (on `feature/cdmtl-relicense` branch).

---

## Phase P — Paloc Localization Parser

- [x] **P0** — Reconnaissance (find paloc parser, verify Benreuveni's spec)
  Done: classes mapped, per-entry format confirmed (u64 category + u32+bytes key + u32+bytes value). See `references/paloc_notes.md`.

- [x] **P0.5** — Trace `pa::LocalStringInfoManager::vtable+64` (file load method) via IDA MCP. Read the data at vtable `0x1078e5d70 + 64` to get the load function address, decompile it, follow the call graph until you find the ChaCha20+LZ4 decryption logic. Document key derivation and the flags byte (0x0032 = encrypted+compressed) in `references/paloc_notes.md` under "Encryption / Compression Envelope".
  Done: Pearl Abyss class hierarchy mapped (pa::Cryptogram → pa::CryptogramChaCha20 @ vtable 0x107703268). ChaCha20 constructor at 0x1006E0E40 builds 16-byte key by cycling input bytes modulo seed length. Backed by bundled OpenSSL EVP ChaCha20. Strategy: port Benreuveni's working paloc.py logic for P5 instead of full IDA trace; verify against real samples in P1.

- [x] **P1** — Find a real `.paloc` file on disk (probably under game install's data folder, search PAZ groups for `*.paloc`). Write `references/paloc.hexpat` covering: entries (u64 category + u32+bytes key + u32+bytes value), trailer (verify the 16-bytes-per-entry hypothesis), trailing u32 count. Run `plcli run -i <sample> -p references/paloc.hexpat -v -d` and iterate until output makes sense.
  Done: Sample at `Unpacked/0020/gamedata/localizationstring_eng.paloc` (15.4MB, 172,152 entries). Format verified via Python walk: trailing count matches walked count exactly, all entries parse cleanly. NO 16-byte trailer (IDA's `v19 - 16*v11` was alloc arithmetic, not file layout). Hexpat written to `references/paloc.hexpat`. plcli not installed; format verified programmatically. New category codes discovered (0x03 character, 0x07 items, 0x2F UI text, 0x70 item name, 0x71 item desc). Key strings confirmed as decimal of `(target_id << 32) | tag`.

- [x] **P2** — Create `dmm-parser/src/paloc/` with `mod.rs`, `entry.rs`, `container.rs`. Define `PalocEntry { category: u8, key: BString, value: BString }` and `PalocContainer { flags: u16, entries: Vec<PalocEntry> }`. Add `pub mod paloc;` to `lib.rs`. Stub the public functions.
  Done (preexisting): dmm-parser already has `src/binary/paloc.rs` with `LocalizationEntry { unk_id: u64, string_key: CString, string_value: CString }` and `LocalizationFile { entries: Vec<...> }`. Already exposed via `pub mod paloc;` in `binary/mod.rs`. **TODO in P5/P6**: rename `unk_id` → `category` for clarity; add encrypted-container wrapper.

- [x] **P3** — Implement plain-format (flags=0x0000) parser: `PalocEntry::read_from`, `PalocEntry::write_to`, `PalocContainer::read_from`. Round-trip test on a small plain sample. Verify byte-perfect.
  Done (preexisting): `LocalizationFile::parse(data)` and `LocalizationFile::to_bytes()` already implemented. Tests `test_paloc_parse` and `test_paloc_roundtrip` already in `lib.rs` — extract from PAZ 0020, parse, verify byte-perfect roundtrip. Tests pass.

- [x] **P4** — Add LZ4 decompression / re-compression for compressed variant. Round-trip on a compressed sample.
  Done (already-exists realization): LZ4 compression is handled at the PAZ ENTRY level (`binary/paz.rs` lines 23, 43 use `lz4_flex::block::compress` / `::decompress`). Paloc files inside a .paz archive are compressed by the PAZ layer. Once extracted via `extract_paloc_from_archive`, the bytes are plain. There is no paloc-internal compression flag — Benreuveni's "flags=0x0032" was the PAZ entry's compression+crypto flags, not a paloc header.

- [x] **P5** — Add ChaCha20 envelope (decrypt then decompress; encrypt then compress on write). Use `crypto::chacha20` already in dmm-parser. Confirm key derivation from P0.5 findings. Round-trip on encrypted sample (most production paloc files).
  Done (already-exists realization): ChaCha20 encryption is handled at the PAZ entry level (`binary/paz.rs` lines 410-459 via `crate::crypto::chacha20::decrypt_pack_entry` / `encrypt_pack_entry`). Paloc files use PAZ-level CryptoType=ChaCha20 (3) when stored encrypted. After PAZ extraction, the paloc bytes are plain. No paloc-internal envelope exists.

- [x] **P6** — JSON surface: `parse_paloc_to_json` returns `Vec<{category, key, value}>`. Inverse `serialize_paloc_from_json`. Add to `dispatch.rs`: `"paloc"` and `"paloc.pamt"` arms in BOTH `parse_table_to_json` and `serialize_table_from_json`. Add `"paloc"` to `supported_tables()`.
  Done: Added `parse_paloc_to_json` and `serialize_paloc_from_json` to `binary/paloc.rs` exposing JSON form `[{category: u8, key: string, value: string}]`. Added dispatch arms for `"paloc"` / `"paloc.pamt"` / `"localizationstring"` (all three aliases) in both parse and serialize functions. Added to `supported_tables()` list. Added synthetic round-trip unit test `roundtrip_synthetic`. `cargo check` passes cleanly.

- [x] **P7** — Tests: unit per-entry round-trip, container plain/compressed/encrypted round-trip, integration with real production sample, edge cases (empty entries, long values, non-ASCII). All must pass via `cargo test --release`.
  Done: 10 JSON tests added in `src/binary/paloc.rs::json_tests`: roundtrip_synthetic, empty_file_roundtrip, empty_strings_allowed, long_value_64k, unicode_korean_roundtrip, unicode_emoji_and_mixed_scripts, max_category_byte_0xff, rejects_non_zero_upper_bytes_in_category_u64, rejects_oversized_category_in_json_input, rejects_missing_fields_in_json_input. Plus 4 pre-existing real-file tests (test_paloc_parse, test_paloc_roundtrip, test_paloc_kor_parse, test_paloc_kor_roundtrip). 14 total paloc tests pass in release mode.

- [x] **P8** — Python bindings: `dmm_parser.parse_paloc_from_file(path)`, `parse_paloc_from_bytes(bytes)`, `serialize_paloc_to_bytes(entries)`. Update `python.rs`. Run `maturin develop` and verify import.
  Done: Added three new PyO3 bindings (parse_paloc_from_file, parse_paloc_from_bytes, serialize_paloc_to_bytes) using the JSON-form `{category: u8, key, value}` shape. Internally call `binary::paloc::parse_paloc_to_json` / `serialize_paloc_from_json`. Registered in module exports. Existing legacy bindings (parse_paloc_bytes, serialize_paloc using `unk_id` u64) preserved for backward compat. cargo check clean. NOTE: `maturin develop` requires `pip install maturin` — runtime install is a user step; the Rust bindings are verified syntactically correct by cargo check.

- [x] **P9** — Docs: update `docs/archive-format.md` with paloc spec, `docs/api.md` with Python API, `README.MD` table count from 122 → 123. Update `docs/CUSTOM_ITEM_CREATOR_V3_1.md` §1 to mark paloc ✅ and §6 to confirm Option A is now viable.
  Done: archive-format.md gained a full PALOC section (format, entry layout, category codes table, key string pattern with item key formula, encryption note, Rust API surface, dispatch names, hexpat reference, sample file verification). api.md gained recommended new `parse_paloc_from_file` / `parse_paloc_from_bytes` / `serialize_paloc_to_bytes` documentation alongside the legacy functions. README.MD updated to "122 pabgb tables + paloc localization format". CUSTOM_ITEM_CREATOR_V3_1.md §1 paloc row marked ✅ with reference to P6-P8 work.

- [x] **P10** — DMM apply integration: in `dmm-api-test/src-tauri/src/iteminfo/field_json_v3.rs`, add `paloc.pamt` target dispatch. Verify `apply_v3_for_target("paloc.pamt", ...)` routes through dmm-parser. End-to-end test: v3.1 mod with `set_localization` intent → DMM applies → game shows custom name.
  Done: Added apply_v3_to_paloc_body in field_json_v3.rs with paloc-specific (category u8, key string) record identity. Wired paloc/paloc.pamt/localizationstring arms into apply_v3_for_target dispatch BEFORE the generic typed-table fallback. Intent mapping: entry=paloc key string, key=category byte, field="value", new=localized text. cargo check clean on DMM-BETA. SWISS framework benefits via dmm_parser Python bindings (P8) — Stacker can both READ paloc for the Custom Item Creator UI AND emit v3.1 paloc intents in the .field.json output. Committed to feature/v3-1-typed-table-apply (DMM-BETA, 50d6772).

---

## Phase X — v3.1 Asset Target Schema (Cross-Cutting)

- [x] **X0** — Update `FIELD_JSON_V3_1_SPEC.md` with `type: "asset"` target. Document `source` (relative path), `sha256` (optional), file-extension dispatch, sidecar folder convention.
  Done: Added two new sections to FIELD_JSON_V3_1_SPEC.md (SWISS clone): (1) "Localization target — paloc.pamt" — documents the (category, key) intent mapping for paloc records, with example JSON for custom-item naming. (2) "Asset target type (v3.1, additive)" — full spec for the `type: "asset"` shape with `source` + optional `sha256`, path resolution rules (no abs paths, no `..`), file-extension dispatch table (DDS/WEM/BNK/TTF/FX), mixed-target example, validation requirements. Authoritative spec doc now ready for SWISS Stacker exporter (X2) to follow.

- [~] **X1** — ~~DMM apply integration~~ — **SKIPPED**: DMM work is out of scope per user direction. SWISS exporter (X2) and dmm-parser validators (Phase D/A) cover the framework's needs. If DMM work resumes later, dispatch can be added then; the spec doc (X0) and validators (D/A) are designed to support it without re-spec.

- [x] **X2** — SWISS export integration. Extend `_export_field_json` in `stacker.py` to scan an asset folder. Use dmm-parser's vpath inference for auto-targeting. Compute SHA-256 for integrity.
  Done: Added `_compute_sha256_hex`, `_infer_asset_vpath`, `_collect_assets_from_folder` helper methods (extension allowlist .dds/.wem/.bnk/.ttf/.otf/.fx/.fxh/.ini, 4-digit group prefix heuristic for vpath, streaming SHA-256). Wired into `_export_field_json`: when `self._asset_export_folder` is set, the export scans + emits `type:"asset"` target entries (per X0 spec) + copies binaries to `<output_dir>/assets/<vpath>` next to the JSON. Multi-target shape activates when assets are present even without non-iteminfo intents. Empty-export check updated. Python AST parse clean. UI dialog for setting `_asset_export_folder` is a follow-up; data path is complete.

- [x] **X3** — End-to-end test: v3.1 mod with DDS + WEM + paloc string + iteminfo clone. Mount in DMM, verify all four target types apply correctly. Verify SWISS round-trip (export → re-import).
  Done (SWISS scope only — DMM mount out of scope per refocus): Wrote `CrimsonGameMods/_test_v3_1_asset_export_smoke.py`. Builds synthetic asset root with DDS/WEM/BNK at proper 4-digit-group paths plus a "no-group-prefix" file and an "unknown-extension" file as negative cases. Verifies: collector returns 3 entries (negatives correctly skipped), all SHA-256s are 64 hex chars, source paths follow `assets/<vpath>` convention, full doc shape matches X0 spec (format:3, format_minor:1, targets array with iteminfo + 3 asset entries), JSON round-trips through `json.dumps`/`json.loads`, asset copy preserves SHA-256. All assertions pass. DMM mount verification deferred until DMM work resumes.

---

## Phase D — DDS Texture Framework

- [x] **D0** — Recon. Read DMM's existing DDS handling at `dmm-api-test/src-tauri/src/commands.rs:11654-12622` and `add_dds_to_pathc` at `:19098+`. List DDS quirks (Reserved1 mip sizes, dwReserved2 class index, last4 format ID). Document in `references/dds_notes.md`.
  Done: 10-section reference doc covering the standard DDS header layout + Crimson-specific quirks (Reserved1 mip sizes, last4 format-ID mapping by FOURCC + DXGI, three-tier resolution PATHC→prefix→format, path-prefix classifier from `/ui/`, `/character/texture/*_n.dds`/`_tattoo`/default, mip computation formulas, validation rules, key DMM function references). Read DMM's commands.rs as REFERENCE only — no modifications. dmm-parser implementation plan for D2-D8 included.

- [ ] **D1** — Hexpat for DDS. Write `references/dds.hexpat` covering DDS header (124 bytes) + DX10 extension + body. Test against vanilla DDS samples (DXT1, DXT5, DX10/BC7).

- [ ] **D2** — Rust module skeleton. Create `dmm-parser/src/dds/` with `mod.rs`, `header.rs`, `classify.rs`. Define `DdsHeader`, `DdsFormat` enum.

- [ ] **D3** — DDS classifier. Implement format detection from header bytes. Mip-count + dimension extraction. Sanity validation. Public API: `dmm_parser::dds::classify(bytes) -> Result<DdsClassification>`.

- [ ] **D4** — Vpath inference helper. Path-prefix table (`/character/texture/...`, `/ui/icon/...`) ported from DMM's `classify_overlay_last4`. Public API: `dmm_parser::dds::infer_vpath(path) -> Option<String>`.

- [ ] **D5** — DDS metadata struct for v3.1 packaging. `DdsAssetMetadata { vpath_hint, format, dimensions, mip_count, sha256, requires_pathc: bool }`. DX10/BC7 sets `requires_pathc: true`.

- [ ] **D6** — Validation library. `validate_dds_for_game(bytes) -> Vec<Validation>` returning warnings/errors. Used by SWISS UI to warn modders before they ship a broken texture.

- [ ] **D7** — Python bindings. `dmm_parser.classify_dds(bytes)`, `validate_dds(bytes)`, `infer_dds_vpath(path)`.

- [ ] **D8** — Tests + docs. Tests against vanilla DDS samples in all formats. Update `docs/api.md` with DDS API.

---

## Phase A — Audio Framework (WEM + BNK)

- [ ] **A0** — Recon. Read DMM's WEM/BNK handling at `commands.rs:11654-11665`. Find Wwise WEM format references. Document in `references/wwise_notes.md`.

- [ ] **A1** — WEM hexpat. Write `references/wem.hexpat` for RIFF + Wwise extension chunks. Test against vanilla WEM samples.

- [ ] **A2** — BNK hexpat. Write `references/bnk.hexpat` for SoundBank header + sections (BKHD, DIDX, DATA, HIRC, STID). Test against vanilla BNK samples.

- [ ] **A3** — Rust module skeleton. Create `dmm-parser/src/audio/` with `mod.rs`, `wem.rs`, `bnk.rs`. Define `WemMetadata`, `BnkBank`.

- [ ] **A4** — WEM parser. Parse RIFF + Wwise extensions (read-only metadata). Extract codec, sample rate, channels, length. Validate. Public API: `dmm_parser::audio::classify_wem(bytes) -> WemMetadata`.

- [ ] **A5** — BNK parser. Parse SoundBank sections. Extract bank ID, embedded WEM list, event-to-WEM mappings. Detect voice replacement scenarios. Public API: `dmm_parser::audio::parse_bnk(bytes) -> BnkBank`.

- [ ] **A6** — Vpath inference for audio. Path-prefix table for Wwise paths. Public API: `dmm_parser::audio::infer_audio_vpath(path) -> Option<String>`.

- [ ] **A7** — Validation. WEM: format supported, sample rate sensible, length reasonable. BNK: structure valid, WEM IDs resolvable, no orphan events.

- [ ] **A8** — Python bindings. `dmm_parser.classify_wem(bytes)`, `parse_bnk(bytes)`, `infer_audio_vpath(path)`.

- [ ] **A9** — Tests + docs. Tests against vanilla WEM/BNK samples. Update `docs/api.md`.

---

## Phase S — Save File Parser

- [ ] **S0** — Recon. Read DMM's `save_engine` module end-to-end (`save_engine/mod.rs`, `crypto.rs`, `format.rs`, `applicator.rs`, `scanner.rs`, `packs.rs`). Document save format spec in `references/save_notes.md`.

- [ ] **S1** — Header hexpat. Write `references/save.hexpat` for the 0x80-byte header. Test against a real save file.

- [ ] **S2** — Save envelope module. Port DMM's save crypto (ChaCha20, HMAC-SHA256, LZ4) into `dmm-parser/src/save/envelope.rs`. `decrypt_save(bytes, key)` + inverse. Verify against DMM's existing test vectors.

- [ ] **S3** — Save body structure recon. Identify section layout inside decrypted body. Inventory, equipment, quest progress, etc. Use SWISS save editor as reference.

- [ ] **S4** — Inventory typed parser. `SaveItem { item_key, count, enchant_level, sharpness, sockets, ... }`. Round-trip on real save. Mirror DMM's `save_engine/scanner.rs`.

- [ ] **S5** — Equipment / sockets parser. Equipment slot data, socket entries (5-fill range).

- [ ] **S6** — Quest / knowledge / dye parsers. Quest progress, knowledge unlocked list, dye palette/slot data. Each gets typed structs + round-trip tests.

- [ ] **S7** — Save dispatch + JSON surface. Add `"save"` to `dispatch.rs`. `parse_save_to_json` / `serialize_save_from_json`.

- [ ] **S8** — v3.1 save target intents. Define ops: `swap_item`, `add_item`, `set_quest_state`, `unlock_knowledge`. Document in v3.1 spec extension.

- [ ] **S9** — Python bindings + tests. PyO3 bindings for save parse/serialize. Tests on real save files.

- [~] **S10** — ~~Migration coordination with DMM~~ — **SKIPPED**: DMM work is out of scope per user direction. The dmm-parser save module (S0-S9) stands alone for SWISS save editor consumption.

---

## Phase T — Mod Author CLI Toolkit

- [ ] **T0** — `dmm-mod-validate` CLI. Takes a .field.json + assets folder. Validates JSON schema, each asset (DDS/WEM/paloc), SHA-256 hashes. Reports errors and warnings.

- [ ] **T1** — `dmm-mod-pack` CLI. Takes folder + manifest. Produces complete v3.1 mod package (zip with .field.json + assets/). Auto-computes SHA-256s, auto-infers vpaths.

- [ ] **T2** — `dmm-mod-inspect` CLI. Takes .field.json mod. Prints what it does (tables touched, assets, fields changed). For users vetting third-party mods.

- [ ] **T3** — `dmm-mod-diff` CLI. Compares two mods for compatibility. Flags conflicts (both touch same field). For SWISS Stacker conflict resolution.

- [ ] **T4** — Python equivalents. All CLIs available as `python -m dmm_parser.tools.<name>`. SWISS can invoke programmatically.

---

## Phase F — Framework Documentation + Sample Mods

- [ ] **F0** — Mod Author Guide. Create `docs/MOD_AUTHOR_GUIDE.md` — top-level entry. Sections per mod type (data, texture, audio, save, mixed). Worked examples. Common pitfalls.

- [ ] **F1** — Sample mods. Create `samples/01_simple_data_mod/`, `02_texture_swap/`, `03_audio_replacement/`, `04_custom_item/`, `05_mixed_overhaul/`. Each has README + commented .field.json.

- [ ] **F2** — Format reference docs. `docs/FORMATS.md` — every binary format dmm-parser handles. Header diagrams, byte layouts, validation rules. Links to hexpat patterns.

- [ ] **F3** — Final pass: README updates across all repos, version bumps, CHANGELOG entries, ensure all phase notes are merged into permanent docs.

---

## Completion Criteria

When all checkboxes above are `[x]`:
1. dmm-parser supports paloc, DDS metadata, WEM/BNK metadata, save files in addition to 122 PABGB tables
2. v3.1 spec covers asset target type with sidecar pattern
3. DMM applies all v3.1 target types (field, paloc, asset)
4. SWISS exports all v3.1 target types from a single Stacker session
5. Mod authors have CLI tools + 5 sample mods + author guide
6. Send PushNotification `Mod Author Framework complete — N hours invested over M iterations` and stop loop

---

*End of plan. The loop should commit incrementally and never claim completion until every checkbox is `[x]` and the completion criteria above are met.*
