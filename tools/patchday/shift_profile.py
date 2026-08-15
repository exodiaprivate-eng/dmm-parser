"""Shift profile: where does OLD stop lining up with NEW, and by how much?

difflib is O(n^2)-ish and gets unusable on big records (interaction_info /
store_info hang it). It is also too clever — it will happily slide an insert
inside a zero run.

This does the blunt, exact thing instead: for each offset i in the OLD record,
find the shift d such that OLD[i:i+W] == NEW[i+d:i+d+W]. d starts at 0 and only
ever increases, so the offsets where d steps up ARE the insert points, and the
step size is the inserted width. Anchored on a real matching window, so a zero
run cannot fake a match unless the window is entirely zeros — which is why W is
generous and why the report says how many records agree.

    python shift_profile.py <table> [--w 16] [--n 5]
"""
import argparse
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rec

OLD = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-7"
NEW = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-15"


def profile(o, n, w):
    """[(old_offset, added_width)] — where the alignment steps."""
    steps = []
    d = 0
    total = len(n) - len(o)
    i = 0
    while i + w <= len(o):
        if o[i:i + w] == n[i + d:i + d + w]:
            i += 1
            continue
        # alignment broke — find the new shift
        for nd in range(d + 1, total + 1):
            if o[i:i + w] == n[i + nd:i + nd + w]:
                steps.append((i, nd - d))
                d = nd
                break
        else:
            return steps, False       # content differs here, not just a shift
        i += 1
    return steps, d == total


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("table")
    ap.add_argument("--w", type=int, default=16, help="match window")
    ap.add_argument("--n", type=int, default=5)
    ap.add_argument("--delta", type=int, help="only records with this size delta")
    a = ap.parse_args()

    base = a.table.replace("_", "")
    ob = dict(rec.recs(OLD, base))
    nb = dict(rec.recs(NEW, base))
    shared = [k for k in ob if k in nb]

    pats = collections.Counter()
    clean = 0
    ex = []
    for k in shared:
        o, n = ob[k], nb[k]
        if len(n) <= len(o):
            continue
        if a.delta is not None and len(n) - len(o) != a.delta:
            continue
        steps, ok = profile(o, n, a.w)
        if ok:
            clean += 1
        # Even when content ALSO changed, the steps found before the first
        # content divergence are still real — keep them and say so.
        if steps:
            pats[tuple(steps)] += 1
            if len(ex) < a.n:
                ex.append((k, o, n, steps))

    print(f"{a.table}: {clean} records explained by inserts ALONE; "
          f"{sum(pats.values())} yielded at least one reliable step\n")
    print("insert pattern [(old_offset, width), ...] -> count")
    for p, c in pats.most_common(12):
        print(f"  {list(p)}  x{c}")

    for k, o, n, steps in ex:
        print(f"\nkey 0x{k:X}  {len(o)} -> {len(n)}   steps {steps}")
        d = 0
        for off, wdt in steps:
            print(f"   @old {off} (= {len(o)-off} from end), +{wdt}: "
                  f"{n[off+d:off+d+wdt].hex(' ')}")
            print(f"      old context: {o[max(0,off-8):off].hex(' ')} || "
                  f"{o[off:off+8].hex(' ')}")
            d += wdt


if __name__ == "__main__":
    main()
