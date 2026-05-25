#!/usr/bin/env python3
"""
Rewrite all table roundtrip tests to use crate::testenv::resolve() instead of
hardcoded const paths.  Run from the repo root:

    python scripts/fix_test_paths.py [--dry-run]

What it does per file:
  1. Replaces `const PABGB_PATH / PABGB: &str = r"..."` with
     `fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("name.pabgb") }`
  2. Removes any companion `const PABGH_PATH / PABGH: &str = ...` line.
  3. Rewrites every use of those consts in test bodies:
       fs::read(PABGB_PATH)   → fs::read(pabgb_path())
       &PABGB_PATH.replace(".pabgb", ".pabgh")
         → &pabgb_path().with_extension("pabgh").to_string_lossy()
       load_pabgh_offsets(PABGH_PATH) / load_pabgh_offsets(&PABGB_PATH.replace(...))
         → load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy())
       eprintln!("...{}", PABGB_PATH)  → eprintln!("SKIP: fixture not found")
       eprintln!("...{}", PABGH_PATH)  → eprintln!("SKIP: pabgh not found")
"""

import sys
import re
from pathlib import Path

DRY_RUN = "--dry-run" in sys.argv

ROOT = Path(__file__).parent.parent / "src"

# ── regex patterns ─────────────────────────────────────────────────────────────

# Matches both single-line and two-line const declarations (some wrapped):
#   const PABGB_PATH: &str = r"...pabgb";
#   const PABGB_PATH: &str =
#       r"...pabgb";
PABGB_CONST_RE = re.compile(
    r'[ \t]*const\s+(PABGB_PATH|PABGB)\s*:\s*&str\s*=\s*\n?[ \t]*r?"([^"]*\.pabgb)";\s*\n?',
    re.MULTILINE,
)

PABGH_CONST_RE = re.compile(
    r'[ \t]*const\s+(PABGH_PATH|PABGH)\s*:\s*&str\s*=\s*\n?[ \t]*r?"[^"]*\.pabgh";\s*\n?',
    re.MULTILINE,
)

PABGB_NAMES = ["PABGB_PATH", "PABGB"]
PABGH_NAMES = ["PABGH_PATH", "PABGH"]

# ── helpers ────────────────────────────────────────────────────────────────────

def extract_filename(path_str: str) -> str:
    """Extract just the filename from any path string."""
    # Normalise separators and grab the last segment
    return path_str.replace("\\", "/").split("/")[-1]


def apply_body_replacements(content: str, pabgb_const: str) -> str:
    """Replace all uses of PABGB_PATH / PABGB / PABGH_PATH / PABGH in test bodies."""

    # 1. fs::read(PABGB_PATH) / fs::read(PABGB)
    for cn in PABGB_NAMES:
        content = content.replace(f"std::fs::read({cn})", "std::fs::read(pabgb_path())")

    # 2a. &PABGB_PATH.replace(".pabgb", ".pabgh")
    for cn in PABGB_NAMES:
        content = content.replace(
            f'&{cn}.replace(".pabgb", ".pabgh")',
            '&pabgb_path().with_extension("pabgh").to_string_lossy()',
        )

    # 2b. load_pabgh_offsets(&PABGB_PATH.replace(".pabgb", ".pabgh"))
    for cn in PABGB_NAMES:
        content = content.replace(
            f'load_pabgh_offsets(&{cn}.replace(".pabgb", ".pabgh"))',
            'load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy())',
        )

    # 3. load_pabgh_offsets(PABGH_PATH) / load_pabgh_offsets(PABGH)
    for cn in PABGH_NAMES:
        content = content.replace(
            f"load_pabgh_offsets({cn})",
            'load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy())',
        )

    # 4. eprintln / format with PABGB_PATH or PABGB
    for cn in PABGB_NAMES:
        # Match the whole eprintln!("...{}...", PABGB_PATH) call (single arg path at end)
        content = re.sub(
            rf'eprintln!\("[^"]*\{{\}}[^"]*",\s*{re.escape(cn)}\)',
            'eprintln!("SKIP: fixture not found")',
            content,
        )

    for cn in PABGH_NAMES:
        content = re.sub(
            rf'eprintln!\("[^"]*\{{\}}[^"]*",\s*{re.escape(cn)}\)',
            'eprintln!("SKIP: pabgh not found")',
            content,
        )

    return content


def process_file(filepath: Path) -> bool:
    text = filepath.read_text(encoding="utf-8")

    # Does this file have a PABGB const at all?
    m = PABGB_CONST_RE.search(text)
    if not m:
        return False  # nothing to do

    const_name = m.group(1)   # PABGB_PATH or PABGB
    full_path   = m.group(2)  # the raw path string value
    filename    = extract_filename(full_path)

    # Indent level of the const (usually 4 spaces inside mod tests {})
    # We'll use 4 spaces for the function too.
    indent = "    "

    fn_def = f'{indent}fn pabgb_path() -> std::path::PathBuf {{ crate::testenv::resolve("{filename}") }}\n'

    # Replace the const declaration
    new_text = PABGB_CONST_RE.sub(fn_def, text, count=1)

    # Remove any PABGH_PATH / PABGH const
    new_text = PABGH_CONST_RE.sub("", new_text)

    # Rewrite test-body usages
    new_text = apply_body_replacements(new_text, const_name)

    # Sanity: if any residual bare const name remains (outside comments), warn
    for cn in PABGB_NAMES + PABGH_NAMES:
        # Quick heuristic: look for the identifier NOT inside a string or comment
        residual = re.findall(rf'(?<!["\w]){re.escape(cn)}(?![\w"])', new_text)
        if residual:
            print(f"  WARN {filepath.relative_to(ROOT)}: {len(residual)} residual '{cn}' references — may need manual fix")

    if new_text == text:
        return False  # unchanged (shouldn't happen if the const was found)

    if not DRY_RUN:
        filepath.write_text(new_text, encoding="utf-8")
    return True


def main():
    changed = []
    skipped = []

    # Process all info.rs files under src/tables/
    tables_dir = ROOT / "tables"
    all_rs = list(tables_dir.rglob("*.rs"))

    # Also process binary/variants/diagnose_*.rs and similar
    all_rs += list((ROOT / "binary" / "variants").glob("diagnose_*.rs"))
    all_rs += list((ROOT / "binary" / "variants").glob("validate_*.rs"))

    # Also check dispatch.rs, intents/apply.rs, item_info/, resolve.rs, lib.rs
    for extra in ["dispatch.rs", "resolve.rs", "lib.rs"]:
        p = ROOT / extra
        if p.exists():
            all_rs.append(p)
    for extra_dir in ["item_info", "intents"]:
        all_rs += list((ROOT / extra_dir).glob("*.rs"))

    for filepath in sorted(set(all_rs)):
        try:
            if process_file(filepath):
                changed.append(filepath.relative_to(ROOT))
                print(f"  UPDATED {filepath.relative_to(ROOT)}")
            else:
                skipped.append(filepath.relative_to(ROOT))
        except Exception as e:
            print(f"  ERROR  {filepath.relative_to(ROOT)}: {e}")

    print(f"\n{'[DRY RUN] ' if DRY_RUN else ''}Done: {len(changed)} updated, {len(skipped)} unchanged")
    if changed:
        print("\nUpdated files:")
        for f in changed:
            print(f"  {f}")


if __name__ == "__main__":
    main()
