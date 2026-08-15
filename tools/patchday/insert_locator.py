"""Locate the inserted bytes in a drifted table, generically.

Two views, because each is blind where the other sees:

  FROM START — right for fields near the head of the record.
  FROM END   — right for tail fields. Records carry a variable-length
               string_key, so absolute offsets differ per record while
               distance-from-end is stable. This is what pins a tail insert.

Inserts that land inside a run of zeros are genuinely ambiguous: the bytes
cannot say whether the new zero went before or after its neighbours. Those are
reported as a WINDOW, not a point (the 1.16 lesson: difflib slides inserts
inside zero runs and its offsets are not field boundaries). A record whose
inserted value is NON-ZERO pins the position exactly — those are called out.

    python insert_locator.py <table> [--n 6] [--hex 32]
"""
import argparse
import collections
import difflib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rec

OLD = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-7"
NEW = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-15"


def inserts(o, n):
    """All inserted byte-runs, as (old_offset, bytes)."""
    sm = difflib.SequenceMatcher(None, o, n, autojunk=False)
    out = []
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == "insert":
            out.append((i1, n[j1:j2]))
        elif tag == "replace":
            # a replace of unequal length is an insert plus changed content
            if (j2 - j1) > (i2 - i1):
                out.append((i1, n[j1:j2]))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("table")
    ap.add_argument("--n", type=int, default=6)
    ap.add_argument("--hex", type=int, default=32)
    ap.add_argument("--old", default=OLD)
    ap.add_argument("--new", default=NEW)
    a = ap.parse_args()

    base = a.table.replace("_", "")
    ob = dict(rec.recs(a.old, base))
    nb = dict(rec.recs(a.new, base))
    shared = [k for k in ob if k in nb]
    print(f"{a.table}: old {len(ob)} recs, new {len(nb)} recs, {len(shared)} shared\n")

    from_end = collections.Counter()
    widths = collections.Counter()
    pinned = []          # records where the inserted bytes are NOT all zero
    rows = []
    for k in shared:
        o, n = ob[k], nb[k]
        ins = inserts(o, n)
        rows.append((k, o, n, ins))
        for off, blob in ins:
            from_end[(len(o) - off, len(blob))] += 1
            widths[len(blob)] += 1
            if any(blob):
                pinned.append((k, len(o) - off, blob))

    print("=== insert sites, measured FROM THE END of the old record ===")
    print("bytes-from-end | width | count")
    for (fe, w), c in from_end.most_common(20):
        print(f"  {fe:<14} | {w:<5} | x{c}")

    print(f"\ninserted-run widths: {dict(widths)}")

    print(f"\n=== records where the inserted bytes are NON-ZERO (position is exact) ===")
    if not pinned:
        print("  none — every inserted byte is 0x00, so position is ambiguous "
              "within its zero run. Trust the field-name oracle for ordering.")
    for k, fe, blob in pinned[:15]:
        print(f"  key 0x{k:X}  {fe} from end  <- {blob.hex(' ')}")

    print(f"\n=== first {a.n} records ===")
    for k, o, n, ins in rows[: a.n]:
        print(f"\n key 0x{k:X}  {len(o)} -> {len(n)}   inserts: "
              + ", ".join(f"@{off}(={len(o)-off} from end) {b.hex(' ')}"
                          for off, b in ins))
        for off, _ in ins[:2]:
            lo = max(0, off - a.hex // 2)
            print(f"   OLD ..{off}: {o[lo:off].hex(' ')} || {o[off:off + a.hex].hex(' ')}")
            j = off + sum(len(b) for p, b in ins if p < off)
            print(f"   NEW ..{off}: {n[lo:j].hex(' ')} || {n[j:j + a.hex].hex(' ')}")


if __name__ == "__main__":
    main()
