//! Reward overlay for the gimmick tail blob.
//!
//! The gimmick post-body is not field-decoded on the current game build — the
//! whole tail rides as a raw blob (`post_blob` or the `_b64` early-fallback).
//! The reward data a modder wants — the world-gathering item COUNT and the
//! pet/NPC FRIENDSHIP delta — lives in `_dropInfoDataList` (and its sibling
//! `_buyableDropItem`) inside that blob, as `DropInfoData` records.
//!
//! Rather than rewrite the sequential decoder (fully undecoded on this build,
//! several tangled failure modes), we SCAN the raw blob for well-formed
//! `_dropInfoDataList` structures, expose the reward values as editable fields,
//! and PATCH edited values back into the blob at their byte offsets on write.
//!
//! Safety against false positives: a candidate list is only accepted when every
//! item-bearing entry's key resolves to a real `iteminfo` key (the caller
//! supplies the predicate). A random byte run almost never has all-real item
//! keys, so this reliably isolates the genuine reward lists. See
//! `drop_info_scan` unit tests + the whole-table round-trip gate.
//!
//! Layout (verified against live 1.13 data — `sub_100DCD204` / `sub_101FD5A0C`):
//! `_dropInfoDataList` = u32 count + count × { present:u8 [+DropInfoData] + u32 }.
//! `DropInfoData` = 63-byte header + a `_typeTag`-selected union body.

/// `DropInfoData` fixed-header size, in bytes.
const HEADER: usize = 63;
/// Byte offset of `_minValue` (i64) inside a `DropInfoData` record.
const OFF_MIN: usize = 45;
/// Byte offset of `_maxValue` (i64) inside a `DropInfoData` record.
const OFF_MAX: usize = 53;
/// Byte offset of `_typeTag` (u8) inside a `DropInfoData` record.
const OFF_TYPE_TAG: usize = 8;
/// Byte offset of `_varyFriendly` (i64) inside the tag-7/8/15 friendship body,
/// measured from the start of the union body (`header + 18`).
const OFF_VARY_IN_BODY: usize = 18;

/// Upper bound for a plausible reward min/max count. Real gather counts are
/// small (observed 1–10); a generous cap rejects garbage i64s from false matches
/// while never clipping a legitimate reward.
const COUNT_CAP: i64 = 1_000_000;

/// Union-body byte size for each `_typeTag`. `None` = unknown tag (reject).
fn case_size(tag: u8) -> Option<usize> {
    Some(match tag {
        0 | 1 | 2 | 3 | 4 | 5 | 6 | 9 | 12 => 4,
        7 | 8 | 15 => 32, // friendship
        10 => 8,
        11 => 0,
        13 => 5,
        14 => 3,
        _ => return None,
    })
}

/// Item-bearing tags — the `_keyRaw` low u32 is an `iteminfo` key we can
/// cross-validate. Friendship/faction/event tags reference other tables.
fn is_item_tag(tag: u8) -> bool {
    matches!(tag, 0 | 1 | 2 | 3 | 4 | 5 | 6 | 9 | 12)
}

/// One decoded reward entry, with the absolute blob offsets of its editable
/// values so a write can patch them back in place.
#[derive(Debug, Clone, PartialEq)]
pub struct RewardEntry {
    /// `_keyRaw` low 32 bits — the item key for item tags.
    pub item_key: u32,
    pub type_tag: u8,
    pub min_value: i64,
    pub max_value: i64,
    /// Present only for friendship tags (7/8/15).
    pub vary_friendly: Option<i64>,
    // Absolute blob offsets of the i64 values, for patch-back.
    pub off_min: usize,
    pub off_max: usize,
    pub off_vary: Option<usize>,
}

/// A reward list found at `offset` in the blob (either `_dropInfoDataList` or
/// `_buyableDropItem` — indistinguishable by content, both editable the same way).
#[derive(Debug, Clone, PartialEq)]
pub struct RewardList {
    pub offset: usize,
    pub entries: Vec<RewardEntry>,
}

#[inline]
fn rd_u32(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}
#[inline]
fn rd_i64(b: &[u8], o: usize) -> Option<i64> {
    b.get(o..o + 8).map(|s| i64::from_le_bytes(s.try_into().unwrap()))
}

/// Parse a single `DropInfoData` at `off`. Returns the entry + the offset just
/// past it, or `None` if the bytes don't form a valid record.
fn read_drop_info(b: &[u8], off: usize) -> Option<(RewardEntry, usize)> {
    let tag = *b.get(off + OFF_TYPE_TAG)?;
    let cs = case_size(tag)?;
    let end = off + HEADER + cs;
    if end > b.len() {
        return None;
    }
    let key_raw = rd_i64(b, off)?;
    let min_value = rd_i64(b, off + OFF_MIN)?;
    let max_value = rd_i64(b, off + OFF_MAX)?;
    let (vary_friendly, off_vary) = if matches!(tag, 7 | 8 | 15) {
        let vo = off + HEADER + OFF_VARY_IN_BODY;
        (Some(rd_i64(b, vo)?), Some(vo))
    } else {
        (None, None)
    };
    Some((
        RewardEntry {
            item_key: (key_raw as u64 & 0xFFFF_FFFF) as u32,
            type_tag: tag,
            min_value,
            max_value,
            vary_friendly,
            off_min: off + OFF_MIN,
            off_max: off + OFF_MAX,
            off_vary,
        },
        end,
    ))
}

