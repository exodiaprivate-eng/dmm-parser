"""Insert a json_roundtrip test into each table's info.rs.

Heuristic-based: parses each file to find
  - the outer Info struct name (used in `XInfo::read_from`)
  - the pabgb-path constant name (e.g. PABGB / PABGB_PATH)
  - whether the table parses sequentially (`read_from`) or per-entry
    via `read_with_size` (skipped for now — different test pattern).
Skips tables already containing `fn json_roundtrip`.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGETS = ROOT / "scripts" / "json_targets.txt"


def emit_test(struct: str, pabgb_const: str, key_is_int: bool) -> str:
    if key_is_int:
        key_fmt = '"entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e'
        diverge_fmt = '"entry {} key=0x{:x}: JSON round-trip diverges from typed write",\n                i, item.key'
    else:
        key_fmt = '"entry {}: write_from_json_dict: {}", i, e'
        diverge_fmt = '"entry {}: JSON round-trip diverges from typed write", i'
    return f"""
    #[test]
    fn json_roundtrip() {{
        let Ok(data) = std::fs::read({pabgb_const}) else {{
            eprintln!("SKIP: missing fixture {{}}", {pabgb_const});
            return;
        }};
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {{
            items.push({struct}::read_from(&data, &mut offset).unwrap());
        }}
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {{
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            {struct}::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!({key_fmt}));
            assert_eq!(
                from_json, from_typed,
                {diverge_fmt}
            );
        }}
    }}
"""


INT_KEY_TYPES = {"u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"}


def parse_table(path: Path):
    """Return (struct, pabgb_const, key_is_int) or None to skip."""
    text = path.read_text(encoding="utf-8")
    if "fn json_roundtrip" in text:
        return None  # already done
    # Locate the existing roundtrip test's read invocation.
    m_read = re.search(r"([A-Z][A-Za-z0-9]*Info)::read_from\(&data,\s*&mut\s+offset\)", text)
    if not m_read:
        return None  # uses read_with_size or non-standard reader
    struct = m_read.group(1)
    # The outer struct must be defined inside py_binary_struct! to have
    # auto-generated to_json_dict / write_from_json_dict.
    m_outer = re.search(
        r"py_binary_struct!\s*\{[^}]*?pub struct " + re.escape(struct),
        text,
        re.DOTALL,
    )
    if not m_outer:
        return None
    # Find any const ending in pabgb path inside the test mod.
    m_const = re.search(
        r"const\s+(PABGB(?:_PATH)?)\s*:\s*&str\s*=\s*r?\"[^\"]*\.pabgb\";",
        text,
    )
    if not m_const:
        return None
    # Determine key type by finding `pub key: TYPE,` inside the matched
    # struct's body.
    body_start = m_outer.end()
    body_end = text.find("}", body_start)
    body = text[body_start:body_end]
    m_key = re.search(r"pub key:\s*([^,]+?)\s*,", body)
    key_is_int = bool(m_key and m_key.group(1).strip() in INT_KEY_TYPES)
    return struct, m_const.group(1), key_is_int


def insert_test(path: Path, struct: str, pabgb_const: str, key_is_int: bool) -> None:
    text = path.read_text(encoding="utf-8")
    stripped = text.rstrip()
    assert stripped.endswith("}"), path
    body = stripped[:-1].rstrip()  # drop the final `}` (mod-tests close)
    test = emit_test(struct, pabgb_const, key_is_int)
    path.write_text(body + "\n" + test + "}\n", encoding="utf-8")


def main():
    targets = [
        line.strip().split("|", 1)
        for line in TARGETS.read_text().splitlines()
        if line.strip()
    ]
    written = 0
    skipped = []
    for dirname, _ in targets:
        info = ROOT / "src" / "tables" / dirname / "info.rs"
        if not info.exists():
            skipped.append((dirname, "missing"))
            continue
        parsed = parse_table(info)
        if parsed is None:
            skipped.append((dirname, "skip-by-shape"))
            continue
        struct, pabgb, key_is_int = parsed
        insert_test(info, struct, pabgb, key_is_int)
        written += 1
        keytag = "int" if key_is_int else "non-int"
        print(f"+ {dirname:45} {struct} ({pabgb}, {keytag} key)")
    print(f"\ninserted {written}, skipped {len(skipped)}")
    for d, why in skipped:
        print(f"  - {d}: {why}")


if __name__ == "__main__":
    main()
