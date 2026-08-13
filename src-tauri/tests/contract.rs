//! QA.2 — contract-test harness for the IPC surface (T-QA.2-1..4).
//!
//! "Direct Tauri command-invocation harness — no HTTP layer, no browser or
//! network mocking needed": `tauri::test::mock_app()` builds a real,
//! in-process `App` (managed state, no window/webview) and the
//! `#[tauri::command]`-annotated functions are called exactly like any
//! other Rust function — no IPC string dispatch involved.

use bvconsole_lib::commands::{self, ALL_COMMAND_NAMES, UNAUTHENTICATED_COMMAND_NAMES};
use bvconsole_lib::db;
use bvconsole_lib::db_state::DbState;
use bvconsole_lib::error::AppError;
use bvconsole_lib::m1_members::{AddMemberInput, AddMemberOutcome, CreateRootMemberInput};
use bvconsole_lib::m2_entries::{EditEntryInput, RecordEntryInput};
use bvconsole_lib::m8_auth::{CredentialInput, SetupFirstRunInput};
use bvconsole_lib::paths::AppPaths;
use bvconsole_lib::session::SessionState;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

/// `edit_entry` always takes an `AppPaths` parameter (its closed-period
/// path copies a real file), so every caller needs one managed even when a
/// given test only ever exercises the open-period branch. The directory is
/// leaked rather than threaded through every existing caller's return type
/// — harmless for a test-only temp dir, and the OS reclaims it eventually.
fn app_with_seeded_db() -> tauri::App<tauri::test::MockRuntime> {
    let app = tauri::test::mock_app();
    app.manage(SessionState::new());
    app.manage(DbState::with_connection(
        db::open_seeded_in_memory().unwrap(),
    ));
    let dir = TempAppDir::new("seeded-db");
    app.manage(AppPaths {
        db_path: dir.0.join("console.db"),
        auth_path: dir.0.join("auth.json"),
        app_data_dir: dir.0.clone(),
    });
    std::mem::forget(dir);
    app
}

