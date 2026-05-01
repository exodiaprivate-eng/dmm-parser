# DMM Mod Ecosystem Protocol v1.0 — Implementation Guide

**Companion to [DMM_ECOSYSTEM_PROTOCOL_v1.md](./DMM_ECOSYSTEM_PROTOCOL_v1.md).**

This guide provides drop-in reference implementations for both DMM (Rust) and SWISS (Python), plus a migration plan from the current unsigned v1 state to signed v2.

> **Licensed under CDMTL v1.0.** Reading this guide constitutes acceptance of §4.9/§4.10.

---

## Phase 0 — Key Generation (One-Time, Off-Repo)

These keys never go in git. Generate once, store in your password manager / KeePass / 1Password / hardware token, then bake the PUBLIC keys into both binaries.

### `scripts/generate_protocol_keys.py` (run once, then DELETE the private-key output from disk after copying to secrets store)

```python
"""Generate ed25519 keypairs for DMM Mod Ecosystem Protocol v1.0.

Run once. Copy the private keys to your secrets store (KeePass/1Password/HSM).
Public keys go into the source tree at keys/*.pub (committable — they're public).
"""
import base64
import json
from pathlib import Path
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives import serialization

KEY_IDS = ["dmm-2026-05", "swiss-2026-05"]
OWNERS = {"dmm-2026-05": "DMM", "swiss-2026-05": "CrimsonGameMods"}

OUTPUT_DIR = Path("keys")
OUTPUT_DIR.mkdir(exist_ok=True)

for key_id in KEY_IDS:
    priv = Ed25519PrivateKey.generate()
    pub = priv.public_key()

    priv_bytes = priv.private_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PrivateFormat.Raw,
        encryption_algorithm=serialization.NoEncryption(),
    )
    pub_bytes = pub.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )

    # Public key — safe to commit
    (OUTPUT_DIR / f"{key_id}.pub").write_bytes(pub_bytes)

    # Private key — DO NOT COMMIT, copy to secrets store, then delete
    (OUTPUT_DIR / f"{key_id}.priv").write_bytes(priv_bytes)

    print(f"Generated {key_id} (owner: {OWNERS[key_id]})")
    print(f"  pub:  {OUTPUT_DIR / f'{key_id}.pub'}  ({len(pub_bytes)} bytes)")
    print(f"  priv: {OUTPUT_DIR / f'{key_id}.priv'} ({len(priv_bytes)} bytes) ← MOVE TO SECRETS")
    print(f"  pub b64: {base64.b64encode(pub_bytes).decode()}")
```

**After running:**

1. Copy `keys/dmm-2026-05.priv` and `keys/swiss-2026-05.priv` to your secrets store (KeePass entry, encrypted backup, etc.)
2. **DELETE** the `.priv` files from disk
3. Add `keys/*.priv` to `.gitignore` in EVERY repo
4. Commit only the `.pub` files

Add to `.gitignore` in dmm-parser, dmm-api-test, and CRIMSON-DESERT-... repos:

```gitignore
# Protocol private keys — NEVER commit
keys/*.priv
keys/*.private
keys/*.pem
```

---

## Phase 1 — DMM (Rust) Implementation

### 1.1 Add Dependencies

`dmm-api-test/src-tauri/Cargo.toml`:

```toml
[dependencies]
# ... existing deps ...
ed25519-dalek = { version = "2.1", default-features = false, features = ["std"] }
base64 = "0.22"
serde_json = "1"  # likely already present
chrono = { version = "0.4", features = ["serde"] }  # likely already present
```

### 1.2 Create the Protocol Module

`dmm-api-test/src-tauri/src/protocol/mod.rs`:

```rust
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.

//! DMM Mod Ecosystem Protocol v1.0 implementation.
//! See docs/DMM_ECOSYSTEM_PROTOCOL_v1.md.

pub mod trust_roots;
pub mod canonical;
pub mod state_file;
pub mod sign;
pub mod verify;

pub use state_file::{StateFile, OverlayEntry, AuditEntry};
pub use sign::sign_entry;
pub use verify::{verify_entry, VerifyResult};
```

### 1.3 Trust Roots (Public Keys Embedded at Build Time)

`dmm-api-test/src-tauri/src/protocol/trust_roots.rs`:

```rust
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0

pub struct TrustRoot {
    pub key_id: &'static str,
    pub owner: &'static str,
    pub public_key: [u8; 32],
    pub valid_from: &'static str,    // ISO-8601
    pub valid_until: Option<&'static str>,
}

// Public keys baked at compile time. Place corresponding files at
// dmm-api-test/keys/*.pub in the repo (committable; they're public).
pub const TRUST_ROOTS: &[TrustRoot] = &[
    TrustRoot {
        key_id: "dmm-2026-05",
        owner: "DMM",
        public_key: *include_bytes!("../../../keys/dmm-2026-05.pub"),
        valid_from: "2026-05-01T00:00:00Z",
        valid_until: None,
    },
    TrustRoot {
        key_id: "swiss-2026-05",
        owner: "CrimsonGameMods",
        public_key: *include_bytes!("../../../keys/swiss-2026-05.pub"),
        valid_from: "2026-05-01T00:00:00Z",
        valid_until: None,
    },
];

pub fn lookup(key_id: &str) -> Option<&'static TrustRoot> {
    TRUST_ROOTS.iter().find(|r| r.key_id == key_id)
}
```

