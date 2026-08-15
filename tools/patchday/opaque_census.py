"""Per-table OPAQUE-RECORD census, old build vs new.

WHY THIS EXISTS (1.18.00): triaging by pass/FAIL SETS is not enough. `character_info`
failed `cargo test` on BOTH 1.17 and 1.18, so a set-diff filed it "pre-existing" — while
its opaque count had gone 232 -> 7226 (EVERY record). The V3 gate said OK because
blob-fallback round-trips byte-exact. Net effect: every characterinfo mod silently died
(Random Boss Encounters Reborn applied 4 of 1061 intents) and nothing flagged it.

⇒ SEVERITY, not just pass/fail. Run this every patch and diff the counts.
    python tools/patchday/opaque_census.py --old <dir> --new <dir>
"""
import argparse, re, subprocess, sys

import os, glob
EXE = os.path.join("target", "release", "examples", "verify_table.exe")

def stems(fixdir):
    return sorted(os.path.basename(p)[:-6]
                  for p in glob.glob(os.path.join(fixdir, "*.pabgb")))

def census(fixdir):
    """verify_table takes ONE stem per call; the release binary is already built,
    so invoke it directly rather than paying cargo's overhead 129 times."""
    if not os.path.exists(EXE):
        print(f"!! {EXE} missing — run: cargo build --release --example verify_table")
        sys.exit(1)
    env = {**os.environ, "DMM_PARSER_PABGB_DIR": fixdir}
    res = {}
    for stem in stems(fixdir):
        out = subprocess.run([EXE, stem], capture_output=True, text=True,
                             errors="replace", env=env)
        m = re.search(r"^(\w+) \(\w+\): (\d+) records, opaque\(_b64\)=(\d+)",
                      out.stdout + out.stderr, re.M)
        if m:
            res[m.group(1)] = (int(m.group(2)), int(m.group(3)))
    return res

ap = argparse.ArgumentParser()
ap.add_argument("--old", required=True); ap.add_argument("--new", required=True)
a = ap.parse_args()
old, new = census(a.old), census(a.new)
if not old or not new:
    print("!! verify_table produced no per-table lines — run it manually to check"); sys.exit(1)
print(f"{'table':38} {'old recs/opaque':>18}  {'new recs/opaque':>18}   verdict")
bad = 0
for t in sorted(set(old) | set(new)):
    o = old.get(t); n = new.get(t)
    if not o or not n: print(f"{t:38} {'-' if not o else o}  {'-' if not n else n}   ONLY ONE BUILD"); continue
    worse = n[1] > o[1]
    if worse: bad += 1
    if worse or o[1] != n[1]:
        print(f"{t:38} {o[0]:>8}/{o[1]:<9} {n[0]:>8}/{n[1]:<9}   "
              f"{'*** WORSE ***' if worse else 'improved'}")
print(f"\n{bad} table(s) got MORE opaque on the new build")
