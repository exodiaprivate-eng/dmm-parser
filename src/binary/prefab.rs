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
//! ## Scope of this module
//!
//! **Phase 1 — same-length string set.** A same-length overwrite changes
//! ZERO structure (no length prefixes, offsets, preamble file-size field,
//! or the header id move), so it is safe regardless of the un-decoded
//! object-graph framing (e.g. Master Sword Patches: socket-name swaps
//! Pelvis_*→Spine2_*). The header 8-byte id is NOT a content checksum
//! (verified: two different-content prefabs share half of it; the format is
//! fully self-describing so the engine reads the embedded schema rather than
//! validating bytes), so a value edit leaves it valid.
//!
//! **Phase 2 — differing-length string set** (`apply_differing_length`).
//! A length change shifts every byte after the value. Because the format is
//! schema-driven (the engine walks fields by their embedded types), the only
//! position-dependent fields are the whole-file `FILE_LEN` (a u32 == the
//! buffer length) and child-object pointers stored as absolute byte offsets.
//! Phase 2 splices the new value, rewrites `FILE_LEN`, and shifts every
//! DATA-section offset that points past the edit — then RE-VALIDATES (re-
//! parse to the same schema + identical string set with only the edited
//! value swapped + coherent `FILE_LEN`). Any structural doubt → the edit is
//! `Skipped` and the bytes are left untouched, so a mount can never emit a
//! corrupt prefab. The scan is DATA-section-only: the schema's 8-byte field
//! descriptors can hold offset-range values that are NOT offsets.
//!
//! Field resolution is content-based (robust against the data section
//! ordering differing from schema order): socket names, `.pac`,
//! `.sockets.xml`, and the `CD_*` component name each have a distinct value
//! shape, so the right value is found by pattern, not position.

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

    let mut out = body.to_vec();
    let mut outcomes = Vec::with_capacity(intents.len());

    for intent in intents {
        // Re-tokenize the CURRENT body each iteration: a prior differing-
        // length edit shifts every offset after it, so cached tokens would
        // be stale. The schema is never edited, so `data_start` is stable.
        let toks = data_strings(&out, data_start);
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
        if *cur == intent.new {
            outcomes.push(PrefabOutcome::NoOp);
            continue;
        }
        if intent.new.len() == len {
            // Phase 1: same-length, byte-faithful in-place overwrite.
            out[off..off + len].copy_from_slice(intent.new.as_bytes());
            outcomes.push(PrefabOutcome::Applied);
        } else {
            // Phase 2: differing-length. Splice the value, fix the FILE_LEN
            // field + every data-section offset pointing past the edit, then
            // re-validate. On ANY structural doubt, `apply_differing_length`
            // returns None and we Skip — a mount NEVER emits a corrupt prefab.
            match apply_differing_length(&out, data_start, off - 4, len, &intent.new) {
                Some(nb) => {
                    out = nb;
                    outcomes.push(PrefabOutcome::Applied);
                }
                None => outcomes.push(PrefabOutcome::Skipped(format!(
                    "field '{}': differing-length edit ({}->{} bytes) failed structural \
                     re-validation (left untouched)",
                    intent.field,
                    len,
                    intent.new.len()
                ))),
            }
        }
    }

    Ok((out, outcomes))
}

