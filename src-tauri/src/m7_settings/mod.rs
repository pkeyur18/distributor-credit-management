// M7 — Settings (04-technical-architecture.md §6.4; 02-business-rules.md §6,
// the 16-row inventory). US-M7.1 (slab table, API-23/24/25), US-M7.2
// (royalty/structure-guidance/reporting/access, API-21/22), US-M7.4's
// schedule/retention config (API-37/38) — all S10. `console_backup_folder`
// (row 16) travels with schedule/retention through API-37/38 rather than
// through `update_settings`, matching the frontend's own `ConsoleBackupSettings`
// contract (`m7-settings.ts`). The mid-period recalculation warning
// (RQ-18/V7.6, API-33) is US-M7.3, S11 — every save here lands silently,
// exactly as this sprint's own exit gate expects.
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::m3_calc;

fn today_iso() -> String {
    chrono::Local::now().date_naive().to_string()
}

/// D-12/D-13's `setting` entity type, `settings_change` cause — covers both
/// a slab-row write (`entity_id` = that row's own id) and a general
/// settings-key write (no natural row, so `entity_id` is 0 — unconstrained,
/// same convention `write_console_backup_copy`'s caller uses for a backup
/// with no owning period).
fn write_audit(
    conn: &Connection,
    entity_id: i64,
    field: &str,
    new_value: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO audit_log (entity_type, entity_id, field, old_value, new_value, changed_at, cause)
         VALUES ('setting', ?1, ?2, NULL, ?3, ?4, 'settings_change')",
        rusqlite::params![entity_id, field, new_value, today_iso()],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlabRow {
    pub id: i64,
    pub threshold: i64,
    pub percentage: i64,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlabRowInput {
    pub threshold: i64,
    pub percentage: i64,
}

/// V7.1/V7.2 — per-field range checks only. **Never** a cross-row
/// monotonicity check (Rule-41/ADR-009/V7.5) — the client explicitly
/// declined that safeguard. Do not add one here.
fn validate_slab_row(input: &SlabRowInput) -> Result<(), AppError> {
    if input.threshold <= 0 {
        return Err(AppError::Validation {
            field: "threshold".into(),
            message: "Threshold must be a positive number.".into(),
        });
    }
    if !(0..=100).contains(&input.percentage) {
        return Err(AppError::Validation {
            field: "percentage".into(),
            message: "Percentage must be between 0 and 100.".into(),
        });
    }
    Ok(())
}

/// T-M7.1-2: refused outright, before any warning is offered (there is no
/// warning this sprint — M7.3 is S11).
fn refuse_duplicate_threshold(
    conn: &Connection,
    threshold: i64,
    excluding_id: Option<i64>,
) -> Result<(), AppError> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM slab_table WHERE threshold = ?1 AND id IS NOT ?2)",
        rusqlite::params![threshold, excluding_id],
        |r| r.get(0),
    )?;
    if exists {
        return Err(AppError::Conflict {
            message: "A slab row with this threshold already exists.".into(),
        });
    }
    Ok(())
}

fn load_slab_row(conn: &Connection, id: i64) -> Result<SlabRow, AppError> {
    conn.query_row(
        "SELECT id, threshold, percentage, sort_order FROM slab_table WHERE id = ?1",
        [id],
        |r| {
            Ok(SlabRow {
                id: r.get(0)?,
                threshold: r.get(1)?,
                percentage: r.get(2)?,
                sort_order: r.get(3)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "Slab row not found.".into(),
    })
}

fn recalculate_after_slab_change(conn: &Connection) -> Result<(), AppError> {
    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        m3_calc::recalculate_open_period(&tx)?;
        tx.commit()?;
        Ok(())
    } else {
        m3_calc::recalculate_open_period(conn)
    }
}

/// API-23. Rule-27: the table can grow past its current shape; nothing here
/// assumes seven rows.
pub fn add_slab_row(conn: &Connection, input: SlabRowInput) -> Result<SlabRow, AppError> {
    validate_slab_row(&input)?;
    refuse_duplicate_threshold(conn, input.threshold, None)?;

    let next_sort_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM slab_table",
        [],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO slab_table (threshold, percentage, sort_order) VALUES (?1, ?2, ?3)",
        rusqlite::params![input.threshold, input.percentage, next_sort_order],
    )?;
    let id = conn.last_insert_rowid();
    write_audit(
        conn,
        id,
        "slab_row_added",
        &format!("{}/{}%", input.threshold, input.percentage),
    )?;
    recalculate_after_slab_change(conn)?;
    load_slab_row(conn, id)
}

