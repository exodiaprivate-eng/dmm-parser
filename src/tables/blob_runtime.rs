//! Generic JSON runtime for `pabgh_blob_table!` formatted tables.
//!
//! The macro at `binary::variant::pabgh_blob_table!` lays out every entry as
//! `[key:u32][string_key:CString][is_blocked:u8][blob:rest_to_record_end]`.
//! ~120 tables in this crate use that layout (anything not yet given a
//! field-decoded reader). Without this runtime, v3 mods can't target any of
//! them — there's no `parse_X_to_json` per table.
//!
//! This module exposes ONE pair of functions that work uniformly across every
//! blob-format table by reading the wire layout directly. Mod intents can edit
//! `key`, `string_key`, or `is_blocked` by name. The `_blob_b64` field carries
//! the opaque tail as base64 so authors can clone whole records between mods
//! without losing field-decoded fidelity for the parts they don't understand —
//! and so the byte-level round-trip stays exact when no intent touches the blob.
//!
//! Tables with their own JSON parsers (iteminfo, skill, equip_slot_info as of
//! 1.3.4) should bypass this runtime — the dispatcher in DMM picks the
//! specific handler when one exists and falls back here for everything else.

use crate::binary::*;
use crate::binary::variant::{entry_ranges, load_pabgh_offsets_from_bytes};
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{json, Value};
use std::io::{self, Write};

/// Owned mirror of the macro-generated `pabgh_blob_table!` struct shape.
/// Used internally by parse/serialize for any table that fits the layout.
///
/// The `key_width` field captures whether the wire actually stored the key as
/// u32 (pabgh format 1/3) or u16 (pabgh format 2). The same struct holds both
/// so the JSON layer doesn't have to fork — write_to picks the right width
/// from `key_width` and the round-trip is byte-exact for either flavor.
#[derive(Debug)]
pub struct BlobTableRecord {
    pub key: u32,
    pub key_width: u8,
    pub string_key: String,
    pub is_blocked: u8,
    pub blob: Vec<u8>,
}

impl BlobTableRecord {
    /// Read one record from `data` starting at `*offset`, consuming exactly
    /// `entry_size` bytes (matches the macro's read_with_size contract).
    /// `key_width` must be 2 or 4 — caller derives it from the sister pabgh's
    /// detected format (format 2 stores u16 keys; formats 1/3 store u32).
    pub fn read_with_size(
        data: &[u8],
        offset: &mut usize,
        entry_size: usize,
        key_width: u8,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start
            .checked_add(entry_size)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BlobTableRecord: entry_size overflow"))?;
        if entry_end > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                format!("BlobTableRecord: record extends past body ({} > {})", entry_end, data.len())));
        }

        let key: u32 = match key_width {
            2 => u16::read_from(data, offset)? as u32,
            4 => u32::read_from(data, offset)?,
            other => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BlobTableRecord: unsupported key_width {} (expected 2 or 4)", other))),
        };
        let s = CString::read_from(data, offset)?;
        let string_key = std::str::from_utf8(s.data.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("string_key utf8: {}", e)))?
            .to_string();
        let is_blocked = u8::read_from(data, offset)?;
        if *offset > entry_end {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("BlobTableRecord k=0x{:x}: prefix over-consumed", key)));
        }
        let blob = data[*offset..entry_end].to_vec();
        *offset = entry_end;
        Ok(Self { key, key_width, string_key, is_blocked, blob })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self.key_width {
            2 => {
                if self.key > u16::MAX as u32 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                        format!("BlobTableRecord: key 0x{:x} doesn't fit u16 (table uses pabgh format 2)", self.key)));
                }
                (self.key as u16).write_to(w)?;
            }
            4 => self.key.write_to(w)?,
            other => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BlobTableRecord: unsupported key_width {} (expected 2 or 4)", other))),
        }
        // CString wire layout = u32 length + bytes. Inline since we hold an owned
        // String rather than the borrowed CString<'a> the macro uses.
        let bytes = self.string_key.as_bytes();
        (bytes.len() as u32).write_to(w)?;
        w.write_all(bytes)?;
        self.is_blocked.write_to(w)?;
        w.write_all(&self.blob)?;
        Ok(())
    }
}

