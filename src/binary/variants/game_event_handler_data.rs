//! GameEventHandlerData polymorphic family wrapper.
//!
//! Per Win-IDA dispatcher `sub_1415BE5E0`, the wire format is:
//!   - u8 sub_tag (0-4)
//!     - 0 → SetSceneObjectParameterBySceneLevel (32B struct)
//!     - 1 → SetSceneObjectParameter (32B struct)
//!     - 2 → SetUIPlayGuideParameter (32B struct)
//!     - 3 → SetUIFullscreenGuideParameter (24B struct)
//!     - 4 → MakeSnapshotForDev (24B struct)
//!     - 5+ → error (returns null)
//!   - body bytes per sub_tag's vtable[4] reader
//!
//! The consuming table (GameEventHandlerInfo) tail also contains a
//! trailing u8 `_isPendOnBattleState` AFTER the variant body. The
//! wrapper struct here owns the sub_tag + body; the consuming table
//! owns the trailing byte separately (mirrors the C++ reader's call
//! sequence: variant first, then the standalone u8).
//!
//! ## Decode strategy: Decoded | Raw fallback
//!
//! Same pattern as GameCondition and GlobalGameEventExecuteData: typed
//! decode for the common case, byte-perfect Raw fallback otherwise.
//! Body stays as `Vec<u8>` inside Decoded — sub_tag-aware filtering and
//! clone-between-entries work without per-sub_tag typed payload structs.
//! Future enhancement: replace `body: Vec<u8>` with typed payloads.

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub enum GameEventHandlerData {
    /// Recognized sub_tag (0-4); body bytes captured opaquely.
    /// On wire: `[sub_tag, ...body]`.
    Decoded { sub_tag: u8, body: Vec<u8> },
    /// Fallback for unrecognized sub_tag or empty wrapper.
    /// Bytes preserved verbatim — round-trips byte-perfect.
    Raw(Vec<u8>),
}

impl GameEventHandlerData {
    /// `data` should be sized to exactly the wrapper bytes (sub_tag +
    /// body, NOT including any consumer-owned trailing fields). The
    /// consumer is responsible for slicing off any post-wrapper bytes
    /// before calling this.
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let start = *offset;
        if start >= data.len() {
            return Ok(Self::Raw(Vec::new()));
        }
        let sub_tag = data[start];
        if sub_tag <= 4 {
            let body = data[start + 1..].to_vec();
            *offset = data.len();
            Ok(Self::Decoded { sub_tag, body })
        } else {
            let raw = data[start..].to_vec();
            *offset = data.len();
            Ok(Self::Raw(raw))
        }
    }
    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Decoded { sub_tag, body } => {
                sub_tag.write_to(w)?;
                w.write_all(body)
            }
            Self::Raw(bytes) => w.write_all(bytes),
        }
    }
}