/// API-25.
pub fn update_slab_row(
    conn: &Connection,
    id: i64,
    input: SlabRowInput,
) -> Result<SlabRow, AppError> {
    validate_slab_row(&input)?;
    refuse_duplicate_threshold(conn, input.threshold, Some(id))?;

    let changed = conn.execute(
        "UPDATE slab_table SET threshold = ?1, percentage = ?2 WHERE id = ?3",
        rusqlite::params![input.threshold, input.percentage, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound {
            message: "Slab row not found.".into(),
        });
    }
    write_audit(
        conn,
        id,
        "slab_row_updated",
        &format!("{}/{}%", input.threshold, input.percentage),
    )?;
    recalculate_after_slab_change(conn)?;
    load_slab_row(conn, id)
}

/// API-24. T-M7.1-4's second half — the UI disables the last row's remove
/// control (with an explanatory `aria-label`), but the handler refuses on
/// its own if reached another way (V7.3).
pub fn remove_slab_row(conn: &Connection, id: i64) -> Result<(), AppError> {
    let row_count: i64 = conn.query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))?;
    if row_count <= 1 {
        return Err(AppError::Conflict {
            message: "At least one slab row must remain.".into(),
        });
    }
    let changed = conn.execute("DELETE FROM slab_table WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(AppError::NotFound {
            message: "Slab row not found.".into(),
        });
    }
    write_audit(conn, id, "slab_row_removed", "")?;
    recalculate_after_slab_change(conn)
}

// --- get_settings / update_settings (API-21/22) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearlyCycle {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub slab_thresholds: Vec<i64>,
    pub slab_percentages: Vec<i64>,
    pub reference_unit_value: i64,
    pub hierarchy_depth: i64,
    pub level2_width: i64,
    pub level3_width: i64,
    pub level4_width: i64,
    pub royalty_qualifying_count: i64,
    pub royalty_rate_percent: i64,
    pub yearly_cycle: YearlyCycle,
    pub low_contribution_threshold: i64,
    pub default_export_columns: Vec<String>,
    pub session_timeout_minutes: i64,
    pub console_backup_schedule: String,
    pub console_backup_retention_count: i64,
    pub console_backup_folder: String,
}

fn setting_value(conn: &Connection, key: &str) -> Result<String, AppError> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
        r.get(0)
    })
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: format!("Setting '{key}' not found."),
    })
}

fn setting_i64(conn: &Connection, key: &str) -> Result<i64, AppError> {
    setting_value(conn, key)?
        .parse()
        .map_err(|_| AppError::Validation {
            field: key.into(),
            message: format!("Setting '{key}' is not a valid number."),
        })
}

fn write_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

/// API-21. Every one of the 16 rows (C1) — the Settings screen's single
/// full read, slab table and backup config included, even though their
/// writes go through dedicated APIs.
pub fn get_settings(conn: &Connection) -> Result<Settings, AppError> {
    let mut stmt =
        conn.prepare("SELECT threshold, percentage FROM slab_table ORDER BY sort_order")?;
    let slab_rows: Vec<(i64, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    let slab_thresholds = slab_rows.iter().map(|(t, _)| *t).collect();
    let slab_percentages = slab_rows.iter().map(|(_, p)| *p).collect();

    let yearly_cycle: YearlyCycle = serde_json::from_str(&setting_value(conn, "yearly_cycle")?)
        .map_err(|_| AppError::Validation {
            field: "yearly_cycle".into(),
            message: "Stored yearly cycle is not valid JSON.".into(),
        })?;
    let default_export_columns: Vec<String> =
        serde_json::from_str(&setting_value(conn, "default_export_columns")?).map_err(|_| {
            AppError::Validation {
                field: "default_export_columns".into(),
                message: "Stored export columns are not valid JSON.".into(),
            }
        })?;

    Ok(Settings {
        slab_thresholds,
        slab_percentages,
        reference_unit_value: setting_i64(conn, "reference_unit_value")?,
        hierarchy_depth: setting_i64(conn, "hierarchy_depth")?,
        level2_width: setting_i64(conn, "level_2_width")?,
        level3_width: setting_i64(conn, "level_3_width")?,
        level4_width: setting_i64(conn, "level_4_width")?,
        royalty_qualifying_count: setting_i64(conn, "royalty_qualifying_count")?,
        royalty_rate_percent: setting_i64(conn, "royalty_rate_percent")?,
        yearly_cycle,
        low_contribution_threshold: setting_i64(conn, "low_contribution_threshold")?,
        default_export_columns,
        session_timeout_minutes: setting_i64(conn, "session_timeout_minutes")?,
        console_backup_schedule: setting_value(conn, "console_backup_schedule")?,
        console_backup_retention_count: setting_i64(conn, "console_backup_retention_count")?,
        console_backup_folder: setting_value(conn, "console_backup_folder")?,
    })
}

/// The general-settings subset `update_settings` accepts — rows 3–13.
/// Slab rows (1–2) go through `add_slab_row`/`update_slab_row`/
/// `remove_slab_row` only; backup schedule/retention/folder (14–16) go
/// through `update_console_backup_settings` only (see this module's own
/// doc comment).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub reference_unit_value: Option<i64>,
    pub hierarchy_depth: Option<i64>,
    pub level2_width: Option<i64>,
    pub level3_width: Option<i64>,
    pub level4_width: Option<i64>,
    pub royalty_qualifying_count: Option<i64>,
    pub royalty_rate_percent: Option<i64>,
    pub yearly_cycle: Option<YearlyCycle>,
    pub low_contribution_threshold: Option<i64>,
    pub default_export_columns: Option<Vec<String>>,
    pub session_timeout_minutes: Option<i64>,
}