impl ToJsonValue for BlobTableRecord {
    fn to_json_value(&self) -> Value {
        json!({
            "key": self.key,
            // Hidden underscored field captures the wire's key width so
            // round-trip serialization writes the same number of bytes.
            // Without this, a u16-keyed table parsed and re-serialized would
            // come out 2 bytes longer per record and the body would shift.
            "_key_width": self.key_width,
            "string_key": self.string_key,
            "is_blocked": self.is_blocked,
            // Underscored to make it visually distinct from the schema fields —
            // editing it is a "blob clone" operation, not a typed edit. Authors
            // who don't understand the table's binary should leave it alone.
            "_blob_b64": B64.encode(&self.blob),
        })
    }
}

impl WriteJsonValue for BlobTableRecord {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "BlobTableRecord: expected object"))?;
        // key_width may be missing on JSON dicts produced before this field
        // existed (defensive). Default to 4 (u32) to match historical behavior.
        let key_width = obj.get("_key_width")
            .and_then(|v| v.as_u64())
            .map(|x| x as u8)
            .unwrap_or(4);
        let key_val = json_get_field(obj, "key")?
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "key: expected integer"))?;
        match key_width {
            2 => {
                if key_val > u16::MAX as u64 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                        format!("key {} doesn't fit u16 (table uses pabgh format 2)", key_val)));
                }
                w.extend_from_slice(&(key_val as u16).to_le_bytes());
            }
            4 => {
                if key_val > u32::MAX as u64 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                        format!("key {} doesn't fit u32", key_val)));
                }
                w.extend_from_slice(&(key_val as u32).to_le_bytes());
            }
            other => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BlobTableRecord: unsupported _key_width {} (expected 2 or 4)", other))),
        }
        // string_key as plain JSON string (CString wire format).
        let sk = json_get_field(obj, "string_key")?
            .as_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "string_key: expected string"))?;
        let sk_bytes = sk.as_bytes();
        if sk_bytes.len() > u32::MAX as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("string_key too long ({} bytes)", sk_bytes.len())));
        }
        w.extend_from_slice(&(sk_bytes.len() as u32).to_le_bytes());
        w.extend_from_slice(sk_bytes);
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        // Blob comes as base64 — decode and write raw bytes.
        let blob_str = json_get_field(obj, "_blob_b64")?
            .as_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "_blob_b64: expected base64 string"))?;
        let blob = B64.decode(blob_str).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData, format!("_blob_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&blob);
        Ok(())
    }
}

/// Detect the wire key width from a sister pabgh by matching the file size
/// against the 3 known formats. Returns 2 (format 2: u16 keys) or 4 (formats
/// 1/3: u32 keys).
fn detect_key_width(pabgh: &[u8]) -> io::Result<u8> {
    if pabgh.len() < 4 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "pabgh too small to detect format"));
    }
    let c16 = u16::from_le_bytes(pabgh[0..2].try_into().unwrap()) as usize;
    let c32 = u32::from_le_bytes(pabgh[0..4].try_into().unwrap()) as usize;
    if 2 + c16 * 8 == pabgh.len() {
        Ok(4) // format 1
    } else if 2 + c16 * 6 == pabgh.len() {
        Ok(2) // format 2
    } else if 4 + c32 * 8 == pabgh.len() {
        Ok(4) // format 3
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData, "pabgh: unrecognized format (cannot detect key width)"))
    }
}

