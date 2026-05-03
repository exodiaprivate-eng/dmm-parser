// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `DdsAssetMetadata` — the aggregate struct a v3.1 packager (SWISS
//! Stacker, mod author CLI) needs about a DDS file on disk.
//!
//! Wraps the format classification (D3) and vpath inference (D4) plus
//! file integrity (SHA-256 + size) into a single value the caller can
//! drop straight into a v3.1 `type:"asset"` target entry.

use std::io;
use std::path::Path;

use super::classify::{classify, DdsClassification, DdsFormat};
use super::vpath::infer_vpath_from_disk_path;

/// Aggregate metadata for a DDS file being packaged into a v3.1 mod.
#[derive(Debug, Clone, PartialEq)]
pub struct DdsAssetMetadata {
    /// Suggested vpath relative to the asset root (e.g. `0009/character/
    /// texture/macduff/diffuse.dds`). `None` when the file's location
    /// can't be inferred (caller must set the target manually).
    pub vpath_hint: Option<String>,

    /// Logical pixel format detected from FOURCC + DXGI extension.
    pub format: DdsFormat,

    /// (width, height) in pixels.
    pub dimensions: (u32, u32),

    /// Mip-map count. Always at least 1.
    pub mip_count: u32,

    /// File size in bytes (full DDS file including header + body).
    pub size: u64,

    /// Lowercase hex SHA-256 digest.
    pub sha256: String,

    /// True when the format requires PATHC template registration at
    /// mount time (BC6H/BC7 / DX10 textures). v3.1 loaders that bundle
    /// textures must surface this flag so DMM-side handling kicks in.
    pub requires_pathc: bool,

    /// Underlying classification for callers that need deeper detail
    /// (e.g. `is_dx10`, `dxgi_format`, `crimson_last4`).
    pub classification: DdsClassification,
}

impl DdsAssetMetadata {
    /// Build metadata by reading a DDS file from disk and (optionally)
    /// inferring its vpath relative to a mod's asset root.
    ///
    /// `sha256` must be the lowercase hex SHA-256 digest of the file —
    /// caller computes it (SWISS uses Python's `hashlib`; CLI tools can
    /// use any SHA-256 implementation). dmm-parser does not bundle a
    /// SHA-256 implementation to avoid adding a hashing dependency that
    /// only this metadata struct would need.
    ///
    /// `asset_root: None` skips vpath inference (`vpath_hint` will be
    /// `None`). Useful for one-off inspections.
    pub fn from_path(
        file_path: &Path,
        asset_root: Option<&Path>,
        sha256: String,
    ) -> io::Result<Self> {
        let bytes = std::fs::read(file_path)?;
        let mut meta = Self::from_bytes(&bytes, sha256)?;
        if let Some(root) = asset_root {
            meta.vpath_hint = infer_vpath_from_disk_path(root, file_path);
        }
        Ok(meta)
    }

    /// Build metadata from in-memory bytes. `vpath_hint` is `None` —
    /// callers using this directly are responsible for setting the
    /// target if they care about it. `sha256` is provided by the caller
    /// (see `from_path` rationale).
    pub fn from_bytes(bytes: &[u8], sha256: String) -> io::Result<Self> {
        let classification = classify(bytes)?;
        Ok(DdsAssetMetadata {
            vpath_hint: None,
            format: classification.format,
            dimensions: (classification.width, classification.height),
            mip_count: classification.mip_count,
            size: bytes.len() as u64,
            sha256,
            requires_pathc: classification.requires_pathc,
            classification,
        })
    }

