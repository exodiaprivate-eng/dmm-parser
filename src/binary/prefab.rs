// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.prefab` — PA Reflect serialized SceneObject (binary).
//!
//! Layout (RE'd 2026-06-13 against 11 vanilla weapon prefabs + IDA
//! `pa::SimpleReflectProperty` confirmation):
//!
//! ```text
//! [HEADER 20B]  ff ff | u32 | u64 type/asset-id | u32 | u16
//! [SCHEMA]      type table — each type:
//!                 [u32 namelen][typename][u16 fieldcount]
//!                 fieldcount × field:
//!                   [u32 namelen][fieldname][u32 typelen][typename][8B descriptor]
//! [DATA]        a serialized instance of the root type (SceneObject):
//!                 a preamble + field values; STRING values are
//!                 [u32 len][bytes] (IndexedStringA / staticstringA /
//!                 NormalizedPathA). Non-string values interspersed.
//! ```
//!
//! ## Scope of this module (Phase 1 — same-length string set)
//!
//! Full reflect-VM data traversal (with the object-graph preamble's
//! pointers/offsets) is not decoded. Instead this module does
//! **byte-faithful, same-length string replacement** keyed by field
//! name, which is exactly what socket-repositioning prefab mods need
//! (e.g. Master Sword Patches: every edit is a same-length socket-name
//! swap, Pelvis_*→Spine2_*). A same-length overwrite changes ZERO
//! structure — no length prefixes, offsets, preamble file-size field, or
//! the header id move — so it is safe regardless of the un-decoded
//! object-graph framing.
//!
//! The header 8-byte id is NOT a content checksum (verified: two
//! different-content prefabs share half of it; the format is fully
//! self-describing so the engine reads the embedded schema rather than
//! validating bytes) and is independent of the edited fields, so a
//! same-length value edit leaves it valid.
//!
//! Differing-length edits are reported `Skipped` (would require decoding
//! the data object-graph + rewriting size/offset fields — a future
//! phase). Resolution is content-based (robust against the data section
//! ordering differing from schema order): socket names, `.pac`,
//! `.sockets.xml`, and the `CD_*` component name each have a distinct
//! value shape, so the right value is found by pattern, not position.

