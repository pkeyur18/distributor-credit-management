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
use bvconsole_lib::m8_auth::{CredentialInput, SetupFirstRunInput};
use bvconsole_lib::paths::AppPaths;
use bvconsole_lib::session::SessionState;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

fn app_with_seeded_db() -> tauri::App<tauri::test::MockRuntime> {
    let app = tauri::test::mock_app();
    app.manage(SessionState::new());
    app.manage(DbState::with_connection(
        db::open_seeded_in_memory().unwrap(),
    ));
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
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bvconsole-contract-{label}-{nanos}"));
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
    // reactivate_member/search_members (M1.2/M1.3/M1.4, S5) and
    // setup_first_run/login/check_data_readable (M8.1/M8.2, S5) have real
    // logic and their own dedicated tests below — this loop covers the
    // remaining 31 stub commands via `call_stub_by_name`.
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