    /// Render this metadata as a v3.1 asset-target JSON entry, ready to
    /// drop into a `targets[]` array per the FIELD_JSON_V3_1_SPEC.
    /// Caller may override the resulting `target.file` if `vpath_hint`
    /// isn't what they want.
    pub fn to_v3_1_asset_entry(&self, source_relative: &str) -> serde_json::Value {
        let target_file = self.vpath_hint
            .clone()
            .unwrap_or_else(|| source_relative.to_string());
        serde_json::json!({
            "target": { "file": target_file },
            "type": "asset",
            "source": source_relative,
            "sha256": self.sha256,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::header::DDS_FILE_HEADER_SIZE;

    fn make_dds(fourcc: &[u8; 4], width: u32, height: u32) -> Vec<u8> {
        let mut buf = vec![0u8; DDS_FILE_HEADER_SIZE + 64];
        buf[0..4].copy_from_slice(b"DDS ");
        buf[4..8].copy_from_slice(&124u32.to_le_bytes());
        buf[12..16].copy_from_slice(&height.to_le_bytes());
        buf[16..20].copy_from_slice(&width.to_le_bytes());
        buf[24..28].copy_from_slice(&1u32.to_le_bytes());
        buf[28..32].copy_from_slice(&4u32.to_le_bytes());
        buf[76..80].copy_from_slice(&32u32.to_le_bytes());
        buf[80..84].copy_from_slice(&4u32.to_le_bytes());
        buf[84..88].copy_from_slice(fourcc);
        buf
    }

    const FAKE_SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn metadata_from_dxt5_bytes() {
        let buf = make_dds(b"DXT5", 256, 256);
        let m = DdsAssetMetadata::from_bytes(&buf, FAKE_SHA.to_string()).unwrap();
        assert_eq!(m.format, DdsFormat::Dxt5);
        assert_eq!(m.dimensions, (256, 256));
        assert_eq!(m.mip_count, 4);
        assert_eq!(m.size as usize, buf.len());
        assert_eq!(m.sha256, FAKE_SHA);
        assert!(!m.requires_pathc);
        assert!(m.vpath_hint.is_none());
    }

    #[test]
    fn metadata_from_bc7_bytes_requires_pathc() {
        let mut buf = make_dds(b"DX10", 512, 512);
        if buf.len() < 148 {
            buf.resize(148 + 64, 0);
        }
        buf[128..132].copy_from_slice(&98u32.to_le_bytes()); // BC7
        let m = DdsAssetMetadata::from_bytes(&buf, FAKE_SHA.to_string()).unwrap();
        assert_eq!(m.format, DdsFormat::Bc7Unorm);
        assert!(m.requires_pathc, "BC7 must surface PATHC requirement");
    }

    #[test]
    fn renders_v3_1_asset_entry_with_inferred_vpath() {
        let buf = make_dds(b"DXT1", 64, 64);
        let mut m = DdsAssetMetadata::from_bytes(&buf, FAKE_SHA.to_string()).unwrap();
        m.vpath_hint = Some("0009/character/texture/foo.dds".to_string());
        let entry = m.to_v3_1_asset_entry("assets/0009/character/texture/foo.dds");
        assert_eq!(
            entry["target"]["file"],
            "0009/character/texture/foo.dds"
        );
        assert_eq!(entry["type"], "asset");
        assert_eq!(entry["source"], "assets/0009/character/texture/foo.dds");
        assert_eq!(entry["sha256"], FAKE_SHA);
    }

    #[test]
    fn renders_v3_1_entry_falls_back_to_source_when_vpath_missing() {
        let buf = make_dds(b"DXT1", 64, 64);
        let m = DdsAssetMetadata::from_bytes(&buf, FAKE_SHA.to_string()).unwrap();
        let entry = m.to_v3_1_asset_entry("assets/foo.dds");
        // No vpath_hint → target.file falls back to source path
        assert_eq!(entry["target"]["file"], "assets/foo.dds");
    }

    #[test]
    fn from_path_reads_file_and_records_size() {
        // Round-trip via tempfile to exercise from_path.
        let tmp = std::env::temp_dir().join("dmm_parser_dds_test_d5.dds");
        let buf = make_dds(b"DXT5", 32, 32);
        std::fs::write(&tmp, &buf).unwrap();
        let m = DdsAssetMetadata::from_path(&tmp, None, FAKE_SHA.to_string()).unwrap();
        assert_eq!(m.format, DdsFormat::Dxt5);
        assert_eq!(m.size as usize, buf.len());
        assert_eq!(m.sha256, FAKE_SHA);
        std::fs::remove_file(&tmp).ok();
    }
}