### 1.4 Canonical Serialization (Sign-Over-Bytes)

`dmm-api-test/src-tauri/src/protocol/canonical.rs`:

```rust
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0

use serde_json::{Map, Value};

/// Serialize to canonical JSON: keys sorted, no insignificant whitespace.
/// Excludes signature/key_id fields (they ARE the signature).
pub fn canonical_bytes(entry: &Value) -> Vec<u8> {
    let cleaned = strip_signature_fields(entry);
    let sorted = sort_recursively(cleaned);
    serde_json::to_vec(&sorted).expect("canonical JSON")
}

fn strip_signature_fields(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut clean = Map::new();
            for (k, val) in m {
                if k == "signature" || k == "key_id" {
                    continue;
                }
                clean.insert(k.clone(), strip_signature_fields(val));
            }
            Value::Object(clean)
        }
        _ => v.clone(),
    }
}

fn sort_recursively(v: Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<String> = m.keys().cloned().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                if let Some(val) = m.get(&k) {
                    out.insert(k, sort_recursively(val.clone()));
                }
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.into_iter().map(sort_recursively).collect()),
        _ => v,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_strips_signature_and_sorts() {
        let entry = json!({
            "signature": "ed25519:abc",
            "key_id": "dmm-2026-05",
            "owner": "DMM",
            "files": ["iteminfo.pabgb"],
            "content": "test",
            "updated": "2026-05-01T00:00:00Z",
            "owner_version": "1.3.4"
        });
        let bytes = canonical_bytes(&entry);
        let s = std::str::from_utf8(&bytes).unwrap();
        // Expected: signature/key_id excluded, alphabetical order
        assert_eq!(
            s,
            r#"{"content":"test","files":["iteminfo.pabgb"],"owner":"DMM","owner_version":"1.3.4","updated":"2026-05-01T00:00:00Z"}"#
        );
    }
}
```

### 1.5 Sign

`dmm-api-test/src-tauri/src/protocol/sign.rs`:

```rust
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0

use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::Value;

use super::canonical::canonical_bytes;

/// Sign an entry with the given private key. Returns (signature_string, key_id).
/// The caller writes these into entry.signature and entry.key_id before persisting.
pub fn sign_entry(
    private_key_bytes: &[u8; 32],
    key_id: &str,
    entry: &Value,
) -> (String, String) {
    let signing_key = SigningKey::from_bytes(private_key_bytes);
    let canonical = canonical_bytes(entry);
    let sig = signing_key.sign(&canonical);
    let sig_b64 = STANDARD.encode(sig.to_bytes());
    (format!("ed25519:{sig_b64}"), key_id.to_string())
}
```

### 1.6 Verify

`dmm-api-test/src-tauri/src/protocol/verify.rs`:

```rust
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0

use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::Value;

use super::canonical::canonical_bytes;
use super::trust_roots::{lookup, TrustRoot};

/// Mod formats that REQUIRE a valid signature. Per spec §2.5.
const SIGNING_REQUIRED_FORMATS: &[&str] = &["v3", "v3.1"];

/// Mod formats that may be present but never need a signature.
const LEGACY_FORMATS: &[&str] = &["v1", "v2", "legacy"];

#[derive(Debug, Clone, PartialEq)]
pub enum VerifyResult {
    /// Signature present and verified against a known Trust Root.
    Valid,
    /// Legacy/unsigned entry. Format is v1/v2/legacy/v3, or signature is
    /// missing and the format does not require signing.
    /// THIS IS AN ACCEPT STATE — entry is honored.
    LegacyUnsigned,
    /// Signing required (v3.1) but signature is missing. REJECT.
    SignatureRequired,
    /// Signing key not in TRUST_ROOTS. REJECT.
    UnknownKey,
    /// Key was found, but its owner doesn't match the entry's owner. REJECT.
    OwnerMismatch,
    /// Signature present but cryptographically invalid. REJECT for v3.1;
    /// for legacy formats, log and ACCEPT as LegacyUnsigned.
    InvalidSignature,
    /// Signature field could not be parsed. REJECT for v3.1;
    /// for legacy formats, log and ACCEPT as LegacyUnsigned.
    MalformedSignature,
    KeyExpired,
}

impl VerifyResult {
    /// True when DMM should honor the entry (mount/coordinate with it).
    pub fn is_accepted(&self) -> bool {
        matches!(self, VerifyResult::Valid | VerifyResult::LegacyUnsigned)
    }
}

pub fn verify_entry(entry: &Value) -> VerifyResult {
    let mod_format = entry.get("mod_format").and_then(|v| v.as_str()).unwrap_or("legacy");
    let signing_required = SIGNING_REQUIRED_FORMATS.contains(&mod_format);
    let is_legacy = LEGACY_FORMATS.contains(&mod_format);

    // No signature?
    let sig_str = match entry.get("signature").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            if signing_required {
                return VerifyResult::SignatureRequired;
            }
            return VerifyResult::LegacyUnsigned;
        }
    };

    // Has signature — try to verify, but downgrade to LegacyUnsigned for legacy formats
    // if anything goes wrong (preserves backward compat).
    let key_id = match entry.get("key_id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return if signing_required {
                VerifyResult::MalformedSignature
            } else {
                VerifyResult::LegacyUnsigned
            };
        }
    };
    let claimed_owner = match entry.get("owner").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return if signing_required {
                VerifyResult::MalformedSignature
            } else {
                VerifyResult::LegacyUnsigned
            };
        }
    };

    let sig_b64 = match sig_str.strip_prefix("ed25519:") {
        Some(s) => s,
        None => {
            return if signing_required {
                VerifyResult::MalformedSignature
            } else {
                VerifyResult::LegacyUnsigned
            };
        }
    };
    let sig_bytes = match STANDARD.decode(sig_b64) {
        Ok(b) => b,
        Err(_) => {
            return if signing_required {
                VerifyResult::MalformedSignature
            } else {
                VerifyResult::LegacyUnsigned
            };
        }
    };
    let sig_array: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => {
            return if signing_required {
                VerifyResult::MalformedSignature
            } else {
                VerifyResult::LegacyUnsigned
            };
        }
    };
    let signature = Signature::from_bytes(&sig_array);

    let root: &TrustRoot = match lookup(key_id) {
        Some(r) => r,
        None => {
            return if signing_required {
                VerifyResult::UnknownKey
            } else {
                // Legacy entry with unknown key — log but accept
                log::warn!("Legacy entry with unknown key_id: {key_id}");
                VerifyResult::LegacyUnsigned
            };
        }
    };
    if root.owner != claimed_owner {
        return if signing_required {
            VerifyResult::OwnerMismatch
        } else {
            log::warn!("Legacy entry has owner mismatch: claimed={claimed_owner}, key_owner={}", root.owner);
            VerifyResult::LegacyUnsigned
        };
    }

    let verifying_key = match VerifyingKey::from_bytes(&root.public_key) {
        Ok(k) => k,
        Err(_) => return VerifyResult::UnknownKey,
    };
    let canonical = canonical_bytes(entry);
    match verifying_key.verify(&canonical, &signature) {
        Ok(()) => VerifyResult::Valid,
        Err(_) => {
            if signing_required {
                VerifyResult::InvalidSignature
            } else {
                log::warn!("Legacy entry signature failed verification — accepting as legacy");
                VerifyResult::LegacyUnsigned
            }
        }
    }

    // Note: is_legacy is computed but unused; future Protocol amendments
    // can swap branches based on it (e.g., to deprecate v1 globally).
    let _ = is_legacy;
}
```

### 1.7 State File Read/Write

`dmm-api-test/src-tauri/src/protocol/state_file.rs`:

```rust
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::verify::{verify_entry, VerifyResult};

pub const STATE_FILENAME: &str = "crimson_modding_state.json";
pub const PROTOCOL_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFile {
    pub protocol_version: u32,
    pub updated: String,
    #[serde(default)]
    pub overlays: HashMap<String, OverlayEntry>,
    #[serde(default)]
    pub audit_log: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayEntry {
    pub owner: String,
    pub owner_version: String,
    /// Mod format: "v1" | "v2" | "legacy" | "v3" | "v3.1".
    /// Signing required only for "v3.1". Defaults to "legacy" for backward compat.
    #[serde(default = "default_mod_format")]
    pub mod_format: String,
    pub content: String,
    pub updated: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
}

fn default_mod_format() -> String {
    "legacy".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub ts: String,
    pub actor: String,
    pub action: String,
    pub group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
}

impl StateFile {
    pub fn load(game_dir: &Path) -> Result<Self, String> {
        let path = game_dir.join(STATE_FILENAME);
        if !path.exists() {
            return Ok(Self::empty());
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let state: StateFile = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        Ok(state)
    }

    pub fn save(&self, game_dir: &Path) -> Result<(), String> {
        let path = game_dir.join(STATE_FILENAME);
        let json = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        crate::fs_util::robust_atomic_write(&path, &json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn empty() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            updated: chrono::Utc::now().to_rfc3339(),
            overlays: HashMap::new(),
            audit_log: Vec::new(),
        }
    }

    /// Return entries that should be honored:
    ///   - Valid (signed v3/v3.1 entry, signature verified)
    ///   - LegacyUnsigned (v1/v2/legacy entry, no verification needed)
    ///
    /// Rejected: SignatureRequired (v3.1 missing sig), UnknownKey (v3.1 with unknown
    /// signer), OwnerMismatch (key signed for wrong owner), InvalidSignature
    /// (v3.1 with broken sig).
    pub fn verified_overlays(&self) -> HashMap<String, &OverlayEntry> {
        let mut out = HashMap::new();
        for (slot, entry) in &self.overlays {
            let value = serde_json::to_value(entry).unwrap();
            let result = verify_entry(&value);
            if result.is_accepted() {
                out.insert(slot.clone(), entry);
            } else {
                log::warn!("Rejected state entry {slot}: {:?}", result);
            }
        }
        out
    }
}
```

### 1.8 Wire Into Existing Mount Paths (Per-Format Signing)

**Critical scope rule**: Sign ONLY for v3 and v3.1 mounts. v1/v2/legacy mounts continue to use unsigned entries indefinitely so DMM can keep loading old mods.

