#!/usr/bin/env python3
"""Two-fixture BYTE DIFF — the "which tables drifted, and how" triage.

Run this FIRST on patch day, before triage.py and before opening IDA.

`patchday/README.md` documents this procedure being done BY HAND for buff_info
("1523 diffs, all -1, lowest 37 -> 36" pinned a removed variant with no
disassembly). It was never scripted, so on 1.16 it was re-derived per table.
This is that procedure, for every table, in one command.

    python bytediff.py                        # all tables, OLD vs NEW
    python bytediff.py buffinfo iteminfo      # just these
    python bytediff.py --json out.json        # machine-readable
    python bytediff.py --old <dir> --new <dir>

What it reports per table, and why each matters:

  size deltas          A CONSTANT non-zero delta = a field was added/removed.
                       The value IS the field width.
  enum renumber        Every differing byte on same-size records is collected,
                       then filtered to SMALL deltas (|d| <= 8). If those agree,
                       a variant index shifted, and `lowest X -> Y` is where the
                       add/remove happened. Found without touching IDA.
                       Validated against the documented buff_info result:
                       "1523 diffs of -1, lowest 37 -> 36".
  first-diff offset    Where records start diverging. A tight cluster localises
                       the drift; a smear means the shift compounds downstream.

Both signals are reported INDEPENDENTLY -- a table can gain a field AND renumber
an enum in one patch (buff_info did exactly that), and the enum half is far
cheaper to act on.

Record matching is by KEY, not index, because a patch can reorder records --
and 1.16 hash-remapped iteminfo's keys outright. When key overlap is poor the
tool falls back to index matching and SAYS SO, because an index-matched diff of
a reordered table is noise.
"""
import argparse
import json
import os
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rec  # noqa: E402  (same-dir helper: pabgh header autodetect + offset sort)

# Windows consoles default to cp1252 and die on any non-latin-1 output. The
# toolkit README flags this for Korean decompiler text; it bites here too, on a
# single star in a heading -- and it crashed the run mid-report. Force UTF-8.
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, ValueError):
    pass

OLD_DEFAULT = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-7-16"
NEW_DEFAULT = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-1"

# A record can differ in a couple of bytes and still be an enum renumber; more
# than this and it is structural, not a value shift.
SINGLE_BYTE_MAX = 2
# Only trust "uniform delta" when this share of small diffs agree.
UNIFORM_SHARE = 0.90
# An enum/variant index shifts by a small amount; anything larger is byte-shift
# noise from a moved region.
ENUM_DELTA_MAX = 8


def tables_in(d):
    return {
        f[:-6]
        for f in os.listdir(d)
        if f.endswith(".pabgb") and os.path.exists(os.path.join(d, f[:-6] + ".pabgh"))
    }


def load(d, t):
    try:
        return rec.recs(d, t)
    except Exception as e:  # unknown pabgh shape, truncated dump, ...
        return e