/// Reject a pabgh index that describes a LARGER body than the one supplied.
///
/// A `.pabgb` body and its `.pabgh` index are a matched pair — the index is a
/// list of record offsets INTO that exact body. Pair a body from one game build
/// with an index from another and the offsets are simply wrong: they run past
/// the end, and every caller that slices `&data[s..e]` panics.
///
/// This is not hypothetical. On 2026-08-15 two unrelated users hit
/// `range end index 1235101 out of range for slice of length 1234057` — the same
/// number on different machines with different mod lists, because 1,235,101 is a
/// record offset out of game 1.18's stock `skill.pabgh` and both had a mod
/// shipping a game-1.17 `skill.pabgb`. Inside a PyO3 call that panic became
/// "dispatch panic in apply_mods" and killed the mount with no usable message.
///
/// `entry_ranges` now clamps, so the panic is gone either way. But clamping alone
/// would turn a version mismatch into a table full of blob-fallback records —
/// silently wrong, which is worse than loudly broken and is exactly how a dead
/// table has slipped past a green gate before. So refuse it here, and name both
/// sizes so the report says which side is stale.
fn check_pabgh_matches_body(entries: &[(u32, usize)], body_len: usize) -> io::Result<()> {
    // The index legitimately ends AT body_len (the last record runs to the end),
    // so only a start strictly past the end is unambiguous evidence of a mismatch.
    let Some(&(key, worst)) = entries.iter().max_by_key(|(_, off)| *off) else {
        return Ok(());
    };
    if worst > body_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "pabgh/pabgb mismatch: index describes a record at offset {} (key 0x{:x}) \
                 but the body is only {} bytes. The .pabgb and .pabgh are from different \
                 builds — a mod is shipping a table built for another game version, or \
                 only one half of the pair was replaced.",
                worst, key, body_len
            ),
        ));
    }
    Ok(())
}

/// Parse any pabgh_blob_table-formatted body using its sister pabgh for record
/// boundaries. Returns one JSON dict per record in pabgh order — same calling
/// convention as `parse_skill_to_json_with_pabgh`.
///
/// Auto-detects whether the table uses u32 keys (pabgh formats 1/3) or u16
/// keys (pabgh format 2) from the pabgh layout. Tables that don't fit the
/// `[key][string_key:CString][is_blocked:u8][...rest...]` prefix shape will
/// fail to parse cleanly — callers should round-trip-check before applying
/// edits.
pub fn parse_blob_table_to_json_with_pabgh(
    data: &[u8],
    pabgh: &[u8],
) -> io::Result<Vec<Value>> {
    let key_width = detect_key_width(pabgh)?;
    let entries = load_pabgh_offsets_from_bytes(pabgh).ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, "blob_table: pabgh parse failed"))?;
    check_pabgh_matches_body(&entries, data.len())?;
    let ranges = entry_ranges(&entries, data.len());
    let mut out = Vec::with_capacity(ranges.len());
    for (k, s, e) in ranges {
        let mut c = s;
        let rec = BlobTableRecord::read_with_size(data, &mut c, e - s, key_width).map_err(|err| io::Error::new(
            err.kind(), format!("blob_table k=0x{:x}: {}", k, err)))?;
        if c != e {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("blob_table k=0x{:x}: under/over-consumed {}/{}", k, c - s, e - s)));
        }
        out.push(rec.to_json_value());
    }
    Ok(out)
}

/// Serialize a JSON list (as produced by `parse_blob_table_to_json_with_pabgh`)
/// back to pabgb bytes. The caller must rebuild the sister pabgh separately
/// — the offset map shifts whenever any record's `string_key` length changes.
pub fn serialize_blob_table_from_json(items: &[Value]) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(items.len() * 256);
    for (i, v) in items.iter().enumerate() {
        BlobTableRecord::write_from_json(&mut out, v).map_err(|e| io::Error::new(
            e.kind(), format!("blob_table[{}]: {}", i, e)))?;
    }
    Ok(out)
}

// ── Typed-prefix runtime (Tier 1.5 tables) ──────────────────────────────────
//
// `pabgh_typed_blob_table!` generates `read_with_size`, `write_to`,
// `to_json_dict`, and `write_from_json_dict` per table. These two helpers
// drive that surface across an entire pabgb body using the sister pabgh for
// entry boundaries — equivalent to `parse_blob_table_to_json_with_pabgh`
// but for tables that decode the typed prefix individually instead of
// folding everything into an opaque blob.

