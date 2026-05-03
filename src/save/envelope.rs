// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Save file envelope: the 0x80-byte header + ChaCha20/LZ4/HMAC pipeline.
//!
//! Format reference: `references/save_notes.md` §1.
//!
//! ## Why HMAC is a closure
//!
//! The save key is a derived secret. Embedding it in this public crate
//! would publish material the upstream tool keeps (loosely) gated. To
//! stay a clean format library, [`decrypt_envelope`] and
//! [`encrypt_envelope`] take an `hmac` closure — the caller (SWISS,
//! CrimsonSaveEditor, a CI rig with the key in env) computes the
//! HMAC-SHA256 themselves and we just verify/embed bytes. ChaCha20 is
//! provided by the existing `chacha20_crate` dep. LZ4 by `lz4_flex`.

use chacha20_crate::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use std::fmt;

/// Header size in bytes (everything before the encrypted payload).
pub const HEADER_SIZE: usize = 0x80;

/// File magic — ASCII "SAVE" at offset 0.
pub const MAGIC: [u8; 4] = *b"SAVE";

const VERSION_OFFSET: usize = 0x04;
const FLAGS_OFFSET: usize = 0x06;
const RESERVED_A_OFFSET: usize = 0x08;
const RESERVED_A_SIZE: usize = 10;
const UNCOMP_SIZE_OFFSET: usize = 0x12;
const PAYLOAD_SIZE_OFFSET: usize = 0x16;
const NONCE_OFFSET: usize = 0x1A;
const NONCE_SIZE: usize = 16;
const HMAC_OFFSET: usize = 0x2A;
const HMAC_SIZE: usize = 32;

/// Parsed plaintext header. Round-trips with `to_bytes()`.
#[derive(Debug, Clone)]
pub struct SaveHeader {
    pub version: u16,
    pub flags: u16,
    /// Bytes 0x08..0x12 — preserved on rewrite, semantics TBD.
    pub reserved_a: [u8; RESERVED_A_SIZE],
    pub uncompressed_size: u32,
    pub payload_size: u32,
    pub nonce: [u8; NONCE_SIZE],
    pub hmac: [u8; HMAC_SIZE],
}

impl SaveHeader {
    /// Serialize to the 0x80-byte plaintext header layout. The trailing
    /// 0x4A..0x80 reserved region is zeroed.
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut out = [0u8; HEADER_SIZE];
        out[..4].copy_from_slice(&MAGIC);
        out[VERSION_OFFSET..VERSION_OFFSET + 2].copy_from_slice(&self.version.to_le_bytes());
        out[FLAGS_OFFSET..FLAGS_OFFSET + 2].copy_from_slice(&self.flags.to_le_bytes());
        out[RESERVED_A_OFFSET..RESERVED_A_OFFSET + RESERVED_A_SIZE]
            .copy_from_slice(&self.reserved_a);
        out[UNCOMP_SIZE_OFFSET..UNCOMP_SIZE_OFFSET + 4]
            .copy_from_slice(&self.uncompressed_size.to_le_bytes());
        out[PAYLOAD_SIZE_OFFSET..PAYLOAD_SIZE_OFFSET + 4]
            .copy_from_slice(&self.payload_size.to_le_bytes());
        out[NONCE_OFFSET..NONCE_OFFSET + NONCE_SIZE].copy_from_slice(&self.nonce);
        out[HMAC_OFFSET..HMAC_OFFSET + HMAC_SIZE].copy_from_slice(&self.hmac);
        out
    }
}

/// Result of a successful envelope decrypt: the parsed header, the
/// decompressed body, and any non-fatal warnings (HMAC mismatch is
/// treated as a warning so SWISS can choose its policy).
#[derive(Debug)]
pub struct SaveEnvelope {
    pub header: SaveHeader,
    pub body: Vec<u8>,
    pub warnings: Vec<EnvelopeWarning>,
}

/// Non-fatal issues observed during decrypt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeWarning {
    /// The stored HMAC didn't match the recomputed one. Body still
    /// returned — caller decides whether to refuse the file.
    HmacMismatch,
}

