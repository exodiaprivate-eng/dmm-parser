#!/usr/bin/env python3
"""
Snapshot SHA256 hashes of key Crimson Desert game files.
Run BEFORE and AFTER a game update to instantly identify what changed.

Usage:
  # Take baseline (run this now, before the update):
  python game_hash_snapshot.py "F:\\Program\\Steam\\steamapps\\common\\Crimson Desert"

  # Diff after the update drops:
  python game_hash_snapshot.py "F:\\Program\\Steam\\steamapps\\common\\Crimson Desert" --diff hashes_pre_patch.json
"""

import sys
import json
import hashlib
from pathlib import Path
from datetime import datetime


GAME_DIR_DEFAULT = r"D:\SteamLibrary\steamapps\common\Crimson Desert"


def hash_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def snapshot(game_dir: Path) -> dict:
    results = {}

    # All .pabgb table files
    gamedata = game_dir / "gamedata"
    if gamedata.exists():
        for f in sorted(gamedata.glob("*.pabgb")):
            rel = f.relative_to(game_dir).as_posix()
            results[rel] = {"sha256": hash_file(f), "size": f.stat().st_size}
        # Also grab any other formats that show up in gamedata
        for ext in ("*.papgt", "*.pamt", "*.paz", "*.paloc"):
            for f in sorted(gamedata.glob(ext)):
                rel = f.relative_to(game_dir).as_posix()
                if rel not in results:
                    results[rel] = {"sha256": hash_file(f), "size": f.stat().st_size}

    # PAPGT / PAMT / PAZ from numbered group directories
    for group_dir in sorted(game_dir.iterdir()):
        if not group_dir.is_dir():
            continue
        if not group_dir.name.isdigit():
            continue
        for fname in ("0.papgt", "0.pamt", "0.paz"):
            f = group_dir / fname
            if f.exists():
                rel = f.relative_to(game_dir).as_posix()
                results[rel] = {"sha256": hash_file(f), "size": f.stat().st_size}

    # meta/ — the master group tree, path/hash table and version stamp.
    # NOT picked up by the loop above: it only visits DIGIT-named directories,
    # and "meta" is not one. A 1.16 hotfix changed these while the snapshot
    # reported nothing, so a container-format move could pass unnoticed.
    # 0.papgt = group tree, 0.pathc = path/hash table, 0.paver = version.
    meta = game_dir / "meta"
    if meta.is_dir():
        for f in sorted(meta.iterdir()):
            if f.is_file() and f.suffix in (".papgt", ".pathc", ".paver", ".pamt"):
                rel = f.relative_to(game_dir).as_posix()
                results[rel] = {"sha256": hash_file(f), "size": f.stat().st_size}

    # Main executable (tracks game version without needing a version file).
    # ⚠ It lives in bin64/, NOT the game root — this looked only at the root, so
    # the exe was never in the snapshot at all. The 1.16 hotfix shrank it by
    # 10.5 MB and the diff report stayed silent, which matters because every ASI
    # mod's AOB signatures are pinned to that binary.
    for sub in ("bin64", "."):
        for exe_name in ("CrimsonDesert.exe", "CrimsonDesert_BE.exe", "CrimsonDesert_Steam.exe"):
            exe = game_dir / sub / exe_name
            if exe.is_file():
                rel = exe.resolve().relative_to(game_dir.resolve()).as_posix()
                if rel not in results:
                    results[rel] = {"sha256": hash_file(exe), "size": exe.stat().st_size}

    return results


def diff_snapshots(before: dict, after: dict):
    all_keys = set(before) | set(after)
    changed = []
    added = []
    removed = []
    unchanged_count = 0

    for key in sorted(all_keys):
        if key not in before:
            added.append(key)
        elif key not in after:
            removed.append(key)
        elif before[key]["sha256"] != after[key]["sha256"]:
            size_diff = after[key]["size"] - before[key]["size"]
            changed.append((key, before[key]["size"], after[key]["size"], size_diff))
        else:
            unchanged_count += 1

    print(f"\n{'='*65}")
    print(f"  GAME UPDATE DIFF REPORT")
    print(f"{'='*65}")
    print(f"  Changed : {len(changed)}")
    print(f"  Added   : {len(added)}")
    print(f"  Removed : {len(removed)}")
    print(f"  Same    : {unchanged_count}")
    print(f"{'='*65}")

    if changed:
        print(f"\n[CHANGED]")
        for key, old_sz, new_sz, diff in changed:
            sign = "+" if diff >= 0 else ""
            print(f"  {key}")
            print(f"    {old_sz:>12,} -> {new_sz:>12,} bytes  ({sign}{diff:,})")

    if added:
        print(f"\n[ADDED — new files]")
        for key in added:
            print(f"  {key}  ({after[key]['size']:,} bytes)")

    if removed:
        print(f"\n[REMOVED — deleted files]")
        for key in removed:
            print(f"  {key}")

    if not changed and not added and not removed:
        print("\n  No game files changed — update did not touch parser-relevant files.")

    # Highlight the files that matter most for the parser
    parser_critical = [k for k, *_ in changed if k.endswith(".pabgb")]
    container_changed = [k for k, *_ in changed if any(k.endswith(e) for e in (".papgt", ".pamt", ".paz"))]

    if parser_critical:
        print(f"\n[PARSER ACTION REQUIRED — table schemas may have changed]")
        for k in parser_critical:
            print(f"  {k}")

    if container_changed:
        print(f"\n[CONTAINER FORMAT CHECK — verify PAPGT/PAMT/PAZ roundtrip]")
        for k in container_changed:
            print(f"  {k}")

    print()


def main():
    args = sys.argv[1:]

    if not args:
        game_dir = Path(GAME_DIR_DEFAULT)
    else:
        game_dir = Path(args[0])
        args = args[1:]

    if not game_dir.exists():
        print(f"Error: game directory not found: {game_dir}")
        print(f"Edit GAME_DIR_DEFAULT at the top of this script or pass the path as the first argument.")
        sys.exit(1)

    if "--diff" in args:
        diff_idx = args.index("--diff")
        before_path = Path(args[diff_idx + 1])
        if not before_path.exists():
            print(f"Error: baseline snapshot not found: {before_path}")
            sys.exit(1)
        with open(before_path) as f:
            before_data = json.load(f)
        print(f"Hashing current game files in: {game_dir}")
        after_data = snapshot(game_dir)
        diff_snapshots(before_data["files"], after_data)
    else:
        out_path = Path(args[0]) if args else Path("hashes_pre_patch.json")
        print(f"Hashing: {game_dir}")
        data = snapshot(game_dir)
        output = {
            "timestamp": datetime.now().isoformat(),
            "game_dir": str(game_dir),
            "file_count": len(data),
            "files": data,
        }
        with open(out_path, "w") as f:
            json.dump(output, f, indent=2)
        print(f"Saved {len(data)} file hashes -> {out_path}")
        print()
        print(f"After the update drops, run:")
        print(f'  python game_hash_snapshot.py "{game_dir}" --diff {out_path}')


if __name__ == "__main__":
    main()