/// Rebuild the prefab body with a differing-length string value written at
/// `sp` (the `[u32 len]` prefix position; the current value's bytes are at
/// `sp + 4`, length `old_len`).
///
/// A length change shifts every byte after the value, so any *absolute
/// offset* recorded in the data object-graph that points past the edit must
/// be decremented/incremented by the same delta, and the whole-file
/// `FILE_LEN` field rewritten. This format is schema-driven (the engine
/// walks fields by their embedded types), so the only size/offset fields are
/// (a) the single `FILE_LEN` (a u32 == the buffer length) and (b) child-
/// object pointers stored as absolute byte offsets. Both are found by
/// scanning the DATA section (never the schema, whose 8-byte field
/// descriptors can hold offset-range values that are NOT offsets) for u32s
/// that are either `== old_len_total` or a valid in-file offset pointing
/// past the edited value.
///
/// Returns `None` (fail-safe → caller Skips) if the spliced result does not
/// re-parse to the same schema with the same string set (only the edited
/// value swapped) and a coherent `FILE_LEN` — so a wrong fixup can never
/// ship a corrupt prefab.
fn apply_differing_length(
    body: &[u8],
    data_start: usize,
    sp: usize,
    old_len: usize,
    new_val: &str,
) -> Option<Vec<u8>> {
    let n = body.len();
    let new_len = new_val.len();
    let delta: isize = new_len as isize - old_len as isize;
    let edit_end = sp + 4 + old_len;

    // String content spans — excluded from the offset scan so ASCII payload
    // bytes are never mistaken for offsets.
    let toks = data_strings(body, data_start);
    let str_ranges: Vec<(usize, usize)> = toks.iter().map(|&(o, l, _)| (o, o + l)).collect();
    let in_str = |p: usize| str_ranges.iter().any(|&(a, b)| a <= p && p < b);

    // Collect fixups on the ORIGINAL body, DATA SECTION ONLY (`>= data_start`).
    let mut fixups: Vec<(usize, u32)> = Vec::new(); // (orig_pos, new_value)
    let mut i = data_start;
    while i + 4 <= n {
        if in_str(i) {
            i += 1;
            continue;
        }
        let v = u32(body, i)? as usize;
        if v == n {
            // FILE_LEN (or any offset pointing at EOF) → new total length.
            fixups.push((i, (n as isize + delta) as u32));
            i += 4;
        } else if (data_start..=n).contains(&v) && v > edit_end {
            // Absolute offset into the tail (past the edit) → shift by delta.
            fixups.push((i, (v as isize + delta) as u32));
            i += 4;
        } else {
            i += 1;
        }
    }

    // Splice: prefix..sp | new [u32 len] | new bytes | tail (edit_end..).
    let mut nb: Vec<u8> = Vec::with_capacity((n as isize + delta) as usize);
    nb.extend_from_slice(&body[..sp]);
    nb.extend_from_slice(&(new_len as u32).to_le_bytes());
    nb.extend_from_slice(new_val.as_bytes());
    nb.extend_from_slice(&body[edit_end..]);

    // Write each fixup at its POST-splice position (fields after the edit
    // moved by `delta`; the value itself was already computed with `delta`).
    for (p, nv) in fixups {
        let np = if p < sp { p } else { (p as isize + delta) as usize };
        nb.get_mut(np..np + 4)?.copy_from_slice(&nv.to_le_bytes());
    }

    // ── Fail-safe re-validation ──
    if nb.len() as isize != n as isize + delta {
        return None;
    }
    let (ds2, _) = parse_schema(&nb)?;
    if ds2 != data_start {
        return None; // schema region must be byte-identical
    }
    // The re-tokenized string set must equal the original with ONLY the
    // edited value swapped, in the SAME order — proves the tail didn't
    // desync (a wrong offset/length shifts a later string's prefix).
    let want: Vec<&str> = toks
        .iter()
        .map(|&(o, _, ref v)| if o == sp + 4 { new_val } else { v.as_str() })
        .collect();
    let got: Vec<String> = data_strings(&nb, ds2).into_iter().map(|(_, _, v)| v).collect();
    if got.len() != want.len() || got.iter().zip(&want).any(|(g, w)| g != w) {
        return None;
    }
    // A FILE_LEN field equal to the new total must be present.
    let new_n = nb.len() as u32;
    if !(0..nb.len().saturating_sub(3)).any(|p| u32(&nb, p) == Some(new_n)) {
        return None;
    }
    Some(nb)
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

    const TOWER: &[u8] = include_bytes!("testdata/tower_shield.prefab"); // cd_phm_03_towershield_0098

    #[test]
    fn differing_length_applies_or_skips_never_corrupts() {
        // Phase 2: a longer socket name (Pelvis_L_Socket 15 -> 20). Must
        // EITHER apply and re-parse cleanly OR skip leaving bytes untouched —
        // never a corrupt in-between.
        let intents = vec![PrefabIntent {
            field: "_attachedSocketName".into(),
            new: "Spine2_R_Root_Socket".into(),
        }];
        let (out, oc) = apply_prefab_intents(SAMPLE, &intents).unwrap();
        match &oc[0] {
            PrefabOutcome::Applied => {
                assert_eq!(out.len(), SAMPLE.len() + 5, "grew by 5 (15 -> 20)");
                let (ds, _) = parse_schema(&out).expect("re-parses after differing-length edit");
                let vals: Vec<String> = data_strings(&out, ds).into_iter().map(|t| t.2).collect();
                assert!(vals.contains(&"Spine2_R_Root_Socket".to_string()));
                assert!(!vals.contains(&"Pelvis_L_Socket".to_string()));
            }
            PrefabOutcome::Skipped(_) => assert_eq!(out, SAMPLE, "skip leaves bytes untouched"),
            PrefabOutcome::NoOp => panic!("unexpected no-op"),
        }
    }

    #[test]
    fn tower_shield_part_rebind_shrinks_and_revalidates() {
        // The real use case: rebind the tower shield's render part from
        // CD_MainWeapon_TowerShield_L (27) -> CD_MainWeapon_Shield_L (22) so
        // the small-shield combat stance sustains it — keeping the tower mesh.
        let intents = vec![PrefabIntent {
            field: "_components.item[0].SkinnedMeshComponent.Parameter.name".into(),
            new: "CD_MainWeapon_Shield_L".into(),
        }];
        let (out, oc) = apply_prefab_intents(TOWER, &intents).unwrap();
        assert_eq!(oc[0], PrefabOutcome::Applied);
        assert_eq!(out.len(), TOWER.len() - 5, "shrank by 5 (27 -> 22)");
        let (ds, _) = parse_schema(&out).expect("re-parses");
        let vals: Vec<String> = data_strings(&out, ds).into_iter().map(|t| t.2).collect();
        // part rebound, no TowerShield left, tower MESH preserved
        assert!(vals.contains(&"CD_MainWeapon_Shield_L".to_string()));
        assert!(!vals.iter().any(|v| v.contains("TowerShield")));
        assert!(vals.iter().any(|v| v.ends_with("cd_phm_03_towershield_0098.pac")));
        // FILE_LEN field now equals the new total length
        let new_n = out.len() as u32;
        assert!((0..out.len() - 3).any(|p| u32(&out, p) == Some(new_n)), "FILE_LEN updated");
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
