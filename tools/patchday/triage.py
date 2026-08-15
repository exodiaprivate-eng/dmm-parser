"""Triage every drifted table at once: walk OLD (1.15) and NEW (1.16) fixtures
with the table's own Rust struct and report where each stops.

A table that walks 100% on OLD and 0% on NEW has a clean 1.16 structural drift
and the stop-field names it. A table that already fails on OLD was broken before
this patch and needs its own RE, not a patch-diff.
"""
import sys, glob, collections, os
sys.path.insert(0, '.')
from ruststruct import parse_structs, Walker
from rec import recs

SRC = r"C:\Users\justi\Desktop\Project\dmm-parser\src"
OLD = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-7-16"
NEW = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-1"
COMMON = glob.glob(SRC + r"\binary\variants\*.rs") + [SRC + r"\binary\types.rs"]

TABLES = [l.split("|") for l in sys.stdin.read().strip().split("\n") if "|" in l]

def run(table, fixture, root, limit=200):
    files = [os.path.join(SRC, "tables", table, "info.rs")] + COMMON
    S = parse_structs(files)
    if root not in S:
        return None, "root struct %r not parsed" % root
    res = {}
    for tag, D in (("old", OLD), ("new", NEW)):
        try:
            B = recs(D, fixture)[:limit]
        except Exception as e:
            res[tag] = (0, 0, {"<no fixture>": 1}); continue
        ok = 0; stops = collections.Counter()
        for k, rec in B:
            w = Walker(S)
            try:
                if w.walk(rec, root) == len(rec): ok += 1
                else: stops["<len mismatch>"] += 1
            except Exception as e:
                stops[str(e).split(":")[0]] += 1
        res[tag] = (ok, len(B), stops)
    return res, None

print("%-30s %-12s %-12s  %s" % ("TABLE", "1.15", "1.16", "TOP STOP ON 1.16"))
for table, fixture, root in TABLES:
    res, err = run(table, fixture, root)
    if err:
        print("%-30s %s" % (table, err)); continue
    (oo, ot, _), (no, nt, ns) = res["old"], res["new"]
    top = ns.most_common(1)[0][0] if ns else "-"
    print("%-30s %-12s %-12s  %s" % (table, "%d/%d" % (oo, ot), "%d/%d" % (no, nt), top[:70]))