/// Parse a Tier 1.5 typed-prefix-plus-tail pabgb body to JSON.
///
/// The caller supplies the per-table `read_with_size` and `to_json_dict`
/// functions (or a thin closure over them). This indirection keeps the
/// runtime fully generic without monomorphising one copy per table inside
/// dmm-parser — DMM and other consumers can reuse it for any pabgh_typed_blob_table
/// table.
pub fn parse_typed_blob_table_to_json_with_pabgh<F, G>(
    data: &[u8],
    pabgh: &[u8],
    mut read_one: F,
    mut read_partial: G,
) -> io::Result<Vec<Value>>
where
    F: FnMut(&[u8], &mut usize, usize) -> io::Result<serde_json::Map<String, Value>>,
    G: FnMut(&[u8], usize, usize) -> serde_json::Map<String, Value>,
{
    let entries = load_pabgh_offsets_from_bytes(pabgh).ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, "typed_blob_table: pabgh parse failed"))?;
    check_pabgh_matches_body(&entries, data.len())?;
    let ranges = entry_ranges(&entries, data.len());
    let mut out = Vec::with_capacity(ranges.len());
    for (k, s, e) in ranges {
        let mut c = s;
        match read_one(data, &mut c, e - s) {
            Ok(dict) if c == e => {
                out.push(Value::Object(dict));
            }
            Ok(dict) => {
                out.push(Value::Object(dict));
            }
            Err(_err) => {
                // First 3 by default so a broken table is obvious without
                // drowning the log. `DMM_BLOB_VERBOSE=1` reports every failure —
                // needed when the failures start deep in a large table
                // (characterinfo's begin around index 6266, so the first-3 rule
                // hid them entirely).
                if out.len() < 3 || std::env::var_os("DMM_BLOB_VERBOSE").is_some() {
                    eprintln!("BLOB_FALLBACK k=0x{:x} size={}: {}", k, e - s, _err);
                }
                // Typed parse failed for this entry — fall back to raw blob.
                // Store the entire entry bytes as _blob_b64 (key-width agnostic).
                // Use the pabgh key `k` for the JSON key field.
                let entry = &data[s..e];

                use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
                // PARTIAL DECODE: keep the fields that DID parse instead of
                // discarding the whole record. One unmodelled field near the end
                // of a 205-field struct used to hide everything before it — in
                // live 1.16 characterinfo that buried 232 records (all the pet
                // cats and dogs, the kakapo) whose name/desc/lookup_22/f38 parse
                // perfectly and sit far earlier in the struct.
                //
                // The blob stays authoritative: `_blob_fallback` still routes
                // this record to `write_blob_fallback_entry`, which writes ONLY
                // `_blob_b64` verbatim. These fields are read-only and can never
                // feed a re-serialisation, so byte-roundtrip is unchanged.
                let mut dict = read_partial(data, s, e - s);
                // pabgh's key wins over any partially-read one.
                dict.insert("key".into(), Value::Number(k.into()));
                dict.insert("_blob_b64".into(), Value::String(B64.encode(entry)));
                dict.insert("_blob_fallback".into(), Value::Bool(true));
                out.push(Value::Object(dict));
            }
        }
    }
    Ok(out)
}

/// Serialize a list of typed-prefix dicts back to pabgb bytes.
///
/// Mirrors `serialize_blob_table_from_json` but routes each entry through
/// the per-table `write_from_json_dict` (passed in as a closure). The
/// caller rebuilds the sister pabgh separately — every record's tail size
/// is preserved verbatim, so for replace-only edits the vanilla pabgh
/// stays valid byte-for-byte.
pub fn serialize_typed_blob_table_from_json<F, G>(
    items: &[Value],
    mut write_one: F,
    mut write_partial: G,
) -> io::Result<Vec<u8>>
where
    F: FnMut(&mut Vec<u8>, &serde_json::Map<String, Value>) -> io::Result<()>,
    G: FnMut(&mut Vec<u8>, &serde_json::Map<String, Value>, usize) -> io::Result<()>,
{
    let mut out = Vec::with_capacity(items.len() * 256);
    for (i, v) in items.iter().enumerate() {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("typed_blob_table[{}]: expected object, got {}", i, type_name(v))))?;
        if obj.contains_key("_blob_fallback") {
            write_blob_fallback_entry_partial(&mut out, obj, &mut write_partial)?;
        } else {
            write_one(&mut out, obj).map_err(|e| io::Error::new(
                e.kind(), format!("typed_blob_table[{}]: {}", i, e)))?;
        }
    }
    Ok(out)
}

