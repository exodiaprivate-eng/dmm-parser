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

#[derive(Debug, Clone, PartialEq)]
pub enum VerifyResult {
    Valid,
    UnknownKey,
    OwnerMismatch,
    InvalidSignature,
    MalformedSignature,
    LegacyUnsigned,    // v1 entry, no signature field — soft-trust per spec §2.4
    KeyExpired,
}

pub fn verify_entry(entry: &Value) -> VerifyResult {
    // Pull signature + key_id
    let sig_str = match entry.get("signature").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return VerifyResult::LegacyUnsigned,
    };
    let key_id = match entry.get("key_id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return VerifyResult::MalformedSignature,
    };
    let claimed_owner = match entry.get("owner").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return VerifyResult::MalformedSignature,
    };

    // Parse "ed25519:<base64>"
    let sig_b64 = match sig_str.strip_prefix("ed25519:") {
        Some(s) => s,
        None => return VerifyResult::MalformedSignature,
    };
    let sig_bytes = match STANDARD.decode(sig_b64) {
        Ok(b) => b,
        Err(_) => return VerifyResult::MalformedSignature,
    };
    let sig_array: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return VerifyResult::MalformedSignature,
    };
    let signature = Signature::from_bytes(&sig_array);

    // Look up trust root
    let root: &TrustRoot = match lookup(key_id) {
        Some(r) => r,
        None => return VerifyResult::UnknownKey,
    };
    if root.owner != claimed_owner {
        return VerifyResult::OwnerMismatch;
    }
    // (Optional: check valid_from / valid_until against current time)

    // Verify
    let verifying_key = match VerifyingKey::from_bytes(&root.public_key) {
        Ok(k) => k,
        Err(_) => return VerifyResult::UnknownKey,
    };
    let canonical = canonical_bytes(entry);
    match verifying_key.verify(&canonical, &signature) {
        Ok(()) => VerifyResult::Valid,
        Err(_) => VerifyResult::InvalidSignature,
    }
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
    pub content: String,
    pub updated: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
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

    /// Return only entries whose signature verifies.
    /// Unsigned legacy entries are returned as Unknown owner if they claim DMM/CrimsonGameMods.
    pub fn verified_overlays(&self) -> HashMap<String, &OverlayEntry> {
        let mut out = HashMap::new();
        for (slot, entry) in &self.overlays {
            let value = serde_json::to_value(entry).unwrap();
            match verify_entry(&value) {
                VerifyResult::Valid => {
                    out.insert(slot.clone(), entry);
                }
                VerifyResult::LegacyUnsigned => {
                    // Soft-trust mode: accept v1 entries from known good owners
                    if entry.owner == "DMM" || entry.owner == "CrimsonGameMods" || entry.owner == "JMM" {
                        out.insert(slot.clone(), entry);
                    }
                }
                _ => {
                    log::warn!("Rejected state entry {slot}: {:?}", verify_entry(&value));
                }
            }
        }
        out
    }
}
```

### 1.8 Wire Into Existing v3_overlay.rs

In `dmm-api-test/src-tauri/src/iteminfo/v3_overlay.rs`, after a successful mount, replace the implicit state update with a signed write:

```rust
use crate::protocol::{StateFile, OverlayEntry, sign::sign_entry};

// After successful PAPGT registration in apply_v3_overlay():
let mut state = StateFile::load(game_dir).unwrap_or_else(|_| StateFile::empty());

let mut entry = OverlayEntry {
    owner: "DMM".to_string(),
    owner_version: env!("CARGO_PKG_VERSION").to_string(),
    content: format!("v3.1 field-level intents on {target_table}"),
    updated: chrono::Utc::now().to_rfc3339(),
    files: vec![target_table.to_string()],
    signature: None,
    key_id: None,
};

// Sign it
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

> ⚠ **Build-time secret**: `include_bytes!("keys/dmm-2026-05.priv")` requires the priv file at build time. Set up a CI/build secrets injection step that places the file before `cargo build`, then deletes it after. Never commit the priv file. Alternative: load from environment variable at runtime.

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
"""Verify protocol entry signatures."""

import base64
from enum import Enum

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
from cryptography.exceptions import InvalidSignature

from .canonical import canonical_bytes
from .trust_roots import lookup


class VerifyResult(Enum):
    VALID = "valid"
    UNKNOWN_KEY = "unknown_key"
    OWNER_MISMATCH = "owner_mismatch"
    INVALID_SIGNATURE = "invalid_signature"
    MALFORMED_SIGNATURE = "malformed_signature"
    LEGACY_UNSIGNED = "legacy_unsigned"


