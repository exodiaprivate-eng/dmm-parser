// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Surgical byte-span patcher for PA-engine UTF-8 XML resources
//! (`.sockets.xml`, `character/descriptors/.../*.xml`, etc.).
//!
//! WHY byte-span splicing instead of a DOM round-trip: these descriptors
//! are parsed *strictly* by the game engine, and PA's "XML" is not
//! standard XML — `.sockets.xml` mixes `<Socket .../>` self-closing
//! children with `<StackEquipInfo>...</StackEquipInfo>` blocks, and
//! character descriptions use anonymous close tags (`</>`) plus
//! interleaved `<!-- … -->` comments and *multiple top-level elements*
//! (no single document root). A DOM library would reformat the whole
//! file (attribute quoting, whitespace, self-closing style) — drift the
//! engine may reject. This patcher only ever rewrites the exact byte
//! span being changed; every other byte stays identical to vanilla.
//!
//! Supported V3 intent ops:
//!   - `set`: replace (or add) an attribute on the element addressed by
//!     `entry` + the field path. Leaf path segment = attribute name.
//!     e.g. `entry="SocketBoneData" field="SocketList.Count"`.
//!   - `add`: insert a new self-closing child element under the element
//!     addressed by `entry` + the field path minus its last segment.
//!     The last segment is `Element[index]`; attributes come from the
//!     `new` JSON object. e.g. `field="SocketList.Socket[52]"`.
//!
//! Path grammar: dot-separated segments, each `Name` or `Name[index]`.
//! A bare `Name` resolves to the first (index 0) direct child of that
//! name; `Name[k]` resolves to the k-th (0-based) direct child of that
//! name. `entry` is the scope element searched among the file's
//! top-level elements. Child counting skips comments — so
//! `PartInOutSocket[1]` is the 2nd `<PartInOutSocket>` element,
//! regardless of comments between them.

use serde_json::Value;

/// One XML edit request, mirroring the DMM V3 intent shape but
/// decoupled from DMM's `Intent` struct.
#[derive(Debug, Clone)]
pub struct XmlIntent {
    /// Scope element name (a top-level element of the file).
    pub entry: String,
    /// Dot/bracket field path relative to `entry`.
    pub field: String,
    /// `"set"` or `"add"`.
    pub op: String,
    /// `set`: scalar value (string/number/bool). `add`: object of attrs.
    pub new: Value,
}

/// Per-intent result, aligned 1:1 with the input slice order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentOutcome {
    Applied,
    Skipped(String),
}

/// Apply one mod's worth of XML intents to `body`. Edits are computed
/// against the *single* input `body` and spliced in one pass, so callers
/// that want last-writer-wins across mods should thread the returned
/// bytes into the next call (apply mod A → feed result to mod B). Within
/// a single call, intents must not target overlapping byte spans (an
/// author editing the same attribute twice in one mod); that returns an
/// `Err` rather than silently corrupting.
///
/// Returns the new bytes plus a per-intent outcome vec (same length /
/// order as `intents`). A bad intent (unresolved element, out-of-range
/// index, malformed path) is recorded as `Skipped` and does not abort
/// the others.
pub fn apply_xml_intents(
    body: &[u8],
    intents: &[XmlIntent],
) -> Result<(Vec<u8>, Vec<IntentOutcome>), String> {
    // Preserve a BOM verbatim; offsets below are absolute into `body`.
    let text = std::str::from_utf8(body)
        .map_err(|e| format!("xml_patch: body not UTF-8: {}", e))?;
    let bytes = text.as_bytes();

    let mut outcomes: Vec<IntentOutcome> = Vec::with_capacity(intents.len());
    let mut edits: Vec<Edit> = Vec::new();

    for (order, intent) in intents.iter().enumerate() {
        match build_edit(bytes, intent, order) {
            Ok(edit) => {
                edits.push(edit);
                outcomes.push(IntentOutcome::Applied);
            }
            Err(reason) => outcomes.push(IntentOutcome::Skipped(reason)),
        }
    }

    let out = splice(bytes, &mut edits)?;
    Ok((out, outcomes))
}

// ── Edit model ───────────────────────────────────────────────────────