fn write_blob_fallback_entry(out: &mut Vec<u8>, obj: &serde_json::Map<String, Value>) -> io::Result<()> {
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    // _blob_b64 contains the entire entry bytes (key-width agnostic).
    if let Some(blob_v) = obj.get("_blob_b64").and_then(|v| v.as_str()) {
        let blob = B64.decode(blob_v).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData, format!("blob fallback decode: {}", e)))?;
        out.extend_from_slice(&blob);
    }
    Ok(())
}

/// Blob-fallback writer that lets an EDIT to a partially-decoded prefix land.
///
/// `read_partial_json` salvages the fields that decoded before the schema broke
/// and records `_partial_fields` / `_partial_prefix_len`. Writing `_blob_b64`
/// verbatim (as `write_blob_fallback_entry` does) silently discards any edit to
/// those fields — that is why the 1.5.9 cat/dog swaps applied and changed
/// nothing. Here we instead:
///   1. re-serialise the `_partial_fields` decoded fields from the JSON, and
///   2. append the ORIGINAL entry bytes from `_partial_prefix_len` onward.
/// The undecoded remainder is preserved byte-for-byte, so an untouched record
/// still round-trips exactly while `lookup_22` (or any decoded field) can change.
///
/// Falls back to the verbatim write whenever anything is missing or the
/// re-serialised prefix would not line up — never guess with a binary format.
fn write_blob_fallback_entry_partial<G>(
    out: &mut Vec<u8>,
    obj: &serde_json::Map<String, Value>,
    write_partial: &mut G,
) -> io::Result<()>
where
    G: FnMut(&mut Vec<u8>, &serde_json::Map<String, Value>, usize) -> io::Result<()>,
{
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    let Some(blob_v) = obj.get("_blob_b64").and_then(|v| v.as_str()) else {
        return Ok(());
    };
    let blob = B64.decode(blob_v).map_err(|e| io::Error::new(
        io::ErrorKind::InvalidData, format!("blob fallback decode: {}", e)))?;

    let n = obj.get("_partial_fields").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let plen = obj.get("_partial_prefix_len").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    if n == 0 || plen == 0 || plen > blob.len() {
        out.extend_from_slice(&blob);
        return Ok(());
    }

    let mut prefix = Vec::with_capacity(plen);
    if write_partial(&mut prefix, obj, n).is_err() {
        // Could not rebuild the prefix — emit the original untouched.
        out.extend_from_slice(&blob);
        return Ok(());
    }
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&blob[plen..]);
    Ok(())
}

/// Same as [`serialize_typed_blob_table_from_json`] but also returns the
/// `(key, byte_offset)` pair for each record. Used by the apply-intents
/// pipeline to rebuild the sister `pabgh` index when records are added,
/// removed, or change size.
///
/// The key is extracted permissively from each record's JSON via
/// [`extract_record_key`] — accepts both scalar `key: <int>` and the
/// iteminfo-style `key: {"value": <int>}` wrapper shape.
pub fn serialize_typed_blob_table_from_json_tracked<F, G>(
    items: &[Value],
    mut write_one: F,
    mut write_partial: G,
) -> io::Result<(Vec<u8>, Vec<(u32, u32)>)>
where
    F: FnMut(&mut Vec<u8>, &serde_json::Map<String, Value>) -> io::Result<()>,
    G: FnMut(&mut Vec<u8>, &serde_json::Map<String, Value>, usize) -> io::Result<()>,
{
    let mut out = Vec::with_capacity(items.len() * 256);
    let mut offsets = Vec::with_capacity(items.len());
    for (i, v) in items.iter().enumerate() {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("typed_blob_table[{}]: expected object, got {}", i, type_name(v))))?;
        let key = extract_record_key(v).ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("typed_blob_table[{}]: missing or non-integer 'key' field", i)))?;
        let offset = u32::try_from(out.len()).map_err(|_| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("typed_blob_table[{}]: body offset {} exceeds u32 range", i, out.len())))?;
        offsets.push((key, offset));
        if obj.contains_key("_blob_fallback") {
            // Same partial-splice as the untracked path — THIS is the function the
            // v3 apply pipeline uses, so without it every mod edit to a salvaged
            // field is silently dropped.
            write_blob_fallback_entry_partial(&mut out, obj, &mut write_partial)?;
        } else {
            write_one(&mut out, obj).map_err(|e| io::Error::new(
                e.kind(), format!("typed_blob_table[{}]: {}", i, e)))?;
        }
    }
    Ok((out, offsets))
}