#### 1.8.1 v3 / v3.1 Mount Path — `iteminfo/v3_overlay.rs`

After a successful v3 or v3.1 mount, write a SIGNED entry:

```rust
use crate::protocol::{StateFile, OverlayEntry, sign::sign_entry};

// After successful PAPGT registration in apply_v3_overlay():
let mut state = StateFile::load(game_dir).unwrap_or_else(|_| StateFile::empty());

let mut entry = OverlayEntry {
    owner: "DMM".to_string(),
    owner_version: env!("CARGO_PKG_VERSION").to_string(),
    mod_format: "v3.1".to_string(),     // ← signing-required format
    content: format!("v3.1 field-level intents on {target_table}"),
    updated: chrono::Utc::now().to_rfc3339(),
    files: vec![target_table.to_string()],
    signature: None,
    key_id: None,
};

// Sign it (v3 and v3.1 require it)
let entry_json = serde_json::to_value(&entry).unwrap();
let dmm_priv = include_bytes!("../../../../keys/dmm-2026-05.priv");
let (sig, kid) = sign_entry(dmm_priv, "dmm-2026-05", &entry_json);
entry.signature = Some(sig);
entry.key_id = Some(kid);

state.overlays.insert(group_name.clone(), entry);
state.protocol_version = 2;
state.updated = chrono::Utc::now().to_rfc3339();
state.save(game_dir)?;
```

#### 1.8.2 Legacy Mount Paths — v1/v2/byte-replace

For v1/v2/legacy mounts (e.g., `iteminfo/legacy_merge.rs`, `iteminfo/v3_byte_replace.rs`, `iteminfo/hybrid_merge.rs`, the file-replacement overlay path), write an UNSIGNED entry:

```rust
use crate::protocol::{StateFile, OverlayEntry};

// After successful legacy mount:
let mut state = StateFile::load(game_dir).unwrap_or_else(|_| StateFile::empty());

let entry = OverlayEntry {
    owner: "DMM".to_string(),
    owner_version: env!("CARGO_PKG_VERSION").to_string(),
    mod_format: "v2".to_string(),          // or "v1" / "legacy" depending on path
    content: format!("byte-replace mod on {target_file}"),
    updated: chrono::Utc::now().to_rfc3339(),
    files: vec![target_file.to_string()],
    signature: None,                        // ← legacy: unsigned
    key_id: None,
};

state.overlays.insert(group_name.clone(), entry);
state.save(game_dir)?;
```

#### 1.8.3 Format Determination Helper

To keep the signing decision in one place, add a small helper:

```rust
// dmm-api-test/src-tauri/src/protocol/format_policy.rs

/// Returns true if a mount of this format must be signed per spec §2.5.
pub fn signing_required(mod_format: &str) -> bool {
    matches!(mod_format, "v3" | "v3.1")
}

/// Build a state file entry, signing only if the format requires it.
pub fn build_entry(
    owner: &str,
    owner_version: &str,
    mod_format: &str,
    content: &str,
    files: Vec<String>,
    sign_with: Option<(&[u8; 32], &str)>,   // (priv_key, key_id) — Some only for v3.1
) -> super::OverlayEntry {
    let mut entry = super::OverlayEntry {
        owner: owner.to_string(),
        owner_version: owner_version.to_string(),
        mod_format: mod_format.to_string(),
        content: content.to_string(),
        updated: chrono::Utc::now().to_rfc3339(),
        files,
        signature: None,
        key_id: None,
    };

    if signing_required(mod_format) {
        let (priv_key, key_id) = sign_with.expect("v3/v3.1 mount requires signing key");
        let val = serde_json::to_value(&entry).unwrap();
        let (sig, kid) = super::sign::sign_entry(priv_key, key_id, &val);
        entry.signature = Some(sig);
        entry.key_id = Some(kid);
    }

    entry
}
```

Then v3, v3.1, and legacy mount paths all call `build_entry` with the appropriate `mod_format` string, and signing is automatically applied or skipped.

> ⚠ **Build-time secret**: `include_bytes!("keys/dmm-2026-05.priv")` requires the priv file at build time. Set up a CI/build secrets injection step that places the file before `cargo build`, then deletes it after. Never commit the priv file. Alternative: load from environment variable at runtime.

> ⚠ **No-key fallback**: If the dev/release build is missing the priv key (e.g., contributor PR build), v3/v3.1 mounts will fail with a clear error. Recommended: a build-time check that errors if the priv key is missing for `--features rps-sign` builds. Contributor PR builds without the key cannot produce signed mounts and SHOULD NOT be distributed as official releases.

---

## Phase 2 — SWISS (Python) Implementation

### 2.1 Add Dependency

`CrimsonGameMods/requirements.txt` (or `requirements_v3_1.txt`):

```
cryptography>=41.0
```

### 2.2 Create Protocol Module

`CrimsonGameMods/protocol/__init__.py`:

```python
# SPDX-License-Identifier: LicenseRef-CDMTL-1.0
"""DMM Mod Ecosystem Protocol v1.0 — Python reference implementation."""

from .canonical import canonical_bytes
from .sign import sign_entry
from .verify import verify_entry, VerifyResult
from .state_file import StateFile, OverlayEntry, AuditEntry

__all__ = [
    "canonical_bytes", "sign_entry", "verify_entry", "VerifyResult",
    "StateFile", "OverlayEntry", "AuditEntry",
]
```

### 2.3 Trust Roots

`CrimsonGameMods/protocol/trust_roots.py`:

```python
# SPDX-License-Identifier: LicenseRef-CDMTL-1.0
"""Trust Roots for DMM Mod Ecosystem Protocol v1.0."""

from pathlib import Path
from typing import Optional, NamedTuple

KEYS_DIR = Path(__file__).parent.parent / "keys"


class TrustRoot(NamedTuple):
    key_id: str
    owner: str
    public_key: bytes              # 32 raw bytes
    valid_from: str
    valid_until: Optional[str]


def _load_pub(filename: str) -> bytes:
    return (KEYS_DIR / filename).read_bytes()


TRUST_ROOTS: list[TrustRoot] = [
    TrustRoot(
        key_id="dmm-2026-05",
        owner="DMM",
        public_key=_load_pub("dmm-2026-05.pub"),
        valid_from="2026-05-01T00:00:00Z",
        valid_until=None,
    ),
    TrustRoot(
        key_id="swiss-2026-05",
        owner="CrimsonGameMods",
        public_key=_load_pub("swiss-2026-05.pub"),
        valid_from="2026-05-01T00:00:00Z",
        valid_until=None,
    ),
]


def lookup(key_id: str) -> Optional[TrustRoot]:
    for r in TRUST_ROOTS:
        if r.key_id == key_id:
            return r
    return None
```

### 2.4 Canonical Serialization

`CrimsonGameMods/protocol/canonical.py`:

```python
# SPDX-License-Identifier: LicenseRef-CDMTL-1.0
"""Canonical JSON serialization for signing."""

import json


def canonical_bytes(entry: dict) -> bytes:
    """Strip signature/key_id, sort keys recursively, serialize compact UTF-8."""
    cleaned = _strip(entry)
    return json.dumps(cleaned, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=False).encode("utf-8")


def _strip(v):
    if isinstance(v, dict):
        return {k: _strip(val) for k, val in v.items()
                if k not in ("signature", "key_id")}
    if isinstance(v, list):
        return [_strip(x) for x in v]
    return v
```

### 2.5 Sign

`CrimsonGameMods/protocol/sign.py`:

```python
# SPDX-License-Identifier: LicenseRef-CDMTL-1.0
"""Sign protocol entries with ed25519."""

import base64
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

from .canonical import canonical_bytes


def sign_entry(private_key_bytes: bytes, key_id: str, entry: dict) -> tuple[str, str]:
    """Returns (signature_string, key_id). Caller writes them into the entry."""
    if len(private_key_bytes) != 32:
        raise ValueError(f"private key must be 32 bytes, got {len(private_key_bytes)}")
    signing_key = Ed25519PrivateKey.from_private_bytes(private_key_bytes)
    canonical = canonical_bytes(entry)
    sig = signing_key.sign(canonical)
    sig_b64 = base64.b64encode(sig).decode("ascii")
    return f"ed25519:{sig_b64}", key_id
```

### 2.6 Verify

`CrimsonGameMods/protocol/verify.py`:

```python
# SPDX-License-Identifier: LicenseRef-CDMTL-1.0
"""Verify protocol entry signatures with mod-format-aware policy."""

import base64
import logging
from enum import Enum

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
from cryptography.exceptions import InvalidSignature

from .canonical import canonical_bytes
from .trust_roots import lookup

log = logging.getLogger(__name__)

# Per spec §2.5 — v3 and v3.1 mounts MUST be signed.
SIGNING_REQUIRED_FORMATS = {"v3", "v3.1"}
LEGACY_FORMATS = {"v1", "v2", "legacy"}


class VerifyResult(Enum):
    VALID = "valid"
    LEGACY_UNSIGNED = "legacy_unsigned"          # ACCEPT for v1/v2/legacy/v3
    SIGNATURE_REQUIRED = "signature_required"     # REJECT — v3.1 missing sig
    UNKNOWN_KEY = "unknown_key"                   # REJECT (or downgrade for legacy)
    OWNER_MISMATCH = "owner_mismatch"
    INVALID_SIGNATURE = "invalid_signature"
    MALFORMED_SIGNATURE = "malformed_signature"

    def is_accepted(self) -> bool:
        return self in (VerifyResult.VALID, VerifyResult.LEGACY_UNSIGNED)


def verify_entry(entry: dict) -> VerifyResult:
    mod_format = entry.get("mod_format", "legacy")
    signing_required = mod_format in SIGNING_REQUIRED_FORMATS

    sig_str = entry.get("signature")

    # No signature?
    if not sig_str:
        if signing_required:
            return VerifyResult.SIGNATURE_REQUIRED
        return VerifyResult.LEGACY_UNSIGNED

    # Has signature — try to verify, but downgrade to LegacyUnsigned for
    # legacy formats if anything goes wrong (preserves backward compat).
    def _legacy_or(reject: VerifyResult) -> VerifyResult:
        if signing_required:
            return reject
        log.warning(f"Legacy entry signature problem ({reject.value}) — accepting as legacy")
        return VerifyResult.LEGACY_UNSIGNED

    key_id = entry.get("key_id")
    claimed_owner = entry.get("owner")
    if not key_id or not claimed_owner:
        return _legacy_or(VerifyResult.MALFORMED_SIGNATURE)

    if not sig_str.startswith("ed25519:"):
        return _legacy_or(VerifyResult.MALFORMED_SIGNATURE)
    try:
        sig_bytes = base64.b64decode(sig_str[len("ed25519:"):])
    except Exception:
        return _legacy_or(VerifyResult.MALFORMED_SIGNATURE)
    if len(sig_bytes) != 64:
        return _legacy_or(VerifyResult.MALFORMED_SIGNATURE)

    root = lookup(key_id)
    if root is None:
        return _legacy_or(VerifyResult.UNKNOWN_KEY)
    if root.owner != claimed_owner:
        return _legacy_or(VerifyResult.OWNER_MISMATCH)

    try:
        pub = Ed25519PublicKey.from_public_bytes(root.public_key)
        pub.verify(sig_bytes, canonical_bytes(entry))
        return VerifyResult.VALID
    except InvalidSignature:
        return _legacy_or(VerifyResult.INVALID_SIGNATURE)
```