/// Try to parse a `_dropInfoDataList` starting at `off`. Accepts only when the
/// list is well-formed AND at least one item-tag entry resolves via `is_real_item`
/// with NO item-tag entry failing to resolve (the false-positive guard).
fn try_read_list(
    b: &[u8],
    off: usize,
    is_real_item: &dyn Fn(u32) -> bool,
) -> Option<(RewardList, usize)> {
    let count = rd_u32(b, off)?;
    if count == 0 || count > 64 {
        return None;
    }
    let mut p = off + 4;
    let mut entries = Vec::with_capacity(count as usize);
    let mut item_hits = 0usize;
    for _ in 0..count {
        let present = *b.get(p)?;
        if present > 1 {
            return None;
        }
        p += 1;
        if present == 1 {
            let (e, next) = read_drop_info(b, p)?;
            // Value-plausibility guard: a real reward's min/max count is a small
            // non-negative range. Garbage byte runs decode to wild i64s — reject
            // them so a spurious "list" can't be exposed (and then mis-edited).
            if !(0..=COUNT_CAP).contains(&e.min_value)
                || !(e.min_value..=COUNT_CAP).contains(&e.max_value)
            {
                return None;
            }
            if is_item_tag(e.type_tag) {
                // Every item-tag key must be real — the false-positive guard.
                if !is_real_item(e.item_key) {
                    return None;
                }
                item_hits += 1;
            }
            entries.push(e);
            p = next;
        }
        // trailing per-entry u32
        rd_u32(b, p)?;
        p += 4;
    }
    // Require at least one resolved item key so a run of zero-length/empty
    // entries can't masquerade as a reward list.
    if item_hits == 0 {
        return None;
    }
    Some((RewardList { offset: off, entries }, p))
}

/// Scan a blob for every non-overlapping, iteminfo-validated reward list.
/// Genuine records normally yield 1 (just `_dropInfoDataList`) or 2 (plus
/// `_buyableDropItem`); the item-key guard rejects spurious matches.
pub fn scan_reward_lists(b: &[u8], is_real_item: &dyn Fn(u32) -> bool) -> Vec<RewardList> {
    let mut out = Vec::new();
    let mut o = 0usize;
    while o + 4 <= b.len() {
        if let Some((list, end)) = try_read_list(b, o, is_real_item) {
            out.push(list);
            o = end;
        } else {
            o += 1;
        }
    }
    out
}

/// Patch an entry's (possibly edited) values back into the blob at its offsets.
/// Writing the same value is a no-op (the round-trip safety property).
pub fn patch_entry(b: &mut [u8], e: &RewardEntry) {
    b[e.off_min..e.off_min + 8].copy_from_slice(&e.min_value.to_le_bytes());
    b[e.off_max..e.off_max + 8].copy_from_slice(&e.max_value.to_le_bytes());
    if let (Some(vo), Some(v)) = (e.off_vary, e.vary_friendly) {
        b[vo..vo + 8].copy_from_slice(&v.to_le_bytes());
    }
}

// ── JSON overlay + item-key context ─────────────────────────────────────────

use serde_json::{Map, Value};
use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    /// Real `iteminfo` keys, supplied by the caller so the reward scan can reject
    /// spurious matches (a byte run whose "item keys" aren't real). Set before
    /// parsing/serializing gimmick_info when reward editing matters; when unset,
    /// the scan falls back to a plausible-item-id range (safe for round-trip,
    /// looser for edits).
    static ITEM_KEYS: RefCell<Option<HashSet<u32>>> = const { RefCell::new(None) };
}

/// Install the real-item-key set for this thread's subsequent gimmick reward
/// scans. Pass `None` to clear.
pub fn set_item_keys(keys: Option<HashSet<u32>>) {
    ITEM_KEYS.with(|c| *c.borrow_mut() = keys);
}

/// The active item-key predicate: the installed set if present, else a plausible
/// item-id range (covers the observed reward item-key band).
fn is_real_item_default(k: u32) -> bool {
    ITEM_KEYS.with(|c| match &*c.borrow() {
        Some(set) => set.contains(&k),
        None => (700_000..=1_100_000).contains(&k),
    })
}