/// A pending replacement of `body[start..end]` with `replacement`.
/// Insertions have `start == end` (zero-length range). `order` is the
/// intent index, used to keep insertions at the same offset stable.
struct Edit {
    start: usize,
    end: usize,
    replacement: String,
    order: usize,
}

/// Apply all edits to `bytes` in one pass. Edits are sorted by
/// (start asc, order asc). Overlapping replacements are an error.
fn splice(bytes: &[u8], edits: &mut [Edit]) -> Result<Vec<u8>, String> {
    edits.sort_by(|a, b| a.start.cmp(&b.start).then(a.order.cmp(&b.order)));
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len() + 256);
    let mut cursor = 0usize;
    for e in edits.iter() {
        if e.start < cursor {
            return Err(format!(
                "xml_patch: overlapping edits at byte {} (cursor {}) — two intents target the same span",
                e.start, cursor
            ));
        }
        out.extend_from_slice(&bytes[cursor..e.start]);
        out.extend_from_slice(e.replacement.as_bytes());
        cursor = e.end;
    }
    out.extend_from_slice(&bytes[cursor..]);
    Ok(out)
}

// ── Intent → Edit ────────────────────────────────────────────────────

fn build_edit(bytes: &[u8], intent: &XmlIntent, order: usize) -> Result<Edit, String> {
    let segments = parse_path(&intent.field)?;
    if segments.is_empty() {
        return Err(format!("empty field path for entry '{}'", intent.entry));
    }

    // Locate the scope (`entry`) element among the top-level elements,
    // then navigate every segment except the last to reach the element
    // that the leaf operates on/under.
    let root_children = direct_children(bytes, 0, bytes.len())?;
    let scope = pick_child(&root_children, &intent.entry, 0).ok_or_else(|| {
        format!("scope element '{}' not found at top level", intent.entry)
    })?;

    // Navigate intermediate segments (all but the last) into `cur`.
    let mut cur = scope.clone();
    for seg in &segments[..segments.len() - 1] {
        let children = direct_children(bytes, cur.content_start, cur.content_end)?;
        let next = pick_child(&children, &seg.name, seg.index).ok_or_else(|| {
            format!(
                "path segment '{}[{}]' not found under '{}'",
                seg.name, seg.index, intent.entry
            )
        })?;
        cur = next.clone();
    }

    let leaf = &segments[segments.len() - 1];
    match intent.op.as_str() {
        "set" => {
            // Leaf is an attribute name on element `cur`.
            if leaf.has_index {
                return Err(format!(
                    "op=set leaf '{}' must be a plain attribute name (no [index])",
                    leaf.name
                ));
            }
            let value = json_scalar_to_attr_string(&intent.new)
                .ok_or_else(|| format!("op=set: 'new' for attribute '{}' is not a scalar", leaf.name))?;
            set_attr_edit(bytes, &cur, &leaf.name, &value, order)
        }
        "add" => {
            // Leaf is the element to insert under `cur` at index.
            let obj = intent.new.as_object().ok_or_else(|| {
                format!("op=add: 'new' for <{}> must be a JSON object of attributes", leaf.name)
            })?;
            let mut attrs: Vec<(String, String)> = Vec::with_capacity(obj.len());
            for (k, v) in obj {
                let s = json_scalar_to_attr_string(v).ok_or_else(|| {
                    format!("op=add: attribute '{}' value is not a scalar", k)
                })?;
                attrs.push((k.clone(), s));
            }
            add_child_edit(bytes, &cur, &leaf.name, leaf.index, &attrs, order)
        }
        other => Err(format!("unsupported xml op '{}'", other)),
    }
}

// ── set: attribute replace / insert ──────────────────────────────────

