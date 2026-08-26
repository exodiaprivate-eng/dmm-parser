//! Test fixture resolution for game-file roundtrip tests.
//!
//! # Quick start (after a game update)
//!
//! 1. Extract the new `.pabgb` / `.pabgh` files from the updated game PAZ archives
//!    into a flat directory (e.g. `C:\CDpabgb\2026-05-xx\`).
//!
//! 2. Point the env var at that directory and run the test suite:
//!    ```text
//!    $env:DMM_PARSER_PABGB_DIR = "C:\CDpabgb\2026-05-xx"
//!    cargo test 2>&1 | Select-String "FAILED|ok|SKIP"
//!    ```
//!
//! Any table whose struct no longer round-trips will fail immediately with the
//! exact byte offset and difference — no hex tracing needed.
//!
//! # Env vars
//!
//! | Variable                  | Meaning                                               |
//! |---------------------------|-------------------------------------------------------|
//! | `DMM_PARSER_PABGB_DIR`    | Directory containing `.pabgb` + `.pabgh` files        |
//! | `DMM_PARSER_ITEMINFO_PATH`| Override for `iteminfo.pabgb` specifically            |
//! | `DMM_PARSER_GAME_DIR`     | Game installation root (for PAPGT / PAMT tests)       |

use std::path::PathBuf;

/// Dev-machine fallback directories tried when `DMM_PARSER_PABGB_DIR` is not set.
/// Newest game-version directories listed first so tests automatically pick up
/// the most-recent files when multiple versions are on disk.
///
/// **Add new directories at the TOP when the game updates** so dev machines pick
/// up the new files without touching any env var.
const FALLBACK_DIRS: &[&str] = &[
    // Add new entries here (newest first) after each game update
    // GAME VERSION 2.00.00 — 2026-08-25, the Enhanced update. 134 tables.
    //
    // ⚠ ADDING THE CAPTURE IS PART OF THE PATCH, NOT A FOLLOW-UP. Until this
    // entry existed, a plain `cargo test` resolved iteminfo to the 1.18 capture
    // below and read it with the 2.00 layout, so
    // `full_table_every_record_consumes_its_range` failed for a reason that had
    // nothing to do with the code under test. The gate only means anything when
    // the newest capture is the one it reaches by default.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-25",
    // GAME VERSION 1.18.01 — 2026-08-16 HOTFIX on top of 1.18.00.
    // ⚠ The version is the 3rd u16 of paver, not the 2nd: `0100 1200 0100` =
    // 1.18.01 vs 1.18.00's `…0000`. The minor (`12` = 18) is UNCHANGED, so a
    // u16@2 comparison sees no patch at all.
    // exe 367,943,576 -> 355,485,592, sha256 974C0446…
    // 268/268 extracted from the UNMOUNTED game. bytediff vs 2026-8-15: only 6
    // tables changed (conditioninfo + its pabgh, itemuseinfo, missioninfo,
    // questinfo, storeinfo) and ALL at identical byte size = value edits, not
    // layout drift. Roundtrip-gated on capture: 130 clean / 0 parse-fail /
    // 0 roundtrip-fail.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-16",
    // GAME VERSION 18 (1.18.00) — 2026-08-15 FULL PATCH. paver u16@2: 17 -> 18 (major
    // bump). exe 361,664,408 -> 367,943,576 bytes, new sha256 9A102A9F…
    // 268/270 extracted from the UNMOUNTED game (Steam's own update had just replaced
    // the tree, so it is vanilla by construction).
    // ⚠ zoneinfo.pabgb / zoneinfo.pabgh are GONE — removed or renamed in 1.18.
    // bytediff vs 2026-8-7: 126 tables drifted, 142 unchanged — far broader than the
    // 1.17 patch's 9. Biggest movers: stageinfo +157K, gimmickinfo +145K, missioninfo
    // +102K, actionpointinfo -66K, dropsetinfo -54K, iteminfo +51K, npcinfo +30K,
    // characterinfo -19K. aimovespeedinfo went 2,005 -> 22,974 (11x) so it is
    // structural, not content. Ours specifically: factionspawndatainfo +9,771 (the
    // 1.6.2 fort-garrison preset), skill +4,538, buffinfo +2,353 (SummonBuffData),
    // knowledgeinfo +4,105 (the new "Quests" category from the patch notes).
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-15",
    // GAME VERSION 17 — 2026-08-07 FULL PATCH. paver u16@2: 16 -> 17 (major bump, not a
    // hotfix). 270 files / 135 tables from the UNMOUNTED game (the patch itself reverted
    // the install to vanilla — no overlay groups present, only stock 0000-0035).
    // bytediff vs 2026-8-5: only 9 tables drifted, 123 unchanged. iteminfo lost 9 records
    // and shows ENUM RENUMBER (9 diffs of -1, lowest 104 -> 103); itemuseinfo and
    // gimmickinfo are STRUCTURAL; gimmickgroupinfo has a variable-length field drift.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-7",
    // GAME VERSION 16 (1.16.04) — 2026-08-05 hotfix. paver 1.16.00 -> 1.16.04 (patch
    // component only). 270 files / 135 tables extracted, 0 missing / 0 err, from the
    // freshly-verified UNMOUNTED game. Table count 132 -> 135, so this hotfix ADDED
    // three tables. Steam applied it mid-session over a live mount, which is what
    // produced the engine's "problem with the game installation" dialog.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-5",
    // GAME VERSION 16 (1.16.00) — 2026-08-01. paver u16@2: 15 -> 16. Trading/banking
    // overhaul patch. 264/264 fixtures extracted (0 missing / 0 err) from the
    // freshly-patched, UNMOUNTED game.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-1",
    // GAME VERSION 14 — 2026-07-16. paver u16@2: 13 -> 14. papgt 577B (unchanged size,
    // sha differs) + pathc same size = group structure intact, minimal file additions.
    // All 264 fixtures extracted 0 missing/0 err. DRIFT: 2 tables.
    //   • vehicle_info — FIXED: one u32 removed from the rider/vehicle spawn-action
    //     trio (every record -4B). Byte-exact roundtrip restored; V3 130 OK / 0 FAIL.
    //   • global_stage_sequencer_info — typed under-read -1B/record, but BLOB-SAFE
    //     (falls back to opaque blob; no-op dispatch roundtrip byte-exact). NOT
    //     DMM-blocking; joins the known typed-drift set. Fix the typed struct later.
    // Roundtrip baseline on v14 = 659 passed / 22 failed (was 661/20 on 1.13 hotfix;
    // +global_stage_sequencer ×2, -vehicle ×2 after fix).
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-7-16",
    r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-7-16",
    // 1.13.00 HOTFIX — 2026-07-08. paver still v13 (0x1000d). 22 tables changed but DATA-ONLY
    // (dropsetinfo +3 recs, stringinfo +1, conditioninfo +2B; missioninfo pre-existing 100% blob).
    // Zero structure drift — V3 all-tables 130 OK / 0 FAIL. No parser change needed.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-7-8",
    r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-7-8",
    // 1.13.00 (game version 13) — 2026-07-03. Captured vanilla after the 1.12.2→1.13.00 patch.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-7-3",
    r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-7-3",
    // 1.12 (game version 12) — 2026-06-19. Captured vanilla after the 1.11→1.12 patch.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-19",
    r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-6-19",
    // 1.11 (game version 11) — 2026-06-11. Captured vanilla after the 1.10→1.11 patch.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-11",
    r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-6-11",
    // 1.10 (game version 10) — 2026-06-04. Captured live after the 1.09→1.10 patch.
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-4",
    r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-6-4",
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-8",
    r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-8",
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-1",
    r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1",
    // Per-machine fallback for machine-specific backups (e.g. inventory)
    r"C:\Users\justi\Desktop\DMM\backups",
];