fn apply_settings_patch(conn: &Connection, patch: &SettingsPatch) -> Result<(), AppError> {
    // V7.4: royalty qualifying count is a positive whole number.
    if let Some(count) = patch.royalty_qualifying_count {
        if count <= 0 {
            return Err(AppError::Validation {
                field: "royaltyQualifyingCount".into(),
                message: "The royalty qualifying count must be a positive whole number.".into(),
            });
        }
    }
    // V6.3: the low-contribution threshold is a positive number.
    if let Some(threshold) = patch.low_contribution_threshold {
        if threshold <= 0 {
            return Err(AppError::Validation {
                field: "lowContributionThreshold".into(),
                message: "The low-contribution threshold must be a positive number.".into(),
            });
        }
    }

    if let Some(v) = patch.reference_unit_value {
        write_setting(conn, "reference_unit_value", &v.to_string())?;
        write_audit(conn, 0, "reference_unit_value", &v.to_string())?;
    }
    if let Some(v) = patch.hierarchy_depth {
        write_setting(conn, "hierarchy_depth", &v.to_string())?;
        write_audit(conn, 0, "hierarchy_depth", &v.to_string())?;
    }
    if let Some(v) = patch.level2_width {
        write_setting(conn, "level_2_width", &v.to_string())?;
        write_audit(conn, 0, "level_2_width", &v.to_string())?;
    }
    if let Some(v) = patch.level3_width {
        write_setting(conn, "level_3_width", &v.to_string())?;
        write_audit(conn, 0, "level_3_width", &v.to_string())?;
    }
    if let Some(v) = patch.level4_width {
        write_setting(conn, "level_4_width", &v.to_string())?;
        write_audit(conn, 0, "level_4_width", &v.to_string())?;
    }
    if let Some(v) = patch.royalty_qualifying_count {
        write_setting(conn, "royalty_qualifying_count", &v.to_string())?;
        write_audit(conn, 0, "royalty_qualifying_count", &v.to_string())?;
    }
    if let Some(v) = patch.royalty_rate_percent {
        write_setting(conn, "royalty_rate_percent", &v.to_string())?;
        write_audit(conn, 0, "royalty_rate_percent", &v.to_string())?;
    }
    if let Some(v) = &patch.yearly_cycle {
        let json = serde_json::to_string(v).expect("YearlyCycle always serializes");
        write_setting(conn, "yearly_cycle", &json)?;
        write_audit(conn, 0, "yearly_cycle", &json)?;
    }
    if let Some(v) = patch.low_contribution_threshold {
        write_setting(conn, "low_contribution_threshold", &v.to_string())?;
        write_audit(conn, 0, "low_contribution_threshold", &v.to_string())?;
    }
    if let Some(v) = &patch.default_export_columns {
        let json = serde_json::to_string(v).expect("Vec<String> always serializes");
        write_setting(conn, "default_export_columns", &json)?;
        write_audit(conn, 0, "default_export_columns", &json)?;
    }
    if let Some(v) = patch.session_timeout_minutes {
        write_setting(conn, "session_timeout_minutes", &v.to_string())?;
        write_audit(conn, 0, "session_timeout_minutes", &v.to_string())?;
    }
    Ok(())
}

