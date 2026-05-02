// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! DDS vpath helpers for the Crimson Desert mod-author framework.
//!
//! Two related but distinct operations modders need:
//!
//! 1. **Disk path → game vpath**: given a file at `<root>/0009/character/
//!    texture/foo/bar.dds`, infer the game vpath `0009/character/texture/
//!    foo/bar.dds`. Used by SWISS Stacker when collecting a mod's assets
//!    folder for v3.1 packaging.
//!
//! 2. **vpath → "last4" classifier**: given a vpath like
//!    `/character/texture/foo/bar_n.dds`, return the Crimson-specific
//!    "last4" identifier (dwReserved2 byte at DDS offset 124) the game
//!    expects when the PATHC vanilla lookup misses. Used by validators
//!    to compute the expected last4 without round-tripping through the
//!    full DDS header.
//!
//! These mirror DMM's `classify_overlay_last4` and folder-walk logic but
//! live in dmm-parser so SWISS can call them via Python bindings.
//!
//! See `references/dds_notes.md` §3 (last4 lookup) and §5 (path-prefix
//! classifier) for the authoritative reference.

use std::path::Path;

/// Crimson-specific "last4" classification of a virtual game path. Used
/// as tier 2 of the three-tier resolution: tier 1 reads the value from
/// vanilla PATHC, tier 2 (this function) infers from path prefix, tier 3
/// falls back to the DDS format-derived value.
///
/// Mirrors DMM's `classify_overlay_last4`. Path-prefix table:
///
/// | Path pattern | last4 |
/// |---|---|
/// | `/ui/*` | `0x00001580` |
/// | `/character/texture/*_n.dds` | `0x00000480` (normal map) |
/// | `/character/texture/*tattoo*` | `0x00001380` (tattoo / decal) |
/// | `/character/texture/*` (default) | `0x00001280` (generic character texture) |
/// | (other) | `None` (caller should use format-derived last4) |
///
/// Path comparisons are case-insensitive. Leading slash optional.
pub fn classify_vpath_last4(vpath: &str) -> Option<u32> {
    let lower = vpath.to_ascii_lowercase();
    // Strip leading slash if present so callers can pass "/ui/..." or "ui/..."
    let p = lower.strip_prefix('/').unwrap_or(&lower);

    if p.starts_with("ui/") || p == "ui" {
        Some(0x0000_1580)
    } else if p.starts_with("character/texture/") {
        // Suffix-based: normal maps use _n.dds
        if p.ends_with("_n.dds") {
            Some(0x0000_0480)
        } else if p.contains("tattoo") {
            Some(0x0000_1380)
        } else {
            Some(0x0000_1280)
        }
    } else {
        None
    }
}

/// Infer the game vpath for a DDS (or other asset) on disk relative to
/// a mod's asset root directory.
///
/// Convention: assets are arranged under a 4-digit PAZ group prefix
/// (`0009/`, `0012/`, etc.) inside the mod's asset folder. The vpath
/// relative to the asset root is what gets registered in the v3.1 mod's
/// asset target entry.
///
/// Returns `None` if:
/// - `file_path` is not under `asset_root`
/// - The first path segment after `asset_root` is not a 4-digit group
///   prefix (in which case the file is malformed for v3.1 bundling)
///
/// The returned string uses forward slashes regardless of host OS.
pub fn infer_vpath_from_disk_path(asset_root: &Path, file_path: &Path) -> Option<String> {
    let rel = match (asset_root.canonicalize(), file_path.canonicalize()) {
        (Ok(root), Ok(file)) => file.strip_prefix(&root).ok().map(|p| p.to_path_buf()),
        _ => {
            // Fall back to non-canonicalized strip_prefix when canonicalize
            // fails (e.g., paths don't exist yet — useful for tests).
            file_path.strip_prefix(asset_root).ok().map(|p| p.to_path_buf())
        }
    }?;

    let s = path_to_forward_slashes(&rel);
    let mut parts = s.split('/');
    let first = parts.next()?;
    if first.len() != 4 || !first.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(s)
}

fn path_to_forward_slashes(p: &Path) -> String {
    p.components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── classify_vpath_last4 ────────────────────────────────────────────

    #[test]
    fn ui_path_classifies_to_1580() {
        assert_eq!(classify_vpath_last4("/ui/icon/sword.dds"), Some(0x1580));
        assert_eq!(classify_vpath_last4("ui/banner/main.dds"), Some(0x1580));
    }

    #[test]
    fn character_normal_map_uses_0480() {
        assert_eq!(
            classify_vpath_last4("/character/texture/macduff/diffuse_n.dds"),
            Some(0x0480),
        );
    }

    #[test]
    fn character_tattoo_uses_1380() {
        assert_eq!(
            classify_vpath_last4("/character/texture/elite/tattoo_dragon.dds"),
            Some(0x1380),
        );
    }

    #[test]
    fn character_default_uses_1280() {
        assert_eq!(
            classify_vpath_last4("/character/texture/macduff/diffuse.dds"),
            Some(0x1280),
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(classify_vpath_last4("/UI/Icon/Sword.DDS"), Some(0x1580));
        assert_eq!(
            classify_vpath_last4("/Character/Texture/foo/Bar_N.DDS"),
            Some(0x0480),
        );
    }

    #[test]
    fn unknown_paths_return_none() {
        assert_eq!(classify_vpath_last4("/level/world/0001.dds"), None);
        assert_eq!(classify_vpath_last4("random/path.dds"), None);
        assert_eq!(classify_vpath_last4(""), None);
    }

    // ── infer_vpath_from_disk_path ──────────────────────────────────────

    #[test]
    fn infers_simple_vpath() {
        let root = PathBuf::from("/tmp/mod/assets");
        let file = PathBuf::from("/tmp/mod/assets/0009/character/macduff/diffuse.dds");
        let v = infer_vpath_from_disk_path(&root, &file);
        assert_eq!(v, Some("0009/character/macduff/diffuse.dds".to_string()));
    }

    #[test]
    fn rejects_no_4_digit_prefix() {
        let root = PathBuf::from("/tmp/mod/assets");
        let file = PathBuf::from("/tmp/mod/assets/no_prefix/foo.dds");
        assert_eq!(infer_vpath_from_disk_path(&root, &file), None);
    }

    #[test]
    fn rejects_3_digit_prefix() {
        let root = PathBuf::from("/tmp/mod/assets");
        let file = PathBuf::from("/tmp/mod/assets/009/character/foo.dds");
        assert_eq!(infer_vpath_from_disk_path(&root, &file), None);
    }

    #[test]
    fn rejects_alpha_prefix() {
        let root = PathBuf::from("/tmp/mod/assets");
        let file = PathBuf::from("/tmp/mod/assets/abcd/foo.dds");
        assert_eq!(infer_vpath_from_disk_path(&root, &file), None);
    }

    #[test]
    fn rejects_file_outside_root() {
        let root = PathBuf::from("/tmp/mod/assets");
        let file = PathBuf::from("/tmp/elsewhere/0009/foo.dds");
        assert_eq!(infer_vpath_from_disk_path(&root, &file), None);
    }
}