/// M8.1/M8.2 write real files (the sidecar credential file, then the
/// encrypted database once a master key is recovered) — every test that
/// exercises `setup_first_run`/`login` gets its own throwaway directory, so
/// the suite never touches the real OS app-data directory and tests can run
/// concurrently without colliding on the same path.
struct TempAppDir(std::path::PathBuf);
impl TempAppDir {
    fn new(label: &str) -> Self {
        // Nanos alone can collide between parallel test threads; a
        // process-wide counter alongside it makes every dir unique.
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bvconsole-contract-{label}-{nanos}-{unique}"));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}
impl Drop for TempAppDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn app_with_temp_paths(label: &str) -> (tauri::App<tauri::test::MockRuntime>, TempAppDir) {
    let dir = TempAppDir::new(label);
    let app = tauri::test::mock_app();
    app.manage(SessionState::new());
    app.manage(DbState::new());
    app.manage(AppPaths {
        db_path: dir.0.join("console.db"),
        auth_path: dir.0.join("auth.json"),
        app_data_dir: dir.0.clone(),
    });
    (app, dir)
}

/// US-M7.4's pulled-forward commands (`run_console_backup_now`,
/// `restore_from_backup`, `restore_from_backup_file`) do real file I/O
/// against `AppPaths.db_path` (`std::fs::copy`, unconditionally) —
/// `app_with_seeded_db`'s in-memory connection has no such file on disk.
/// This fixture opens a real encrypted file at the managed `db_path`
/// instead, exactly like `setup_first_run`/`login` do.
fn app_with_seeded_db_on_disk(label: &str) -> (tauri::App<tauri::test::MockRuntime>, TempAppDir) {
    let dir = TempAppDir::new(label);
    let app = tauri::test::mock_app();
    app.manage(SessionState::new());
    let db_path = dir.0.join("console.db");
    let conn = db::open_encrypted(&db_path, "test-key").unwrap();
    app.manage(DbState::with_connection(conn));
    app.manage(AppPaths {
        db_path,
        auth_path: dir.0.join("auth.json"),
        app_data_dir: dir.0.clone(),
    });
    (app, dir)
}

fn root_input(phone: &str) -> CreateRootMemberInput {
    CreateRootMemberInput {
        name: "Top Member".into(),
        phone: phone.into(),
        address: "1 Main Street".into(),
        email: None,
        consent_given: true,
    }
}

// T-QA.2-3: the surface holds exactly 40 commands, API-01 to API-40, no
// gaps — and this list is the same one `lib.rs` feeds to
// `generate_handler!` and `build.rs` feeds to the ACL generator (via
// `command_names.rs`), so the three can never quietly drift apart.
#[test]
fn the_command_surface_holds_exactly_forty_commands() {
    assert_eq!(
        ALL_COMMAND_NAMES.len(),
        40,
        "API-01 to API-40, no gaps (C2)"
    );

    let capabilities = include_str!("../capabilities/default.json");
    let allow_count = capabilities.matches("\"allow-").count();
    assert_eq!(
        allow_count, 40,
        "the Tauri capability allowlist must have exactly 40 allow-* entries"
    );
    for name in ALL_COMMAND_NAMES {
        // Tauri's ACL identifiers are hyphen-only (autogenerate_command_permissions
        // slugifies `_` to `-`); the command name itself stays snake_case.
        let slug = name.replace('_', "-");
        assert!(
            capabilities.contains(&format!("\"allow-{slug}\"")),
            "capabilities/default.json is missing allow-{slug} (for command {name})"
        );
    }
}

// T-QA.2-2: the unauthenticated set is exactly seven, named — not six, and
// closed. Every other command must refuse with AuthRequired when no
// session exists, *before* touching any business logic.
#[test]
fn the_unauthenticated_set_is_exactly_the_named_seven() {
    assert_eq!(UNAUTHENTICATED_COMMAND_NAMES.len(), 7);
    assert_eq!(
        UNAUTHENTICATED_COMMAND_NAMES,
        &[
            "login",
            "setup_first_run",
            "use_recovery_code",
            "check_data_readable",
            "list_restore_points",
            "restore_from_backup",
            "restore_from_backup_file",
        ]
    );
    for name in UNAUTHENTICATED_COMMAND_NAMES {
        assert!(
            ALL_COMMAND_NAMES.contains(name),
            "{name} is in the unauthenticated list but not in the full command list"
        );
    }
}

// The structural property T-QA.2-2 actually needs: with no session, every
// command *not* in the unauthenticated list refuses with `auth_required`,
// and every command *in* it never does (a stub `not_implemented` refusal
// is fine — reaching that error at all proves the gate was never hit).
#[test]
fn every_authenticated_command_refuses_without_a_session_and_only_those() {
    let app = tauri::test::mock_app();
    app.manage(SessionState::new());
    app.manage(DbState::new());
    let session = app.state::<SessionState>();

    // create_root_member/add_member (M1.1), edit_member/deactivate_member/
    // reactivate_member/search_members (M1.2/M1.3/M1.4, S5),
    // setup_first_run/login/check_data_readable (M8.1/M8.2, S5) and
    // record_entry/edit_entry/lock_session/unlock_session (M2.1/M2.2/M8.3,
    // S7) have real logic and their own dedicated tests below — this loop
    // covers the remaining stub commands via `call_stub_by_name`.
    const HAS_REAL_LOGIC: &[&str] = &[
        "create_root_member",
        "add_member",
        "edit_member",
        "deactivate_member",
        "reactivate_member",
        "search_members",
        "setup_first_run",
        "login",
        "check_data_readable",
        "record_entry",
        "edit_entry",
        "lock_session",
        "unlock_session",
        "get_member_detail",
        "get_direct_children_chart",
        "use_recovery_code",
        // US-M7.1/M7.2/M7.4, S10 (US-M8.5/M8.6's own commands pulled
        // forward — see `commands.rs`'s "M8 remainder" comment).
        "get_settings",
        "update_settings",
        "add_slab_row",
        "remove_slab_row",
        "update_slab_row",
        "get_console_backup_settings",
        "update_console_backup_settings",
        "run_console_backup_now",
        "list_restore_points",
        "restore_from_backup",
        "restore_from_backup_file",
        // US-M7.3/US-M5.1, S11.
        "preview_settings_impact",
        "get_outstanding_periods",
        "begin_close",
        "confirm_backup_and_close",
        "manual_backup_current_period",
        // US-M5.2/M5.3/M2.3/M2.4, S12.
        "get_period_lock_status",
        "get_outstanding_alert",
    ];
    for &name in ALL_COMMAND_NAMES
        .iter()
        .filter(|&&n| !HAS_REAL_LOGIC.contains(&n))
    {
        let result = commands::call_stub_by_name(name, session.clone());
        let is_unauthenticated = UNAUTHENTICATED_COMMAND_NAMES.contains(&name);
        match result {
            Err(AppError::AuthRequired) => assert!(
                !is_unauthenticated,
                "{name} is in the unauthenticated list but hit the session gate"
            ),
            Err(AppError::NotImplemented { .. }) | Ok(_) => assert!(
                is_unauthenticated,
                "{name} is not in the unauthenticated list but did not hit the session gate"
            ),
            Err(other) => panic!("{name}: unexpected error variant {other:?}"),
        }
    }
}

#[test]
fn create_root_member_requires_a_session() {
    let app = app_with_seeded_db();
    // Deliberately not marking the session authenticated.
    let result = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876511111"),
    );
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn create_root_member_succeeds_once_authenticated_then_refuses_a_second_root() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();

    let first = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876522222"),
    );
    assert!(first.is_ok());

