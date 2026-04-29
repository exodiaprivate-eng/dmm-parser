# Parallel Lanes — Coordination Doc

We're running 3-4 Claude instances in parallel against the same repo.
Each instance owns a distinct file scope so we never trample each
other's work.

**Last updated**: 2026-04-29

---

## The setup

```
C:\Users\corin\Desktop\CD DUMPING TOOLS\
  ├─ dmm-parser/      ← Instance A (lead, on `main`)
  ├─ dmm-parser-b/    ← Instance B (on `lane-b` branch)
  └─ dmm-parser-c/    ← Instance C (on `lane-c` branch)
```

All three are git worktrees of the same repo, sharing one `.git`. Each
worktree has its own checkout so file edits don't collide.

---

## Lane assignments

Pick the row for your instance and stick to its files. If you need to
edit anything outside your lane, **stop and message the human first**.

| Instance | Branch  | Lane                                    | Files you own |
|---|---|---|---|
| **Snow** | (separate) | Skill/Buff Python bindings + API docs | `src/python.rs`, `src/tables/skill_info/*`, `src/tables/buff_info/*`, `docs/api.md` |
| **A** | `main` | EffectData inner_map typing + ConditionInfo etc. | `src/binary/variants/effect_data.rs`, `src/binary/variants/condition_*.rs`, `src/binary/variants/game_condition.rs`, `src/tables/effect_info/*`, `src/tables/condition_info/*` |
| **B** | `lane-b` | Per-sub_tag typed payloads inside polymorphic families | `src/binary/variants/global_game_event_execute_data.rs`, `src/binary/variants/game_event_handler_data.rs`, `src/tables/global_game_event_info/*`, `src/tables/game_event_handler_info/*` |
| **C** | `lane-c` | JSON exposure for hand-corrected tables that lack it | one specific table at a time from the list in **Instance C task list** below |

### Shared files — touch carefully

These files are touched by everyone. Make changes **small and atomic**,
push fast, and pull before you start any new work:

- `src/binary/variants/mod.rs`  — module declarations
- `src/binary/mod.rs`            — base traits / array impls
- `src/json_traits.rs`            — JSON trait base impls
- `Cargo.toml`                    — deps
- `docs/PARALLEL_LANES.md`        — this file (to record your status)

---

## Instance B — concrete starting task

**Goal**: Replace `body: Vec<u8>` opacity inside the polymorphic family
wrappers with per-sub_tag typed payload structs.

Currently:
```rust
// in src/binary/variants/global_game_event_execute_data.rs
pub enum GlobalGameEventExecuteData {
    Absent,
    Present { sub_tag: u8, body: Vec<u8> },   // ← body is opaque
    Raw(Vec<u8>),
}
```

Desired:
```rust
pub enum GlobalGameEventExecuteData {
    Absent,
    Present(GlobalGameEventExecuteDataBody),  // ← typed
    Raw(Vec<u8>),
}

pub enum GlobalGameEventExecuteDataBody {
    VaryTradeItemPrice(VaryTradeItemPricePayload),  // sub_tag 0
    OpenRoyalSupply(OpenRoyalSupplyPayload),        // sub_tag 1
    InPlace,                                         // sub_tag 2 (no alloc)
}
```

### IDA reference (already decompiled — see commit history `4b30791`)

- Win-IDA dispatcher: `sub_141156680`
- sub_tag 0 (VaryTradeItemPrice) reader: `sub_141155000` →
  `sub_1410FFAC0` (CArray<u16>) + `sub_141155530`
  (CArray<VaryTradeItemPriceData>) +
  `read_u32_lookup_DA30` + `read_LocalizableString`
- sub_tag 1 (OpenRoyalSupply) reader: `sub_141155300` →
  `sub_1411553D0` (single helper)
- sub_tag 2: in-place reuse of existing struct (no body bytes)

### Steps

1. `cd C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser-b`
2. `git pull origin main && git rebase origin/main` to sync.
3. Read `src/binary/variants/global_game_event_execute_data.rs` for the
   current Decoded|Raw shape and check the existing dispatcher comments.
