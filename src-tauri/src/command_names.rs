// Plain data, no logic — `include!`-ed verbatim into `build.rs` (which
// can't depend on this crate's own lib target) so the ACL generation list
// and the runtime `generate_handler!`/contract-test list can never drift
// apart. See `commands.rs` for the command implementations themselves.

/// The complete, closed list — API-01 to API-45, no gaps (C2, amended for
/// API-43/44's addition — see 06-decision-log-and-open-items.md; amended
/// again for API-45's addition — correction panel "Add record", Rule-39
/// extended to creation).
pub const ALL_COMMAND_NAMES: &[&str] = &[
    "create_root_member",
    "add_member",
    "edit_member",
    "deactivate_member",
    "reactivate_member",
    "search_members",
    "record_entry",
    "edit_entry",
    "add_closed_month_entry",
    "get_period_lock_status",
    "list_period_entries",
    "preview_settings_impact",
    "get_member_detail",
    "get_direct_children_chart",
    "get_ancestor_chain",
    "get_outstanding_periods",
    "begin_close",
    "confirm_backup_and_close",
    "manual_backup_current_period",
    "export_monthly",
    "export_yearly_average",
    "export_low_contribution",
    "list_backups",
    "redownload_backup",
    "preview_monthly_data",
    "preview_yearly_average",
    "get_settings",
    "update_settings",
    "add_slab_row",
    "remove_slab_row",
    "update_slab_row",
    "get_console_backup_settings",
    "update_console_backup_settings",
    "lock_session",
    "unlock_session",
    "get_outstanding_alert",
    "run_console_backup_now",
    "get_audit_log",
    "setup_first_run",
    "login",
    "use_recovery_code",
    "check_data_readable",
    "list_restore_points",
    "restore_from_backup",
    "restore_from_backup_file",
];

/// The closed list of seven — must stay identical between
/// `04-api-specification.md` §3 and `06-security-authorization-matrix.md` §3.
pub const UNAUTHENTICATED_COMMAND_NAMES: &[&str] = &[
    "login",
    "setup_first_run",
    "use_recovery_code",
    "check_data_readable",
    "list_restore_points",
    "restore_from_backup",
    "restore_from_backup_file",
];