    let second = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876533333"),
    );
    assert!(matches!(second, Err(AppError::Conflict { .. })));
}

#[test]
fn add_member_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();

    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876544444"),
    )
    .unwrap();

    let outcome = commands::add_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        AddMemberInput {
            name: "Asha Patel".into(),
            phone: "9876555555".into(),
            address: "2 Side Street".into(),
            email: None,
            consent_given: true,
            introducer_member_id: root.id,
        },
    )
    .unwrap();

    match outcome {
        AddMemberOutcome::Created { member, .. } => {
            assert_eq!(member.introducer_member_id, Some(root.id));
        }
        AddMemberOutcome::ReactivationOffer { .. } => panic!("expected Created"),
    }
}

// US-M1.2/M1.3/M1.4 (S5) command-layer wiring — business logic is covered
// exhaustively in m1_members::tests; this proves the session gate applies.

#[test]
fn edit_deactivate_reactivate_and_search_all_require_a_session() {
    let app = app_with_seeded_db();
    let edit = commands::edit_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        bvconsole_lib::m1_members::EditMemberInput {
            id: 1,
            name: None,
            phone: None,
            email: None,
            address: None,
        },
    );
    assert!(matches!(edit, Err(AppError::AuthRequired)));

    let deactivate =
        commands::deactivate_member(app.state::<SessionState>(), app.state::<DbState>(), 1);
    assert!(matches!(deactivate, Err(AppError::AuthRequired)));

    let reactivate =
        commands::reactivate_member(app.state::<SessionState>(), app.state::<DbState>(), 1);
    assert!(matches!(reactivate, Err(AppError::AuthRequired)));

    let search = commands::search_members(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        "anyone".into(),
        None,
    );
    assert!(matches!(search, Err(AppError::AuthRequired)));
}

#[test]
fn edit_member_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876599901"),
    )
    .unwrap();

    let updated = commands::edit_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        bvconsole_lib::m1_members::EditMemberInput {
            id: root.id,
            name: Some("Renamed".into()),
            phone: None,
            email: None,
            address: None,
        },
    )
    .unwrap();
    assert_eq!(updated.name, "Renamed");
}