4. Decompile each sub_tag's reader via the IDA MCP (you have access).
   Walk the wire format. Same playbook as the GameEventHandlerData
   decode you can read in `git log 3fe208e --stat`.
5. Define a typed payload struct per sub_tag inside the variants
   module. Use `py_binary_struct!` for simple field-list payloads.
6. Replace `Present { sub_tag, body }` with a typed body enum that
   dispatches per sub_tag at read time.
7. Update `src/tables/global_game_event_info/info.rs` to reflect the
   new shape (the JSON should expose typed body fields, not just
   `_execute_data_b64`).
8. Run `cargo test --release` — must pass. The roundtrip test in
   `tables/global_game_event_info` proves bytes match.
9. Commit on `lane-b`: `git add ... && git commit -m "..."`
10. **Sync routine to push to main**:
    ```bash
    git fetch origin
    git rebase origin/main             # rebase your work onto current main
    cargo test --release               # re-verify after rebase
    git push origin lane-b:main        # fast-forward main if no conflict
    ```
    If `git push` fails with non-fast-forward, someone else pushed in
    the meantime — `git fetch origin && git rebase origin/main` again
    and retry.
11. After successful push, optionally start GameEventHandlerData per-
    sub_tag typing (5 sub_tags: SetSceneObjectParameterBySceneLevel /
    SetSceneObjectParameter / SetUIPlayGuideParameter /
    SetUIFullscreenGuideParameter / MakeSnapshotForDev). Same pattern.

### Stats currently on `main` (your starting point)

```
GlobalGameEventInfo: 80 entries, 100% Decoded as Present
GameEventHandlerInfo: 682 entries, 100% Decoded
  sub_tag=2 (SetUIPlayGuideParameter): 422
  sub_tag=3 (SetUIFullscreenGuideParameter): 260
```

Snow's lane and Instance A's lane don't touch your files. You're free.

---

## Instance C — concrete starting task

**Goal**: Add `to_json_dict` / `write_from_json_dict` to hand-corrected
tables that have working byte parsers but no JSON. Each of these is a
1-commit win.

### Pick ONE table per session from this list

| Table | File | Why |
|---|---|---|
| `aimove_speed_info` | `src/tables/aimove_speed_info/info.rs` | Tiny, fast win |
| `auto_spawn_filter_info` | `src/tables/auto_spawn_filter_info/info.rs` | Tiny |
| `dye_color_group_info` | `src/tables/dye_color_group_info/info.rs` | Mod-relevant (cosmetics) |
| `fail_message_info` | `src/tables/fail_message_info/info.rs` | Mod-relevant (i18n) |
| `mercenary_info` | `src/tables/mercenary_info/info.rs` | Mod-relevant (NPCs) |
| `part_prefab_dye_slot_info` | `src/tables/part_prefab_dye_slot_info/info.rs` | Cosmetics |
| `house_info` | `src/tables/house_info/info.rs` | Mod-relevant (housing) |
| `level_action_point_info` | `src/tables/level_action_point_info/info.rs` | Level data |
| `field_level_name_table_info` | `src/tables/field_level_name_table_info/info.rs` | Level data |

### How to add JSON to a table (5-minute pattern)

Look at `src/tables/condition_info/info.rs` (commit `9f1be1d`) or
`src/tables/effect_info/info.rs` for the canonical pattern. The shape is:

```rust
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
// existing imports stay

impl<'a> MyInfo<'a> {
    // ...existing read_with_size / write_to stay unchanged...

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("field_a".to_string(), self.field_a.to_json_value());
        m.insert("field_b".to_string(), self.field_b.to_json_value());
        // ... one entry per public field ...
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <FieldType as WriteJsonValue>::write_from_json(w, json_get_field(obj, "field_a")?)?;
        <FieldType as WriteJsonValue>::write_from_json(w, json_get_field(obj, "field_b")?)?;
        // ... one line per public field, in wire order ...
        Ok(())
    }
}
```