/// API-22. §5.7: structure guidance, reporting and the reference value save
/// silently — they change nothing already calculated. Only a royalty
/// qualifying-count or rate change recalculates the current open period
/// (T-M7.2-2); the other sections never do.
pub fn update_settings(conn: &Connection, patch: SettingsPatch) -> Result<Settings, AppError> {
    let recalculates =
        patch.royalty_qualifying_count.is_some() || patch.royalty_rate_percent.is_some();

    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        apply_settings_patch(&tx, &patch)?;
        if recalculates {
            m3_calc::recalculate_open_period(&tx)?;
        }
        tx.commit()?;
    } else {
        apply_settings_patch(conn, &patch)?;
        if recalculates {
            m3_calc::recalculate_open_period(conn)?;
        }
    }
    get_settings(conn)
}

// --- get_console_backup_settings / update_console_backup_settings (API-37/38) ---

const VALID_SCHEDULES: &[&str] = &["off", "daily", "weekly", "monthly"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleBackupSettings {
    pub schedule: String,
    pub retention_count: i64,
    pub folder: String,
}

/// API-37.
pub fn get_console_backup_settings(conn: &Connection) -> Result<ConsoleBackupSettings, AppError> {
    Ok(ConsoleBackupSettings {
        schedule: setting_value(conn, "console_backup_schedule")?,
        retention_count: setting_i64(conn, "console_backup_retention_count")?,
        folder: setting_value(conn, "console_backup_folder")?,
    })
}

/// API-38. Rule-43: schedule is a closed enum, retention count >= 1. Never
/// recalculates — backup config touches no calculated figure.
pub fn update_console_backup_settings(
    conn: &Connection,
    input: ConsoleBackupSettings,
) -> Result<ConsoleBackupSettings, AppError> {
    if !VALID_SCHEDULES.contains(&input.schedule.as_str()) {
        return Err(AppError::Validation {
            field: "schedule".into(),
            message: "Schedule must be one of off, daily, weekly, monthly.".into(),
        });
    }
    if input.retention_count < 1 {
        return Err(AppError::Validation {
            field: "retentionCount".into(),
            message: "Retention count must be at least 1.".into(),
        });
    }

    write_setting(conn, "console_backup_schedule", &input.schedule)?;
    write_setting(
        conn,
        "console_backup_retention_count",
        &input.retention_count.to_string(),
    )?;
    write_setting(conn, "console_backup_folder", &input.folder)?;
    get_console_backup_settings(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded() -> Connection {
        db::open_seeded_in_memory().unwrap()
    }

    // --- slab table ---

    #[test]
    fn add_slab_row_grows_past_the_default_seven_rows() {
        let conn = seeded();
        let row = add_slab_row(
            &conn,
            SlabRowInput {
                threshold: 2_000_000,
                percentage: 16,
            },
        )
        .unwrap();
        assert_eq!(row.threshold, 2_000_000);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 8);
    }

    #[test]
    fn add_slab_row_refuses_a_duplicate_threshold_before_anything_else() {
        let conn = seeded();
        let err = add_slab_row(
            &conn,
            SlabRowInput {
                threshold: 10_000, // already the 2% row's threshold
                percentage: 99,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Conflict { .. }));
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 7, "a refused row must never be inserted");
    }

    #[test]
    fn add_slab_row_refuses_a_non_positive_threshold() {
        let conn = seeded();
        let err = add_slab_row(
            &conn,
            SlabRowInput {
                threshold: 0,
                percentage: 5,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn add_slab_row_refuses_a_percentage_outside_zero_to_a_hundred() {
        let conn = seeded();
        let err = add_slab_row(
            &conn,
            SlabRowInput {
                threshold: 2_000_000,
                percentage: 400,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn update_slab_row_allows_keeping_its_own_threshold() {
        let conn = seeded();
        let id: i64 = conn
            .query_row(
                "SELECT id FROM slab_table WHERE threshold = 10000",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let row = update_slab_row(
            &conn,
            id,
            SlabRowInput {
                threshold: 10_000,
                percentage: 3,
            },
        )
        .unwrap();
        assert_eq!(row.percentage, 3);
    }

    #[test]
    fn update_slab_row_moves_a_threshold_matching_the_clients_own_example() {
        // The 6% row moved to 1,000 (×100 = 100,000) — one of the two named
        // client examples in Rule-4/AC-33.
        let conn = seeded();
        let id: i64 = conn
            .query_row("SELECT id FROM slab_table WHERE percentage = 6", [], |r| {
                r.get(0)
            })
            .unwrap();
        let row = update_slab_row(
            &conn,
            id,
            SlabRowInput {
                threshold: 100_000,
                percentage: 6,
            },
        )
        .unwrap();
        assert_eq!(row.threshold, 100_000);
    }

    #[test]
    fn update_slab_row_refuses_colliding_with_a_different_rows_threshold() {
        let conn = seeded();
        let id: i64 = conn
            .query_row("SELECT id FROM slab_table WHERE percentage = 6", [], |r| {
                r.get(0)
            })
            .unwrap();
        let err = update_slab_row(
            &conn,
            id,
            SlabRowInput {
                threshold: 10_000, // the 2% row's threshold
                percentage: 6,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Conflict { .. }));
    }

    #[test]
    fn remove_slab_row_shrinks_the_table() {
        let conn = seeded();
        let id: i64 = conn
            .query_row("SELECT id FROM slab_table WHERE percentage = 14", [], |r| {
                r.get(0)
            })
            .unwrap();
        remove_slab_row(&conn, id).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 6);
    }

    #[test]
    fn remove_slab_row_refuses_the_last_remaining_row_with_a_named_message() {
        let conn = seeded();
        let ids: Vec<i64> = {
            let mut stmt = conn.prepare("SELECT id FROM slab_table").unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };
        for id in &ids[..ids.len() - 1] {
            remove_slab_row(&conn, *id).unwrap();
        }
        let last = ids[ids.len() - 1];
        let err = remove_slab_row(&conn, last).unwrap_err();
        match err {
            AppError::Conflict { message } => {
                assert_eq!(message, "At least one slab row must remain.")
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "the last row must survive the refused removal");
    }

    /// T-M7.1-6: the deliberate negative test. A non-monotonic table is not
    /// blocked, and the resulting negative differential computes and
    /// displays as-is — not silently clamped (Rule-9's caveat, Rule-41).
    #[test]
    fn a_non_monotonic_table_is_not_blocked_and_produces_a_negative_differential() {
        let conn = seeded();
        let month = chrono::Local::now().format("%Y-%m").to_string();
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
            [&month],
        )
        .unwrap();
        let period: i64 = conn.last_insert_rowid();

        let parent = insert_member(&conn, None);
        let child = insert_member(&conn, Some(parent));
        insert_entry(&conn, child, &month, 1_000_000); // child TBV lands on the 14% row
        m3_calc::recalculate_chain(&conn, child, period).unwrap();

        // Break monotonicity the way a misconfigured admin actually would:
        // add a new top row at a *lower* percentage than the row beneath
        // it (V3.4/Rule-41 — add_slab_row performs no cross-row check).
        add_slab_row(
            &conn,
            SlabRowInput {
                threshold: 2_000_000,
                percentage: 5,
            },
        )
        .unwrap();

        // Parent's TBV climbs past the new row's threshold while the
        // child's does not — parent's TBV is still structurally >= the
        // child's (Rule-9), but its slab is now *lower*.
        insert_entry(&conn, parent, &month, 1_100_000);
        m3_calc::recalculate_chain(&conn, parent, period).unwrap();

        let (parent_slab, differential): (i64, i64) = conn
            .query_row(
                "SELECT slab_pct, differential FROM member_period_totals
                 WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![parent, period],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            parent_slab, 5,
            "parent's TBV must land on the newly added top row"
        );
        assert!(
            differential < 0,
            "an accepted-risk misconfiguration must compute a real negative differential, not a clamped zero"
        );
    }

    fn insert_member(conn: &Connection, introducer: Option<i64>) -> i64 {
        static NEXT_PHONE: std::sync::atomic::AtomicI64 =
            std::sync::atomic::AtomicI64::new(9_100_000_000);
        let phone = NEXT_PHONE
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string();
        conn.execute(
            "INSERT INTO members
                (name, phone, address, introducer_member_id, level, is_active,
                 joining_date, consent_given, consent_date, created_at)
             VALUES ('T', ?1, 'addr', ?2, 1, 1, '2026-01-01', 1, '2026-01-01', '2026-01-01')",
            rusqlite::params![phone, introducer],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_entry(conn: &Connection, member_id: i64, month: &str, amount: i64) {
        conn.execute(
            "INSERT INTO business_volume_entries
                (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                member_id,
                amount,
                format!("{month}-15"),
                month,
                format!("{month}-15")
            ],
        )
        .unwrap();
    }

    // --- get_settings / update_settings ---

    #[test]
    fn get_settings_reads_all_sixteen_rows() {
        let conn = seeded();
        let settings = get_settings(&conn).unwrap();
        assert_eq!(settings.slab_thresholds.len(), 7);
        assert_eq!(settings.hierarchy_depth, 4);
        assert_eq!(settings.session_timeout_minutes, 15);
        assert_eq!(settings.console_backup_schedule, "off");
        assert_eq!(settings.console_backup_retention_count, 10);
    }

    #[test]
    fn update_settings_writes_only_the_patched_keys() {
        let conn = seeded();
        let updated = update_settings(
            &conn,
            SettingsPatch {
                session_timeout_minutes: Some(30),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(updated.session_timeout_minutes, 30);
        assert_eq!(
            updated.hierarchy_depth, 4,
            "an unpatched key must be untouched"
        );
    }

    #[test]
    fn update_settings_refuses_a_non_positive_royalty_qualifying_count() {
        let conn = seeded();
        let err = update_settings(
            &conn,
            SettingsPatch {
                royalty_qualifying_count: Some(0),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn update_settings_refuses_a_non_positive_low_contribution_threshold() {
        let conn = seeded();
        let err = update_settings(
            &conn,
            SettingsPatch {
                low_contribution_threshold: Some(0),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn a_royalty_rate_change_recalculates_the_open_period() {
        let conn = seeded();
        let month = chrono::Local::now().format("%Y-%m").to_string();
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
            [&month],
        )
        .unwrap();
        let period: i64 = conn.last_insert_rowid();
        let parent = insert_member(&conn, None);
        for _ in 0..3 {
            let child = insert_member(&conn, Some(parent));
            insert_entry(&conn, child, &month, 1_000_000);
            m3_calc::recalculate_chain(&conn, child, period).unwrap();
        }
        let royalty_before: i64 = conn
            .query_row(
                "SELECT royalty FROM member_period_totals WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![parent, period],
                |r| r.get(0),
            )
            .unwrap();

        update_settings(
            &conn,
            SettingsPatch {
                royalty_rate_percent: Some(5),
                ..Default::default()
            },
        )
        .unwrap();

        let royalty_after: i64 = conn
            .query_row(
                "SELECT royalty FROM member_period_totals WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![parent, period],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            royalty_after > royalty_before,
            "raising the royalty rate must recalculate the open period immediately"
        );
    }

    #[test]
    fn a_structure_guidance_change_does_not_recalculate() {
        // §5.7: structure guidance, reporting and the reference value save
        // silently — no open period is even required for these to succeed.
        let conn = seeded();
        let updated = update_settings(
            &conn,
            SettingsPatch {
                hierarchy_depth: Some(5),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(updated.hierarchy_depth, 5);
    }

    // --- console backup settings ---

    #[test]
    fn console_backup_settings_round_trip() {
        let conn = seeded();
        let updated = update_console_backup_settings(
            &conn,
            ConsoleBackupSettings {
                schedule: "weekly".into(),
                retention_count: 20,
                folder: "backups".into(),
            },
        )
        .unwrap();
        assert_eq!(updated.schedule, "weekly");
        assert_eq!(updated.retention_count, 20);

        let reread = get_console_backup_settings(&conn).unwrap();
        assert_eq!(reread.schedule, "weekly");
        assert_eq!(reread.retention_count, 20);
    }

    #[test]
    fn console_backup_settings_refuses_an_invalid_schedule() {
        let conn = seeded();
        let err = update_console_backup_settings(
            &conn,
            ConsoleBackupSettings {
                schedule: "hourly".into(),
                retention_count: 10,
                folder: "backups".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn console_backup_settings_refuses_a_retention_count_below_one() {
        let conn = seeded();
        let err = update_console_backup_settings(
            &conn,
            ConsoleBackupSettings {
                schedule: "daily".into(),
                retention_count: 0,
                folder: "backups".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }
}
