//! Dispatches `setup_first_run` through the real Tauri IPC path (macro-
//! generated arg deserialization included), unlike `contract.rs` which
//! calls the command function directly and so never exercises how the
//! `#[tauri::command]` macro binds the JSON payload to the `input`
//! parameter. Reproduces the "Something went wrong." bug reported on the
//! Setup screen: the frontend sent `{"pin": "..."}` but Tauri binds a
//! single struct parameter under its own name (`{"input": {"pin": "..."}}`).

use std::time::{SystemTime, UNIX_EPOCH};

use bvconsole_lib::commands;
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
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
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
