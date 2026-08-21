//! Pre-provisions app-data for the E2E perf-at-scale spec
//! (`e2e/specs/perf-at-scale.e2e.js`) — a hierarchy + a year of entries,
//! built the same real-engine way `qa_dataset`/`tests/performance.rs`
//! already do, but with an actual `auth.json` credential envelope so the
//! *real compiled app*, launched separately by WebdriverIO, can log in and
//! show the seeded data.
//!
//! Neither existing dev tool does this: `generate_dataset` opens its
//! output with a hardcoded key and writes no `auth.json` (a real app
//! pointed at that file would show Setup, then fail to reopen it — wrong
//! key); `import_test_data` requires `auth.json` to already exist (real
//! Setup wizard). This seeds both, with a known PIN, from one master key.
//!
//! `seed_into` takes `app_data_dir` as a parameter specifically so it can
//! be exercised safely against a throwaway directory — see the first test
//! below, which is fast and runs on every `cargo test`. The second test
//! is the dangerous one: it writes into the *real* OS app-data directory
//! (paths.rs: "no automated test should ever touch the real OS app-data
//! directory" — this is a deliberate, narrow, CI-only exception to that
//! rule, made explicitly for this one spec). `#[ignore]`d and guarded to
//! refuse an app-data directory that already has a credential in it; only
//! ever run this against a fresh CI runner, never a developer machine.
use std::path::{Path, PathBuf};

use bvconsole_lib::db;
use bvconsole_lib::m8_auth::crypto::sqlcipher_raw_key_pragma;
use bvconsole_lib::m8_auth::{self, CredentialInput, SetupFirstRunInput};
use bvconsole_lib::qa_dataset::generate_dataset_into;

const IDENTIFIER: &str = "com.siddharthpatel.bvconsole"; // src-tauri/tauri.conf.json "identifier"

/// Same resolution Tauri's `app.path().app_data_dir()` performs — hand-
/// replicated because seeding runs before any Tauri app exists to ask.
/// Same approach `src/bin/import_test_data.rs`'s own `default_app_data_dir`
/// already takes, for the same reason.
fn real_app_data_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("HOME/USERPROFILE not set");
    if cfg!(target_os = "macos") {
        PathBuf::from(home)
            .join("Library/Application Support")
            .join(IDENTIFIER)
    } else if cfg!(target_os = "windows") {
        let appdata =
            std::env::var("APPDATA").unwrap_or_else(|_| format!("{home}/AppData/Roaming"));
        PathBuf::from(appdata).join(IDENTIFIER)
    } else {
        let xdg = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{home}/.local/share"));
        PathBuf::from(xdg).join(IDENTIFIER)
    }
}

// Rule-29: 6 digits. Fixed and documented — this is an E2E-only,
// throwaway-CI-runner credential, never a real one.
const KNOWN_PIN: &str = "246810";

fn seed_into(app_data_dir: &Path, scale: usize) -> Vec<i64> {
    std::fs::create_dir_all(app_data_dir).expect("create app-data dir");
    let auth_path = app_data_dir.join("auth.json");
    let db_path = app_data_dir.join("console.db");

    let (_, master_key) = m8_auth::setup_first_run(
        &auth_path,
        SetupFirstRunInput {
            pin: Some(KNOWN_PIN.to_string()),
            password: None,
        },
    )
    .expect("setup_first_run");

    // No `run_period_catchup` here — on an empty database it immediately
    // inserts the current month's period row itself, which then collides
    // with `generate_dataset_into`'s own (it builds the full 12-month
    // period state — 11 closed, 1 open — as part of its own contract).
    // The two are alternative ways to establish period state, not meant
    // to compose.
    let conn = db::open_encrypted(&db_path, &sqlcipher_raw_key_pragma(&master_key))
        .expect("open_encrypted");

    generate_dataset_into(&conn, 42, scale, 8, None)
}

#[test]
fn seed_into_a_temp_directory_produces_a_loginable_database() {
    let dir = std::env::temp_dir().join(format!(
        "bvconsole-e2e-seed-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let member_ids = seed_into(&dir, 50);
    assert_eq!(member_ids.len(), 50);

    // The point of this test: prove the credential this just wrote is
    // actually the one a real login (`unlock_session`'s own path) would
    // accept, and that it decrypts the seeded database.
    let master_key = m8_auth::login(
        &dir.join("auth.json"),
        CredentialInput {
            pin: Some(KNOWN_PIN.to_string()),
            password: None,
        },
    )
    .expect("login with the known PIN must succeed");
    let conn = db::open_encrypted(
        &dir.join("console.db"),
        &sqlcipher_raw_key_pragma(&master_key),
    )
    .expect("the logged-in key must open the seeded database");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM members", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 50);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
#[ignore = "writes into the REAL OS app-data directory for e2e/specs/perf-at-scale.e2e.js \
            to launch the app against — only ever run this on a throwaway CI runner, \
            never a developer machine. See this file's module doc."]
fn seed_the_real_app_data_directory_for_e2e_perf_at_scale() {
    let scale: usize = std::env::var("E2E_SEED_SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(25_000);
    let dir = real_app_data_dir();
    assert!(
        !dir.join("auth.json").exists(),
        "refusing to seed {} — a credential already exists there. This must only ever run \
         against a fresh CI runner with no prior app data, never a developer machine.",
        dir.display()
    );

    let member_ids = seed_into(&dir, scale);
    println!(
        "seeded {} members into {} (PIN {KNOWN_PIN})",
        member_ids.len(),
        dir.display()
    );
}
