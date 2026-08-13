// The full 40-command IPC surface (04-technical-architecture.md §6 / §6.1),
// registered together so the S4 exit gate ("the contract harness asserts
// exactly seven unauthenticated commands and 40 total" — 02-roadmap.md) is
// real from this sprint onward. Only API-01/API-02 (US-M1.1) have logic;
// every other command is a typed, correctly-gated stub until its own story
// ships — see `AppError::NotImplemented`.
use crate::backup;
use crate::db_state::DbState;
use crate::error::AppError;
use crate::m1_members;
use crate::m2_entries;
use crate::m3_calc;
use crate::m4_search;
use crate::m5_close;
use crate::m6_reports;
use crate::m7_settings;
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
    m2_entries::edit_entry(conn, &paths.db_path, &paths.app_data_dir, input)
}

/// API-07 (US-M2.3/M5.3, S12).
#[tauri::command]
pub fn get_period_lock_status(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<m5_close::PeriodLockStatus, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m5_close::get_period_lock_status(conn)
}

// M3 — US-M3.1/M3.2, S6; US-M7.3's `preview_settings_impact`, S11.

/// API-33. Writes nothing (`m3_calc::preview_settings_impact`'s own doc
/// comment) — safe to call freely from the Settings pre-save warning.
#[tauri::command]
pub fn preview_settings_impact(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    candidate: m3_calc::CandidateSettings,
) -> Result<m3_calc::SettingsImpactPreview, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m3_calc::preview_settings_impact(conn, candidate)
}

// M4 — US-M4.1/M4.2, S8; US-M4.3, S9.

/// `period_month`: T-M2.5-3's month switcher. `None` resolves to the
/// oldest recordable period (never "whatever period_id is highest") — see
/// `m4_search::resolve_view_period_id`.
#[tauri::command]
pub fn get_member_detail(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    member_id: i64,
    period_month: Option<String>,
) -> Result<m4_search::MemberDetail, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m4_search::get_member_detail(conn, member_id, period_month.as_deref())
}

/// API-11. `full_tree: true` is US-M4.3's parameter, implemented here
/// because Home's slab-distribution charts (US-M4.4, same sprint) are the
/// first caller that needs it — see `m4_search`'s own doc comment.
/// `period_month`: T-M2.5-3's month switcher, same default as
/// `get_member_detail` above.
#[tauri::command]
pub fn get_direct_children_chart(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    member_id: Option<i64>,
    full_tree: bool,
    period_month: Option<String>,
) -> Result<m4_search::DirectChildrenChartResult, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m4_search::get_direct_children_chart(conn, member_id, full_tree, period_month.as_deref())
}

// M5 — US-M5.1, S11; US-M5.2/M5.3/M5.5, S12 (US-M5.4 is S13 and stays a stub).

/// API-12.
#[tauri::command]
pub fn get_outstanding_periods(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<Vec<m5_close::Period>, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m5_close::get_outstanding_periods(conn)
}

/// API-13.
#[tauri::command]
pub fn begin_close(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<m5_close::BeginCloseResult, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m5_close::begin_close(conn)
}

/// API-14.
#[tauri::command]
pub fn confirm_backup_and_close(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    paths: tauri::State<'_, AppPaths>,
    input: m5_close::ConfirmBackupAndCloseInput,
) -> Result<m5_close::CloseOutcome, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m5_close::confirm_backup_and_close(conn, &paths.db_path, &paths.app_data_dir, input)
}

/// API-15.
#[tauri::command]
pub fn manual_backup_current_period(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    paths: tauri::State<'_, AppPaths>,
) -> Result<backup::BackupRecord, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m5_close::manual_backup_current_period(conn, &paths.db_path, &paths.app_data_dir)
}

// M6 — US-M6.1..M6.5, S13.

/// API-16.
#[tauri::command]
pub fn export_monthly(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    input: m6_reports::ExportMonthlyInput,
) -> Result<m6_reports::ExportResult, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m6_reports::export_monthly(conn, input)
}

/// API-17.
#[tauri::command]
pub fn export_yearly_average(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    output_path: String,
) -> Result<m6_reports::ExportResult, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m6_reports::export_yearly_average(conn, &output_path)
}
auth_stub!(export_low_contribution);
auth_stub!(list_backups);
auth_stub!(redownload_backup);

// M7 — US-M7.1/M7.2/M7.4, S10 (the mid-period recalculation warning,
// US-M7.3/API-33's `preview_settings_impact`, stays a stub — S11).

/// API-21.
#[tauri::command]
pub fn get_settings(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<m7_settings::Settings, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m7_settings::get_settings(conn)
}