/// Tracked sister of [`serialize_blob_table_from_json`] — generic blob-
/// fallback path. Same offsets contract as
/// [`serialize_typed_blob_table_from_json_tracked`].
pub fn serialize_blob_table_from_json_tracked(
    items: &[Value],
) -> io::Result<(Vec<u8>, Vec<(u32, u32)>)> {
    let mut out = Vec::with_capacity(items.len() * 256);
    let mut offsets = Vec::with_capacity(items.len());
    for (i, v) in items.iter().enumerate() {
        let key = extract_record_key(v).ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("blob_table[{}]: missing or non-integer 'key' field", i)))?;
        let offset = u32::try_from(out.len()).map_err(|_| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("blob_table[{}]: body offset {} exceeds u32 range", i, out.len())))?;
        offsets.push((key, offset));
        BlobTableRecord::write_from_json(&mut out, v).map_err(|e| io::Error::new(
            e.kind(), format!("blob_table[{}]: {}", i, e)))?;
    }
    Ok((out, offsets))
}

/// Read a record's identifying key from its JSON. Permissive: accepts
/// scalar `record["key"] = <int>` (most pabgh-bounded tables) and the
/// `record["key"] = {"value": <int>}` wrapper shape (used by iteminfo's
/// `ItemKey`). Returns `None` when neither shape is present or the value
/// doesn't fit `u32`.
pub fn extract_record_key(record: &Value) -> Option<u32> {
    let key = record.get("key")?;
    if let Some(n) = key.as_u64() {
        return u32::try_from(n).ok();
    }
    if let Some(n) = key.as_i64() {
        return u32::try_from(n).ok();
    }
    if let Some(inner) = key.get("value") {
        if let Some(n) = inner.as_u64() { return u32::try_from(n).ok(); }
        if let Some(n) = inner.as_i64() { return u32::try_from(n).ok(); }
    }
    None
}