/// Resolve a `.pabgb` fixture path by its canonical filename.
///
/// Priority:
///   1. `$DMM_PARSER_PABGB_DIR/<filename>` — set this env var after a game update
///   2. First entry in [`FALLBACK_DIRS`] that contains the file
///   3. Bare `filename` as a relative path — test will skip on `fs::read` failure
///      (no panic, no false-positive)
///
/// # Usage in a table test
///
/// ```ignore
/// fn pabgb_path() -> std::path::PathBuf {
///     crate::testenv::resolve("actionpointinfo.pabgb")
/// }
///
/// #[test]
/// fn roundtrip() {
///     let Ok(data) = std::fs::read(pabgb_path()) else {
///         eprintln!("SKIP: fixture not found");
///         return;
///     };
///     let entries = load_pabgh_offsets(
///         &pabgb_path().with_extension("pabgh").to_string_lossy()
///     );
///     // ...
/// }
/// ```
pub fn resolve(filename: &str) -> PathBuf {
    // 1. Explicit env var override
    if let Ok(dir) = std::env::var("DMM_PARSER_PABGB_DIR") {
        // Return even if the file doesn't exist yet — the test will skip on
        // fs::read failure rather than panicking, giving a clear SKIP message.
        return PathBuf::from(&dir).join(filename);
    }

    // 2. Scan known dev-machine fallback directories
    for base in FALLBACK_DIRS {
        let p = PathBuf::from(base).join(filename);
        if p.exists() {
            return p;
        }
    }

    // 3. No candidate found — return bare filename so test skips cleanly
    PathBuf::from(filename)
}
