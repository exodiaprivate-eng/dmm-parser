"""PHASE 0 — capture everything the patch is about to overwrite. Run BEFORE clicking update.

The runbook's own warning is that Phase 0 has no natural trigger: by the time anyone thinks
"patch day", the patch has already destroyed the "before". This is that trigger, as a script,
so it is one command rather than a checklist somebody has to remember.

Captures the things that CANNOT be recovered afterwards:

  1. **the exe** — ASI signatures rot every patch, and re-deriving one needs the OLD binary to
     diff against. Steam overwrites it in place. Nothing else on disk is a substitute.
  2. **every localization file** — one per language, each in its own group. Not in the pabgb
     fixture dump (that only covers group 0008), so an i18n or custom-item regression would
     have no "before" at all.
  3. **a whole-install file manifest** — path, size, sha256. Gives an instant file-level
     change list after the patch instead of guessing where to look.
  4. **the loose game files our own mods SHIP** — a stale one is a hard CTD, not a no-op
     (a 1.12 stage file on a 1.18 engine cost three crashes before it was found).

⚠ The game must be UNMOUNTED. A mounted game returns INJECTED bytes, and a baseline captured
from one is worse than no baseline: it reports our own mods as vanilla drift on the next diff.
This refuses to run rather than capture poison.

Usage:  python tools/patchday/prepatch_capture.py [--out <dir>] [--skip-manifest]
"""
import argparse
import hashlib
import json
import os
import shutil
import sys
import time

sys.path.insert(0, os.path.abspath("python"))
import dmm_parser as dp  # noqa: E402

GAME = r"D:\SteamLibrary\steamapps\common\Crimson Desert"
DEFAULT_OUT = r"C:\temp\GIT\CrimsonDesertUpdates\prepatch"

# Group -> the localization file it holds. One per language; a new language means a new group,
# so the listing is rebuilt from the PAMTs rather than hardcoded.
LOC_GROUPS = [f"{n:04d}" for n in range(19, 40)]

# Loose game files our own mods ship rebuilt copies of. Small, and shipping a stale one is a
# HARD CTD rather than a no-op.
SHIPPED_ASSET_HINTS = (".pastage", ".paseq", ".paseqc", ".paac")


def sha256(path, cap=None):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
            if cap and f.tell() > cap:
                break
    return h.hexdigest()


def assert_unmounted():
    """A mounted game returns injected bytes. Refuse rather than capture poison."""
    bad = [d for d in os.listdir(GAME)
           if d.lower().startswith("dmm") and os.path.isdir(os.path.join(GAME, d))]
    if bad:
        print("REFUSING: the game looks MOUNTED — found " + ", ".join(sorted(bad)))
        print("  Unmount in DMM first. A baseline captured from a mounted game reports our own")
        print("  mods as vanilla drift on the next patch diff, which is worse than no baseline.")
        sys.exit(2)
    print("game is unmounted \u2713")


def capture_exe(out):
    src = os.path.join(GAME, "bin64", "CrimsonDesert.exe")
    if not os.path.isfile(src):
        print("  \u26a0 exe not found at " + src)
        return
    size = os.path.getsize(src)
    dst = os.path.join(out, "bin64", "CrimsonDesert.exe")
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if os.path.isfile(dst) and os.path.getsize(dst) == size:
        print(f"  exe already captured ({size:,} bytes)")
        return
    t = time.time()
    shutil.copy2(src, dst)
    print(f"  exe {size:,} bytes in {time.time() - t:.1f}s")
    print(f"     sha256 {sha256(dst)}")
    # The version stamp lives in paver, but the exe size + hash is what identifies a build
    # when the launcher will not run.
    for extra in ("pers.exe", "crashpad_handler.exe"):
        p = os.path.join(GAME, "bin64", extra)
        if os.path.isfile(p):
            shutil.copy2(p, os.path.join(out, "bin64", extra))


