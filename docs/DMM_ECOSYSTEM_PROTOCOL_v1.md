# DMM Mod Ecosystem Protocol v1.0
## Specification for Cross-Tool Coordination in the Crimson Desert Modding Suite

**Copyright (C) 2026 RicePaddySoftware. All Rights Reserved.**
**Licensed under CDMTL v1.0 — see [LICENSE.txt](../LICENSE.txt).**
**Effective Date: 2026-05-01**

> **Trademark notice:** "DMM Mod Ecosystem Protocol", "DMM", "Definitive Mod Manager", "SWISS Suite", "CrimsonGameMods", and "Field JSON v3.1" are trademarks of RicePaddySoftware. Use of these names by competing tools, or use of this Protocol's reserved namespace identifiers, is prohibited without written authorization. See CDMTL v1.0 §4.5.

---

## 0. Purpose

This Protocol specifies the cross-tool coordination convention used by **DMM** (Definitive Mod Manager) and the **SWISS Suite** (CrimsonGameMods Stacker, Save Editor, Game Mods) to safely share mounted overlays in a Crimson Desert game directory.

The Protocol enables:

1. Multiple authorized tools to mount mods to the same game without overwriting each other.
2. Cross-tool visibility — each tool can see what others have mounted.
3. Cryptographic verification — each tool's claims are signed and cannot be forged.
4. Deterministic conflict resolution — last-mounted-wins with explicit deregistration of conflicting overlays.

This Protocol is intentionally narrow in scope: it covers ownership claims, slot reservation, and state coordination. It does NOT specify how individual tools transform mods or interact with the game runtime — each authorized tool retains its own implementation.

---

## 1. Definitions

**"Authorized Tool"** means a software program operated by RicePaddySoftware or its designated agents, whose public signing key is registered as a Trust Root (Section 6). The current set of Authorized Tools is:

  (a) **DMM** (Definitive Mod Manager) — Tauri/Rust desktop application
  (b) **CrimsonGameMods** (Stacker, Save Editor, Game Mods) — PySide6/Python application