def compare(old_recs, new_recs):
    """Diff two record lists. Returns a dict of findings."""
    o = {k: v for k, v in old_recs}
    n = {k: v for k, v in new_recs}
    shared = set(o) & set(n)
    # Key overlap decides whether keys are trustworthy this patch.
    overlap = len(shared) / max(1, min(len(o), len(n)))
    mode = "key"
    if overlap < 0.5:
        mode = "index"
        pairs = list(zip([v for _, v in old_recs], [v for _, v in new_recs]))
    else:
        pairs = [(o[k], n[k]) for k in shared]

    size_delta = Counter()
    byte_delta = Counter()      # signed (new-old) for single-byte-diff records
    first_diff = Counter()      # bucketed offset of first differing byte
    lowest_by_delta = {}        # delta -> lowest OLD byte value that moved
    single_byte_recs = 0
    changed = 0

    for a, b in pairs:
        if a == b:
            continue
        changed += 1
        size_delta[len(b) - len(a)] += 1
        if len(a) != len(b):
            m = min(len(a), len(b))
            for i in range(m):
                if a[i] != b[i]:
                    first_diff[i // 16 * 16] += 1
                    break
            else:
                first_diff[m // 16 * 16] += 1
            continue
        # Same size -> look for the enum-renumber fingerprint.
        # ★ Collect EVERY differing byte, not only records that differ in one
        # place. The documented buff_info find was "1523 diffs, all -1" across
        # the whole table -- its records differ in several positions each, so a
        # "records with exactly one changed byte" filter finds NOTHING there.
        # The uniformity test below is what separates signal from noise.
        diffs = [i for i in range(len(a)) if a[i] != b[i]]
        if not diffs:
            continue
        first_diff[diffs[0] // 16 * 16] += 1
        if len(diffs) <= SINGLE_BYTE_MAX:
            single_byte_recs += 1
        for i in diffs:
            d = b[i] - a[i]
            byte_delta[d] += 1
            # lowest OLD value per delta: for an enum renumber this is the
            # variant index that moved, i.e. where the add/remove happened.
            if d not in lowest_by_delta or a[i] < lowest_by_delta[d]:
                lowest_by_delta[d] = a[i]

    return {
        "match_mode": mode,
        "key_overlap": round(overlap, 3),
        "old_records": len(o),
        "new_records": len(n),
        "compared": len(pairs),
        "changed": changed,
        "size_delta": dict(size_delta.most_common()),
        "byte_delta": dict(byte_delta.most_common()),
        "single_byte_records": single_byte_recs,
        "lowest_by_delta": lowest_by_delta,
        "first_diff_offsets": dict(first_diff.most_common(5)),
    }


def enum_signal(r):
    """The buff_info fingerprint: uniform single-byte deltas on same-size records.

    ★ Reported INDEPENDENTLY of size deltas. A table can gain/lose a field AND
    renumber a variant enum in the same patch, and the enum signal is the
    cheaper of the two to act on -- masking it behind "mixed size deltas" is
    what made this a hand-derived job on 1.16.
    """
    # ★ Only SMALL deltas can be an enum renumber. Large ones (-186, -240 ...)
    # are shifted-region noise: when bytes move, every byte in the moved span
    # "changes" by an arbitrary amount. On buff_info that noise outnumbered the
    # real signal 9:1 and hid it completely.
    small = {d: c for d, c in r["byte_delta"].items() if 0 < abs(d) <= ENUM_DELTA_MAX}
    if not small:
        return None
    tot = sum(small.values())
    top, cnt = max(small.items(), key=lambda kv: kv[1])
    if tot < 8 or cnt / tot < UNIFORM_SHARE:
        return None
    lo = r.get("lowest_by_delta", {}).get(top)
    return (
        f"ENUM RENUMBER: {cnt} diffs of {top:+d}"
        + (f", lowest {lo} -> {lo + top}" if lo is not None else "")
    )


def verdict(r):
    """One line naming the most likely drift shape, and what to do next."""
    if r["changed"] == 0:
        return "unchanged"
    sd = r["size_delta"]
    nonzero = {k: v for k, v in sd.items() if k != 0}
    total = sum(sd.values()) or 1
    enum = enum_signal(r)

    # constant non-zero size delta => a field of exactly that width moved in/out
    if nonzero and len(nonzero) == 1:
        d = next(iter(nonzero))
        if nonzero[d] / total >= UNIFORM_SHARE:
            what = "ADDED" if d > 0 else "REMOVED"
            base = f"FIELD {what}: every changed record is {d:+d} B -> a {abs(d)}-byte field"
            return base + (f"  ||  {enum}" if enum else "")

    if not nonzero:
        if enum:
            return enum + "  (variant added/removed -- fix the enum, skip IDA)"
        return "same-size value changes (data edit, or a field changed type in place)"

    base = (
        "STRUCTURAL: many distinct size deltas -> use ruststruct.py shift profile"
        if len(nonzero) > 6
        else f"mixed size deltas {sorted(nonzero)} -> variable-length field (CArray/CString) drifted"
    )
    return base + (f"  ||  {enum}" if enum else "")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("tables", nargs="*", help="table basenames; default = all in both dirs")
    ap.add_argument("--old", default=OLD_DEFAULT)
    ap.add_argument("--new", default=NEW_DEFAULT)
    ap.add_argument("--json", metavar="PATH", help="also write machine-readable results")
    ap.add_argument("--all", action="store_true", help="list unchanged tables too")
    args = ap.parse_args()

    for d in (args.old, args.new):
        if not os.path.isdir(d):
            sys.exit(f"not a directory: {d}")

    o_t, n_t = tables_in(args.old), tables_in(args.new)
    todo = sorted(args.tables) if args.tables else sorted(o_t & n_t)

    print(f"OLD {args.old}\nNEW {args.new}")
    only_new, only_old = sorted(n_t - o_t), sorted(o_t - n_t)
    if only_new:
        print(f"\n★ tables ONLY IN NEW ({len(only_new)}): {', '.join(only_new)}")
    if only_old:
        print(f"★ tables ONLY IN OLD ({len(only_old)}): {', '.join(only_old)}")

    results, errors = {}, []
    for t in todo:
        a, b = load(args.old, t), load(args.new, t)
        if isinstance(a, Exception) or isinstance(b, Exception):
            errors.append((t, a if isinstance(a, Exception) else b))
            continue
        results[t] = compare(a, b)

    # Most-changed first: that is the patch-day work queue.
    ranked = sorted(
        results.items(),
        key=lambda kv: (kv[1]["changed"] / max(1, kv[1]["compared"]), kv[1]["changed"]),
        reverse=True,
    )
    print(f"\n{'table':<34} {'recs old->new':>16} {'changed':>12}  verdict")
    print("-" * 118)
    shown = 0
    for t, r in ranked:
        if r["changed"] == 0 and not args.all:
            continue
        shown += 1
        pct = 100.0 * r["changed"] / max(1, r["compared"])
        recs_s = f"{r['old_records']}->{r['new_records']}"
        flag = " [IDX]" if r["match_mode"] == "index" else ""
        print(f"{t:<34} {recs_s:>16} {r['changed']:>7} {pct:5.1f}%  {verdict(r)}{flag}")

    unchanged = sum(1 for _, r in results.items() if r["changed"] == 0)
    print("-" * 118)
    print(f"{shown} table(s) drifted, {unchanged} unchanged, {len(errors)} unreadable")
    if any(r["match_mode"] == "index" for _, r in results.items()):
        print("[IDX] = keys did not line up (remapped/reordered); matched by index, treat as approximate")
    for t, e in errors:
        print(f"  UNREADABLE {t}: {e}")

    if args.json:
        with open(args.json, "w", encoding="utf-8") as f:
            json.dump({"old": args.old, "new": args.new, "tables": results}, f, indent=1)
        print(f"wrote {args.json}")


if __name__ == "__main__":
    main()