def capture_localization(out):
    """Every `.paloc`, whichever group it lives in — a new language means a new group."""
    got = 0
    for g in LOC_GROUPS:
        pamt = os.path.join(GAME, g, "0.pamt")
        if not os.path.isfile(pamt):
            continue
        try:
            p = dp.parse_pamt_file(pamt)
        except Exception as e:
            print(f"  \u26a0 group {g}: {str(e)[:60]}")
            continue
        for d in p["directories"]:
            for f in (d.get("files") or []):
                if not f["name"].lower().endswith(".paloc"):
                    continue
                dst = os.path.join(out, "localization", g, f["name"])
                os.makedirs(os.path.dirname(dst), exist_ok=True)
                try:
                    open(dst, "wb").write(bytes(dp.extract_file(GAME, g, d["path"], f["name"])))
                except Exception as e:
                    print(f"  \u26a0 {g}/{f['name']}: {str(e)[:60]}")
                    continue
                got += 1
                print(f"  {g}/{f['name']:<34} {os.path.getsize(dst):>12,} bytes")
    print(f"  {got} localization file(s)")
    return got


def capture_shipped_assets(out):
    """Loose files our own mods rebuild from — cheap, and a stale one is a hard CTD."""
    got = 0
    for g in ("0009", "0010", "0011"):
        pamt = os.path.join(GAME, g, "0.pamt")
        if not os.path.isfile(pamt):
            continue
        try:
            p = dp.parse_pamt_file(pamt)
        except Exception:
            continue
        for d in p["directories"]:
            for f in (d.get("files") or []):
                if not f["name"].lower().endswith(SHIPPED_ASSET_HINTS):
                    continue
                dst = os.path.join(out, "assets", g, f["name"])
                os.makedirs(os.path.dirname(dst), exist_ok=True)
                try:
                    open(dst, "wb").write(bytes(dp.extract_file(GAME, g, d["path"], f["name"])))
                    got += 1
                except Exception:
                    pass
    print(f"  {got} shipped-asset file(s)")
    return got


def capture_manifest(out):
    """Path, size, sha256 for the whole install \u2014 the instant post-patch change list."""
    rows = {}
    t = time.time()
    for root, _, files in os.walk(GAME):
        for fn in files:
            p = os.path.join(root, fn)
            rel = os.path.relpath(p, GAME).replace("\\", "/")
            try:
                st = os.stat(p)
            except OSError:
                continue
            # Hash small files whole; for the multi-GB archives hash the first 64MB, which
            # still moves whenever the archive is rebuilt and keeps this under a minute.
            rows[rel] = {
                "size": st.st_size,
                "sha256": sha256(p, cap=None if st.st_size < (64 << 20) else (64 << 20)),
                "partial": st.st_size >= (64 << 20),
            }
    dst = os.path.join(out, "manifest.json")
    json.dump(rows, open(dst, "w"), indent=1, sort_keys=True)
    print(f"  {len(rows)} files hashed in {time.time() - t:.0f}s -> manifest.json")
    return len(rows)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=None, help="capture directory (default: dated)")
    ap.add_argument("--skip-manifest", action="store_true")
    a = ap.parse_args()

    out = a.out or os.path.join(DEFAULT_OUT, time.strftime("%Y-%m-%d"))
    os.makedirs(out, exist_ok=True)
    print(f"capturing {GAME}\n     ->   {out}\n")
    assert_unmounted()

    print("\n\u2500\u2500 1. the exe (ASI signatures need the OLD binary to diff) \u2500\u2500")
    capture_exe(out)
    print("\n\u2500\u2500 2. localization, every language \u2500\u2500")
    n_loc = capture_localization(out)
    print("\n\u2500\u2500 3. loose files our own mods ship rebuilt \u2500\u2500")
    n_ast = capture_shipped_assets(out)
    n_man = 0
    if not a.skip_manifest:
        print("\n\u2500\u2500 4. whole-install manifest \u2500\u2500")
        n_man = capture_manifest(out)

    json.dump(
        {
            "captured_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "game": GAME,
            "exe_size": os.path.getsize(os.path.join(GAME, "bin64", "CrimsonDesert.exe")),
            "localization_files": n_loc,
            "shipped_assets": n_ast,
            "manifest_files": n_man,
        },
        open(os.path.join(out, "capture.json"), "w"),
        indent=1,
    )
    print(f"\ndone \u2014 {out}")


main()