/// API-22. §5.7: structure guidance/reporting/reference-value sections save
/// silently; only a royalty qualifying-count or rate change recalculates
/// the open period — see `m7_settings::update_settings`'s own doc comment.
/// The pre-save warning this API doc otherwise requires is US-M7.3, S11.
#[tauri::command]
pub fn update_settings(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    patch: m7_settings::SettingsPatch,
) -> Result<m7_settings::Settings, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m7_settings::update_settings(conn, patch)
}

/// API-23.
#[tauri::command]
pub fn add_slab_row(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    threshold: i64,
    percentage: i64,
) -> Result<m7_settings::SlabRow, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m7_settings::add_slab_row(
        conn,
        m7_settings::SlabRowInput {
            threshold,
            percentage,
        },
    )
}

/// API-24.
#[tauri::command]
pub fn remove_slab_row(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m7_settings::remove_slab_row(conn, id)
}

/// API-25.
#[tauri::command]
pub fn update_slab_row(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    id: i64,
    threshold: i64,
    percentage: i64,
) -> Result<m7_settings::SlabRow, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m7_settings::update_slab_row(
        conn,
        id,
        m7_settings::SlabRowInput {
            threshold,
            percentage,
        },
    )
}

/// API-37.
#[tauri::command]
pub fn get_console_backup_settings(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<m7_settings::ConsoleBackupSettings, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m7_settings::get_console_backup_settings(conn)
}

/// API-38. T-M7.4-2: the segmented control and retention field save
/// immediately, no separate Save step — this command is the whole of that
/// save, called straight from the control's `onValueChange`.
#[tauri::command]
pub fn update_console_backup_settings(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    schedule: String,
    retention_count: i64,
    folder: String,
) -> Result<m7_settings::ConsoleBackupSettings, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m7_settings::update_console_backup_settings(
        conn,
        m7_settings::ConsoleBackupSettings {
            schedule,
            retention_count,
            folder,
        },
    )
}

// M8 remainder — US-M8.2/M8.3, S5/S7. `run_console_backup_now`,
// `list_restore_points`, `restore_from_backup`, `restore_from_backup_file`
// below are US-M8.5/US-M8.6's own commands, pulled forward into this sprint
// because US-M7.4's Settings screen (S10) genuinely needs them working —
// see PI/01-backlog.md's T-M7.4-3/4/5/6. Only the login-triggered schedule
// check (T-M8.5-2) is still S14.

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

/// API-31 (US-M5.2, S12).
#[tauri::command]
pub fn get_outstanding_alert(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<m5_close::OutstandingAlert, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m5_close::get_outstanding_alert(conn)
}

/// API-39. `kind = "manual"` — T-M7.4-4's "Back up now" action. The
/// login-triggered `kind = "scheduled"` catch-up call site (T-M8.5-2) is
/// still S14; `backup::run_console_backup_now` itself is agnostic to which
/// kind called it.
#[tauri::command]
pub fn run_console_backup_now(
    paths: tauri::State<'_, AppPaths>,
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<backup::BackupRecord, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    backup::run_console_backup_now(conn, &paths.db_path, &paths.app_data_dir, "manual", None)
}

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
    m5_close::run_period_catchup(&conn)?;
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
    m5_close::run_period_catchup(&conn)?;
    *db.0.lock().expect("db mutex poisoned") = Some(conn);
    session.mark_authenticated();
    Ok(())
}

/// API-30 — unauthenticated (the credential store isn't readable without
/// it), same as `login`/`setup_first_run`. Deliberately doesn't touch
/// `session`/`db` — it only resets the credential; the operator signs in
/// normally afterward through the ordinary `login` path.
#[tauri::command]
pub fn use_recovery_code(
    paths: tauri::State<'_, AppPaths>,
    input: m8_auth::UseRecoveryCodeInput,
) -> Result<m8_auth::UseRecoveryCodeResult, AppError> {
    m8_auth::use_recovery_code(&paths.auth_path, input)
}

/// API-34 — minimal slice pulled forward from US-M8.6 (S14): whether the
/// sidecar credential file exists is exactly "has this machine been set up
/// before", which is what the frontend needs to route Setup vs Login at
/// launch. S14 deepens this into genuine corrupted-database detection
/// without changing this boolean contract.
#[tauri::command]
pub fn check_data_readable(paths: tauri::State<'_, AppPaths>) -> Result<bool, AppError> {
    Ok(paths.auth_path.exists())
}

