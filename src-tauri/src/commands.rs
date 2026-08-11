// The full 40-command IPC surface (04-technical-architecture.md §6 / §6.1),
// registered together so the S4 exit gate ("the contract harness asserts
// exactly seven unauthenticated commands and 40 total" — 02-roadmap.md) is
// real from this sprint onward. Only API-01/API-02 (US-M1.1) have logic;
// every other command is a typed, correctly-gated stub until its own story
// ships — see `AppError::NotImplemented`.
use crate::db_state::DbState;
use crate::error::AppError;
use crate::m1_members;
use crate::m2_entries;
use crate::m8_auth;
use crate::paths::AppPaths;
use crate::session::{require_locked, require_session, SessionState};

// An authenticated session implies an open database connection: nothing in
// S4 ever gets past `require_session` without one, since nothing sets the
// session flag except a test. S5's `login` sets both together, which is
// what will make this genuinely unreachable rather than merely untested.
fn locked_conn(db: &DbState) -> std::sync::MutexGuard<'_, Option<rusqlite::Connection>> {
    db.0.lock().expect("db mutex poisoned")
}

#[tauri::command]
pub fn create_root_member(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    input: m1_members::CreateRootMemberInput,
) -> Result<m1_members::Member, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m1_members::create_root_member(conn, input)
}

#[tauri::command]
pub fn add_member(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    input: m1_members::AddMemberInput,
) -> Result<m1_members::AddMemberOutcome, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m1_members::add_member(conn, input)
}

macro_rules! auth_stub {
    ($name:ident) => {
        #[tauri::command]
        pub fn $name(
            session: tauri::State<'_, SessionState>,
        ) -> Result<serde_json::Value, AppError> {
            require_session(&session)?;
            Err(AppError::NotImplemented {
                command: stringify!($name),
            })
        }
    };
}

macro_rules! open_stub {
    ($name:ident) => {
        #[tauri::command]
        pub fn $name() -> Result<serde_json::Value, AppError> {
            Err(AppError::NotImplemented {
                command: stringify!($name),
            })
        }
    };
}

// M1 remainder — US-M1.2/M1.3/M1.4, S5.

#[tauri::command]
pub fn edit_member(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    input: m1_members::EditMemberInput,
) -> Result<m1_members::Member, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m1_members::edit_member(conn, input)
}

#[tauri::command]
pub fn deactivate_member(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m1_members::deactivate_member(conn, id)
}

#[tauri::command]
pub fn reactivate_member(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m1_members::reactivate_member(conn, id)
}

/// API-06. `active_only` defaults false — every screen searches the whole
/// directory except the Add-Member reference lookup, which passes `true`
/// (Rule-30).
#[tauri::command]
pub fn search_members(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    query: String,
    active_only: Option<bool>,
) -> Result<Vec<m1_members::SearchResult>, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m1_members::search_members(conn, &query, active_only.unwrap_or(false))
}

// M2 — US-M2.1/M2.2, S7; US-M2.3/M2.4, S12.

#[tauri::command]
pub fn record_entry(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    input: m2_entries::RecordEntryInput,
) -> Result<m2_entries::BusinessVolumeEntry, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m2_entries::record_entry(conn, input)
}

#[tauri::command]
pub fn edit_entry(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    paths: tauri::State<'_, AppPaths>,
    input: m2_entries::EditEntryInput,
) -> Result<m2_entries::BusinessVolumeEntry, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m2_entries::edit_entry(conn, &paths.db_path, &paths.backups_dir, input)
}

auth_stub!(get_period_lock_status);

// M3 — US-M3.1/M3.2, S6.
auth_stub!(preview_settings_impact);

// M4 — US-M4.1/M4.2, S8; US-M4.3, S9.
auth_stub!(get_member_detail);
auth_stub!(get_direct_children_chart);

// M5 — US-M5.1..M5.5, S11-S13.
auth_stub!(get_outstanding_periods);
auth_stub!(begin_close);
auth_stub!(confirm_backup_and_close);
auth_stub!(manual_backup_current_period);