/// One prefab edit (op = "set"). `new` is the replacement string value.
#[derive(Debug, Clone)]
pub struct PrefabIntent {
    /// V3 field path, e.g. `_attachedSocketName` or
    /// `_components.item[0].SkinnedMeshComponent._skinnedMeshFileName`.
    pub field: String,
    pub new: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefabOutcome {
    /// Value changed (same-length overwrite).
    Applied,
    /// Field resolved and current value already equals `new` — no write.
    NoOp,
    /// Not applied; reason for the mount log.
    Skipped(String),
}

fn u16(b: &[u8], o: usize) -> Option<u16> {
    b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}
fn u32(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}
fn ascii_at(b: &[u8], o: usize, len: usize) -> bool {
    b.get(o..o + len)
        .map(|s| s.iter().all(|&c| (0x20..0x7f).contains(&c)))
        .unwrap_or(false)
}

/// Parse the schema/type-table and return the byte offset where the data
/// section begins, plus the ordered type names. Returns `None` if the
/// bytes don't look like a PA Reflect prefab.
///
/// Header is 20 bytes; each type is `[u32 namelen][name][u16 fieldcount]`
/// followed by `fieldcount` fields of
/// `[u32 namelen][name][u32 typelen][type][8B descriptor]`.
fn parse_schema(b: &[u8]) -> Option<(usize, Vec<String>)> {
    let n = b.len();
    if n < 24 || b[0] != 0xFF || b[1] != 0xFF {
        return None;
    }
    let mut p = 20usize;
    let mut types: Vec<String> = Vec::new();
    while p + 6 <= n {
        let nl = u32(b, p)? as usize;
        if !(1..=64).contains(&nl) || p + 4 + nl > n || !ascii_at(b, p + 4, nl) {
            break;
        }
        let name = String::from_utf8_lossy(&b[p + 4..p + 4 + nl]).into_owned();
        let mut q = p + 4 + nl;
        let fc = u16(b, q)? as usize;
        q += 2;
        if fc > 200 {
            break;
        }
        let mut ok = true;
        for _ in 0..fc {
            if q + 8 > n {
                ok = false;
                break;
            }
            let fnl = u32(b, q)? as usize;
            if !(1..=64).contains(&fnl) || !ascii_at(b, q + 4, fnl) {
                ok = false;
                break;
            }
            q += 4 + fnl;
            let ftl = u32(b, q)? as usize;
            if !(1..=64).contains(&ftl) || !ascii_at(b, q + 4, ftl) {
                ok = false;
                break;
            }
            q += 4 + ftl + 8; // type + 8-byte descriptor
        }
        if !ok {
            break;
        }
        types.push(name);
        p = q;
    }
    if types.is_empty() {
        None
    } else {
        Some((p, types))
    }
}

/// Tokenize the data section into `(value_bytes_offset, len, value)` for
/// every `[u32 len][bytes]` printable string. `value_bytes_offset` points
/// at the string bytes (after the 4-byte length prefix).
fn data_strings(b: &[u8], data_start: usize) -> Vec<(usize, usize, String)> {
    let n = b.len();
    let mut out = Vec::new();
    let mut i = data_start;
    while i + 4 <= n {
        let l = u32(b, i).unwrap() as usize;
        if (2..=130).contains(&l) && i + 4 + l <= n && ascii_at(b, i + 4, l) {
            out.push((i + 4, l, String::from_utf8_lossy(&b[i + 4..i + 4 + l]).into_owned()));
            i += 4 + l;
        } else {
            i += 1;
        }
    }
    out
}

/// Content-based field → candidate data-string resolver. Returns indices
/// into `toks` that match the field's value shape.
fn candidates(field: &str, toks: &[(usize, usize, String)]) -> Vec<usize> {
    let leaf = field.rsplit('.').next().unwrap_or(field);
    let pred: Box<dyn Fn(&str) -> bool> = if field == "_attachedSocketName" {
        Box::new(|v: &str| v.ends_with("_Socket") && !v.ends_with("_ChildSocket"))
    } else if field == "_pivotSocketName" {
        Box::new(|v: &str| v.ends_with("_ChildSocket"))
    } else if leaf == "_skinnedMeshFileName" {
        Box::new(|v: &str| v.ends_with(".pac"))
    } else if leaf == "_socketFileName" {
        Box::new(|v: &str| v.ends_with(".sockets.xml"))
    } else if leaf == "name" {
        Box::new(|v: &str| v.starts_with("CD_"))
    } else {
        return Vec::new();
    };
    toks.iter()
        .enumerate()
        .filter(|(_, (_, _, v))| pred(v))
        .map(|(i, _)| i)
        .collect()
}

/// Apply prefab intents (op=set, string values) via same-length, content-
/// matched, byte-faithful overwrite. Returns the new bytes (same length)
/// and a per-intent outcome aligned to `intents`.
///
/// Returns `Err` only if the bytes don't parse as a SceneObject prefab —
/// the caller then surfaces "unsupported prefab shape" and leaves the
/// file untouched.
pub fn apply_prefab_intents(
    body: &[u8],
    intents: &[PrefabIntent],
) -> Result<(Vec<u8>, Vec<PrefabOutcome>), String> {
    let (data_start, types) =
        parse_schema(body).ok_or_else(|| "not a recognized PA Reflect prefab".to_string())?;
    // Gate: only the SceneObject prefab shape is supported.
    if types.first().map(|s| s.as_str()) != Some("SceneObject") {
        return Err(format!(
            "unsupported prefab root type {:?} (expected SceneObject)",
            types.first()
        ));
    }

    let toks = data_strings(body, data_start);
    let mut out = body.to_vec();
    let mut outcomes = Vec::with_capacity(intents.len());

    for intent in intents {
        let cands = candidates(&intent.field, &toks);
        if cands.len() != 1 {
            outcomes.push(PrefabOutcome::Skipped(format!(
                "field '{}' resolved {} candidates (need exactly 1)",
                intent.field,
                cands.len()
            )));
            continue;
        }
        let (off, len, ref cur) = toks[cands[0]];
        if intent.new.len() != len {
            outcomes.push(PrefabOutcome::Skipped(format!(
                "field '{}': differing-length edit ({}->{} bytes) not yet supported",
                intent.field,
                len,
                intent.new.len()
            )));
            continue;
        }
        if *cur == intent.new {
            outcomes.push(PrefabOutcome::NoOp);
            continue;
        }
        out[off..off + len].copy_from_slice(intent.new.as_bytes());
        outcomes.push(PrefabOutcome::Applied);
    }

    // Same-length only: output length must equal input length.
    debug_assert_eq!(out.len(), body.len());
    Ok((out, outcomes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[u8] = include_bytes!("testdata/sample.prefab"); // cd_phm_01_sword_0023_r
    const SAMPLE2: &[u8] = include_bytes!("testdata/sample2.prefab"); // cd_phm_01_swordbelt_0001

    #[test]
    fn schema_parses_sceneobject() {
        let (ds, types) = parse_schema(SAMPLE).expect("parse");
        assert_eq!(types[0], "SceneObject");
        assert_eq!(
            types,
            vec![
                "SceneObject",
                "SkinnedMeshComponent",
                "ResourceReferencePath_CharacterSkeleton",
                "ResourceReferencePath_SkinnedMesh"
            ]
        );
        assert!(ds > 1000 && ds < SAMPLE.len());
    }

    #[test]
    fn data_strings_are_the_five_values() {
        let (ds, _) = parse_schema(SAMPLE).unwrap();
        let toks = data_strings(SAMPLE, ds);
        let vals: Vec<&str> = toks.iter().map(|t| t.2.as_str()).collect();
        assert!(vals.contains(&"Pelvis_L_Socket"));
        assert!(vals.contains(&"Pelvis_L_ChildSocket"));
        assert!(vals.contains(&"CD_MainWeapon_Sword_R"));
        assert!(vals.iter().any(|v| v.ends_with("cd_phm_01_sword_0023.pac")));
        assert!(vals.iter().any(|v| v.ends_with("cd_phm_01_sword_0001_r.sockets.xml")));
    }

    #[test]
    fn same_length_socket_swap_applies_and_preserves_length() {
        let intents = vec![
            PrefabIntent { field: "_attachedSocketName".into(), new: "Spine2_R_Socket".into() },
            PrefabIntent { field: "_pivotSocketName".into(), new: "Spine2_R_ChildSocket".into() },
        ];
        let (out, oc) = apply_prefab_intents(SAMPLE, &intents).unwrap();
        assert_eq!(oc[0], PrefabOutcome::Applied);
        assert_eq!(oc[1], PrefabOutcome::Applied);
        assert_eq!(out.len(), SAMPLE.len(), "same-length: byte count preserved");
        // schema must be byte-identical (only data values changed)
        let (ds, _) = parse_schema(SAMPLE).unwrap();
        assert_eq!(&out[..ds], &SAMPLE[..ds], "schema region untouched");
        // new values present, old gone
        let toks = data_strings(&out, ds);
        let vals: Vec<&str> = toks.iter().map(|t| t.2.as_str()).collect();
        assert!(vals.contains(&"Spine2_R_Socket"));
        assert!(vals.contains(&"Spine2_R_ChildSocket"));
        assert!(!vals.contains(&"Pelvis_L_Socket"));
    }

    #[test]
    fn noop_when_value_matches() {
        let intents = vec![PrefabIntent {
            field: "_components.item[0].SkinnedMeshComponent.Parameter.name".into(),
            new: "CD_MainWeapon_Sword_R".into(),
        }];
        let (out, oc) = apply_prefab_intents(SAMPLE, &intents).unwrap();
        assert_eq!(oc[0], PrefabOutcome::NoOp);
        assert_eq!(out, SAMPLE, "no-op: byte-identical");
    }

    #[test]
    fn differing_length_is_skipped_not_corrupting() {
        let intents = vec![PrefabIntent {
            field: "_attachedSocketName".into(),
            new: "A_Much_Longer_Socket_Name".into(),
        }];
        let (out, oc) = apply_prefab_intents(SAMPLE, &intents).unwrap();
        assert!(matches!(oc[0], PrefabOutcome::Skipped(_)));
        assert_eq!(out, SAMPLE, "skipped edit leaves bytes untouched");
    }

    #[test]
    fn content_matched_paths_resolve_on_swordbelt() {
        // swordbelt ships a distinct .pac (cd_phm_01_swordbelt_0001.pac) —
        // content matching must still pick it for _skinnedMeshFileName.
        let (ds, types) = parse_schema(SAMPLE2).unwrap();
        assert_eq!(types[0], "SceneObject");
        let toks = data_strings(SAMPLE2, ds);
        let c = candidates("_components.item[0].SkinnedMeshComponent._skinnedMeshFileName", &toks);
        assert_eq!(c.len(), 1);
        assert!(toks[c[0]].2.ends_with("cd_phm_01_swordbelt_0001.pac"));
    }

    #[test]
    fn non_prefab_bytes_error() {
        assert!(apply_prefab_intents(b"not a prefab at all........", &[]).is_err());
    }
}
