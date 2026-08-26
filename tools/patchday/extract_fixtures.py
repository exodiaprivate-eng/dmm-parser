"""Extract vanilla table fixtures from the UNMOUNTED game. One script, every patch.

Replaces `dmm_probes/extract_fixtures_<date>.py` — five near-identical copies, each made by
pasting the last one and editing two paths and a version guard. That pattern is the thing the
runbook warns about: the copies drift, and on 2026-08-25 the newest one carried

    if paver[:4] != bytes.fromhex('01001200'): sys.exit("not a 1.18.x build")

which would have REFUSED to run on 2.00.00 — the first major bump since the parser was
written. There was also no copy at all for `2026-8-17`, the set everything is currently
diffed against.

Three things this does that a hardcoded copy cannot:

  * **Refuses a mounted game.** A mounted game returns INJECTED bytes; a fixture set captured
    from one reports our own mods as vanilla drift on the next patch diff, which is worse than
    having no set at all.
  * **Refuses to capture the SAME build twice.** The old guard asserted a hardcoded version,
    which breaks on every bump. The real intent was "don't mislabel the set" — so this compares
    the live paver against the PRIOR set's recorded paver and stops if they match, meaning the
    patch has not actually landed yet. Version-agnostic, and it catches the mistake that
    actually happens.
  * **Records the paver INTO the output**, so every set is self-describing rather than
    identified by a folder name someone typed.

Usage
    python tools/patchday/extract_fixtures.py                 # to today's date, vs the newest set
    python tools/patchday/extract_fixtures.py --out 2026-8-26 --prior 2026-8-17
    python tools/patchday/extract_fixtures.py --force         # capture even if paver is unchanged
"""
import argparse
import os
import sys
import time

sys.path.insert(0, os.path.abspath("python"))
import dmm_parser as dp  # noqa: E402

GAME = r"D:\SteamLibrary\steamapps\common\Crimson Desert"
BIN = "gamedata/binary__/client/bin"
ROOT = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb"
PAVER_NOTE = "paver.hex"


def read_paver():
    p = os.path.join(GAME, "meta", "0.paver")
    try:
        return open(p, "rb").read()[:16]
    except OSError as e:
        sys.exit(f"cannot read {p}: {e}")


def describe(paver):
    """major.minor.patch — three little-endian u16s. A hotfix moves only the THIRD."""
    if len(paver) < 6:
        return "?"
    u = [int.from_bytes(paver[i:i + 2], "little") for i in (0, 2, 4)]
    return f"{u[0]}.{u[1]:02d}.{u[2]:02d}"


def newest_set():
    if not os.path.isdir(ROOT):
        return None
    dirs = [d for d in os.listdir(ROOT) if os.path.isdir(os.path.join(ROOT, d))]
    if not dirs:
        return None
    return max(dirs, key=lambda d: os.path.getmtime(os.path.join(ROOT, d)))


def group_tables():
    """Every table the LIVE build ships in group 0008 — not just the ones we knew about.

    The extraction walks the PRIOR set's file list so the suites compare like-for-like, which
    means a table ADDED by the patch would never be captured and never be missed. Consolidating
    post-launch content into a base game is exactly the shape that adds one.
    """
    try:
        pamt = dp.parse_pamt_file(os.path.join(GAME, "0008", "0.pamt"))
    except Exception:  # noqa: BLE001
        return None
    out = set()
    for d in pamt["directories"]:
        if d["path"].replace("\\", "/").lower() != BIN:
            continue
        for f in (d.get("files") or []):
            if f["name"].lower().endswith((".pabgb", ".pabgh")):
                out.add(f["name"])
    return out