Then add a JSON round-trip test in the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn json_roundtrip() {
    let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP"); return; };
    let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP"); return; };
    let ranges = entry_ranges(&entries, data.len());
    for (i, (key, start, end)) in ranges.iter().enumerate() {
        let mut cursor = *start;
        let item = MyInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
        let dict = item.to_json_dict();
        let mut typed = Vec::new();
        item.write_to(&mut typed).unwrap();
        let mut from_json = Vec::new();
        MyInfo::write_from_json_dict(&mut from_json, &dict)
            .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: {}", i, key, e));
        assert_eq!(from_json, typed, "entry {} key=0x{:x}: JSON round-trip diverges", i, key);
    }
}
```

### Steps per table

1. `cd C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser-c`
2. `git fetch origin && git rebase origin/main` to sync.
3. Pick ONE table from the list above. Mark it in this doc (under
   "In-progress" below) so other instances know.
4. Read its `info.rs` to understand the field list.
5. Add `to_json_dict` + `write_from_json_dict` + the json_roundtrip test.
6. `cargo test --release <table_name>` — must pass.
7. Commit on `lane-c`: `git commit -m "<table>: add JSON exposure"`
8. **Sync routine to push to main**:
    ```bash
    git fetch origin
    git rebase origin/main
    cargo test --release
    git push origin lane-c:main
    ```
9. Update this doc to remove the table from the in-progress list, then
   pick the next one.

### What to AVOID

- **Do not touch** `src/python.rs` (Snow's lane).
- **Do not touch** anything under `src/binary/variants/` (Instance A & B).
- **Do not touch** `src/tables/condition_info/`, `src/tables/effect_info/`,
  `src/tables/global_game_event_info/`, `src/tables/game_event_handler_info/`.
- **Do not modify** Cargo.toml or json_traits.rs unless you absolutely
  need a new helper. If you do, message the human first.

---

## Sync routine — every instance, every session

```bash
# at start of session
cd <your worktree>
git fetch origin
git rebase origin/main          # for B/C; A is already on main and pulls

# work, commit on your branch (or main for A)
git add <files>
git commit -m "..."

# before pushing
cargo test --release            # MUST pass
git fetch origin                # someone else may have pushed
git rebase origin/main          # rebase if needed
cargo test --release            # re-verify
git push origin <branch>:main   # for B (lane-b:main) and C (lane-c:main)
                                # A pushes plain `git push origin main`
```

If `git push` rejects with "non-fast-forward", someone pushed while you
were testing. Just `git fetch && git rebase origin/main && git push`
again. Three-instance traffic is rare enough that this should resolve
in one retry.

---

## In-progress (live status — update as you work)

| Instance | Started | Lane / table | Status |
|---|---|---|---|
| Snow | (long-running) | Skill/Buff Python bindings | active |
| A | now | EffectData inner_map typing | active |
| B | — | (pick: GlobalGameEventExecuteData per-sub_tag bodies) | not started |
| C | 2026-04-29 | auto_spawn_filter_info JSON exposure | done — pick next |

When you start a task, edit your row to `Status: in progress on <table/feature>`.
When you finish + push, edit to `Status: done — pick next`.

---

## Glossary

- **Tier 1**: every typed field is editable through JSON.
- **Tier 1.5**: typed prefix + opaque blob (clone-only, no field access).
- **Tier 2**: whole-tail blob (legacy).
- **`Decoded | Raw` enum**: standard fallback pattern for polymorphic
  family wrappers — see `src/binary/variants/game_condition.rs` for the
  canonical impl. Use this whenever a wire shape might have unknown
  variants; it guarantees byte-perfect round-trip.
- **Wire shape**: the on-disk byte layout. Comes from Win-IDA decompiles
  (use the IDA MCP). For most tables there's already an IDA recipe in
  `dmm-pabgb-aio/mac_extract/`.

---

## When to escalate

Ping the human if:

- Your test fails and the error is in someone else's lane (don't fix
  another lane's code; report it).
- You hit an obfuscated reader (anti-disassembly self-modifying code).
  Flag it and skip — the `Decoded | Raw` fallback handles it.
- A push gets rejected three times in a row.
- You finish your starting task and need a new one.
