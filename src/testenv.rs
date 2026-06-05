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