fn set_attr_edit(
    bytes: &[u8],
    elem: &Child,
    attr: &str,
    value: &str,
    order: usize,
) -> Result<Edit, String> {
    let escaped = escape_attr(value);
    // Search for the attribute within the element's open tag only.
    if let Some((val_start, val_end)) = find_attr_value_span(bytes, elem.open_start, elem.open_end, attr) {
        Ok(Edit { start: val_start, end: val_end, replacement: escaped, order })
    } else {
        // Attribute absent — insert ` attr="value"` just before the tag
        // terminator ('/>' for self-closing, else '>').
        let insert_at = if elem.self_closing {
            // open_end is one past '>'; '/' sits at open_end-2.
            elem.open_end.saturating_sub(2)
        } else {
            elem.open_end.saturating_sub(1)
        };
        // Drop a trailing space if the byte before insert is already
        // whitespace, to keep ` a="1"/>` tidy either way.
        let lead = if insert_at > 0 && bytes[insert_at - 1].is_ascii_whitespace() {
            ""
        } else {
            " "
        };
        Ok(Edit {
            start: insert_at,
            end: insert_at,
            replacement: format!("{}{}=\"{}\"", lead, attr, escaped),
            order,
        })
    }
}

/// Find the byte span of the *value* (between the quotes) of attribute
/// `attr` within `bytes[tag_start..tag_end]`. Matches `attr` only as a
/// whole token: preceded by whitespace (or the start), followed by
/// optional whitespace then `=`. Returns (value_start, value_end)
/// excluding the surrounding quotes.
fn find_attr_value_span(
    bytes: &[u8],
    tag_start: usize,
    tag_end: usize,
    attr: &str,
) -> Option<(usize, usize)> {
    let a = attr.as_bytes();
    let mut i = tag_start;
    while i + a.len() <= tag_end {
        // candidate match of the attr name
        if &bytes[i..i + a.len()] == a {
            let prev_ok = i == tag_start || bytes[i - 1].is_ascii_whitespace();
            // after the name: optional ws then '='
            let mut j = i + a.len();
            while j < tag_end && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if prev_ok && j < tag_end && bytes[j] == b'=' {
                j += 1;
                while j < tag_end && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < tag_end && bytes[j] == b'"' {
                    let val_start = j + 1;
                    let mut k = val_start;
                    while k < tag_end && bytes[k] != b'"' {
                        k += 1;
                    }
                    if k < tag_end {
                        return Some((val_start, k));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

// ── add: insert child element ────────────────────────────────────────

fn add_child_edit(
    bytes: &[u8],
    parent: &Child,
    name: &str,
    index: usize,
    attrs: &[(String, String)],
    order: usize,
) -> Result<Edit, String> {
    if parent.self_closing {
        return Err(format!(
            "op=add: parent of <{}> is self-closing (no content to insert into)",
            name
        ));
    }
    let mut elem = format!("<{}", name);
    for (k, v) in attrs {
        elem.push_str(&format!(" {}=\"{}\"", k, escape_attr(v)));
    }
    elem.push_str("/>");

    let children = direct_children(bytes, parent.content_start, parent.content_end)?;
    let same: Vec<&Child> = children.iter().filter(|c| c.name == name).collect();

    if same.is_empty() {
        // No sibling of this name to align to: insert right before the
        // parent's close tag, on its own line, indented one level deeper
        // than the parent's open tag.
        let base_indent = leading_indent(bytes, parent.open_start);
        let indent = format!("{}\t", base_indent);
        let insert_at = parent.content_end; // '<' of the close tag
        Ok(Edit {
            start: insert_at,
            end: insert_at,
            replacement: format!("{}{}\n{}", indent, elem, base_indent),
            order,
        })
    } else if index >= same.len() {
        // Append after the last existing sibling, matching its indent.
        let last = same[same.len() - 1];
        let indent = leading_indent(bytes, last.open_start);
        Ok(Edit {
            start: last.elem_end,
            end: last.elem_end,
            replacement: format!("\n{}{}", indent, elem),
            order,
        })
    } else {
        // Insert before the index-th existing sibling, taking its indent.
        let at = same[index];
        let indent = leading_indent(bytes, at.open_start);
        // Insert at the start of the sibling's indentation so the new
        // element occupies the line above it.
        let line_start = at.open_start - indent.len();
        Ok(Edit {
            start: line_start,
            end: line_start,
            replacement: format!("{}{}\n", indent, elem),
            order,
        })
    }
}

/// The run of space/tab bytes immediately preceding `pos`, back to the
/// previous newline (or start of file). Used to mirror sibling
/// indentation when inserting a new element.
fn leading_indent(bytes: &[u8], pos: usize) -> String {
    let mut start = pos;
    while start > 0 {
        let c = bytes[start - 1];
        if c == b' ' || c == b'\t' {
            start -= 1;
        } else {
            break;
        }
    }
    String::from_utf8_lossy(&bytes[start..pos]).into_owned()
}

// ── XML scanning ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Child {
    name: String,
    open_start: usize, // index of '<'
    open_end: usize,   // index just past the '>' that ends the open tag
    self_closing: bool,
    content_start: usize, // == open_end
    content_end: usize,   // index of '<' of the matching close (== open_end if self-closing)
    elem_end: usize,      // index just past the element (after close '>' or == open_end)
}

fn pick_child<'a>(children: &'a [Child], name: &str, index: usize) -> Option<&'a Child> {
    children.iter().filter(|c| c.name == name).nth(index)
}

/// Enumerate the *direct* child elements within `bytes[range_start..range_end]`.
/// Skips comments / prologs / doctype. Stops at the first close tag seen
/// at this level (that close belongs to the enclosing element).
fn direct_children(bytes: &[u8], range_start: usize, range_end: usize) -> Result<Vec<Child>, String> {
    let mut out = Vec::new();
    let mut i = range_start;
    while i < range_end {
        // advance to next '<'
        while i < range_end && bytes[i] != b'<' {
            i += 1;
        }
        if i >= range_end {
            break;
        }
        // comment / prolog / doctype / close
        if starts_with(bytes, i, b"<!--") {
            i = find_seq(bytes, i + 4, range_end, b"-->")
                .map(|p| p + 3)
                .unwrap_or(range_end);
            continue;
        }
        if i + 1 < range_end && bytes[i + 1] == b'?' {
            i = find_seq(bytes, i + 2, range_end, b"?>")
                .map(|p| p + 2)
                .unwrap_or(range_end);
            continue;
        }
        if i + 1 < range_end && bytes[i + 1] == b'!' {
            // <!DOCTYPE ...> or <![CDATA[...]]> — skip to next '>'
            i = find_byte(bytes, i + 1, range_end, b'>')
                .map(|p| p + 1)
                .unwrap_or(range_end);
            continue;
        }
        if i + 1 < range_end && bytes[i + 1] == b'/' {
            // close tag at this level → belongs to enclosing element
            break;
        }

        // open tag of a child element
        let open_start = i;
        let name_start = i + 1;
        let mut j = name_start;
        while j < range_end
            && !bytes[j].is_ascii_whitespace()
            && bytes[j] != b'>'
            && bytes[j] != b'/'
        {
            j += 1;
        }
        let name = String::from_utf8_lossy(&bytes[name_start..j]).into_owned();
        let (open_end, self_closing) = parse_open_tag_end(bytes, j, range_end)?;
        if self_closing {
            out.push(Child {
                name,
                open_start,
                open_end,
                self_closing: true,
                content_start: open_end,
                content_end: open_end,
                elem_end: open_end,
            });
            i = open_end;
        } else {
            let (content_end, elem_end) = find_matching_close(bytes, open_end, range_end)?;
            out.push(Child {
                name,
                open_start,
                open_end,
                self_closing: false,
                content_start: open_end,
                content_end,
                elem_end,
            });
            i = elem_end;
        }
    }
    Ok(out)
}

/// Given `k` positioned at/after the element name inside an open tag,
/// scan (quote-aware) to the '>' that ends the tag. Returns
/// (index past '>', is_self_closing).
fn parse_open_tag_end(bytes: &[u8], mut k: usize, end: usize) -> Result<(usize, bool), String> {
    let mut in_quote = false;
    let mut last_nonspace = 0u8;
    while k < end {
        let c = bytes[k];
        if in_quote {
            if c == b'"' {
                in_quote = false;
            }
        } else if c == b'"' {
            in_quote = true;
        } else if c == b'>' {
            let self_closing = last_nonspace == b'/';
            return Ok((k + 1, self_closing));
        }
        if !c.is_ascii_whitespace() {
            last_nonspace = c;
        }
        k += 1;
    }
    Err("xml_patch: unterminated open tag".to_string())
}

/// From `from` (just past an open tag's '>'), find the matching close
/// tag at depth 0. Returns (content_end = index of '<' of the close,
/// elem_end = index past the close tag's '>'). Depth-based and
/// name-agnostic, so anonymous `</>` closes work.
fn find_matching_close(bytes: &[u8], from: usize, end: usize) -> Result<(usize, usize), String> {
    let mut i = from;
    let mut depth = 0usize;
    while i < end {
        while i < end && bytes[i] != b'<' {
            i += 1;
        }
        if i >= end {
            break;
        }
        if starts_with(bytes, i, b"<!--") {
            i = find_seq(bytes, i + 4, end, b"-->").map(|p| p + 3).unwrap_or(end);
            continue;
        }
        if i + 1 < end && bytes[i + 1] == b'?' {
            i = find_seq(bytes, i + 2, end, b"?>").map(|p| p + 2).unwrap_or(end);
            continue;
        }
        if i + 1 < end && bytes[i + 1] == b'!' {
            i = find_byte(bytes, i + 1, end, b'>').map(|p| p + 1).unwrap_or(end);
            continue;
        }
        if i + 1 < end && bytes[i + 1] == b'/' {
            // close tag
            let close_start = i;
            let tag_end = find_byte(bytes, i, end, b'>')
                .map(|p| p + 1)
                .ok_or("xml_patch: unterminated close tag")?;
            if depth == 0 {
                return Ok((close_start, tag_end));
            }
            depth -= 1;
            i = tag_end;
            continue;
        }
        // nested open tag
        let name_start = i + 1;
        let mut j = name_start;
        while j < end && !bytes[j].is_ascii_whitespace() && bytes[j] != b'>' && bytes[j] != b'/' {
            j += 1;
        }
        let (open_end, self_closing) = parse_open_tag_end(bytes, j, end)?;
        if !self_closing {
            depth += 1;
        }
        i = open_end;
    }
    Err("xml_patch: unterminated element (no matching close tag)".to_string())
}

// ── small byte helpers ───────────────────────────────────────────────

fn starts_with(bytes: &[u8], at: usize, pat: &[u8]) -> bool {
    at + pat.len() <= bytes.len() && &bytes[at..at + pat.len()] == pat
}

fn find_seq(bytes: &[u8], from: usize, end: usize, pat: &[u8]) -> Option<usize> {
    if pat.is_empty() || from >= end {
        return None;
    }
    let mut i = from;
    while i + pat.len() <= end {
        if &bytes[i..i + pat.len()] == pat {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_byte(bytes: &[u8], from: usize, end: usize, b: u8) -> Option<usize> {
    (from..end).find(|&i| bytes[i] == b)
}

/// JSON scalar → attribute string. Strings pass through; numbers render
/// without quotes/JSON formatting; bools as `true`/`false`. Objects /
/// arrays / null are rejected (`None`).
fn json_scalar_to_attr_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Minimal XML attribute-value escaping. Inputs here are clean bone /
/// path / numeric identifiers, but escape defensively so a stray `&`,
/// `<`, or `"` can never break the tag.
fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn set(entry: &str, field: &str, new: Value) -> XmlIntent {
        XmlIntent { entry: entry.into(), field: field.into(), op: "set".into(), new }
    }
    fn add(entry: &str, field: &str, new: Value) -> XmlIntent {
        XmlIntent { entry: entry.into(), field: field.into(), op: "add".into(), new }
    }
    fn apply(body: &str, intents: &[XmlIntent]) -> (String, Vec<IntentOutcome>) {
        let (b, o) = apply_xml_intents(body.as_bytes(), intents).expect("apply");
        (String::from_utf8(b).unwrap(), o)
    }

    const SOCKETS: &str = "<SocketBoneData>\n\
        \t<SocketList Count=\"2\">\n\
        \t\t<Socket Name=\"A\" Parent=\"PA\" Rotation=\"0 0 0 1\" Translation=\"0 0 0\"/>\n\
        \t\t<Socket Name=\"B\" Parent=\"PB\" Rotation=\"0 0 0 1\" Translation=\"0 0 0\"/>\n\
        \t</SocketList>\n\
        \t<StackEquipInfo Count=\"1\" EquipTypeName=\"Back\">\n\
        \t\t<Socket Name=\"A\"/>\n\
        \t</StackEquipInfo>\n\
        </SocketBoneData>\n";

    #[test]
    fn set_attribute_replaces_value() {
        let (out, oc) = apply(SOCKETS, &[set("SocketBoneData", "SocketList.Count", json!(4))]);
        assert_eq!(oc[0], IntentOutcome::Applied);
        assert!(out.contains("<SocketList Count=\"4\">"));
        // The StackEquipInfo nested <Socket> count must be untouched.
        assert!(out.contains("<StackEquipInfo Count=\"1\""));
    }

    #[test]
    fn socket_index_scopes_to_socketlist_not_stackequip() {
        // SocketList has exactly 2 <Socket> children; appending at [2]
        // must land inside SocketList, before its </SocketList>.
        let (out, oc) = apply(
            SOCKETS,
            &[add(
                "SocketBoneData",
                "SocketList.Socket[2]",
                json!({"Name":"C","Parent":"PC","Rotation":"0 0 0 1","Translation":"0 0 0"}),
            )],
        );
        assert_eq!(oc[0], IntentOutcome::Applied);
        let sl = &out[out.find("<SocketList").unwrap()..out.find("</SocketList>").unwrap()];
        assert!(sl.contains("Name=\"C\""), "new socket inside SocketList: {}", sl);
        // exactly one new socket line, tab-indented like siblings
        assert!(out.contains("\t\t<Socket Name=\"C\" Parent=\"PC\" Rotation=\"0 0 0 1\" Translation=\"0 0 0\"/>"));
    }

    #[test]
    fn two_appends_in_order() {
        let (out, _) = apply(
            SOCKETS,
            &[
                set("SocketBoneData", "SocketList.Count", json!(4)),
                add("SocketBoneData", "SocketList.Socket[2]", json!({"Name":"C","Parent":"PC","Rotation":"r","Translation":"t"})),
                add("SocketBoneData", "SocketList.Socket[3]", json!({"Name":"D","Parent":"PD","Rotation":"r","Translation":"t"})),
            ],
        );
        let ca = out.find("Name=\"C\"").unwrap();
        let da = out.find("Name=\"D\"").unwrap();
        let close = out.find("</SocketList>").unwrap();
        assert!(ca < da && da < close, "C before D before close");
        assert!(out.contains("Count=\"4\""));
    }

    #[test]
    fn set_attr_inserts_when_absent() {
        let (out, oc) = apply(SOCKETS, &[set("SocketBoneData", "StackEquipInfo.Axis", json!("z"))]);
        assert_eq!(oc[0], IntentOutcome::Applied);
        assert!(out.contains("Axis=\"z\""));
        // existing attrs preserved
        assert!(out.contains("EquipTypeName=\"Back\""));
    }

    const DESC: &str = "<!-- desc -->\n\
        \n\
        <PoseModifierDesc>\n\
        \t<FootIK Type=\"Biped\"/>\n\
        </>\n\
        <PhysicsData StepUpDistance=\"0.5\"/>\n\
        <PartInOutSocketInfo>\n\
        \t<!-- hyperspace -->\n\
        \t<PartInOutSocket PartName=\"CD_HyperspacePlug\" Visible=\"Out\"/>\n\
        \t<!-- sword -->\n\
        \t<PartInOutSocket PartName=\"CD_MainWeapon_Sword_R\" InSocketBone=\"Pelvis_L_Socket\" InChildSocketBone=\"Pelvis_L_ChildSocket\"/>\n\
        \t<PartInOutSocket PartName=\"CD_MainWeapon_Sword_IN_R\" InSocketBone=\"Pelvis_L_Socket\" OutSocketBone=\"Pelvis_L_Socket\"/>\n\
        </PartInOutSocketInfo>\n";

    #[test]
    fn fragment_with_anonymous_close_and_comments() {
        // index 1 = 2nd <PartInOutSocket> (sword_R), skipping comments.
        let (out, oc) = apply(
            DESC,
            &[
                set("PartInOutSocketInfo", "PartInOutSocket[1].InSocketBone", json!("Spine2_R_Socket")),
                set("PartInOutSocketInfo", "PartInOutSocket[2].OutSocketBone", json!("Spine2_R_Socket")),
            ],
        );
        assert_eq!(oc[0], IntentOutcome::Applied);
        assert_eq!(oc[1], IntentOutcome::Applied);
        // The sword_R element got InSocketBone changed; hyperspace (index 0) untouched.
        assert!(out.contains("PartName=\"CD_MainWeapon_Sword_R\" InSocketBone=\"Spine2_R_Socket\""));
        assert!(out.contains("PartName=\"CD_HyperspacePlug\" Visible=\"Out\""));
        assert!(out.contains("PartName=\"CD_MainWeapon_Sword_IN_R\" InSocketBone=\"Pelvis_L_Socket\" OutSocketBone=\"Spine2_R_Socket\""));
    }

    #[test]
    fn unchanged_bytes_are_byte_identical() {
        // A no-op-ish single set must change only the targeted span.
        let (out, _) = apply(SOCKETS, &[set("SocketBoneData", "SocketList.Count", json!(2))]);
        assert_eq!(out, SOCKETS, "setting Count to its current value rewrites only the value, identical here");
    }

    #[test]
    fn missing_scope_is_skipped_not_fatal() {
        let (out, oc) = apply(SOCKETS, &[set("NoSuchRoot", "X.Y", json!("z"))]);
        assert!(matches!(oc[0], IntentOutcome::Skipped(_)));
        assert_eq!(out, SOCKETS);
    }

    #[test]
    fn oob_socket_index_for_set_is_skipped() {
        let (_out, oc) = apply(SOCKETS, &[set("SocketBoneData", "SocketList.Socket[9].Name", json!("Z"))]);
        assert!(matches!(oc[0], IntentOutcome::Skipped(_)));
    }

    #[test]
    fn set_nested_socket_attribute() {
        let (out, oc) = apply(SOCKETS, &[set("SocketBoneData", "SocketList.Socket[1].Parent", json!("NEWP"))]);
        assert_eq!(oc[0], IntentOutcome::Applied);
        assert!(out.contains("Name=\"B\" Parent=\"NEWP\""));
        assert!(out.contains("Name=\"A\" Parent=\"PA\""));
    }

    #[test]
    fn bom_preserved() {
        let mut body = String::from("\u{feff}");
        body.push_str(SOCKETS);
        let (out, oc) = apply_xml_intents(body.as_bytes(), &[set("SocketBoneData", "SocketList.Count", json!(5))]).unwrap();
        assert_eq!(oc[0], IntentOutcome::Applied);
        assert!(out.starts_with(&[0xEF, 0xBB, 0xBF]));
        assert!(String::from_utf8(out).unwrap().contains("Count=\"5\""));
    }

    #[test]
    fn path_parse_rejects_garbage() {
        assert!(parse_path("a[b]").is_err());
        assert!(parse_path("a[").is_err());
        let p = parse_path("SocketList.Socket[52]").unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[1].name, "Socket");
        assert_eq!(p[1].index, 52);
        assert!(p[1].has_index);
        assert!(!p[0].has_index);
    }
}

// ── path parsing ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Segment {
    name: String,
    index: usize,
    has_index: bool,
}

/// Parse a dotted path into segments. Each segment is `Name` or
/// `Name[index]`. Whitespace is not expected. Errors on malformed
/// brackets or empty names.
fn parse_path(path: &str) -> Result<Vec<Segment>, String> {
    let mut segments = Vec::new();
    for raw in path.split('.') {
        if raw.is_empty() {
            return Err(format!("empty path segment in '{}'", path));
        }
        if let Some(open) = raw.find('[') {
            if !raw.ends_with(']') {
                return Err(format!("malformed index in segment '{}'", raw));
            }
            let name = &raw[..open];
            let idx_str = &raw[open + 1..raw.len() - 1];
            if name.is_empty() {
                return Err(format!("empty element name in segment '{}'", raw));
            }
            let index: usize = idx_str
                .parse()
                .map_err(|_| format!("non-numeric index in segment '{}'", raw))?;
            segments.push(Segment { name: name.to_string(), index, has_index: true });
        } else {
            segments.push(Segment { name: raw.to_string(), index: 0, has_index: false });
        }
    }
    Ok(segments)
}