### 2.7 State File Read/Write

`CrimsonGameMods/protocol/state_file.py`:

```python
# SPDX-License-Identifier: LicenseRef-CDMTL-1.0
"""Protocol v2 State File reader/writer."""

import json
import logging
from datetime import datetime, timezone
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

from .verify import verify_entry, VerifyResult

log = logging.getLogger(__name__)

STATE_FILENAME = "crimson_modding_state.json"
PROTOCOL_VERSION = 2


@dataclass
class OverlayEntry:
    owner: str
    owner_version: str
    content: str
    updated: str
    mod_format: str = "legacy"          # "v1"|"v2"|"legacy"|"v3"|"v3.1" — signing required for "v3" and "v3.1"
    files: list[str] = field(default_factory=list)
    signature: Optional[str] = None
    key_id: Optional[str] = None


@dataclass
class AuditEntry:
    ts: str
    actor: str
    action: str
    group: str
    signature: Optional[str] = None
    key_id: Optional[str] = None


@dataclass
class StateFile:
    protocol_version: int = PROTOCOL_VERSION
    updated: str = ""
    overlays: dict[str, OverlayEntry] = field(default_factory=dict)
    audit_log: list[AuditEntry] = field(default_factory=list)

    @classmethod
    def load(cls, game_dir: Path) -> "StateFile":
        path = game_dir / STATE_FILENAME
        if not path.exists():
            return cls(updated=_now())
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            overlays = {
                k: OverlayEntry(**v) for k, v in data.get("overlays", {}).items()
            }
            audit = [AuditEntry(**a) for a in data.get("audit_log", [])]
            return cls(
                protocol_version=data.get("protocol_version", 1),
                updated=data.get("updated", _now()),
                overlays=overlays,
                audit_log=audit,
            )
        except Exception as e:
            log.error(f"Failed to load state file: {e}")
            return cls(updated=_now())

    def save(self, game_dir: Path) -> None:
        path = game_dir / STATE_FILENAME
        self.updated = _now()
        data = {
            "protocol_version": self.protocol_version,
            "updated": self.updated,
            "overlays": {k: _asdict_compact(v) for k, v in self.overlays.items()},
            "audit_log": [_asdict_compact(a) for a in self.audit_log],
        }
        # Atomic write via temp file
        tmp = path.with_suffix(".tmp")
        tmp.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")
        tmp.replace(path)

    def verified_overlays(self) -> dict[str, OverlayEntry]:
        """Return entries that should be honored (Valid OR LegacyUnsigned)."""
        out = {}
        for slot, entry in self.overlays.items():
            result = verify_entry(asdict(entry))
            if result.is_accepted():
                out[slot] = entry
            else:
                log.warning(f"Rejected state entry {slot}: {result.value}")
        return out


def _now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")


def _asdict_compact(obj) -> dict:
    """asdict but skips None fields."""
    return {k: v for k, v in asdict(obj).items() if v is not None}
```

### 2.8 Wire Into Existing overlay_coordinator.py

**Critical scope rule**: Sign ONLY for v3 and v3.1 mounts. Stacker output (v3/v3.1) signs; legacy mod paths (v1/v2/byte-replace exports) write unsigned entries.