#[test]
fn deactivate_and_reactivate_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876599902"),
    )
    .unwrap();
    let child = match commands::add_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        AddMemberInput {
            name: "Child Member".into(),
            phone: "9876599903".into(),
            address: "3 Side Street".into(),
            email: None,
            consent_given: true,
            introducer_member_id: root.id,
        },
    )
    .unwrap()
    {
        AddMemberOutcome::Created { member, .. } => member,
        _ => panic!("expected Created"),
    };

    commands::deactivate_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        child.id,
    )
    .unwrap();
    commands::reactivate_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        child.id,
    )
    .unwrap();

    let root_deactivate =
        commands::deactivate_member(app.state::<SessionState>(), app.state::<DbState>(), root.id);
    assert!(matches!(root_deactivate, Err(AppError::Conflict { .. })));
}

#[test]
fn search_members_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876599904"),
    )
    .unwrap();

    let results = commands::search_members(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        "Top Member".into(),
        None,
    )
    .unwrap();
    assert!(results.iter().any(|r| r.id == root.id));
}

// US-M4.1/M4.2 (S8) command-layer wiring.

#[test]
fn get_member_detail_requires_a_session() {
    let app = app_with_seeded_db();
    let result =
        commands::get_member_detail(app.state::<SessionState>(), app.state::<DbState>(), 1, None);
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn get_direct_children_chart_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::get_direct_children_chart(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        None,
        false,
        None,
    );
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn get_member_detail_and_get_direct_children_chart_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876599905"),
    )
    .unwrap();

    let detail = commands::get_member_detail(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root.id,
        None,
    )
    .unwrap();
    assert_eq!(detail.member.id, root.id);
    assert_eq!(detail.leg_count, 0);

    // `member_id: None` resolves to the (only) root member.
    let chart = commands::get_direct_children_chart(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        None,
        false,
        None,
    )
    .unwrap();
    assert_eq!(chart.nodes[0].member_id, root.id);
    assert_eq!(chart.slab_table.len(), 7);
}

// US-M8.1/M8.2 (S5) command-layer wiring — the business logic itself
// (envelope crypto, the lockout ladder, dual-credential login) is
// exhaustively covered by `m8_auth`'s own unit tests against temp file
// paths directly. What's specific to the command layer, and worth proving
// here, is: no session required to call these; a successful call actually
// opens the database (not just recovers a key) and marks the session
// authenticated; `check_data_readable` flips once setup has run.