/// Hard failures during decrypt or encrypt.
#[derive(Debug)]
pub enum EnvelopeError {
    TooSmall { actual: usize, min: usize },
    BadMagic { got: [u8; 4] },
    PayloadOverflow { payload_size: u32, file_size: usize },
    Lz4(lz4_flex::block::DecompressError),
    Lz4Compress(lz4_flex::block::CompressError),
    Lz4SizeMismatch { decoded: usize, expected: u32 },
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvelopeError::TooSmall { actual, min } =>
                write!(f, "save file too small: {} < {}", actual, min),
            EnvelopeError::BadMagic { got } =>
                write!(f, "bad magic: expected 'SAVE', got {:?}", got),
            EnvelopeError::PayloadOverflow { payload_size, file_size } =>
                write!(f, "payload_size {} exceeds remaining file ({} bytes)", payload_size, file_size),
            EnvelopeError::Lz4(e) => write!(f, "lz4 decompress: {}", e),
            EnvelopeError::Lz4Compress(e) => write!(f, "lz4 compress: {}", e),
            EnvelopeError::Lz4SizeMismatch { decoded, expected } =>
                write!(f, "lz4 produced {} bytes, header expected {}", decoded, expected),
        }
    }
}

impl std::error::Error for EnvelopeError {}

/// Parse just the 0x80 header without touching the payload. Useful for
/// inspecting a save without holding its key.
pub fn parse_header(bytes: &[u8]) -> Result<SaveHeader, EnvelopeError> {
    if bytes.len() < HEADER_SIZE {
        return Err(EnvelopeError::TooSmall { actual: bytes.len(), min: HEADER_SIZE });
    }
    let mut magic = [0u8; 4];
    magic.copy_from_slice(&bytes[..4]);
    if magic != MAGIC {
        return Err(EnvelopeError::BadMagic { got: magic });
    }

    let version = u16::from_le_bytes([bytes[VERSION_OFFSET], bytes[VERSION_OFFSET + 1]]);
    let flags = u16::from_le_bytes([bytes[FLAGS_OFFSET], bytes[FLAGS_OFFSET + 1]]);

    let mut reserved_a = [0u8; RESERVED_A_SIZE];
    reserved_a.copy_from_slice(&bytes[RESERVED_A_OFFSET..RESERVED_A_OFFSET + RESERVED_A_SIZE]);

    let uncompressed_size = u32::from_le_bytes(
        bytes[UNCOMP_SIZE_OFFSET..UNCOMP_SIZE_OFFSET + 4].try_into().unwrap()
    );
    let payload_size = u32::from_le_bytes(
        bytes[PAYLOAD_SIZE_OFFSET..PAYLOAD_SIZE_OFFSET + 4].try_into().unwrap()
    );

    let mut nonce = [0u8; NONCE_SIZE];
    nonce.copy_from_slice(&bytes[NONCE_OFFSET..NONCE_OFFSET + NONCE_SIZE]);

    let mut hmac = [0u8; HMAC_SIZE];
    hmac.copy_from_slice(&bytes[HMAC_OFFSET..HMAC_OFFSET + HMAC_SIZE]);

    Ok(SaveHeader { version, flags, reserved_a, uncompressed_size, payload_size, nonce, hmac })
}

