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
    app.manage(AppPaths {
        db_path: dir.0.join("console.db"),
        auth_path: dir.0.join("auth.json"),
        backups_manifest_path: dir.0.join("backups-manifest.json"),
        app_data_dir: dir.0.clone(),
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

// Reproduces the "something went wrong" export bug reported on the Reports
// screen: `export_monthly` and `export_low_contribution` each take a single
// struct param (`input`), same binding rule as `update_settings`'s `patch`
// above — but `src/lib/ipc/m6-reports.ts` was spreading the input flat
// (`{ ...input }`) instead of nesting it (`{ input }`). The macro rejects
// the flat shape with a raw string error, not a typed `AppError`, so the
// frontend's `toErrorPresentation` can't recognize it and falls back to its
// generic "Something went wrong." message. `export_yearly_average` takes a
// plain scalar (`output_path`) and was never affected — included here only
// as a regression guard against a future "fix" nesting it too.

fn invoke_reports(cmd: &str, body: InvokeBody) -> Result<serde_json::Value, serde_json::Value> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::export_monthly,
            commands::export_yearly_average,
            commands::export_low_contribution,
        ])
        .build(mock_context(noop_assets()))
        .unwrap();

    app.manage(SessionState::new());
    app.state::<SessionState>().mark_authenticated();
    let conn = db::open_seeded_in_memory().unwrap();
    conn.execute(
        "INSERT INTO periods (period_month, status) VALUES ('2026-08', 'open')",
        [],
    )
    .unwrap();
    app.manage(DbState::with_connection(conn));

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

/// The bug as it shipped: `exportMonthly()`'s flat `{ ...input }` spread.
#[test]
fn export_monthly_flat_payload_is_rejected_by_the_real_command_macro() {
    let result = invoke_reports(
        "export_monthly",
        InvokeBody::Json(json!({
            "periodMonth": "2026-08",
            "optionalColumns": [],
            "outputPath": std::env::temp_dir().join("x.xlsx").to_str().unwrap()
        })),
    );
    let err = result.expect_err("flat payload should be rejected, not accepted");
    let message = err.as_str().unwrap_or_default();
    assert!(
        message.contains("missing required key input"),
        "expected a missing-key error, got: {message}"
    );
}

/// The fix: nested under `input`, matching the single struct param's name.
#[test]
fn export_monthly_nested_under_input_succeeds() {
    let result = invoke_reports(
        "export_monthly",
        InvokeBody::Json(json!({
            "input": {
                "periodMonth": "2026-08",
                "optionalColumns": [],
                "sortField": "name",
                "sortDirection": "asc",
                "outputPath": std::env::temp_dir().join("y.xlsx").to_str().unwrap()
            }
        })),
    );
    assert!(result.is_ok(), "expected success, got: {result:?}");
}

/// The bug as it shipped: `exportLowContribution()`'s flat `{ ...input }` spread.
#[test]
fn export_low_contribution_flat_payload_is_rejected_by_the_real_command_macro() {
    let result = invoke_reports(
        "export_low_contribution",
        InvokeBody::Json(json!({
            "threshold": 10000,
            "outputPath": std::env::temp_dir().join("z.xlsx").to_str().unwrap()
        })),
    );
    let err = result.expect_err("flat payload should be rejected, not accepted");
    let message = err.as_str().unwrap_or_default();
    assert!(
        message.contains("missing required key input"),
        "expected a missing-key error, got: {message}"
    );
}

/// The fix: nested under `input`, matching the single struct param's name.
#[test]
fn export_low_contribution_nested_under_input_succeeds() {
    let result = invoke_reports(
        "export_low_contribution",
        InvokeBody::Json(json!({
            "input": {
                "threshold": 10000,
                "sortField": "name",
                "sortDirection": "asc",
                "outputPath": std::env::temp_dir().join("w.xlsx").to_str().unwrap()
            }
        })),
    );
    assert!(result.is_ok(), "expected success, got: {result:?}");
}

/// `exportYearlyAverage()`'s payload — flat, matching its single *scalar*
/// param (`output_path`), never affected by the struct-binding bug above.
#[test]
fn export_yearly_average_flat_scalar_payload_succeeds() {
    let result = invoke_reports(
        "export_yearly_average",
        InvokeBody::Json(json!({
            "outputPath": std::env::temp_dir().join("v.xlsx").to_str().unwrap(),
            "sortField": "name",
            "sortDirection": "asc"
        })),
    );
    assert!(result.is_ok(), "expected success, got: {result:?}");
}
