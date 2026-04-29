//! GlobalGameEventExecuteData polymorphic family wrapper.
//!
//! Per Win-IDA dispatcher `sub_141156680`, the wire format is:
//!   - u8 presence (0 = absent, 1 = present)
//!   - if present:
//!     - u8 sub_tag
//!         - 0 → ConstructorVaryTradeItemPrice (88-byte struct, body via
//!           sub_141155000: CArray<u16> + CArray<VaryTradeItemPriceData>
//!           + u32-hash-lookup + LocalizableString)
//!         - 1 → ConstructorOpenRoyalSupply (32-byte struct, body via
//!           sub_141155300 → sub_1411553D0)
//!         - 2 → in-place reuse of existing pointer (no alloc)
//!         - other → error (return 0)
//!     - body bytes per sub_tag-specific reader
//!
//! ## Decode strategy
//!
//! This wrapper follows the GameCondition pattern: `Decoded | Raw` enum
//! with the Raw variant capturing bytes verbatim if anything looks off.
//! That guarantees byte-perfect round-trip even before per-sub_tag body
//! recipes are fully reversed.
//!
//! `Decoded` parses the presence byte + sub_tag, but stores the body as
//! `Vec<u8>` rather than typed fields. That's enough for clone-between-
//! entries / sub_tag-aware filtering without doing the full body
//! reverse-engineering. Future enhancement: replace `body: Vec<u8>` with
//! per-sub_tag typed payload structs.

use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub enum GlobalGameEventExecuteData {
    /// Wrapper has presence byte == 0; no sub_tag, no body.
    /// On wire: `[0x00]`.
    Absent,
    /// Wrapper has presence byte == 1; sub_tag-driven body.
    /// On wire: `[0x01, sub_tag, ...body]`.
    Present { sub_tag: u8, body: Vec<u8> },
    /// Fallback for unrecognized presence byte or empty wrapper.
    /// Bytes preserved verbatim — round-trips byte-perfect.
    Raw(Vec<u8>),
}

impl GlobalGameEventExecuteData {
    /// `data` should be sized to exactly the wrapper bytes (table-level
    /// tail_blob). The Raw fallback uses `data[start..]` on any decode
    /// path that doesn't cleanly consume all bytes.
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let start = *offset;
        // Empty tail → not even a presence byte → Raw passthrough.
        if start >= data.len() {
            return Ok(Self::Raw(Vec::new()));
        }
        let presence = data[start];
        match presence {
            0 if data.len() - start == 1 => {
                *offset = data.len();
                Ok(Self::Absent)
            }
            1 if data.len() - start >= 2 => {
                let sub_tag = data[start + 1];
                let body = data[start + 2..].to_vec();
                *offset = data.len();
                Ok(Self::Present { sub_tag, body })
            }
            _ => {
                // Unknown presence byte or short data — capture verbatim.
                let raw = data[start..].to_vec();
                *offset = data.len();
                Ok(Self::Raw(raw))
            }
        }
    }
    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Absent => 0u8.write_to(w),
            Self::Present { sub_tag, body } => {
                1u8.write_to(w)?;
                sub_tag.write_to(w)?;
                w.write_all(body)
            }
            Self::Raw(bytes) => w.write_all(bytes),
        }
    }
}