#[test]
fn check_data_readable_is_false_before_setup_and_true_after() {
    let (app, _dir) = app_with_temp_paths("check-data-readable");
    assert!(!commands::check_data_readable(app.state::<AppPaths>()).unwrap());

    commands::setup_first_run(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        SetupFirstRunInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();

    assert!(commands::check_data_readable(app.state::<AppPaths>()).unwrap());
}

#[test]
fn setup_first_run_requires_no_session_and_leaves_one_authenticated_with_a_usable_database() {
    let (app, _dir) = app_with_temp_paths("setup-first-run");
    assert!(!app.state::<SessionState>().is_authenticated());

    let result = commands::setup_first_run(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        SetupFirstRunInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();
    assert_eq!(result.recovery_codes.len(), 10);
    assert!(app.state::<SessionState>().is_authenticated());

    // The database is genuinely open, not just the credential accepted —
    // an authenticated command works immediately after.
    let member = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876566666"),
    );
    assert!(member.is_ok());
}

#[test]
fn login_requires_no_session_and_recovers_the_same_database_setup_wrote() {
    let (app, _dir) = app_with_temp_paths("login");
    commands::setup_first_run(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        SetupFirstRunInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();
    commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876577777"),
    )
    .unwrap();

    // A fresh session/db, same on-disk paths — as if the app were relaunched.
    app.state::<SessionState>().clear();
    *app.state::<DbState>().0.lock().unwrap() = None;
    assert!(!app.state::<SessionState>().is_authenticated());

    commands::login(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        CredentialInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();
    assert!(app.state::<SessionState>().is_authenticated());

    // The root member created before the "relaunch" is still there — same
    // database, reopened with the recovered master key, not a fresh one.
    let second_root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876588888"),
    );
    assert!(matches!(second_root, Err(AppError::Conflict { .. })));
}

#[test]
fn login_with_the_wrong_pin_never_authenticates() {
    let (app, _dir) = app_with_temp_paths("wrong-pin");
    commands::setup_first_run(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        SetupFirstRunInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();
    app.state::<SessionState>().clear();

    let err = commands::login(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        CredentialInput {
            pin: Some("000000".into()),
            password: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidCredential { .. }));
    assert!(!app.state::<SessionState>().is_authenticated());
}

// US-M2.1/M2.2 (S7) command-layer wiring — business logic is covered
// exhaustively in m2_entries::tests; this proves the session gate applies
// and that the command layer reaches the real module correctly.

#[test]
fn record_entry_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::record_entry(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        RecordEntryInput {
            member_id: 1,
            amount: 1_000,
            entry_date: "2026-08-15".into(),
        },
    );
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn edit_entry_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::edit_entry(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        app.state::<AppPaths>(),
        EditEntryInput {
            id: 1,
            amount: 1_000,
            entry_date: "2026-08-15".into(),
        },
    );
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn record_entry_and_edit_entry_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876511111"),
    )
    .unwrap();

    let entry = commands::record_entry(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        RecordEntryInput {
            member_id: root.id,
            amount: 100_000,
            entry_date: "2026-08-15".into(),
        },
    )
    .unwrap();
    assert_eq!(entry.period_month, "2026-08");

    let updated = commands::edit_entry(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        app.state::<AppPaths>(),
        EditEntryInput {
            id: entry.id,
            amount: 250_000,
            entry_date: "2026-08-15".into(),
        },
    )
    .unwrap();
    assert_eq!(updated.amount, 250_000);
}

// US-M8.3 (S7) — proves the deadlock `session.rs`'s doc comment describes
// cannot happen: `unlock_session` must stay reachable after `lock_session`
// clears the normal `Auth` gate, and every other authenticated command must
// refuse while locked.