```python
from dataclasses import asdict
from pathlib import Path
from datetime import datetime, timezone

from protocol import StateFile, OverlayEntry, sign_entry

# Per spec §2.5
SIGNING_REQUIRED_FORMATS = {"v3.1"}


def _signing_required(mod_format: str) -> bool:
    return mod_format in SIGNING_REQUIRED_FORMATS


def post_write(
    game_path: Path,
    group: str,
    owner: str,
    mod_format: str,        # "v3.1" for Stacker exports; "v2"/"legacy" otherwise
    content: str,
    files: list[str],
) -> None:
    """Record a successful overlay mount in the State File.

    Per spec §2.5: v3 and v3.1 mounts are signed. Legacy mod formats
    (v1/v2/byte-replace) write unsigned entries indefinitely so existing
    mods continue to mount.
    """
    state = StateFile.load(game_path)
    state.protocol_version = 2

    entry = OverlayEntry(
        owner=owner,                                # "CrimsonGameMods"
        owner_version=_get_swiss_version(),
        mod_format=mod_format,
        content=content,
        updated=datetime.now(timezone.utc).isoformat(),
        files=files,
    )

    # Sign for v3 and v3.1
    if _signing_required(mod_format):
        priv_path = Path(__file__).parent / "keys" / "swiss-2026-05.priv"
        if not priv_path.exists():
            raise RuntimeError(
                f"v3/v3.1 mount requires signing key at {priv_path}, but it was not found. "
                f"Either install the key or downgrade this export to a legacy mod_format."
            )
        priv = priv_path.read_bytes()
        entry_dict = asdict(entry)
        sig, kid = sign_entry(priv, "swiss-2026-05", entry_dict)
        entry.signature = sig
        entry.key_id = kid

    state.overlays[group] = entry
    state.save(game_path)
```

**Caller examples:**

```python
# Stacker v3.1 export — signs
post_write(
    game_path,
    group="0036",
    owner="CrimsonGameMods",
    mod_format="v3.1",
    content="Stacker merged dropsets",
    files=["dropsetinfo.pabgb"],
)

# Legacy v2 byte-diff export — does NOT sign
post_write(
    game_path,
    group="0036",
    owner="CrimsonGameMods",
    mod_format="v2",
    content="Legacy byte-diff stack",
    files=["dropsetinfo.pabgb"],
)
```

> ⚠ **Build-time secret**: For SWISS bundled distributions, the private key is included at build time via PyInstaller's data bundling. Without obfuscation the priv key in the bundled exe is extractable — accepted risk for v1 of the protocol; rotate keys quarterly. For development, load from `keys/swiss-2026-05.priv` (gitignored).

---

## Phase 3 — Rollout Plan

### Goal

Roll out signed v3.1 entries without breaking existing v1/v2/legacy mod mounts.

### Strategy — No Hard Cutoff for Legacy Formats

Per spec §2.5, **v1/v2/legacy mounts are PERMANENTLY exempt from signing**. The rollout only changes behavior for v3 and v3.1 mounts:

**Stage 1 — Initial Release (Week 1):**
- DMM and SWISS releases ship with the protocol module + embedded Trust Roots
- v3 / v3.1 mounts → SIGNED entries written to State File
- v1 / v2 / legacy mounts → UNSIGNED entries (no behavior change vs current)
- All entries (signed or unsigned) read and honored as before
- State File `protocol_version` field upgrades to 2 the first time a v3+ entry is written

**Stage 2 — Forgery Detection (Week 1+, ongoing):**
- A v3 or v3.1 entry with missing/invalid signature is REJECTED
- The slot is treated as if no entry exists for it (foreign tool conflict path)
- Logged so user can see what happened

**Stage 3 — Future Optional Tightening (Not Currently Planned):**
- If RicePaddySoftware later decides to deprecate v1 or v2 mod formats, the spec will be amended to require signing on those formats too
- Until that explicit Protocol amendment, v1/v2/legacy mounts continue to work indefinitely

### What This Means For Existing Installations

| Scenario | Behavior |
|---|---|
| User has only legacy mods (v1/v2) mounted | No change. State file stays unsigned, mods load normally. |
| User has v3 mods mounted from a pre-Protocol-v2 DMM (no `mod_format` field in state) | Treated as `legacy` (default). Soft-trust accepted. Mounts continue to work. Re-mounting via Protocol-v2 DMM upgrades them to signed entries. |
| User installs a v3.1 mod via Protocol-v2 DMM | New v3.1 entry is signed. Existing legacy entries stay as-is. |
| User has both legacy and v3+ mods mounted | State file mixes signed v3/v3.1 entries with unsigned legacy entries. Both honored. |
| User downgrades DMM to a pre-protocol version | Old DMM ignores new fields (`mod_format`, `signature`, `key_id`); legacy entries continue to work. v3/v3.1 entries become orphaned but harmless. |
| Forgery: external tool writes `"owner": "DMM", "mod_format": "v3.1"` with bad sig | Rejected during read. User sees warning in log. |
| Forgery: external tool writes `"owner": "DMM", "mod_format": "v2"` without sig | Treated as legacy entry (soft-trust). This is a known gap — legacy mounts are intentionally trust-on-faith. |

### User-Facing Communication

In release notes:

> **DMM v1.4 / SWISS v3.2 — Protocol v2 (Field JSON v3 + v3.1 signing).**
>
> Field JSON v3 and v3.1 mods now use cryptographic signing for cross-tool coordination between DMM and SWISS. This prevents unauthorized tools from impersonating DMM or SWISS in your game directory.
>
> **Existing v1/v2/legacy mods are unaffected.** They continue to mount and load exactly as before. Existing v3 mounts from older DMM versions also continue to work — they're treated as legacy until you re-mount them with the new DMM, at which point they upgrade to signed entries automatically.
>
> Only RicePaddySoftware's official DMM and SWISS builds can produce signed v3/v3.1 entries.

---