**"Game Directory"** means the root folder of a Crimson Desert game installation (e.g., `D:\Games\CrimsonDesert\`).

**"Overlay Group"** means a numbered or named subdirectory of the Game Directory containing PAZ overlay data and a `0.pamt` metadata file. The game engine loads overlays in PAPGT iteration order; later entries override earlier ones.

**"Reserved Slot Name"** means an Overlay Group name that is exclusively claimed by a specific Authorized Tool per Section 4.

**"State File"** means `crimson_modding_state.json`, a coordination file living at the root of the Game Directory and read/written by all participating Authorized Tools.

**"Trust Root"** means a public ed25519 signing key embedded in an Authorized Tool's binary, used to verify the authenticity of State File entries claiming a particular owner.

---

## 2. State File: `crimson_modding_state.json`

### 2.1 Location

```
<game_dir>/crimson_modding_state.json
```

The State File MUST live at the root of the Game Directory, alongside the `meta/` and overlay-group folders.

### 2.2 Schema (Protocol Version 2)

```json
{
  "protocol_version": 2,
  "updated": "2026-05-01T12:34:56Z",
  "overlays": {
    "dmmv3": {
      "owner": "DMM",
      "owner_version": "1.3.4",
      "content": "iteminfo field-level intents",
      "updated": "2026-05-01T12:30:00Z",
      "files": ["iteminfo.pabgb"],
      "signature": "ed25519:base64-encoded-64-byte-signature-here",
      "key_id": "dmm-2026-05"
    },
    "0036": {
      "owner": "CrimsonGameMods",
      "owner_version": "3.1.2",
      "content": "Stacker merged dropsets",
      "updated": "2026-05-01T11:00:00Z",
      "files": ["dropsetinfo.pabgb"],
      "signature": "ed25519:...",
      "key_id": "swiss-2026-05"
    }
  },
  "audit_log": [
    {
      "ts": "2026-05-01T11:00:00Z",
      "actor": "CrimsonGameMods",
      "action": "mount",
      "group": "0036"
    }
  ]
}
```

### 2.3 Field Specifications

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `protocol_version` | integer | yes | MUST be `2` for this version of the Protocol |
| `updated` | ISO-8601 string | yes | Last write timestamp (top-level) |
| `overlays` | object | yes | Map of slot-name → entry |
| `overlays.<slot>.owner` | string | yes | One of: `"DMM"`, `"CrimsonGameMods"`, `"Unknown"` |
| `overlays.<slot>.owner_version` | string | yes | Semver of the writing tool |
| `overlays.<slot>.content` | string | yes | Human-readable description |
| `overlays.<slot>.updated` | ISO-8601 | yes | Per-entry last-write timestamp |
| `overlays.<slot>.files` | array of strings | yes | PABGB file names this overlay claims |
| `overlays.<slot>.signature` | string | yes (v2+) | Format: `"ed25519:" + base64(sig)` |
| `overlays.<slot>.key_id` | string | yes (v2+) | Identifier of the public key that verifies the signature |
| `audit_log` | array | optional | Append-only log of mount/unmount events |

### 2.4 Backward Compatibility

Authorized Tools implementing Protocol v2 MUST also be able to read Protocol v1 State Files (no `signature` / `key_id` / `protocol_version`). When reading v1 entries:

  - Entries with known `owner` values (`DMM`, `CrimsonGameMods`, `JMM`) SHOULD be treated as **soft-trust** — accepted at face value but logged as "unsigned legacy entry"
  - Entries with unknown `owner` SHOULD be treated as `"Unknown"` and ignored for ownership purposes

Once an Authorized Tool writes a v2 entry to a v1 State File, the file becomes v2 and all future entries MUST be signed.

---

## 3. Owner Enumeration

The `owner` field MUST be one of the following exact strings:

| Owner String | Authorized Tool | Trademark Status |
|--------------|-----------------|------------------|
| `"DMM"` | Definitive Mod Manager | RicePaddySoftware ™ |
| `"CrimsonGameMods"` | SWISS Suite (Stacker, Game Mods) | RicePaddySoftware ™ |
| `"JMM"` | (Reserved historical) | Recognized for backward compatibility |
| `"Unknown"` | Unrecognized owner | Used when a tool encounters an unverified entry |

**Use of any other string in the `owner` field is prohibited.** Specifically:

  - Competing tools MAY NOT use `"DMM"`, `"CrimsonGameMods"`, or `"SWISS"` as their owner identifier — doing so constitutes trademark infringement under the Lanham Act and CDMTL §4.5.
  - Competing tools MAY NOT introduce alternative owner strings (e.g., `"CDUMM"`, `"DUMM"`) into the State File — this corrupts the Protocol namespace, which is reserved exclusively for Authorized Tools per CDMTL §4.7.

Authorized Tools MAY add new owner strings to this enumeration through a Protocol amendment published by RicePaddySoftware.

---

## 4. Reserved Slot Names

The following Overlay Group names are **reserved** for the Authorized Tools indicated:

| Slot Name | Reserved For | Purpose |
|-----------|--------------|---------|
| `dmmv3` | DMM | Field JSON v3.1 mod application target |
| `dmmsa` | DMM | Save engine apply target |
| `dmmgen` | DMM | Generic mod overlay |
| `dmmequ` | DMM | Equipment slot info overlay |
| `0036` | DMM (legacy) / CrimsonGameMods Stacker | DropSets / merged stacker output |
| `0037` | DMM (legacy) | Legacy mod slot |
| `0058` | (External: ItemBuffs tool) | Recognized as foreign, preserved during conflict scan |

**Use of the `dmm*` prefix in slot names is reserved for DMM and Authorized Tools.** Competing tools that use slot names beginning with `dmm` are infringing the DMM trademark.

Slot names not in this table are FOREIGN — Authorized Tools MUST preserve foreign entries during PAPGT rebuilds unless the foreign entry directly conflicts with an Authorized Tool's claim (Section 7).

---

## 5. Marker File: `.dmm_owned`

When an Authorized Tool creates or owns an Overlay Group folder, it MUST place a marker file at:

```
<game_dir>/<slot_name>/.dmm_owned
```

The marker file's contents MUST be a JSON object:

```json
{
  "owner": "DMM",
  "owner_version": "1.3.4",
  "key_id": "dmm-2026-05",
  "signature": "ed25519:...",
  "created": "2026-05-01T12:30:00Z"
}
```

The signature in the marker file covers the canonical bytes of `owner + owner_version + key_id + created`. This allows ownership verification even when the State File is missing or corrupt.

Removal of the `.dmm_owned` marker by any tool other than the marker's signing owner is a Protocol violation. Removal by a non-Authorized tool is additionally a CDMTL §4.6 (CMI removal) violation.

---

## 6. Trust Roots and Signing

### 6.1 Key Generation

Each Authorized Tool has an ed25519 keypair generated by RicePaddySoftware:

```
DMM:                Private key held by RPS, public key embedded in DMM binary
CrimsonGameMods:    Private key held by RPS, public key embedded in SWISS binary
```

Public keys are also embedded in DMM and SWISS in BOTH directions — each tool can verify the other's signatures.

### 6.2 Trust Root Format

A Trust Root is a public ed25519 verification key plus a key identifier:

```
{
  "key_id": "dmm-2026-05",
  "public_key": "base64-encoded-32-byte-ed25519-public-key",
  "owner": "DMM",
  "valid_from": "2026-05-01T00:00:00Z",
  "valid_until": null
}
```

`valid_until: null` means the key is currently valid. To rotate a key, set `valid_until` to a date and add a new Trust Root with a new `key_id`.

### 6.3 Embedded Trust Roots

Both DMM and SWISS MUST embed the following Trust Roots in their binary at compile time:

```rust
// DMM (Rust) — src-tauri/src/protocol/trust_roots.rs (NEW FILE)
pub const TRUST_ROOTS: &[TrustRoot] = &[
    TrustRoot {
        key_id: "dmm-2026-05",
        owner: "DMM",
        public_key: include_bytes!("../../../keys/dmm_2026_05.pub"),
        valid_from: "2026-05-01T00:00:00Z",
        valid_until: None,
    },
    TrustRoot {
        key_id: "swiss-2026-05",
        owner: "CrimsonGameMods",
        public_key: include_bytes!("../../../keys/swiss_2026_05.pub"),
        valid_from: "2026-05-01T00:00:00Z",
        valid_until: None,
    },
];
```

```python
# SWISS (Python) — CrimsonGameMods/protocol/trust_roots.py (NEW FILE)
TRUST_ROOTS = [
    {
        "key_id": "dmm-2026-05",
        "owner": "DMM",
        "public_key": b"<32 raw bytes>",  # baked at build time
        "valid_from": "2026-05-01T00:00:00Z",
        "valid_until": None,
    },
    {
        "key_id": "swiss-2026-05",
        "owner": "CrimsonGameMods",
        "public_key": b"<32 raw bytes>",
        "valid_from": "2026-05-01T00:00:00Z",
        "valid_until": None,
    },
]
```

### 6.4 Private Keys

Private keys MUST NOT be embedded in any distributed binary. They are held by RicePaddySoftware in a developer-only secrets store. Build-time injection happens via:

  - DMM: `keys/dmm_2026_05.priv` (gitignored, only on RPS dev machines, signs at build via build script)
  - SWISS: same pattern

**Never commit a private key. Never publish a private key. If a private key is leaked, immediately rotate it (Section 6.5).**

### 6.5 Key Rotation

To rotate a compromised key:

  1. Generate a new keypair.
  2. Add the new Trust Root with `valid_from: <today>`.
  3. Mark the old Trust Root with `valid_until: <today>`.
  4. Release new DMM and SWISS versions with both Trust Roots embedded (old + new).
  5. After 90 days, release versions with only the new Trust Root.

This gives users 90 days to update before old-key-signed entries become invalid.

---

## 7. Signing and Verification

### 7.1 Canonical Form

The bytes signed are the **canonical JSON serialization** of the entry, with:

  - Keys sorted lexicographically
  - No insignificant whitespace
  - UTF-8 encoding
  - The `signature` and `key_id` fields excluded from the canonical form (they ARE the signature itself)

Example (canonical bytes):

```
{"content":"iteminfo field-level intents","files":["iteminfo.pabgb"],"owner":"DMM","owner_version":"1.3.4","updated":"2026-05-01T12:30:00Z"}
```

The signature is computed over those exact bytes.

### 7.2 Sign

```
sig_bytes = ed25519_sign(private_key, canonical_entry_bytes)
entry.signature = "ed25519:" + base64(sig_bytes)
entry.key_id = "<your key_id>"
```

### 7.3 Verify

```
1. Look up entry.key_id in the embedded TRUST_ROOTS table.
2. If not found OR key is outside its valid_from/valid_until window:
   → REJECT (treat owner as "Unknown")
3. If trust_root.owner != entry.owner:
   → REJECT (a key signed for a different owner)
4. Re-compute canonical_entry_bytes from entry (excluding signature/key_id).
5. Decode entry.signature from base64.
6. ed25519_verify(trust_root.public_key, canonical_entry_bytes, sig_bytes).
7. If verify fails → REJECT.
8. Otherwise → ACCEPT (entry is authentic).
```

Unsigned entries (Protocol v1 legacy) are accepted only with `owner_version` not set or below documented v2 cutoff. Once a tool writes a v2 entry to a State File, all subsequent writes by all tools MUST be v2 signed.

---

## 8. Conflict Resolution

When an Authorized Tool prepares to mount an Overlay Group claiming PABGB file X:

### 8.1 Scan Phase

  1. Read the State File. Verify all entries (Section 7.3).
  2. For each authentic entry whose `files` includes X:
     - If owned by **the current tool**: this is an update, proceed.
     - If owned by **another Authorized Tool**: this is a peer conflict — resolve per Section 8.2.
     - If owner is **Unknown** (verification failed) or foreign: this is an outside conflict — resolve per Section 8.3.

### 8.2 Peer Conflict Resolution

When two Authorized Tools both claim file X, the tool mounting LATER wins via PAPGT iteration order. The earlier tool's entry stays in the State File (so it can re-mount when the user re-enables) but its PAPGT registration is updated to point earlier in iteration order.

### 8.3 Outside Conflict Resolution

When a foreign or unverified overlay claims file X, the Authorized Tool MUST:

  1. Deregister the foreign entry from PAPGT (game-level effect: it stops loading).
  2. Leave the foreign group's folder on disk (so the foreign tool can re-add itself).
  3. Log the deregistration in `audit_log` with `actor` = current tool, `action` = `"deregister_foreign"`, `group` = the foreign slot name.

### 8.4 Foreign-tool Restoration

If a foreign tool re-mounts after deregistration, the conflict cycle repeats. This is intentional — Authorized Tools have priority over unverified or foreign overlays in their target files.

---

## 9. Audit Log

The `audit_log` array in the State File is **append-only**. Each entry:

```json
{
  "ts": "2026-05-01T12:34:56Z",
  "actor": "DMM",                  // owner enum
  "action": "mount",                // mount|unmount|deregister_foreign|update
  "group": "dmmv3",
  "signature": "ed25519:...",       // signed by actor's key
  "key_id": "dmm-2026-05"
}
```

Audit log entries MUST be signed using the same canonical-form mechanism as overlay entries (Section 7.1).

The audit log can grow indefinitely; tools MAY truncate entries older than 90 days, preserving the most recent 1000 entries minimum.

---

## 10. Compliance and Enforcement

### 10.1 Conformance Levels

A tool that participates in this Protocol MUST be **Compliant** at one of two levels:

  - **Authorized Compliant**: holds an embedded Trust Root, signs its own entries, and is listed in Section 1.
  - **Read-Only Observer**: reads State File entries but does not write them. Does not require Trust Root credentials.

A tool that writes State File entries WITHOUT an embedded Trust Root recognized by RicePaddySoftware is **Non-Compliant** and is operating in violation of this Protocol.

### 10.2 Trademark and Copyright Protections

Per CDMTL §4.5, the strings `"DMM"`, `"CrimsonGameMods"`, `"DMM Mod Ecosystem Protocol"`, `"Field JSON v3.1"`, and the `dmm*` slot-name prefix are trademarks of RicePaddySoftware. Use by Non-Compliant tools constitutes trademark infringement.

This Protocol's specification text is copyrighted as a literary work under CDMTL v1.0. Reading this document constitutes acceptance of CDMTL v1.0 §4.10 (Acceptance by Access).

### 10.3 NexusMods Enforcement

Uploads on NexusMods that:

  - Implement this Protocol while writing `"owner": "DMM"` or `"owner": "CrimsonGameMods"` without holding the corresponding private key, OR
  - Use any reserved slot name (Section 4) without authorization, OR
  - Use the `dmm*` prefix in slot names, marker files, or branding,

are subject to DMCA takedown notices on the basis of trademark infringement and CDMTL §4.5 violations.

### 10.4 No Reverse Engineering for Competing Implementation

Per CDMTL §4.9, recipients of this specification MAY NOT use the Protocol design to develop a competing mod manager for Crimson Desert for a period of three (3) years from first access.

Permitted uses (Section 5 of CDMTL):

  - **Read-Only Observer** tools that only display State File contents
  - **Tools targeting different games** that adopt similar but distinct protocols
  - **Academic study** of the Protocol design

---

## 11. Versioning

This Protocol is versioned as follows:

  - **v1**: Initial unsigned version (current state of `crimson_modding_state.json` in deployed DMM/SWISS as of 2026-04)
  - **v2**: Adds ed25519 signing, Trust Roots, key rotation (this document)

Future versions will be published by RicePaddySoftware. Authorized Tools SHOULD always implement the latest published version while remaining backward-compatible with v1 (unsigned legacy entries).

---

## 12. Appendix: State File State Machine

```
┌─────────────────┐
│  No State File  │
└────────┬────────┘
         │ first tool to mount
         ▼
┌─────────────────┐
│   v2 State File │ ←──────────────┐
│   (signed)      │                │
└────────┬────────┘                │
         │ tool A writes entry     │ tool B writes entry
         │ (signed by A's key)     │ (signed by B's key)
         ▼                         │
┌─────────────────┐                │
│   v2 with two   │ ───────────────┘
│   peer entries  │
└────────┬────────┘
         │ unmount
         ▼
┌─────────────────┐
│   Entry removed │
│   from overlays │
│   audit_log kept│
└─────────────────┘
```

---

## 13. References

- **CDMTL v1.0** — `LICENSE.txt` at the root of every RicePaddySoftware repository
- **Field JSON v3.1 Specification** — `FIELD_JSON_V3_1_SPEC.md` (CrimsonGameMods)
- **DMM v3 Overlay Implementation** — `dmm-api-test/src-tauri/src/iteminfo/v3_overlay.rs`
- **SWISS Overlay Coordinator** — `CrimsonGameMods/overlay_coordinator.py`
- **SWISS Shared State** — `CrimsonGameMods/shared_state.py`
- **ed25519 specification** — RFC 8032

---

*End of DMM Mod Ecosystem Protocol v1.0 specification.*