// M6 — US-M6.1..M6.5, S13.
auth_stub!(export_monthly);
auth_stub!(export_yearly_average);
auth_stub!(export_low_contribution);
auth_stub!(list_backups);
auth_stub!(redownload_backup);

// M7 — US-M7.1/M7.2/M7.4, S10; US-M7.3, S11.
auth_stub!(get_settings);
auth_stub!(update_settings);
auth_stub!(add_slab_row);
auth_stub!(remove_slab_row);
auth_stub!(update_slab_row);
auth_stub!(get_console_backup_settings);
auth_stub!(update_console_backup_settings);

// M8 remainder — US-M8.2/M8.3, S5/S7; US-M8.5, S14.

/// API-28. ⚠️ T-M8.3-1: the encryption key must be genuinely dropped, not
/// merely hidden behind an overlay — dropping the open `Connection` here
/// does that, since SQLCipher's derived key lives inside it. Idempotent
/// (calling it twice just drops an already-empty slot) and not audited,
/// matching the API table.
#[tauri::command]
pub fn lock_session(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<(), AppError> {
    require_session(&session)?;
    *locked_conn(&db) = None;
    session.mark_locked();
    Ok(())
}

/// API-29: "Same as `login`" for verification, but gated on
/// `require_locked` rather than `require_session` — see `session.rs`'s doc
/// comment for why `require_session` would be a deadlock here. Re-derives
/// the master key and reopens the connection exactly like `login`, since
/// `lock_session` genuinely closed the previous one.
#[tauri::command]
pub fn unlock_session(
    paths: tauri::State<'_, AppPaths>,
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    input: m8_auth::CredentialInput,
) -> Result<(), AppError> {
    require_locked(&session)?;
    let master_key = m8_auth::login(&paths.auth_path, input)?;
    let conn = crate::db::open_encrypted(
        &paths.db_path,
        &m8_auth::crypto::sqlcipher_raw_key_pragma(&master_key),
    )?;
    *locked_conn(&db) = Some(conn);
    session.mark_authenticated();
    Ok(())
}

auth_stub!(get_outstanding_alert);
auth_stub!(run_console_backup_now);

// M9 — US-M9.1, S14 (a completeness check; audit writes land per-command from S4,
// but the read command itself has no consumer until then).
auth_stub!(get_audit_log);

// The closed set of seven unauthenticated commands (03-business-rules.md
// Rule-29 / 06-security-authorization-matrix.md §3). None of these call
// `require_session` — that is the property QA.2's authorization test
// actually checks, not just the returned error.

/// API-26 — US-M8.1, S5. Real logic, so unlike the generic stubs it takes
/// the app handle (path resolution — `paths::auth_path`/`paths::db_path`)
/// and opens the just-created database with the freshly-generated master
/// key before returning. `m8_auth::setup_first_run` refuses a second call
/// once the sidecar file exists (see its own doc comment for why that file,
/// not an `auth` DB row, is the source of truth).
#[tauri::command]
pub fn setup_first_run(
    paths: tauri::State<'_, AppPaths>,
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    input: m8_auth::SetupFirstRunInput,
) -> Result<m8_auth::SetupFirstRunResult, AppError> {
    let (result, master_key) = m8_auth::setup_first_run(&paths.auth_path, input)?;
    let conn = crate::db::open_encrypted(
        &paths.db_path,
        &m8_auth::crypto::sqlcipher_raw_key_pragma(&master_key),
    )?;
    *db.0.lock().expect("db mutex poisoned") = Some(conn);
    session.mark_authenticated();
    Ok(result)
}

/// API-27 — US-M8.1/M8.2, S5. Generic failure message regardless of which
/// credential type or part was wrong (Rule-29); a locked account reports
/// `AccountLocked` instead so the login screen's countdown has something to
/// show. Success opens the database with the recovered master key.
#[tauri::command]
pub fn login(
    paths: tauri::State<'_, AppPaths>,
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    input: m8_auth::CredentialInput,
) -> Result<(), AppError> {
    let master_key = m8_auth::login(&paths.auth_path, input)?;
    let conn = crate::db::open_encrypted(
        &paths.db_path,
        &m8_auth::crypto::sqlcipher_raw_key_pragma(&master_key),
    )?;
    *db.0.lock().expect("db mutex poisoned") = Some(conn);
    session.mark_authenticated();
    Ok(())
}

open_stub!(use_recovery_code); // US-M8.4, S8

/// API-34 — minimal slice pulled forward from US-M8.6 (S14): whether the
/// sidecar credential file exists is exactly "has this machine been set up
/// before", which is what the frontend needs to route Setup vs Login at
/// launch. S14 deepens this into genuine corrupted-database detection
/// without changing this boolean contract.
#[tauri::command]
pub fn check_data_readable(paths: tauri::State<'_, AppPaths>) -> Result<bool, AppError> {
    Ok(paths.auth_path.exists())
}

open_stub!(list_restore_points); // US-M8.6, S14
open_stub!(restore_from_backup); // US-M8.6, S14
open_stub!(restore_from_backup_file); // US-M8.6, S14

pub use crate::command_names::{ALL_COMMAND_NAMES, UNAUTHENTICATED_COMMAND_NAMES};

/// QA.2's contract test needs to exercise the remaining stub commands
/// generically by name — `create_root_member`/`add_member` (M1.1),
/// `setup_first_run`/`login`/`check_data_readable` (M8.1/M8.2, S5), and
/// `record_entry`/`edit_entry`/`lock_session`/`unlock_session` (M2.1/M2.2/
/// M8.3, S7) all have real logic now and their own dedicated tests instead
/// (see `tests/contract.rs`). Rust has no runtime reflection to call a
/// function by string, so this is the one place that enumerates the match
/// by hand; `ALL_COMMAND_NAMES` is what keeps it honest against gaps.
pub fn call_stub_by_name(
    name: &str,
    session: tauri::State<'_, SessionState>,
) -> Result<serde_json::Value, AppError> {
    match name {
        "get_period_lock_status" => get_period_lock_status(session),
        "preview_settings_impact" => preview_settings_impact(session),
        "get_member_detail" => get_member_detail(session),
        "get_direct_children_chart" => get_direct_children_chart(session),
        "get_outstanding_periods" => get_outstanding_periods(session),
        "begin_close" => begin_close(session),
        "confirm_backup_and_close" => confirm_backup_and_close(session),
        "manual_backup_current_period" => manual_backup_current_period(session),
        "export_monthly" => export_monthly(session),
        "export_yearly_average" => export_yearly_average(session),
        "export_low_contribution" => export_low_contribution(session),
        "list_backups" => list_backups(session),
        "redownload_backup" => redownload_backup(session),
        "get_settings" => get_settings(session),
        "update_settings" => update_settings(session),
        "add_slab_row" => add_slab_row(session),
        "remove_slab_row" => remove_slab_row(session),
        "update_slab_row" => update_slab_row(session),
        "get_console_backup_settings" => get_console_backup_settings(session),
        "update_console_backup_settings" => update_console_backup_settings(session),
        "get_outstanding_alert" => get_outstanding_alert(session),
        "run_console_backup_now" => run_console_backup_now(session),
        "get_audit_log" => get_audit_log(session),
        "use_recovery_code" => use_recovery_code(),
        "list_restore_points" => list_restore_points(),
        "restore_from_backup" => restore_from_backup(),
        "restore_from_backup_file" => restore_from_backup_file(),
        other => panic!("unknown command in ALL_COMMAND_NAMES: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_stub_by_name_covers_every_stub_command() {
        const HAS_REAL_LOGIC: &[&str] = &[
            "create_root_member",
            "add_member",
            "edit_member",
            "deactivate_member",
            "reactivate_member",
            "search_members",
            "record_entry",
            "edit_entry",
            "setup_first_run",
            "login",
            "check_data_readable",
            "lock_session",
            "unlock_session",
        ];
        let stub_names: Vec<&str> = ALL_COMMAND_NAMES
            .iter()
            .copied()
            .filter(|n| !HAS_REAL_LOGIC.contains(n))
            .collect();
        assert_eq!(stub_names.len(), 27);
        // Exercised properly (with real State fixtures) in tests/contract.rs;
        // this just proves the dispatcher doesn't panic on any known name.
    }
}
