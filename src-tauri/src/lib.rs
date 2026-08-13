// `pub` throughout: `tests/*.rs` (QA.2's contract harness) compiles as a
// crate external to this lib and needs to reach these directly — see
// T-QA.2-1's "direct Tauri command-invocation" requirement.
pub mod backup;
pub mod command_names;
pub mod commands;
pub mod db;
pub mod db_state;
pub mod error;
pub mod m1_members;
pub mod m2_entries;
pub mod m3_calc;
pub mod m4_search;
pub mod m5_close;
pub mod m6_reports;
pub mod m7_settings;
pub mod m8_auth;
pub mod m9_audit;
pub mod paths;
pub mod qa_dataset;
pub mod session;

use db_state::DbState;
use paths::AppPaths;
use session::SessionState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // T-M7.4-5: the native file picker behind Settings' "Restore from a
        // file…" — an official Tauri plugin rather than hand-rolling one.
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(AppPaths::resolve(app.handle())?);
            Ok(())
        })
        .manage(SessionState::new())
        .manage(DbState::new())
        .invoke_handler(tauri::generate_handler![
            commands::create_root_member,
            commands::add_member,
            commands::edit_member,
            commands::deactivate_member,
            commands::reactivate_member,
            commands::search_members,
            commands::record_entry,
            commands::edit_entry,
            commands::get_period_lock_status,
            commands::list_period_entries,
            commands::preview_settings_impact,
            commands::get_member_detail,
            commands::get_direct_children_chart,
            commands::get_outstanding_periods,
            commands::begin_close,
            commands::confirm_backup_and_close,
            commands::manual_backup_current_period,
            commands::export_monthly,
            commands::export_yearly_average,
            commands::export_low_contribution,
            commands::list_backups,
            commands::redownload_backup,
            commands::get_settings,
            commands::update_settings,
            commands::add_slab_row,
            commands::remove_slab_row,
            commands::update_slab_row,
            commands::get_console_backup_settings,
            commands::update_console_backup_settings,
            commands::lock_session,
            commands::unlock_session,
            commands::get_outstanding_alert,
            commands::run_console_backup_now,
            commands::get_audit_log,
            commands::setup_first_run,
            commands::login,
            commands::use_recovery_code,
            commands::check_data_readable,
            commands::list_restore_points,
            commands::restore_from_backup,
            commands::restore_from_backup_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
