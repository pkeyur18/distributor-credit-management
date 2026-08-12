//! Dispatches `setup_first_run` through the real Tauri IPC path (macro-
//! generated arg deserialization included), unlike `contract.rs` which
//! calls the command function directly and so never exercises how the
//! `#[tauri::command]` macro binds the JSON payload to the `input`
//! parameter. Reproduces the "Something went wrong." bug reported on the
//! Setup screen: the frontend sent `{"pin": "..."}` but Tauri binds a
//! single struct parameter under its own name (`{"input": {"pin": "..."}}`).

use std::time::{SystemTime, UNIX_EPOCH};

use bvconsole_lib::commands;
use bvconsole_lib::db;
use bvconsole_lib::db_state::DbState;
use bvconsole_lib::paths::AppPaths;
use bvconsole_lib::session::SessionState;
use serde_json::json;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::webview::InvokeRequest;
use tauri::Manager;

struct TempAppDir(std::path::PathBuf);
impl TempAppDir {
    fn new() -> Self {
        // Nanos alone can collide between parallel test threads — see
        // contract.rs's identical helper.
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bvconsole-ipc-dispatch-{nanos}-{unique}"));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}
impl Drop for TempAppDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn invoke_setup_first_run(body: InvokeBody) -> Result<serde_json::Value, serde_json::Value> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![commands::setup_first_run])
        .build(mock_context(noop_assets()))
        .unwrap();

    let dir = TempAppDir::new();
    app.manage(SessionState::new());
    app.manage(DbState::new());
    let backups_dir = dir.0.join("backups");
    std::fs::create_dir_all(&backups_dir).unwrap();
    app.manage(AppPaths {
        db_path: dir.0.join("console.db"),
        auth_path: dir.0.join("auth.json"),
        backups_dir,
    });

    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    tauri::test::get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "setup_first_run".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body,
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    )
    .map(|b| b.deserialize::<serde_json::Value>().unwrap())
}

/// This is exactly the shape `src/lib/ipc/m8-auth.ts`'s `setupFirstRun`
/// sends today (`invokeCommand("setup_first_run", { ...input })`) — a flat
/// object, not nested under `input`. Tauri's command macro looks the
/// parameter up by name, so this must fail with a missing-key error.
#[test]
fn frontend_flat_payload_is_rejected_by_the_real_command_macro() {
    let result = invoke_setup_first_run(InvokeBody::Json(json!({ "pin": "482913" })));

    let err = result.expect_err("flat payload should be rejected, not accepted");
    let message = err.as_str().unwrap_or_default();
    assert!(
        message.contains("missing required key input"),
        "expected a missing-key error, got: {message}"
    );
}

/// The shape the macro actually requires: the struct nested under the
/// Rust parameter's own name (`input`).
#[test]
fn payload_nested_under_input_succeeds() {
    let result = invoke_setup_first_run(InvokeBody::Json(json!({ "input": { "pin": "482913" } })));

    assert!(result.is_ok(), "expected success, got: {result:?}");
}

// S10's M7 commands mix both payload shapes on purpose (see
// `src/lib/ipc/m7-settings.ts`): `update_settings` takes one struct param
// (`patch`), so the frontend nests under that name (`{ patch: {...} }}`) —
// the same fix this file's own bug report is about, applied correctly from
// the start this time. `add_slab_row` takes two *scalar* params, so the
// frontend spreads them flat (`{ threshold, percentage }`) — a different,
// unrelated binding rule (each scalar param matches its own top-level
// key), easy to conflate with the struct case above. `contract.rs` calls
// these functions directly and would never catch either shape being wrong;
// only dispatching through the real macro, as here, proves it.

fn invoke_authenticated(
    cmd: &str,
    body: InvokeBody,
) -> Result<serde_json::Value, serde_json::Value> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::add_slab_row,
        ])
        .build(mock_context(noop_assets()))
        .unwrap();

    app.manage(SessionState::new());
    app.state::<SessionState>().mark_authenticated();
    app.manage(DbState::with_connection(
        db::open_seeded_in_memory().unwrap(),
    ));

    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    tauri::test::get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body,
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    )
    .map(|b| b.deserialize::<serde_json::Value>().unwrap())
}

/// `updateSettings`'s payload — nested under `patch`, matching the Rust
/// parameter's own name, exactly the shape the macro requires for a
/// single-struct command.
#[test]
fn update_settings_nested_under_its_param_name_succeeds() {
    let result = invoke_authenticated(
        "update_settings",
        InvokeBody::Json(json!({ "patch": { "sessionTimeoutMinutes": 30 } })),
    );
    assert!(result.is_ok(), "expected success, got: {result:?}");
}

/// The single-struct rule does not generalize to a flat spread — proving
/// this catches a regression where someone "fixes" `add_slab_row` by
/// nesting it the same way, which would break the frontend's actual
/// `{ ...input }` call.
#[test]
fn update_settings_flat_payload_is_rejected() {
    let result = invoke_authenticated(
        "update_settings",
        InvokeBody::Json(json!({ "sessionTimeoutMinutes": 30 })),
    );
    let err = result.expect_err("a flat payload must not satisfy the single `patch` parameter");
    let message = err.as_str().unwrap_or_default();
    assert!(
        message.contains("missing required key patch"),
        "got: {message}"
    );
}

/// `addSlabRow`'s payload — flat, matching two scalar parameters, each
/// bound by its own top-level key rather than one wrapping key.
#[test]
fn add_slab_row_flat_scalar_payload_succeeds() {
    let result = invoke_authenticated(
        "add_slab_row",
        InvokeBody::Json(json!({ "threshold": 2_000_000, "percentage": 16 })),
    );
    assert!(result.is_ok(), "expected success, got: {result:?}");
}
