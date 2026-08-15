"""Diff the field-name oracle between two builds — the fastest possible patch-day
answer to "what field was added?".

The Korean reader strings are plain UTF-8 in the binary, so the scan is
ISA-independent: a Mach-O/ARM64 Mac build and a PE/x64 Windows build can be
compared directly. Field NAMES are platform-independent; only address order is
not portable, and we do not rely on it here.

    python oracle_diff.py                         # every table that changed
    python oracle_diff.py TribeInfo VehicleInfo   # just these
"""
import argparse
import sys

from korean_fields_win import harvest

MAC_OLD = r"C:\Users\justi\Desktop\Project\IDA Professional 9.0\1.17\CrimsonDesert.exe"
WIN_NEW = r"D:\SteamLibrary\steamapps\common\Crimson Desert\bin64\CrimsonDesert.exe"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("tables", nargs="*")
    ap.add_argument("--old", default=MAC_OLD)
    ap.add_argument("--new", default=WIN_NEW)
    a = ap.parse_args()

    old = harvest(a.old)
    new = harvest(a.new)
    print(f"OLD {a.old}\n    {len(old)} tables / {sum(len(v) for v in old.values())} fields")
    print(f"NEW {a.new}\n    {len(new)} tables / {sum(len(v) for v in new.values())} fields\n")
    if not old:
        print("!! no reader strings found in OLD — wrong file or stripped build")
        sys.exit(1)

    want = {t.lower() for t in a.tables}
    changed = 0
    for t in sorted(set(old) | set(new)):
        if want and t.lower() not in want:
            continue
        o, n = old.get(t, []), new.get(t, [])
        added = [f for f in n if f not in o]
        removed = [f for f in o if f not in n]
        if not added and not removed:
            continue
        changed += 1
        print(f"=== {t}  ({len(o)} -> {len(n)} fields) ===")
        for f in added:
            # index in the NEW reader order = where it sits in the struct
            print(f"   + {f}   (new idx {n.index(f)}, after "
                  f"{n[n.index(f) - 1] if n.index(f) else '<start>'})")
        for f in removed:
            print(f"   - {f}")
        print()
    print(f"{changed} table(s) changed field set")


if __name__ == "__main__":
    main()
