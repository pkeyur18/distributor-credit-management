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
use bvconsole_lib::session::SessionState;
use tauri::Manager;

fn app_with_seeded_db() -> tauri::App<tauri::test::MockRuntime> {
    let app = tauri::test::mock_app();
    app.manage(SessionState::new());
    app.manage(DbState::with_connection(
        db::open_seeded_in_memory().unwrap(),
    ));
    app
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

    // create_root_member/add_member have real logic and their own
    // dedicated tests below — this loop covers the 38 stub commands via
    // `call_stub_by_name`.
    for &name in ALL_COMMAND_NAMES
        .iter()
        .filter(|&&n| n != "create_root_member" && n != "add_member")
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
        AddMemberOutcome::Created { member } => {
            assert_eq!(member.introducer_member_id, Some(root.id));
        }
        AddMemberOutcome::ReactivationOffer { .. } => panic!("expected Created"),
    }
}