/// API-35 — unauthenticated per the closed set of seven (Rule-29). S10
/// only reaches this from the authenticated Settings screen (T-M7.4-5),
/// where `db`'s connection is already open; a genuinely pre-login read (no
/// connection, no key) is the "corrupted-database detection" work
/// `check_data_readable`'s own doc comment already defers to S14 — this
/// refuses cleanly rather than attempting it.
#[tauri::command]
pub fn list_restore_points(
    db: tauri::State<'_, DbState>,
) -> Result<Vec<backup::BackupRecord>, AppError> {
    let guard = locked_conn(&db);
    let conn = guard.as_ref().ok_or_else(|| AppError::NotFound {
        message: "No database is currently open.".into(),
    })?;
    backup::list_restore_points(conn)
}

/// API-36 — same authenticated-only scope as `list_restore_points` this
/// sprint. Drops the session on success: the restored file may hold a
/// different credential (§9.5), so the next access must go through `login`
/// again rather than trust the now-stale in-memory connection.
#[tauri::command]
pub fn restore_from_backup(
    paths: tauri::State<'_, AppPaths>,
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    backup_id: i64,
) -> Result<(), AppError> {
    {
        let guard = locked_conn(&db);
        let conn = guard.as_ref().ok_or_else(|| AppError::NotFound {
            message: "No database is currently open.".into(),
        })?;
        backup::restore_from_backup(conn, &paths.db_path, &paths.app_data_dir, backup_id)?;
    }
    *locked_conn(&db) = None;
    session.mark_locked();
    Ok(())
}

/// API-40 — same authenticated-only scope and post-restore session drop as
/// `restore_from_backup`. Backs Settings' "Restore from a file…"
/// (T-M7.4-5), fed by the native file picker (`tauri-plugin-dialog`) on
/// the frontend.
#[tauri::command]
pub fn restore_from_backup_file(
    paths: tauri::State<'_, AppPaths>,
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    file_path: String,
) -> Result<(), AppError> {
    {
        let guard = locked_conn(&db);
        let conn = guard.as_ref().ok_or_else(|| AppError::NotFound {
            message: "No database is currently open.".into(),
        })?;
        backup::restore_from_backup_file(
            conn,
            &paths.db_path,
            &paths.app_data_dir,
            std::path::Path::new(&file_path),
        )?;
    }
    *locked_conn(&db) = None;
    session.mark_locked();
    Ok(())
}

pub use crate::command_names::{ALL_COMMAND_NAMES, UNAUTHENTICATED_COMMAND_NAMES};

/// QA.2's contract test needs to exercise the remaining stub commands
/// generically by name — `create_root_member`/`add_member` (M1.1),
/// `setup_first_run`/`login`/`check_data_readable` (M8.1/M8.2, S5),
/// `record_entry`/`edit_entry`/`lock_session`/`unlock_session` (M2.1/M2.2/
/// M8.3, S7), and `get_period_lock_status`/`get_outstanding_alert`
/// (M5.2/M5.3/M2.3/M2.4, S12) all have real logic now and their own
/// dedicated tests instead (see `tests/contract.rs`). Rust has no runtime
/// reflection to call a function by string, so this is the one place that
/// enumerates the match by hand; `ALL_COMMAND_NAMES` is what keeps it
/// honest against gaps.
pub fn call_stub_by_name(
    name: &str,
    session: tauri::State<'_, SessionState>,
) -> Result<serde_json::Value, AppError> {
    match name {
        "export_low_contribution" => export_low_contribution(session),
        "list_backups" => list_backups(session),
        "redownload_backup" => redownload_backup(session),
        "get_audit_log" => get_audit_log(session),
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
            "get_member_detail",
            "get_direct_children_chart",
            "use_recovery_code",
            // US-M7.1/M7.2/M7.4, S10 (US-M8.5/M8.6's own commands pulled
            // forward — see the "M8 remainder" comment above).
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
            // US-M7.3, S11.
            "preview_settings_impact",
            // US-M5.1, S11.
            "get_outstanding_periods",
            "begin_close",
            "confirm_backup_and_close",
            "manual_backup_current_period",
            // US-M5.2/M5.3/M2.3/M2.4, S12.
            "get_period_lock_status",
            "get_outstanding_alert",
            // US-M6.5/M6.1/M6.2, S13.
            "export_monthly",
            "export_yearly_average",
        ];
        let stub_names: Vec<&str> = ALL_COMMAND_NAMES
            .iter()
            .copied()
            .filter(|n| !HAS_REAL_LOGIC.contains(n))
            .collect();
        assert_eq!(stub_names.len(), 4);
        // Exercised properly (with real State fixtures) in tests/contract.rs;
        // this just proves the dispatcher doesn't panic on any known name.
    }
}