/// Decrypt + decompress a save file.
///
/// `hmac` is invoked on the **compressed plaintext** (post-decrypt,
/// pre-decompress) and must return `[u8; 32]` (the HMAC-SHA256 digest
/// using the caller's key). If the digest doesn't match the header's
/// stored HMAC, the body is still returned but `EnvelopeWarning::HmacMismatch`
/// is included.
///
/// `key32` is the 32-byte ChaCha20 key. The 16-byte nonce stored in the
/// header is passed directly to ChaCha20 (its leading u32 LE is the
/// block counter, its trailing 12 bytes are the ChaCha20 nonce — this
/// matches OpenSSL/`cryptography`'s ChaCha20 primitive convention).
pub fn decrypt_envelope<H>(
    bytes: &[u8],
    key32: &[u8; 32],
    hmac: H,
) -> Result<SaveEnvelope, EnvelopeError>
where
    H: FnOnce(&[u8]) -> [u8; 32],
{
    let header = parse_header(bytes)?;

    let payload_start = HEADER_SIZE;
    let payload_end = payload_start
        .checked_add(header.payload_size as usize)
        .ok_or(EnvelopeError::PayloadOverflow {
            payload_size: header.payload_size,
            file_size: bytes.len().saturating_sub(payload_start),
        })?;
    if payload_end > bytes.len() {
        return Err(EnvelopeError::PayloadOverflow {
            payload_size: header.payload_size,
            file_size: bytes.len().saturating_sub(payload_start),
        });
    }
    let ciphertext = &bytes[payload_start..payload_end];

    // 16-byte nonce splits into a u32 LE block counter (first 4 bytes)
    // plus the 12-byte ChaCha20 nonce (last 12 bytes), matching the
    // OpenSSL/`cryptography` ChaCha20 convention.
    let initial_counter = u32::from_le_bytes(header.nonce[0..4].try_into().unwrap());
    let nonce12: [u8; 12] = header.nonce[4..16].try_into().unwrap();
    let mut compressed = ciphertext.to_vec();
    let mut cipher = chacha20_crate::ChaCha20::new(key32.into(), (&nonce12).into());
    cipher.seek(initial_counter as u64 * 64);
    cipher.apply_keystream(&mut compressed);

    let mut warnings = Vec::new();
    let computed_hmac = hmac(&compressed);
    if computed_hmac != header.hmac {
        warnings.push(EnvelopeWarning::HmacMismatch);
    }

    let body = lz4_flex::block::decompress(&compressed, header.uncompressed_size as usize)
        .map_err(EnvelopeError::Lz4)?;
    if body.len() != header.uncompressed_size as usize {
        return Err(EnvelopeError::Lz4SizeMismatch {
            decoded: body.len(),
            expected: header.uncompressed_size,
        });
    }

    Ok(SaveEnvelope { header, body, warnings })
}