/// Scan a blob and return the reward overlay as a JSON array (empty if none).
/// Shape: `[{ item_key, type_tag, min_value, max_value, vary_friendly? }, …]`
/// flattened across every found list (order-stable = scan order).
pub fn reward_list_json(blob: &[u8]) -> Value {
    let lists = scan_reward_lists(blob, &is_real_item_default);
    let mut arr = Vec::new();
    for list in &lists {
        for e in &list.entries {
            let mut m = Map::new();
            m.insert("item_key".into(), Value::from(e.item_key));
            m.insert("type_tag".into(), Value::from(e.type_tag));
            m.insert("min_value".into(), Value::from(e.min_value));
            m.insert("max_value".into(), Value::from(e.max_value));
            if let Some(v) = e.vary_friendly {
                m.insert("vary_friendly".into(), Value::from(v));
            }
            arr.push(Value::Object(m));
        }
    }
    Value::Array(arr)
}

/// Patch a blob in place from an edited `drop_info_data_list` JSON array. The
/// blob is RE-SCANNED to recover offsets (never trusting caller-supplied
/// offsets, so it stays version-robust), then each entry's `min_value` /
/// `max_value` / `vary_friendly` is written back positionally. A shape mismatch
/// (count differs) leaves the blob untouched — a safe no-op rather than a guess.
pub fn patch_blob_from_json(blob: &mut [u8], list_json: &Value) {
    let Some(edits) = list_json.as_array() else { return };
    // Recover the same flattened entry order the read produced.
    let lists = scan_reward_lists(blob, &is_real_item_default);
    let mut flat: Vec<RewardEntry> = Vec::new();
    for l in &lists {
        flat.extend(l.entries.iter().cloned());
    }
    if flat.len() != edits.len() {
        return; // structure drifted — don't risk a mispatch
    }
    for (e, ev) in flat.iter().zip(edits.iter()) {
        let mut patched = e.clone();
        if let Some(v) = ev.get("min_value").and_then(|x| x.as_i64()) {
            patched.min_value = v;
        }
        if let Some(v) = ev.get("max_value").and_then(|x| x.as_i64()) {
            patched.max_value = v;
        }
        if patched.off_vary.is_some() {
            if let Some(v) = ev.get("vary_friendly").and_then(|x| x.as_i64()) {
                patched.vary_friendly = Some(v);
            }
        }
        patch_entry(blob, &patched);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a minimal blob: [count=1][present=1][DropInfoData tag=0][trailing u32].
    fn item_drop_blob(item_key: u32, min: i64, max: i64) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&1u32.to_le_bytes()); // count
        b.push(1); // present
        let start = b.len();
        b.resize(start + HEADER + 4, 0); // header + tag-0 body(4)
        b[start..start + 8].copy_from_slice(&(item_key as i64).to_le_bytes()); // _keyRaw
        b[start + OFF_TYPE_TAG] = 0; // _typeTag
        b[start + OFF_MIN..start + OFF_MIN + 8].copy_from_slice(&min.to_le_bytes());
        b[start + OFF_MAX..start + OFF_MAX + 8].copy_from_slice(&max.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes()); // trailing u32
        b
    }

    #[test]
    fn reads_item_drop_list_and_guards_on_real_keys() {
        let b = item_drop_blob(750006, 1, 2);
        // With the key accepted: one list, correct values.
        let real = |k: u32| k == 750006;
        let lists = scan_reward_lists(&b, &real);
        assert_eq!(lists.len(), 1);
        let e = &lists[0].entries[0];
        assert_eq!(e.item_key, 750006);
        assert_eq!((e.min_value, e.max_value), (1, 2));
        // With the key rejected: the guard drops it (no false positive).
        let none = |_k: u32| false;
        assert!(scan_reward_lists(&b, &none).is_empty());
    }

    #[test]
    fn patch_roundtrips_and_edits() {
        let mut b = item_drop_blob(750006, 1, 2);
        let orig = b.clone();
        let real = |k: u32| k == 750006;
        let mut e = scan_reward_lists(&b, &real)[0].entries[0].clone();
        // patch-same-value = byte-identical (the round-trip safety property)
        patch_entry(&mut b, &e);
        assert_eq!(b, orig);
        // scale the count ×5 and patch → the value changes at exactly its offset
        e.min_value *= 5;
        e.max_value *= 5;
        patch_entry(&mut b, &e);
        let after = scan_reward_lists(&b, &real)[0].entries[0].clone();
        assert_eq!((after.min_value, after.max_value), (5, 10));
    }

    #[test]
    fn friendship_entry_exposes_vary_friendly() {
        // [count=1][present=1][DropInfoData tag=7 friendship][trailing u32]
        let mut b = Vec::new();
        b.extend_from_slice(&1u32.to_le_bytes());
        b.push(1);
        let start = b.len();
        b.resize(start + HEADER + 32, 0);
        b[start + OFF_TYPE_TAG] = 7;
        let vo = start + HEADER + OFF_VARY_IN_BODY;
        b[vo..vo + 8].copy_from_slice(&5i64.to_le_bytes()); // +5 friendship
        b.extend_from_slice(&0u32.to_le_bytes());
        // friendship tags carry no item key, so this list needs an item entry to
        // pass the guard in the real scanner; here we assert the low-level reader.
        let (e, _) = read_drop_info(&b, start).unwrap();
        assert_eq!(e.type_tag, 7);
        assert_eq!(e.vary_friendly, Some(5));
    }
}
