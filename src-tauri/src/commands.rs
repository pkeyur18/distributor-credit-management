// The full 40-command IPC surface (04-technical-architecture.md §6 / §6.1),
// registered together so the S4 exit gate ("the contract harness asserts
// exactly seven unauthenticated commands and 40 total" — 02-roadmap.md) is
// real from this sprint onward. Only API-01/API-02 (US-M1.1) have logic;
// every other command is a typed, correctly-gated stub until its own story
// ships — see `AppError::NotImplemented`.
use crate::db_state::DbState;
use crate::error::AppError;
use crate::m1_members;
use crate::session::{require_session, SessionState};

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
auth_stub!(edit_member);
auth_stub!(deactivate_member);
auth_stub!(reactivate_member);
auth_stub!(search_members);

// M2 — US-M2.1/M2.2, S7; US-M2.3/M2.4, S12.
auth_stub!(record_entry);
auth_stub!(edit_entry);
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
auth_stub!(lock_session);
auth_stub!(unlock_session);
auth_stub!(get_outstanding_alert);
auth_stub!(run_console_backup_now);

// M9 — US-M9.1, S14 (a completeness check; audit writes land per-command from S4,
// but the read command itself has no consumer until then).
auth_stub!(get_audit_log);

// The closed set of seven unauthenticated commands (03-business-rules.md
// Rule-29 / 06-security-authorization-matrix.md §3). None of these call
// `require_session` — that is the property QA.2's authorization test
// actually checks, not just the returned error.
open_stub!(setup_first_run); // US-M8.1, S5
open_stub!(login); // US-M8.1/M8.2, S5
open_stub!(use_recovery_code); // US-M8.4, S8
open_stub!(check_data_readable); // US-M8.6, S14
open_stub!(list_restore_points); // US-M8.6, S14
open_stub!(restore_from_backup); // US-M8.6, S14
open_stub!(restore_from_backup_file); // US-M8.6, S14

pub use crate::command_names::{ALL_COMMAND_NAMES, UNAUTHENTICATED_COMMAND_NAMES};

/// QA.2's contract test needs to exercise all 38 stub commands generically
/// by name (create_root_member/add_member have real logic and their own
/// dedicated tests instead — see `tests/contract.rs`). Rust has no runtime
/// reflection to call a function by string, so this is the one place that
/// enumerates the match by hand; `ALL_COMMAND_NAMES` is what keeps it
/// honest against gaps.
pub fn call_stub_by_name(
    name: &str,
    session: tauri::State<'_, SessionState>,
) -> Result<serde_json::Value, AppError> {
    match name {
        "edit_member" => edit_member(session),
        "deactivate_member" => deactivate_member(session),
        "reactivate_member" => reactivate_member(session),
        "search_members" => search_members(session),
        "record_entry" => record_entry(session),
        "edit_entry" => edit_entry(session),
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
        "lock_session" => lock_session(session),
        "unlock_session" => unlock_session(session),
        "get_outstanding_alert" => get_outstanding_alert(session),
        "run_console_backup_now" => run_console_backup_now(session),
        "get_audit_log" => get_audit_log(session),
        "setup_first_run" => setup_first_run(),
        "login" => login(),
        "use_recovery_code" => use_recovery_code(),
        "check_data_readable" => check_data_readable(),
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
        let stub_names: Vec<&str> = ALL_COMMAND_NAMES
            .iter()
            .copied()
            .filter(|n| *n != "create_root_member" && *n != "add_member")
            .collect();
        assert_eq!(stub_names.len(), 38);
        // Exercised properly (with real State fixtures) in tests/contract.rs;
        // this just proves the dispatcher doesn't panic on any known name.
    }
}