## Phase 4 — Testing

### 4.1 Test Vectors (Cross-Implementation)

Create `tests/protocol/test_vectors.json`:

```json
{
  "test_canonical_form": {
    "input": {
      "signature": "ed25519:should-be-stripped",
      "key_id": "dmm-2026-05",
      "owner": "DMM",
      "owner_version": "1.3.4",
      "content": "test entry",
      "updated": "2026-05-01T12:00:00Z",
      "files": ["iteminfo.pabgb"]
    },
    "expected_canonical_bytes_utf8": "{\"content\":\"test entry\",\"files\":[\"iteminfo.pabgb\"],\"owner\":\"DMM\",\"owner_version\":\"1.3.4\",\"updated\":\"2026-05-01T12:00:00Z\"}"
  },
  "test_known_signature": {
    "private_key_b64": "<32-byte-private-key-base64>",
    "entry": { "owner": "DMM", "owner_version": "1.3.4", "content": "test", "updated": "2026-05-01T00:00:00Z", "files": ["x.pabgb"] },
    "expected_signature": "ed25519:<deterministic-base64-sig>"
  }
}
```

Both DMM (Rust) and SWISS (Python) MUST produce identical canonical bytes and identical signatures from the same input.

### 4.2 Cross-Tool Integration Test

```
1. Generate test keys
2. SWISS writes a state entry with swiss-key
3. DMM reads the state; verify_entry should return Valid
4. DMM writes a state entry with dmm-key
5. SWISS reads the state; verify_entry should return Valid
6. Modify a byte in DMM's entry; SWISS should return InvalidSignature
7. Forge an entry with owner=DMM but signed with swiss-key; should return OwnerMismatch
8. Write an entry with key_id=ghost-2026; should return UnknownKey
```

---

## Phase 5 — Build Process Changes

### DMM (Rust)

`dmm-api-test/build.rs` (NEW or extended):

```rust
// Pre-build: ensure private key file exists for sign-only build target
fn main() {
    if std::env::var("CARGO_FEATURE_RPS_SIGN").is_ok() {
        let priv_path = std::path::Path::new("keys/dmm-2026-05.priv");
        if !priv_path.exists() {
            panic!("Private key not found at {priv_path:?}. \
                    Set RPS_PRIVATE_KEY_DIR or place the key before building.");
        }
    }
    println!("cargo:rerun-if-changed=keys/");
}
```

Add to `Cargo.toml`:
```toml
[features]
default = []
rps-sign = []   # enable in CI build for release
```

Release build: `cargo build --release --features rps-sign`
Dev build (no signing): `cargo build` — produces unsigned v1 entries

### SWISS (Python)

In `bundle_unified.py` or PyInstaller spec:

```python
# Bundle private key into the frozen exe (encrypted-at-rest via py-armor or similar)
datas = [
    ('keys/swiss-2026-05.priv', 'keys'),
    ('keys/swiss-2026-05.pub', 'keys'),
    ('keys/dmm-2026-05.pub', 'keys'),
]
```

> Without obfuscation, the priv key in the bundled exe is extractable. For production, use a code-protection tool (PyArmor, Nuitka with private builds, or a small native helper that holds the key). For initial release, document this as a known limitation and rotate keys quarterly.

---

## Phase 6 — Deployment Checklist

When you're ready to ship Protocol v2:

- [ ] Run `scripts/generate_protocol_keys.py` once on a clean dev machine
- [ ] Move `.priv` files to your secrets store (KeePass entry: "RPS Protocol Keys 2026-05")
- [ ] DELETE `.priv` files from disk after copying
- [ ] Commit `keys/*.pub` files to dmm-parser, dmm-api-test, and SWISS repos
- [ ] Add `keys/*.priv` to `.gitignore` in all three repos
- [ ] Implement Phase 1 (Rust) modules in DMM
- [ ] Implement Phase 2 (Python) modules in SWISS
- [ ] Wire into existing `v3_overlay.rs` / `overlay_coordinator.py`
- [ ] Run cross-implementation test vectors
- [ ] Tag DMM v1.4 and SWISS v3.2 (the dual-version release)
- [ ] Update release notes with the migration explanation
- [ ] Schedule the Stage 2 / Stage 3 milestones (Section 3 above)

---

## Open Decisions for You

1. **Where do private keys live during builds?**
   - Option A: GitHub Actions secrets, injected at CI build time (recommended)
   - Option B: Local dev machine only, manual release builds
   - Option C: Hardware security module (overkill for current scale)

2. **How do you handle bundled SWISS distributions where the priv key ships in the exe?**
   - Acceptable risk: rotate quarterly, accept the priv could be extracted
   - Mitigation: use a small native helper (Rust or Go) that holds the key behind syscall obfuscation
   - Maximalist: per-user signing tied to NexusMods account (requires online auth — kills offline use)

3. **Key rotation cadence?**
   - Recommendation: every 12 months for routine rotation, immediate on suspected leak
   - Document in CHANGELOG.md when keys rotate

4. **Should the audit log persist forever or truncate?**
   - Recommendation: keep last 1000 entries, truncate older — simple and bounded

Tell me your decisions on these four and I can refine the build/key-distribution strategy.

---

*End of Implementation Guide.*