#[inline]
fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod mismatch_guard_tests {
    use super::check_pabgh_matches_body;

    /// A matched pair: the last record legitimately runs to the end of the body,
    /// so an offset EQUAL to body_len must not be treated as a mismatch.
    #[test]
    fn matched_pair_is_accepted_including_a_record_ending_at_the_body_end() {
        let entries = [(0x1, 0usize), (0x2, 500), (0x3, 900)];
        assert!(check_pabgh_matches_body(&entries, 1000).is_ok());
        // Degenerate but legal: an empty trailing record starting exactly at the end.
        assert!(check_pabgh_matches_body(&[(0x1, 0), (0x2, 1000)], 1000).is_ok());
    }

    /// The live failure, in miniature: game 1.18's skill index (a record at
    /// 1,235,101) against a game 1.17 skill body (1,234,380 bytes).
    ///
    /// Before this guard the mismatch reached `&data[s..e]` and panicked through
    /// PyO3 as "dispatch panic in apply_mods", killing the mount. It must now be
    /// an ordinary Err whose text names both sizes, so the report says which side
    /// is stale instead of printing a bare slice index.
    #[test]
    fn index_from_another_build_is_refused_with_both_sizes_named() {
        let entries = [(0x1, 0usize), (0x2, 1_200_000), (0xabc, 1_235_101)];
        let err = check_pabgh_matches_body(&entries, 1_234_380)
            .expect_err("a 1.18 index over a 1.17 body must not be accepted");
        let msg = err.to_string();
        assert!(msg.contains("1235101"), "should name the bad offset: {msg}");
        assert!(msg.contains("1234380"), "should name the body size: {msg}");
        assert!(msg.contains("abc"), "should name the record key: {msg}");
        assert!(
            msg.contains("different") || msg.contains("version"),
            "should explain it is a build mismatch, not just a number: {msg}"
        );
    }

    #[test]
    fn empty_index_is_not_a_mismatch() {
        assert!(check_pabgh_matches_body(&[], 0).is_ok());
        assert!(check_pabgh_matches_body(&[], 1000).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip a known blob-format table to confirm the runtime preserves
    /// vanilla bytes when no intent touches the data. Uses equip_info — small,
    /// well-behaved, and confirmed pabgh_blob_table! in the source.
    #[test]
    fn blob_table_roundtrip() {
        let pabgb_path = r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\equip_info.pabgb";
        let pabgh_path = r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\equip_info.pabgh";
        // Try several candidates — different game-data dumps may not have every file.
        for (pb, ph) in [
            (pabgb_path, pabgh_path),
            (r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\condition_info.pabgb",
             r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\condition_info.pabgh"),
            (r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\buff_info.pabgb",
             r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\buff_info.pabgh"),
        ] {
            let Ok(body) = std::fs::read(pb) else { continue; };
            let Ok(pabgh) = std::fs::read(ph) else { continue; };
            let json = parse_blob_table_to_json_with_pabgh(&body, &pabgh)
                .unwrap_or_else(|e| panic!("parse failed for {}: {}", pb, e));
            let out = serialize_blob_table_from_json(&json)
                .unwrap_or_else(|e| panic!("serialize failed for {}: {}", pb, e));
            assert_eq!(out, body, "round-trip mismatch for {}", pb);
            // Smoke: at least one record has a non-empty string_key for the
            // typical case; if all are empty we likely picked the wrong layout.
            let any_named = json.iter().any(|v| v.get("string_key").and_then(|s| s.as_str()).map(|s| !s.is_empty()).unwrap_or(false));
            // Don't fail on this — some tables genuinely have empty string_keys.
            // Just log via the test runner if we want to inspect.
            let _ = any_named;
            return; // first successful file is enough
        }
        eprintln!("SKIP blob_table_roundtrip: no input files available");
    }

    /// Round-trip a u16-keyed table (pabgh format 2) to prove the auto
    /// detected key_width path preserves bytes. inventory.pabgb is the
    /// canonical example — without u16 detection this would mis-parse the
    /// key column and serialize-back would shift every record.
    #[test]
    fn blob_table_roundtrip_u16_key() {
        for (pb, ph) in [
            (r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/inventory.pabgb",
             r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/inventory.pabgh"),
            (r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\inventory.pabgb",
             r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata\inventory.pabgh"),
        ] {
            let Ok(body) = std::fs::read(pb) else { continue; };
            let Ok(pabgh) = std::fs::read(ph) else { continue; };
            let json = parse_blob_table_to_json_with_pabgh(&body, &pabgh)
                .unwrap_or_else(|e| panic!("parse failed for {}: {}", pb, e));
            let out = serialize_blob_table_from_json(&json)
                .unwrap_or_else(|e| panic!("serialize failed for {}: {}", pb, e));
            assert_eq!(out, body, "u16-key round-trip mismatch for {}", pb);
            // Confirm at least one record actually came back with a useful
            // key value — if the key_width detection silently fell back to
            // u32 we'd see truncated/0 keys and JSON would be useless.
            let max_key = json.iter()
                .filter_map(|v| v.get("key").and_then(|k| k.as_u64()))
                .max()
                .unwrap_or(0);
            assert!(max_key > 0, "no records had non-zero keys — u16 detection probably failed");
            assert!(max_key <= u16::MAX as u64, "key {} exceeds u16 max — wrong width detected", max_key);
            return;
        }
        eprintln!("SKIP blob_table_roundtrip_u16_key: no input files available");
    }
}