/// Compress + encrypt a save body, producing a complete `SAVE`-format
/// file. Caller supplies a fresh 16-byte nonce (e.g. from a CSPRNG) so
/// the envelope module stays deterministic and testable.
///
/// `hmac` is invoked on the compressed plaintext; the resulting digest
/// is embedded in the header. `template_header` lets the caller
/// preserve the original 0x12 prefix bytes (magic/version/flags/reserved_a)
/// from the file being rewritten — pass `None` to start clean.
pub fn encrypt_envelope<H>(
    body: &[u8],
    key32: &[u8; 32],
    nonce16: [u8; NONCE_SIZE],
    template_header: Option<&SaveHeader>,
    hmac: H,
) -> Result<Vec<u8>, EnvelopeError>
where
    H: FnOnce(&[u8]) -> [u8; 32],
{
    let compressed = lz4_flex::block::compress(body);

    let digest = hmac(&compressed);

    let initial_counter = u32::from_le_bytes(nonce16[0..4].try_into().unwrap());
    let nonce12: [u8; 12] = nonce16[4..16].try_into().unwrap();
    let mut ciphertext = compressed.clone();
    let mut cipher = chacha20_crate::ChaCha20::new(key32.into(), (&nonce12).into());
    cipher.seek(initial_counter as u64 * 64);
    cipher.apply_keystream(&mut ciphertext);

    let header = SaveHeader {
        version: template_header.map(|h| h.version).unwrap_or(2),
        flags: template_header.map(|h| h.flags).unwrap_or(0x0080),
        reserved_a: template_header.map(|h| h.reserved_a).unwrap_or([0u8; RESERVED_A_SIZE]),
        uncompressed_size: body.len() as u32,
        payload_size: ciphertext.len() as u32,
        nonce: nonce16,
        hmac: digest,
    };

    let mut out = Vec::with_capacity(HEADER_SIZE + ciphertext.len());
    out.extend_from_slice(&header.to_bytes());
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stub HMAC that returns a deterministic 32-byte digest derived
    /// from the input length. Lets us roundtrip without pulling in
    /// sha2/hmac. Real callers will plug in HMAC-SHA256.
    fn fake_hmac(data: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        let len = data.len() as u64;
        out[..8].copy_from_slice(&len.to_le_bytes());
        // Sprinkle a few bytes from the data so different bodies
        // produce different "digests" in tests.
        for (i, b) in data.iter().take(24).enumerate() {
            out[8 + i] = *b;
        }
        out
    }

    #[test]
    fn header_roundtrip() {
        let h = SaveHeader {
            version: 2,
            flags: 0x0080,
            reserved_a: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            uncompressed_size: 0x1234_5678,
            payload_size: 0x0000_BEEF,
            nonce: [0xAA; 16],
            hmac: [0xCC; 32],
        };
        let bytes = h.to_bytes();
        assert_eq!(&bytes[..4], &MAGIC);
        assert_eq!(bytes.len(), HEADER_SIZE);
        let parsed = parse_header(&bytes).unwrap();
        assert_eq!(parsed.version, h.version);
        assert_eq!(parsed.flags, h.flags);
        assert_eq!(parsed.reserved_a, h.reserved_a);
        assert_eq!(parsed.uncompressed_size, h.uncompressed_size);
        assert_eq!(parsed.payload_size, h.payload_size);
        assert_eq!(parsed.nonce, h.nonce);
        assert_eq!(parsed.hmac, h.hmac);
    }

    #[test]
    fn rejects_too_small() {
        let r = parse_header(b"SAVE");
        assert!(matches!(r, Err(EnvelopeError::TooSmall { .. })));
    }

    #[test]
    fn rejects_bad_magic() {
        let mut buf = [0u8; HEADER_SIZE];
        buf[..4].copy_from_slice(b"NOPE");
        let r = parse_header(&buf);
        assert!(matches!(r, Err(EnvelopeError::BadMagic { .. })));
    }

    #[test]
    fn full_envelope_roundtrip() {
        // A body large enough for LZ4 to do something interesting.
        let body: Vec<u8> = (0..4096).map(|i| (i as u8).wrapping_mul(7)).collect();
        let key = [0x42u8; 32];
        let nonce = [0x55u8; 16];

        let file = encrypt_envelope(&body, &key, nonce, None, fake_hmac).unwrap();
        assert!(file.len() > HEADER_SIZE);
        assert_eq!(&file[..4], &MAGIC);

        let env = decrypt_envelope(&file, &key, fake_hmac).unwrap();
        assert_eq!(env.body, body);
        assert!(env.warnings.is_empty(), "matching HMAC should produce no warnings");
        assert_eq!(env.header.version, 2);
        assert_eq!(env.header.flags, 0x0080);
        assert_eq!(env.header.uncompressed_size, body.len() as u32);
        assert_eq!(env.header.nonce, nonce);
    }

    #[test]
    fn hmac_mismatch_warns_but_returns_body() {
        let body = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let key = [0x99u8; 32];
        let nonce = [0x11u8; 16];

        let file = encrypt_envelope(&body, &key, nonce, None, fake_hmac).unwrap();

        // Decrypt with a different HMAC closure so the digest disagrees.
        let bad_hmac = |_: &[u8]| -> [u8; 32] { [0xDE; 32] };
        let env = decrypt_envelope(&file, &key, bad_hmac).unwrap();
        assert_eq!(env.body, body);
        assert_eq!(env.warnings, vec![EnvelopeWarning::HmacMismatch]);
    }

    #[test]
    fn template_header_preserves_prefix() {
        let body = b"hello world".to_vec();
        let key = [0x33u8; 32];
        let nonce = [0x77u8; 16];

        let template = SaveHeader {
            version: 2,
            flags: 0x0123,
            reserved_a: [9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
            uncompressed_size: 0,
            payload_size: 0,
            nonce: [0; 16],
            hmac: [0; 32],
        };

        let file = encrypt_envelope(&body, &key, nonce, Some(&template), fake_hmac).unwrap();
        let parsed = parse_header(&file).unwrap();
        assert_eq!(parsed.version, template.version);
        assert_eq!(parsed.flags, template.flags);
        assert_eq!(parsed.reserved_a, template.reserved_a);
    }

    #[test]
    fn payload_overflow_detected() {
        let mut buf = vec![0u8; HEADER_SIZE + 4];
        buf[..4].copy_from_slice(&MAGIC);
        // Claim the payload is huge.
        let huge = (0xFFFF_FFFFu32).to_le_bytes();
        buf[PAYLOAD_SIZE_OFFSET..PAYLOAD_SIZE_OFFSET + 4].copy_from_slice(&huge);
        let r = decrypt_envelope(&buf, &[0u8; 32], fake_hmac);
        assert!(matches!(r, Err(EnvelopeError::PayloadOverflow { .. })));
    }
}