def assert_unmounted():
    bad = [d for d in os.listdir(GAME)
           if d.lower().startswith("dmm") and os.path.isdir(os.path.join(GAME, d))]
    if bad:
        sys.exit(
            "REFUSING: the game looks MOUNTED — found " + ", ".join(sorted(bad)) +
            "\n  Unmount in DMM first. A fixture set captured from a mounted game reports our"
            "\n  own mods as vanilla drift on the next patch diff."
        )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=None, help="output set name (default: today)")
    ap.add_argument("--prior", default=None, help="set to diff against (default: newest)")
    ap.add_argument("--force", action="store_true",
                    help="capture even when the paver matches the prior set")
    a = ap.parse_args()

    prior_name = a.prior or newest_set()
    if not prior_name:
        sys.exit(f"no existing fixture set under {ROOT} to mirror the table list from")
    prior = os.path.join(ROOT, prior_name)
    # Matches the existing naming: 2026-8-17, not 2026-08-17. Built from the
    # parts rather than strftime, because the no-pad flag differs by platform.
    y, mo, d = time.localtime()[:3]
    out_name = a.out or f"{y}-{mo}-{d}"
    out = os.path.join(ROOT, out_name)

    paver = read_paver()
    print(f"game   {GAME}")
    print(f"paver  {paver[:8].hex()}  = {describe(paver)}")
    print(f"prior  {prior_name}")
    print(f"out    {out_name}\n")
    assert_unmounted()

    prior_paver_path = os.path.join(prior, PAVER_NOTE)
    if os.path.isfile(prior_paver_path):
        was = bytes.fromhex(open(prior_paver_path).read().strip())
        if was[:6] == paver[:6] and not a.force:
            sys.exit(
                f"REFUSING: the live game is still {describe(paver)}, the same build "
                f"'{prior_name}' was captured from.\n"
                "  The patch has not landed yet — capturing now would mislabel the set.\n"
                "  Pass --force if you really mean to re-capture the same build."
            )
    else:
        print(f"note: '{prior_name}' predates paver recording, so the same-build check is off\n")

    if os.path.abspath(out) == os.path.abspath(prior):
        sys.exit("REFUSING: output and prior are the same directory")
    os.makedirs(out, exist_ok=True)

    names = sorted(n for n in os.listdir(prior) if n != PAVER_NOTE)
    ok, miss, err = 0, [], []
    for name in names:
        try:
            data = bytes(dp.extract_file(GAME, "0008", BIN, name))
            open(os.path.join(out, name), "wb").write(data)
            ok += 1
        except Exception as e:  # noqa: BLE001
            msg = str(e)
            if "not found" in msg.lower() or "no such" in msg.lower():
                miss.append(name)
            else:
                err.append((name, msg[:90]))

    open(os.path.join(out, PAVER_NOTE), "w").write(paver.hex())
    print(f"extracted {ok}/{len(names)} into {out}")
    if miss:
        print(f"\nMISSING from the patched game ({len(miss)}) "
              f"— removed, renamed, or moved to another group:")
        for m in miss:
            print("  -", m)
    if err:
        print(f"\nERRORS ({len(err)}):")
        for n, m in err:
            print(f"  - {n}: {m}")

    # ── what actually drifted ────────────────────────────────────────────────
    print(f"\n--- drift vs {prior_name} ({describe(bytes.fromhex(open(prior_paver_path).read().strip())) if os.path.isfile(prior_paver_path) else 'unknown build'}) ---")
    same, changed = 0, []
    for name in names:
        pa, pb = os.path.join(prior, name), os.path.join(out, name)
        if not (os.path.exists(pa) and os.path.exists(pb)):
            continue
        da, db = open(pa, "rb").read(), open(pb, "rb").read()
        if da == db:
            same += 1
        else:
            changed.append((name, len(da), len(db)))
    print(f"identical: {same}   changed: {len(changed)}")
    resized = [c for c in changed if c[1] != c[2]]
    for n, sa, sb in sorted(changed, key=lambda c: -abs(c[2] - c[1])):
        tag = "same size (value edits)" if sa == sb else f"*** SIZE CHANGED {sb - sa:+,}"
        print(f"  {n:<36} {sa:>11,} -> {sb:>11,}  {tag}")

    print(f"\n{len(resized)} table(s) changed SIZE — those are the layout-drift candidates.")

    # ★ What the build ships that we did not ask for. Captured too, so a table added by
    # the patch is not invisible for a whole cycle.
    live = group_tables()
    if live is None:
        print("\n⚠ could not read the 0008 PAMT — NEW tables were not checked for")
    else:
        fresh = sorted(live - set(names))
        if fresh:
            print(f"\n★ {len(fresh)} table(s) in this build that the prior set never had:")
            for n in fresh:
                try:
                    data = bytes(dp.extract_file(GAME, "0008", BIN, n))
                    open(os.path.join(out, n), "wb").write(data)
                    print(f"  + {n:<36} {len(data):>11,}  captured")
                except Exception as e:  # noqa: BLE001
                    print(f"  + {n:<36} could not extract: {str(e)[:50]}")
        else:
            print("\nno new tables in this build")
    print("\nnext:")
    print(f"  python tools/patchday/bytediff.py --old {prior} --new {out}")
    print(f"  set DMM_PARSER_PABGB_DIR={out} && cargo test --lib      # vs the 670/20 baseline")
    print(f"  # then add '{out_name}' to the TOP of FALLBACK_DIRS in src/testenv.rs")


main()