def verify_entry(entry: dict) -> VerifyResult:
    sig_str = entry.get("signature")
    if not sig_str:
        return VerifyResult.LEGACY_UNSIGNED

    key_id = entry.get("key_id")
    claimed_owner = entry.get("owner")
    if not key_id or not claimed_owner:
        return VerifyResult.MALFORMED_SIGNATURE

    if not sig_str.startswith("ed25519:"):
        return VerifyResult.MALFORMED_SIGNATURE
    try:
        sig_bytes = base64.b64decode(sig_str[len("ed25519:"):])
    except Exception:
        return VerifyResult.MALFORMED_SIGNATURE
    if len(sig_bytes) != 64:
        return VerifyResult.MALFORMED_SIGNATURE

    root = lookup(key_id)
    if root is None:
        return VerifyResult.UNKNOWN_KEY
    if root.owner != claimed_owner:
        return VerifyResult.OWNER_MISMATCH

    try:
        pub = Ed25519PublicKey.from_public_bytes(root.public_key)
        pub.verify(sig_bytes, canonical_bytes(entry))
        return VerifyResult.VALID
    except InvalidSignature:
        return VerifyResult.INVALID_SIGNATURE
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
        """Return only entries whose signature verifies."""
        out = {}
        for slot, entry in self.overlays.items():
            entry_dict = asdict(entry)
            result = verify_entry(entry_dict)
            if result == VerifyResult.VALID:
                out[slot] = entry
            elif result == VerifyResult.LEGACY_UNSIGNED:
                # Soft-trust v1 entries from known good owners
                if entry.owner in ("DMM", "CrimsonGameMods", "JMM"):
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

In `CrimsonGameMods/overlay_coordinator.py`, replace the unsigned record-write with signed:

```python
from protocol import StateFile, OverlayEntry, sign_entry
from datetime import datetime, timezone

def post_write(game_path: Path, group: str, owner: str, content: str, files: list[str]):
    state = StateFile.load(game_path)
    state.protocol_version = 2

    entry = OverlayEntry(
        owner=owner,                               # "CrimsonGameMods"
        owner_version=_get_swiss_version(),
        content=content,
        updated=datetime.now(timezone.utc).isoformat(),
        files=files,
    )

    # Sign it
    priv_path = Path(__file__).parent / "keys" / "swiss-2026-05.priv"
    if priv_path.exists():
        priv = priv_path.read_bytes()
        entry_dict = asdict(entry)
        sig, kid = sign_entry(priv, "swiss-2026-05", entry_dict)
        entry.signature = sig
        entry.key_id = kid
    else:
        log.warning("Private key not present; writing unsigned (legacy v1) entry")

    state.overlays[group] = entry
    state.save(game_path)
```

> ⚠ **Build-time secret**: For SWISS bundled distributions, the private key is included at build time via PyInstaller's data bundling, then encrypted at-rest using a key derived from the binary's hash. For development, load from `keys/swiss-2026-05.priv` (gitignored).

---

## Phase 3 — Migration Plan (Unsigned v1 → Signed v2)

### Goal

Roll out signed entries without breaking existing user installations.

### Strategy

**Stage 1 — Dual-version (Weeks 1-4):**
- New DMM and SWISS releases write v2 signed entries
- They also accept v1 unsigned legacy entries from known good owners (`DMM`, `CrimsonGameMods`, `JMM`)
- This is the `LEGACY_UNSIGNED → soft-trust` path in the verify code

**Stage 2 — Warning (Weeks 5-12):**
- DMM and SWISS show a warning when reading unsigned entries: "Legacy unsigned entry from `<owner>`. Update your other tool to v1.4+ for full protocol compliance."
- All new entries are v2 signed

**Stage 3 — Hard cutoff (Week 13+):**
- Release DMM 1.5 / SWISS 4.0 that REJECT unsigned entries claiming `DMM` or `CrimsonGameMods` ownership
- Unsigned entries are still accepted but marked `Unknown` owner (treated as foreign for conflict purposes)

### User-Facing Communication

In the release notes for the first signed-version release:

> **DMM 1.4 / SWISS 3.2** introduces cryptographic signing for cross-tool coordination. This prevents unauthorized tools from impersonating DMM or SWISS in your game directory.
>
> If you use both DMM and SWISS, **update both tools** for full protocol compliance. Single-tool installations are unaffected.

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