#[test]
fn lock_session_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::lock_session(app.state::<SessionState>(), app.state::<DbState>());
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn unlock_session_is_unreachable_before_anything_has_ever_locked() {
    let (app, _dir) = app_with_temp_paths("unlock-fresh");
    let result = commands::unlock_session(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        CredentialInput {
            pin: Some("482913".into()),
            password: None,
        },
    );
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn lock_then_unlock_round_trips_back_to_an_authenticated_session_with_a_usable_database() {
    let (app, _dir) = app_with_temp_paths("lock-unlock");
    commands::setup_first_run(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        SetupFirstRunInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876522222"),
    )
    .unwrap();

    commands::lock_session(app.state::<SessionState>(), app.state::<DbState>()).unwrap();
    assert!(!app.state::<SessionState>().is_authenticated());
    assert!(
        app.state::<DbState>().0.lock().unwrap().is_none(),
        "lock_session must genuinely drop the open connection, not just the session flag"
    );

    // Every other authenticated command refuses while locked.
    let blocked = commands::search_members(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        "any".into(),
        None,
    );
    assert!(matches!(blocked, Err(AppError::AuthRequired)));

    commands::unlock_session(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        CredentialInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();
    assert!(app.state::<SessionState>().is_authenticated());

    // Same database — the root created before locking is still there.
    let second_root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876533333"),
    );
    assert!(matches!(second_root, Err(AppError::Conflict { .. })));
    let _ = root;
}

#[test]
fn unlock_session_with_the_wrong_pin_stays_locked_not_authenticated() {
    let (app, _dir) = app_with_temp_paths("unlock-wrong-pin");
    commands::setup_first_run(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        SetupFirstRunInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();
    commands::lock_session(app.state::<SessionState>(), app.state::<DbState>()).unwrap();

    let err = commands::unlock_session(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        CredentialInput {
            pin: Some("000000".into()),
            password: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::InvalidCredential { .. }));
    assert!(!app.state::<SessionState>().is_authenticated());
    assert!(
        app.state::<SessionState>().is_locked(),
        "a failed unlock attempt must leave the session locked, not signed all the way out"
    );
}

// US-M8.4 (S8) command-layer wiring. `use_recovery_code`'s own business
// logic (envelope crypto, single-use, master-key reuse) is exhaustively
// covered by `m8_auth`'s own unit tests — this proves it's callable with no
// session (it's in the unauthenticated seven) and that the new credential
// it sets actually opens the database afterward.
#[test]
fn use_recovery_code_end_to_end_through_the_command_layer() {
    let (app, _dir) = app_with_temp_paths("use-recovery-code");
    let setup = commands::setup_first_run(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        SetupFirstRunInput {
            pin: Some("482913".into()),
            password: None,
        },
    )
    .unwrap();

    let result = commands::use_recovery_code(
        app.state::<AppPaths>(),
        bvconsole_lib::m8_auth::UseRecoveryCodeInput {
            code: setup.recovery_codes[0].clone(),
            new_pin: Some("111222".into()),
            new_password: None,
        },
    )
    .unwrap();
    assert_eq!(result.recovery_codes.len(), 10);

    app.state::<SessionState>().clear();
    commands::login(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        CredentialInput {
            pin: Some("111222".into()),
            password: None,
        },
    )
    .unwrap();
    assert!(app.state::<SessionState>().is_authenticated());
}

// US-M7.1/M7.2/M7.4 (S10) command-layer wiring — US-M8.5/M8.6's own
// commands (`run_console_backup_now`, `list_restore_points`,
// `restore_from_backup`, `restore_from_backup_file`) pulled forward, per
// `commands.rs`'s "M8 remainder" comment.

#[test]
fn get_settings_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::get_settings(app.state::<SessionState>(), app.state::<DbState>());
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn get_and_update_settings_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();

    let settings =
        commands::get_settings(app.state::<SessionState>(), app.state::<DbState>()).unwrap();
    assert_eq!(settings.session_timeout_minutes, 15);

    let updated = commands::update_settings(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        bvconsole_lib::m7_settings::SettingsPatch {
            session_timeout_minutes: Some(30),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(updated.session_timeout_minutes, 30);
}

#[test]
fn slab_row_commands_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();

    let added = commands::add_slab_row(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        2_000_000,
        16,
    )
    .unwrap();
    assert_eq!(added.percentage, 16);

    let updated = commands::update_slab_row(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        added.id,
        2_000_000,
        18,
    )
    .unwrap();
    assert_eq!(updated.percentage, 18);

    commands::remove_slab_row(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        added.id,
    )
    .unwrap();
    let settings =
        commands::get_settings(app.state::<SessionState>(), app.state::<DbState>()).unwrap();
    assert_eq!(
        settings.slab_thresholds.len(),
        7,
        "back to the seeded seven rows"
    );
}

#[test]
fn console_backup_settings_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();

    let before =
        commands::get_console_backup_settings(app.state::<SessionState>(), app.state::<DbState>())
            .unwrap();
    assert_eq!(before.schedule, "off");

    let after = commands::update_console_backup_settings(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        "weekly".into(),
        5,
        "backups".into(),
    )
    .unwrap();
    assert_eq!(after.schedule, "weekly");
    assert_eq!(after.retention_count, 5);
}

#[test]
fn run_console_backup_now_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::run_console_backup_now(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
    );
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn run_console_backup_now_end_to_end_produces_a_manual_backup() {
    let (app, _dir) = app_with_seeded_db_on_disk("run-console-backup-now");
    app.state::<SessionState>().mark_authenticated();

    let record = commands::run_console_backup_now(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
    )
    .unwrap();

    assert_eq!(record.kind, "manual");
    let points = commands::list_restore_points(app.state::<DbState>()).unwrap();
    assert_eq!(
        points[0].id, record.id,
        "T-M7.4-4: the new manual backup must lead the Restore card's list"
    );
}

#[test]
fn list_restore_points_refuses_cleanly_with_no_database_open() {
    let app = app_with_seeded_db();
    // `app_with_seeded_db` manages an in-memory connection directly, so
    // clear it to simulate the genuine "nothing open yet" state.
    *app.state::<DbState>().0.lock().unwrap() = None;
    let result = commands::list_restore_points(app.state::<DbState>());
    assert!(matches!(result, Err(AppError::NotFound { .. })));
}

#[test]
fn restore_from_backup_end_to_end_drops_the_session() {
    let (app, _dir) = app_with_seeded_db_on_disk("restore-from-backup");
    app.state::<SessionState>().mark_authenticated();
    let record = commands::run_console_backup_now(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
    )
    .unwrap();

    commands::restore_from_backup(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        record.id,
    )
    .unwrap();

    assert!(
        !app.state::<SessionState>().is_authenticated(),
        "a restored file may hold a different credential — the session must not survive it"
    );
}

#[test]
fn restore_from_backup_file_end_to_end_through_the_command_layer() {
    let (app, dir) = app_with_seeded_db_on_disk("restore-from-backup-file");
    app.state::<SessionState>().mark_authenticated();
    let source_path = dir.0.join("brought-from-another-machine.db");
    std::fs::copy(app.state::<AppPaths>().db_path.clone(), &source_path).unwrap();

    commands::restore_from_backup_file(
        app.state::<AppPaths>(),
        app.state::<SessionState>(),
        app.state::<DbState>(),
        source_path.to_string_lossy().into_owned(),
    )
    .unwrap();

    assert!(!app.state::<SessionState>().is_authenticated());
}

// --- preview_settings_impact (API-33, US-M7.3) ---

#[test]
fn preview_settings_impact_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::preview_settings_impact(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        bvconsole_lib::m3_calc::CandidateSettings::default(),
    );
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn preview_settings_impact_writes_nothing_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9990000001"),
    )
    .unwrap();
    commands::record_entry(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        RecordEntryInput {
            member_id: root.id,
            amount: 100_000,
            entry_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        },
    )
    .unwrap();
    let before =
        commands::get_settings(app.state::<SessionState>(), app.state::<DbState>()).unwrap();

    let preview = commands::preview_settings_impact(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        bvconsole_lib::m3_calc::CandidateSettings {
            royalty_qualifying_count: Some(before.royalty_qualifying_count + 5),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(preview.rewards_before, preview.rewards_after);

    let after =
        commands::get_settings(app.state::<SessionState>(), app.state::<DbState>()).unwrap();
    assert_eq!(
        after.royalty_qualifying_count, before.royalty_qualifying_count,
        "a preview must never write the candidate settings back"
    );
}

// --- M5 close flow (API-12/13/14/15, US-M5.1) ---

fn mark_current_month_awaiting_close(app: &tauri::App<tauri::test::MockRuntime>) -> i64 {
    // No command transitions a period to `awaiting_close` yet (US-M5.5,
    // S12's catch-up) — direct SQL is the only way to reach this state
    // ahead of that story, same as any other pre-S12 test exercising the
    // close flow.
    let db = app.state::<DbState>();
    let guard = db.0.lock().unwrap();
    let conn = guard.as_ref().unwrap();
    let month = chrono::Local::now().format("%Y-%m").to_string();
    conn.execute(
        "UPDATE periods SET status = 'awaiting_close', ended_at = ?1 WHERE period_month = ?1",
        [&month],
    )
    .unwrap();
    conn.query_row(
        "SELECT id FROM periods WHERE period_month = ?1",
        [&month],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn get_outstanding_periods_and_begin_close_require_a_session() {
    let app = app_with_seeded_db();
    assert!(matches!(
        commands::get_outstanding_periods(app.state::<SessionState>(), app.state::<DbState>()),
        Err(AppError::AuthRequired)
    ));
    assert!(matches!(
        commands::begin_close(app.state::<SessionState>(), app.state::<DbState>()),
        Err(AppError::AuthRequired)
    ));
}

#[test]
fn get_period_lock_status_and_get_outstanding_alert_require_a_session() {
    let app = app_with_seeded_db();
    assert!(matches!(
        commands::get_period_lock_status(app.state::<SessionState>(), app.state::<DbState>()),
        Err(AppError::AuthRequired)
    ));
    assert!(matches!(
        commands::get_outstanding_alert(app.state::<SessionState>(), app.state::<DbState>()),
        Err(AppError::AuthRequired)
    ));
}

#[test]
fn confirm_backup_and_close_and_manual_backup_require_a_session() {
    let (app, _dir) = app_with_seeded_db_on_disk("m5-close-session-gate");
    assert!(matches!(
        commands::confirm_backup_and_close(
            app.state::<SessionState>(),
            app.state::<DbState>(),
            app.state::<AppPaths>(),
            bvconsole_lib::m5_close::ConfirmBackupAndCloseInput {
                period_id: 1,
                external_medium_path: None,
            },
        ),
        Err(AppError::AuthRequired)
    ));
    assert!(matches!(
        commands::manual_backup_current_period(
            app.state::<SessionState>(),
            app.state::<DbState>(),
            app.state::<AppPaths>(),
        ),
        Err(AppError::AuthRequired)
    ));
}

#[test]
fn the_full_close_flow_writes_a_permanent_record_and_zeroes_live_figures() {
    let (app, _dir) = app_with_seeded_db_on_disk("m5-close-end-to-end");
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9990000002"),
    )
    .unwrap();
    commands::record_entry(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        RecordEntryInput {
            member_id: root.id,
            amount: 100_000,
            entry_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        },
    )
    .unwrap();
    let period_id = mark_current_month_awaiting_close(&app);

    let outstanding =
        commands::get_outstanding_periods(app.state::<SessionState>(), app.state::<DbState>())
            .unwrap();
    assert_eq!(outstanding.len(), 1);
    assert_eq!(outstanding[0].id, period_id);

    let begun = commands::begin_close(app.state::<SessionState>(), app.state::<DbState>()).unwrap();
    assert_eq!(begun.period_id, period_id);
    assert_eq!(begun.with_entry_count, 1);

    commands::confirm_backup_and_close(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        app.state::<AppPaths>(),
        bvconsole_lib::m5_close::ConfirmBackupAndCloseInput {
            period_id,
            external_medium_path: None,
        },
    )
    .unwrap();

    let still_outstanding =
        commands::get_outstanding_periods(app.state::<SessionState>(), app.state::<DbState>())
            .unwrap();
    assert!(
        still_outstanding.is_empty(),
        "the closed month must drop off the outstanding list"
    );

    let points = commands::list_restore_points(app.state::<DbState>()).unwrap();
    assert!(
        points.iter().any(|p| p.kind == "period_close"),
        "the close must have written a period_close backup"
    );
}

#[test]
fn manual_backup_current_period_end_to_end_through_the_command_layer() {
    let (app, _dir) = app_with_seeded_db_on_disk("manual-backup-current-period");
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9990000003"),
    )
    .unwrap();
    commands::record_entry(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        RecordEntryInput {
            member_id: root.id,
            amount: 50_000,
            entry_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        },
    )
    .unwrap();

    let record = commands::manual_backup_current_period(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        app.state::<AppPaths>(),
    )
    .unwrap();

    assert_eq!(record.kind, "manual");
    let points = commands::list_restore_points(app.state::<DbState>()).unwrap();
    assert_eq!(points[0].id, record.id);
}
