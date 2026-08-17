// M5 — Monthly Close (04-technical-architecture.md §3.1/§7.1;
// 02-business-rules.md Rule-17/18/20/21/38). US-M5.1, S11.
//
// The physical database file is one SQLCipher file regardless of which
// period is "in progress" or "closing" — there is no per-period file. What
// distinguishes a `period_close` backup from a whole-console `manual`/
// `scheduled` one (Rule-43, `backup.rs`) is purely which conceptual event
// triggered the copy and how the `backups` row is labelled, not the bytes
// copied.
use std::path::Path;

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::backup;
use crate::error::AppError;
use crate::m3_calc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub id: i64,
    pub period_month: String,
    pub status: String,
    pub ended_at: Option<String>,
    pub closed_at: Option<String>,
}

fn row_to_period(r: &rusqlite::Row) -> rusqlite::Result<Period> {
    Ok(Period {
        id: r.get(0)?,
        period_month: r.get(1)?,
        status: r.get(2)?,
        ended_at: r.get(3)?,
        closed_at: r.get(4)?,
    })
}

const PERIOD_COLUMNS: &str = "id, period_month, status, ended_at, closed_at";

/// API-12: months awaiting close, oldest first (Rule-20's queue — every
/// outstanding month is listed, only the oldest is closable).
pub fn get_outstanding_periods(conn: &Connection) -> Result<Vec<Period>, AppError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PERIOD_COLUMNS} FROM periods WHERE status = 'awaiting_close' ORDER BY period_month ASC"
    ))?;
    let rows = stmt
        .query_map([], row_to_period)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn oldest_outstanding_period_id(conn: &Connection) -> Result<Option<i64>, AppError> {
    Ok(conn
        .query_row(
            "SELECT id FROM periods WHERE status = 'awaiting_close' ORDER BY period_month ASC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginCloseResult {
    pub period_id: i64,
    pub member_count: i64,
    pub with_entry_count: i64,
    pub top_slab_count: i64,
}

/// API-13: a prepare step, no write (Rule-17 — closing itself stays manual,
/// pressed later in the wizard). Always resolves to the oldest outstanding
/// period — there is no id parameter to request a different one, so
/// AC-21's "only the oldest may begin" holds by construction, not by a
/// rejected input.
pub fn begin_close(conn: &Connection) -> Result<BeginCloseResult, AppError> {
    let period_id = oldest_outstanding_period_id(conn)?.ok_or_else(|| AppError::NotFound {
        message: "No month is currently outstanding.".into(),
    })?;

    let member_count: i64 = conn.query_row("SELECT COUNT(*) FROM members", [], |r| r.get(0))?;
    let with_entry_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM member_period_totals WHERE period_id = ?1 AND business_volume > 0",
        [period_id],
        |r| r.get(0),
    )?;
    let top_slab_pct: Option<i64> =
        conn.query_row("SELECT MAX(percentage) FROM slab_table", [], |r| r.get(0))?;
    let top_slab_count: i64 = match top_slab_pct {
        Some(pct) => conn.query_row(
            "SELECT COUNT(*) FROM member_period_totals WHERE period_id = ?1 AND slab_pct = ?2",
            rusqlite::params![period_id, pct],
            |r| r.get(0),
        )?,
        None => 0,
    };

    Ok(BeginCloseResult {
        period_id,
        member_count,
        with_entry_count,
        top_slab_count,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmBackupAndCloseInput {
    pub period_id: i64,
    pub external_medium_path: Option<String>,
}

/// RQ-16/Rule-23: true only when nothing was ever recorded anywhere in the
/// hierarchy during this calendar month. `business_volume_entries.amount`
/// has a `CHECK (amount > 0)` constraint, so row-existence alone is a
/// correct emptiness check — there is no such thing as a zero-amount entry
/// to filter out.
fn period_has_any_activity(conn: &Connection, period_month: &str) -> Result<bool, AppError> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM business_volume_entries WHERE period_month = ?1)",
        [period_month],
        |r| r.get(0),
    )?)
}

/// Rule-38: an immutable snapshot at version 1 for **every** member, not
/// only those with activity this period — a member with no row yet in
/// `member_period_totals` (nothing entered under them this period) still
/// gets a zero-figure snapshot, same `COALESCE` shape `direct_children_figures`
/// already uses for the same reason. `pub`, not `pub(crate)`: `generate_dataset`
/// (T-QA.5-3) is its own binary crate and reuses this rather than
/// re-deriving the same snapshot shape a second way.
///
/// Callers must check `period_has_any_activity` first (RQ-16/T-M5.4-1): a
/// calendar month that elapses with zero entries anywhere in the hierarchy
/// gets no snapshot at all, not a snapshot of zeroes. This function itself
/// still writes unconditionally for every member of a genuinely active
/// period — the emptiness check is the caller's job, not this one's, since
/// `generate_dataset` never produces an empty period and must keep calling
/// this directly.
pub fn write_period_close_snapshots(
    conn: &Connection,
    period_id: i64,
    created_at: &str,
) -> Result<(), AppError> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.is_active,
                COALESCE(t.business_volume, 0), COALESCE(t.total_business_volume, 0),
                COALESCE(t.slab_pct, 0), COALESCE(t.differential, 0),
                COALESCE(t.royalty, 0), COALESCE(t.own_reward, 0), COALESCE(t.rewards, 0)
         FROM members m
         LEFT JOIN member_period_totals t ON t.member_id = m.id AND t.period_id = ?1",
    )?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(i64, bool, i64, i64, i64, i64, i64, i64, i64)> = stmt
        .query_map([period_id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
            ))
        })?
        .collect::<Result<_, _>>()?;
    drop(stmt);

    for (member_id, is_active, bv, tbv, slab_pct, differential, royalty, own_reward, rewards) in
        rows
    {
        conn.execute(
            "INSERT INTO monthly_snapshots
                (member_id, period_id, version, business_volume, total_business_volume,
                 slab_pct, differential, royalty, own_reward, rewards, is_active_status, created_at)
             VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                member_id,
                period_id,
                bv,
                tbv,
                slab_pct,
                differential,
                royalty,
                own_reward,
                rewards,
                is_active,
                created_at,
            ],
        )?;
    }
    Ok(())
}

/// Rule-38: zeroes everything a snapshot has already captured — Business
/// Volume, TBV, Rewards, royalty. `pub`, not `pub(crate)`: `generate_dataset`
/// reuses this too, so a closed synthetic month ends in exactly the same
/// zeroed-live-figures state a real close leaves.
pub fn zero_period_totals(conn: &Connection, period_id: i64) -> Result<(), AppError> {
    conn.execute(
        "UPDATE member_period_totals SET
            business_volume = 0, total_business_volume = 0, slab_pct = 0,
            differential = 0, royalty = 0, own_reward = 0, rewards = 0
         WHERE period_id = ?1",
        [period_id],
    )?;
    Ok(())
}

/// Rule-18/38's strict order, all inside one transaction: write+verify
/// backup → write snapshots v1 → zero live figures → mark closed. The
/// backup runs **first**, before this transaction's own writes touch the
/// file — SQLite's rollback journal leaves the main file unchanged until
/// COMMIT (`backup.rs`'s own module doc), and nothing has written to it yet
/// at this point, so the copy correctly captures the live pre-close
/// figures Rule-18 requires backed up, not the about-to-be-zeroed state.
/// The external-medium copy (Rule-31/RQ-19) is attempted at the same time
/// but never blocks — a failure there is never propagated as an error, but
/// is still reported back (see `CloseOutcome`) so the caller can remind the
/// operator rather than let it fail silently and unnoticed.
fn write_period_close_backup_and_snapshots(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    period_id: i64,
    external_medium_path: Option<&str>,
    today: &str,
) -> Result<bool, AppError> {
    let version =
        backup::write_backup_copy(conn, db_path, app_data_dir, period_id, "period_close")?;
    let mut external_medium_copy_failed = false;
    if let Some(external_path) = external_medium_path {
        let internal_path: Option<String> = conn
            .query_row(
                "SELECT internal_retained_path FROM backups WHERE period_id = ?1 AND version = ?2",
                rusqlite::params![period_id, version],
                |r| r.get(0),
            )
            .optional()?;
        external_medium_copy_failed = match internal_path {
            Some(internal_path) => std::fs::copy(internal_path, external_path).is_err(),
            None => true,
        };
    }

    let period_month: String = conn.query_row(
        "SELECT period_month FROM periods WHERE id = ?1",
        [period_id],
        |r| r.get(0),
    )?;
    if period_has_any_activity(conn, &period_month)? {
        write_period_close_snapshots(conn, period_id, today)?;
    }
    zero_period_totals(conn, period_id)?;
    conn.execute(
        "UPDATE periods SET status = 'closed', closed_at = ?2 WHERE id = ?1",
        rusqlite::params![period_id, today],
    )?;
    crate::m9_audit::write_audit_entry(
        conn,
        "period",
        period_id,
        "status",
        Some("awaiting_close"),
        Some("closed"),
        "period_close",
    )?;
    Ok(external_medium_copy_failed)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseOutcome {
    /// Rule-31: true only when an external-medium path was actually
    /// requested and the copy to it failed — never blocks the close, but
    /// the caller should remind the operator to back it up separately.
    pub external_medium_copy_failed: bool,
}

/// API-14.
pub fn confirm_backup_and_close(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    input: ConfirmBackupAndCloseInput,
) -> Result<CloseOutcome, AppError> {
    let period_id = input.period_id;
    let status: Option<String> = conn
        .query_row(
            "SELECT status FROM periods WHERE id = ?1",
            [period_id],
            |r| r.get(0),
        )
        .optional()?;
    match status.as_deref() {
        Some("awaiting_close") => {}
        Some(_) => {
            return Err(AppError::Conflict {
                message: "This month is not awaiting close.".into(),
            })
        }
        None => {
            return Err(AppError::NotFound {
                message: "Period not found.".into(),
            })
        }
    }
    if oldest_outstanding_period_id(conn)? != Some(period_id) {
        return Err(AppError::Conflict {
            message: "Only the oldest outstanding month may be closed.".into(),
        });
    }

    let today = chrono::Local::now().date_naive().to_string();
    let external_medium_copy_failed = if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        let failed = write_period_close_backup_and_snapshots(
            &tx,
            db_path,
            app_data_dir,
            period_id,
            input.external_medium_path.as_deref(),
            &today,
        )?;
        tx.commit()?;
        failed
    } else {
        write_period_close_backup_and_snapshots(
            conn,
            db_path,
            app_data_dir,
            period_id,
            input.external_medium_path.as_deref(),
            &today,
        )?
    };
    Ok(CloseOutcome {
        external_medium_copy_failed,
    })
}

/// API-15: on-demand backup of the in-progress month, no zeroing. Same
/// physical write as Settings' "Back up now" (`backup::run_console_backup_now`,
/// kind `manual`) — just a distinct audit cause (`manual_backup`, its own
/// API row, not `console_backup`) and pruned the same way rather than
/// skipping retention just because this call site is different.
pub fn manual_backup_current_period(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
) -> Result<backup::BackupRecord, AppError> {
    m3_calc::current_open_period_id(conn)?.ok_or_else(|| AppError::NotFound {
        message: "No month is currently in progress.".into(),
    })?;

    let id = backup::write_console_backup_copy(conn, db_path, app_data_dir, "manual", None)?;
    crate::m9_audit::write_audit_entry(
        conn,
        "backup",
        id,
        "kind",
        None,
        Some("manual"),
        "manual_backup",
    )?;
    let retention = backup::console_backup_retention_count(conn)?;
    backup::prune_console_backups(conn, app_data_dir, retention)?;
    backup::get_backup_record(conn, id)
}

fn current_calendar_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

fn next_month(period_month: &str) -> String {
    let first = chrono::NaiveDate::parse_from_str(&format!("{period_month}-01"), "%Y-%m-%d")
        .expect("period_month is always a valid YYYY-MM value");
    first
        .checked_add_months(chrono::Months::new(1))
        .expect("chrono month arithmetic never overflows in practice")
        .format("%Y-%m")
        .to_string()
}

fn last_day_of_month(period_month: &str) -> String {
    let first = chrono::NaiveDate::parse_from_str(&format!("{period_month}-01"), "%Y-%m-%d")
        .expect("period_month is always a valid YYYY-MM value");
    first
        .checked_add_months(chrono::Months::new(1))
        .and_then(|next| next.pred_opt())
        .expect("every month has a last day")
        .to_string()
}

/// Rule-20's queue, as bare month strings — `get_outstanding_periods`
/// already owns the query; this just projects `period_month` out of it
/// rather than re-deriving the same `WHERE status = 'awaiting_close'`.
fn outstanding_period_months(conn: &Connection) -> Result<Vec<String>, AppError> {
    Ok(get_outstanding_periods(conn)?
        .into_iter()
        .map(|p| p.period_month)
        .collect())
}

/// US-M5.5 (D-9/D-10) — run once at successful login/setup, before the UI
/// takes over (T-M5.5-4). Two idempotent phases:
///   1. Backfill a row for every calendar month with none yet, up to and
///      including the current one, all inserted `open`.
///   2. Elapse every `open` row whose month has passed to `awaiting_close`,
///      `ended_at` set to that month's own last calendar day (not the run
///      date — the current month can't unlock while an earlier one stays
///      outstanding, so "when this period ended" must mean the calendar
///      boundary itself, not whenever the admin next happened to open the
///      console).
pub fn run_period_catchup(conn: &Connection) -> Result<(), AppError> {
    let current_month = current_calendar_month();

    let latest_existing: Option<String> =
        conn.query_row("SELECT MAX(period_month) FROM periods", [], |r| r.get(0))?;

    let mut month = match &latest_existing {
        Some(latest) if latest >= &current_month => None,
        Some(latest) => Some(next_month(latest)),
        None => Some(current_month.clone()),
    };
    while let Some(m) = month {
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
            [&m],
        )?;
        if m == current_month {
            break;
        }
        month = Some(next_month(&m));
    }

    let mut stmt = conn.prepare(
        "SELECT id, period_month FROM periods WHERE status = 'open' AND period_month < ?1",
    )?;
    let elapsed: Vec<(i64, String)> = stmt
        .query_map([&current_month], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    drop(stmt);
    for (id, period_month) in elapsed {
        let ended_at = last_day_of_month(&period_month);
        conn.execute(
            "UPDATE periods SET status = 'awaiting_close', ended_at = ?2 WHERE id = ?1",
            rusqlite::params![id, ended_at],
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodLockStatus {
    /// Oldest-first. Never a plain boolean (CR-2) — the entry screen needs
    /// to know *which* months it may record into, not just whether it can.
    pub recordable_period_months: Vec<String>,
    /// The month that must close before the current month unlocks — set
    /// only when the current month itself is not recordable.
    pub blocking_month: Option<String>,
}

/// API-07 (US-M2.3/M5.3) — Rule-36 as amended by CR-2.
pub fn get_period_lock_status(conn: &Connection) -> Result<PeriodLockStatus, AppError> {
    let outstanding = outstanding_period_months(conn)?;
    if outstanding.is_empty() {
        Ok(PeriodLockStatus {
            recordable_period_months: vec![current_calendar_month()],
            blocking_month: None,
        })
    } else {
        let blocking_month = outstanding[0].clone();
        Ok(PeriodLockStatus {
            recordable_period_months: outstanding,
            blocking_month: Some(blocking_month),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutstandingAlert {
    pub outstanding_months: Vec<String>,
    pub current_month: String,
}

/// API-31 (US-M5.2) — Rule-20's banner/notification-list data source.
pub fn get_outstanding_alert(conn: &Connection) -> Result<OutstandingAlert, AppError> {
    Ok(OutstandingAlert {
        outstanding_months: outstanding_period_months(conn)?,
        current_month: current_calendar_month(),
    })
}

/// US-M2.3/US-M2.4 (Rule-36 as amended by CR-2) — resolves `entry_date` to
/// the period it must be recorded against, or refuses it. Replaces
/// `m2_entries::get_or_create_open_period`, which was a narrow stand-in for
/// exactly this routine (see that module's own header comment).
pub fn resolve_recording_period(conn: &Connection, entry_date: &str) -> Result<i64, AppError> {
    let target_month = crate::m2_entries::period_month_of_date(entry_date)?;
    let current_month = current_calendar_month();

    if target_month > current_month {
        return Err(AppError::Validation {
            field: "entryDate".into(),
            message: "Entries cannot be dated in the future.".into(),
        });
    }

    let existing: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, status FROM periods WHERE period_month = ?1",
            [&target_month],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    let (period_id, status) = match existing {
        Some(row) => row,
        None if target_month == current_month => {
            conn.execute(
                "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
                [&target_month],
            )?;
            (conn.last_insert_rowid(), "open".to_string())
        }
        None => {
            return Err(AppError::NotFound {
                message: "That month has no recorded period.".into(),
            });
        }
    };

    match status.as_str() {
        "closed" => Err(AppError::PeriodClosed {
            month: target_month,
        }),
        "awaiting_close" => Ok(period_id),
        _ => match outstanding_period_months(conn)?.into_iter().next() {
            Some(blocking_month) => Err(AppError::PeriodNotAcceptingEntries {
                month: target_month,
                blocking_month,
            }),
            None => Ok(period_id),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded() -> Connection {
        db::open_seeded_in_memory().unwrap()
    }

    fn seeded_with_temp_db() -> (Connection, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "bvconsole-m5close-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("console.db");
        let conn = db::open_encrypted(&db_path, "test-key").unwrap();
        (conn, dir)
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

    fn insert_period(conn: &Connection, month: &str, status: &str) -> i64 {
        conn.execute(
            "INSERT INTO periods (period_month, status, ended_at) VALUES (?1, ?2, ?1)",
            [month, status],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_totals(conn: &Connection, member_id: i64, period_id: i64, bv: i64, slab_pct: i64) {
        conn.execute(
            "INSERT INTO member_period_totals
                (member_id, period_id, business_volume, total_business_volume, slab_pct,
                 differential, royalty, own_reward, rewards)
             VALUES (?1, ?2, ?3, ?3, ?4, 0, 0, 0, ?3)",
            rusqlite::params![member_id, period_id, bv, slab_pct],
        )
        .unwrap();
    }

    // T-M5.4-1/3: `period_has_any_activity` reads `business_volume_entries`
    // directly, not `member_period_totals` — `insert_totals` above seeds
    // the rollup table only, which real entry recording never does without
    // also writing an entries row, so any test exercising the empty-month
    // gate needs this instead.
    fn insert_entry(conn: &Connection, member_id: i64, period_month: &str, amount: i64) {
        conn.execute(
            "INSERT INTO business_volume_entries
                (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, ?2, ?3 || '-15', ?3, ?3 || '-15')",
            rusqlite::params![member_id, amount, period_month],
        )
        .unwrap();
    }

    fn ym_offset(months: i64) -> String {
        let today = chrono::Local::now().date_naive();
        let shifted = if months >= 0 {
            today.checked_add_months(chrono::Months::new(months as u32))
        } else {
            today.checked_sub_months(chrono::Months::new((-months) as u32))
        };
        shifted.unwrap().format("%Y-%m").to_string()
    }

    #[test]
    fn run_period_catchup_creates_current_month_on_a_fresh_install() {
        let conn = seeded();
        run_period_catchup(&conn).unwrap();

        let current = ym_offset(0);
        let status: String = conn
            .query_row(
                "SELECT status FROM periods WHERE period_month = ?1",
                [&current],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "open");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn run_period_catchup_backfills_and_elapses_across_three_unopened_months() {
        // Last login was three calendar months ago: only that month's row
        // exists (as `open`, since it was current then). Today, three
        // month boundaries have passed.
        let conn = seeded();
        insert_period(&conn, &ym_offset(-3), "open");

        run_period_catchup(&conn).unwrap();

        let elapsed = [ym_offset(-3), ym_offset(-2), ym_offset(-1)];
        for month in &elapsed {
            let (status, ended_at): (String, Option<String>) = conn
                .query_row(
                    "SELECT status, ended_at FROM periods WHERE period_month = ?1",
                    [month],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(status, "awaiting_close", "{month} must have elapsed");
            let expected_ended_at = last_day_of_month(month);
            assert_eq!(
                ended_at,
                Some(expected_ended_at),
                "{month}'s ended_at must be that month's own last calendar day, not the run date"
            );
        }

        let current = ym_offset(0);
        let current_status: String = conn
            .query_row(
                "SELECT status FROM periods WHERE period_month = ?1",
                [&current],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(current_status, "open");

        let outstanding = get_outstanding_periods(&conn).unwrap();
        assert_eq!(
            outstanding
                .iter()
                .map(|p| p.period_month.clone())
                .collect::<Vec<_>>(),
            elapsed,
            "queued oldest-first"
        );

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 4, "the three elapsed months plus the current one");
    }

    #[test]
    fn run_period_catchup_is_idempotent_on_a_second_login() {
        let conn = seeded();
        insert_period(&conn, &ym_offset(-3), "open");
        run_period_catchup(&conn).unwrap();
        let after_first: i64 = conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();

        run_period_catchup(&conn).unwrap();
        let after_second: i64 = conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();

        assert_eq!(after_first, after_second, "a second login creates nothing");
    }

    #[test]
    fn get_period_lock_status_reports_only_the_current_month_when_nothing_outstanding() {
        let conn = seeded();
        let status = get_period_lock_status(&conn).unwrap();
        assert_eq!(status.recordable_period_months, vec![ym_offset(0)]);
        assert_eq!(status.blocking_month, None);
    }

    #[test]
    fn get_period_lock_status_lists_outstanding_months_and_names_the_blocker() {
        let conn = seeded();
        insert_period(&conn, &ym_offset(-2), "awaiting_close");
        insert_period(&conn, &ym_offset(-1), "awaiting_close");
        insert_period(&conn, &ym_offset(0), "open");

        let status = get_period_lock_status(&conn).unwrap();
        assert_eq!(
            status.recordable_period_months,
            vec![ym_offset(-2), ym_offset(-1)]
        );
        assert_eq!(status.blocking_month, Some(ym_offset(-2)));
    }

    #[test]
    fn get_outstanding_alert_reports_outstanding_and_current_months() {
        let conn = seeded();
        insert_period(&conn, &ym_offset(-1), "awaiting_close");
        insert_period(&conn, &ym_offset(0), "open");

        let alert = get_outstanding_alert(&conn).unwrap();
        assert_eq!(alert.outstanding_months, vec![ym_offset(-1)]);
        assert_eq!(alert.current_month, ym_offset(0));
    }

    #[test]
    fn resolve_recording_period_accepts_an_outstanding_month() {
        let conn = seeded();
        let outstanding = ym_offset(-1);
        let period_id = insert_period(&conn, &outstanding, "awaiting_close");
        insert_period(&conn, &ym_offset(0), "open");

        let resolved = resolve_recording_period(&conn, &format!("{outstanding}-15")).unwrap();
        assert_eq!(resolved, period_id);
    }

    #[test]
    fn resolve_recording_period_refuses_the_current_month_while_earlier_is_outstanding() {
        let conn = seeded();
        let outstanding = ym_offset(-1);
        insert_period(&conn, &outstanding, "awaiting_close");
        let current = ym_offset(0);
        insert_period(&conn, &current, "open");

        let err = resolve_recording_period(&conn, &format!("{current}-05")).unwrap_err();
        match err {
            AppError::PeriodNotAcceptingEntries {
                month,
                blocking_month,
            } => {
                assert_eq!(month, current);
                assert_eq!(blocking_month, outstanding);
            }
            other => panic!("expected PeriodNotAcceptingEntries, got {other:?}"),
        }
    }

    #[test]
    fn resolve_recording_period_directs_a_closed_month_to_correction() {
        let conn = seeded();
        let closed = ym_offset(-2);
        insert_period(&conn, &closed, "closed");

        let err = resolve_recording_period(&conn, &format!("{closed}-10")).unwrap_err();
        match err {
            AppError::PeriodClosed { month } => assert_eq!(month, closed),
            other => panic!("expected PeriodClosed, got {other:?}"),
        }
    }

    #[test]
    fn resolve_recording_period_refuses_a_future_date() {
        let conn = seeded();
        let future = ym_offset(1);
        let err = resolve_recording_period(&conn, &format!("{future}-01")).unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn resolve_recording_period_accepts_the_current_month_when_nothing_outstanding() {
        let conn = seeded();
        let current = ym_offset(0);
        let period_id = insert_period(&conn, &current, "open");

        let resolved = resolve_recording_period(&conn, &format!("{current}-01")).unwrap();
        assert_eq!(resolved, period_id);
    }

    #[test]
    fn resolve_recording_period_auto_creates_the_current_month_row_if_missing() {
        // Defensive fallback for direct callers that skip login/catch-up —
        // in real operation the row always already exists.
        let conn = seeded();
        let current = ym_offset(0);
        let period_id = resolve_recording_period(&conn, &format!("{current}-01")).unwrap();

        let status: String = conn
            .query_row(
                "SELECT status FROM periods WHERE id = ?1",
                [period_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "open");
    }

    #[test]
    fn get_outstanding_periods_lists_oldest_first() {
        let conn = seeded();
        insert_period(&conn, "2026-06", "awaiting_close");
        insert_period(&conn, "2026-05", "awaiting_close");
        insert_period(&conn, "2026-07", "open");

        let periods = get_outstanding_periods(&conn).unwrap();

        assert_eq!(
            periods
                .iter()
                .map(|p| p.period_month.clone())
                .collect::<Vec<_>>(),
            vec!["2026-05", "2026-06"]
        );
    }

    #[test]
    fn begin_close_refuses_when_nothing_is_outstanding() {
        let conn = seeded();
        let err = begin_close(&conn).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn begin_close_resolves_to_the_oldest_with_its_stat_summary() {
        let conn = seeded();
        let older = insert_period(&conn, "2026-05", "awaiting_close");
        insert_period(&conn, "2026-06", "awaiting_close");
        let m1 = insert_member(&conn, None);
        let m2 = insert_member(&conn, Some(m1));
        insert_totals(&conn, m1, older, 100_000, 14); // top slab (14%)
        insert_totals(&conn, m2, older, 0, 0);

        let result = begin_close(&conn).unwrap();

        assert_eq!(result.period_id, older);
        assert_eq!(result.member_count, 2);
        assert_eq!(result.with_entry_count, 1);
        assert_eq!(result.top_slab_count, 1);
    }

    #[test]
    fn confirm_backup_and_close_refuses_a_non_oldest_period() {
        let (conn, dir) = seeded_with_temp_db();
        insert_period(&conn, "2026-05", "awaiting_close");
        let newer = insert_period(&conn, "2026-06", "awaiting_close");

        let err = confirm_backup_and_close(
            &conn,
            &dir.join("console.db"),
            &dir,
            ConfirmBackupAndCloseInput {
                period_id: newer,
                external_medium_path: None,
            },
        )
        .unwrap_err();

        assert!(matches!(err, AppError::Conflict { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn confirm_backup_and_close_writes_snapshot_zeroes_and_closes_in_one_go() {
        let (conn, dir) = seeded_with_temp_db();
        let period = insert_period(&conn, "2026-05", "awaiting_close");
        let with_activity = insert_member(&conn, None);
        let without_activity = insert_member(&conn, Some(with_activity));
        insert_totals(&conn, with_activity, period, 100_000, 4);
        insert_entry(&conn, with_activity, "2026-05", 100_000);

        confirm_backup_and_close(
            &conn,
            &dir.join("console.db"),
            &dir,
            ConfirmBackupAndCloseInput {
                period_id: period,
                external_medium_path: None,
            },
        )
        .unwrap();

        let status: String = conn
            .query_row("SELECT status FROM periods WHERE id = ?1", [period], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(status, "closed");

        let (bv, tbv): (i64, i64) = conn
            .query_row(
                "SELECT business_volume, total_business_volume FROM member_period_totals WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![with_activity, period],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((bv, tbv), (0, 0), "live figures must be zeroed after close");

        let snapshot_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM monthly_snapshots WHERE period_id = ?1",
                [period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            snapshot_count, 2,
            "every member gets a snapshot, including one with no activity this period"
        );
        let snapshotted_bv: i64 = conn
            .query_row(
                "SELECT business_volume FROM monthly_snapshots WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![with_activity, period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            snapshotted_bv, 100_000,
            "the permanent record must hold the pre-zero figure"
        );
        let no_activity_snapshot_bv: i64 = conn
            .query_row(
                "SELECT business_volume FROM monthly_snapshots WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![without_activity, period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(no_activity_snapshot_bv, 0);

        let backup_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM backups WHERE period_id = ?1 AND kind = 'period_close'",
                [period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(backup_count, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn confirm_backup_and_close_writes_no_snapshot_for_a_wholly_empty_month() {
        // T-M5.4-1/RQ-16: a calendar month elapsing with zero entries
        // anywhere in the hierarchy produces no snapshot at all — not a
        // zero snapshot — while the backup gate and close still complete
        // exactly as for any other month (T-M5.5-5).
        let (conn, dir) = seeded_with_temp_db();
        let period = insert_period(&conn, "2026-05", "awaiting_close");
        // A member exists but nothing was ever entered for them this
        // period — no insert_entry call, and (matching real usage, where
        // member_period_totals is only ever populated by recalculation off
        // an actual entry) no insert_totals call either.
        insert_member(&conn, None);

        confirm_backup_and_close(
            &conn,
            &dir.join("console.db"),
            &dir,
            ConfirmBackupAndCloseInput {
                period_id: period,
                external_medium_path: None,
            },
        )
        .unwrap();

        let status: String = conn
            .query_row("SELECT status FROM periods WHERE id = ?1", [period], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(status, "closed", "the close itself still completes");

        let snapshot_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM monthly_snapshots WHERE period_id = ?1",
                [period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(snapshot_count, 0, "no snapshot at all, not a zero snapshot");

        let backup_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM backups WHERE period_id = ?1 AND kind = 'period_close'",
                [period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            backup_count, 1,
            "the whole-database backup gate still runs regardless of emptiness"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn confirm_backup_and_close_also_writes_the_external_medium_copy_best_effort() {
        let (conn, dir) = seeded_with_temp_db();
        let period = insert_period(&conn, "2026-05", "awaiting_close");
        insert_member(&conn, None);
        let external_path = dir.join("external-copy.db");

        let outcome = confirm_backup_and_close(
            &conn,
            &dir.join("console.db"),
            &dir,
            ConfirmBackupAndCloseInput {
                period_id: period,
                external_medium_path: Some(external_path.to_string_lossy().to_string()),
            },
        )
        .unwrap();

        assert!(external_path.is_file());
        assert!(!outcome.external_medium_copy_failed);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn confirm_backup_and_close_reports_but_never_blocks_on_a_failed_external_copy() {
        let (conn, dir) = seeded_with_temp_db();
        let period = insert_period(&conn, "2026-05", "awaiting_close");
        insert_member(&conn, None);
        // A path under a directory that doesn't exist — the internal
        // retained copy still succeeds, only the external one fails.
        let external_path = dir.join("no-such-dir").join("external-copy.db");

        let outcome = confirm_backup_and_close(
            &conn,
            &dir.join("console.db"),
            &dir,
            ConfirmBackupAndCloseInput {
                period_id: period,
                external_medium_path: Some(external_path.to_string_lossy().to_string()),
            },
        )
        .unwrap();

        assert!(outcome.external_medium_copy_failed);
        let status: String = conn
            .query_row("SELECT status FROM periods WHERE id = ?1", [period], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            status, "closed",
            "a failed external copy must never block the close"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn confirm_backup_and_close_mutates_nothing_when_the_backup_write_fails() {
        // T-M5.1-10: simulate a backup-verification failure mid-close and
        // confirm zero data is mutated — no partial zeroing, no orphaned
        // snapshot row. Forced by putting a plain file where the default
        // "backups" subfolder needs to go, so `resolve_backups_dir`'s
        // `create_dir_all` fails before `write_backup_copy` ever copies
        // anything — the transaction is dropped, never committed, and
        // rusqlite's `Transaction` rolls back automatically on drop.
        let (conn, dir) = seeded_with_temp_db();
        let period = insert_period(&conn, "2026-05", "awaiting_close");
        let member = insert_member(&conn, None);
        insert_totals(&conn, member, period, 100_000, 4);
        std::fs::write(dir.join("backups"), b"not a directory").unwrap();

        let err = confirm_backup_and_close(
            &conn,
            &dir.join("console.db"),
            &dir,
            ConfirmBackupAndCloseInput {
                period_id: period,
                external_medium_path: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Io(_)));

        let status: String = conn
            .query_row("SELECT status FROM periods WHERE id = ?1", [period], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(status, "awaiting_close", "status must remain untouched");

        let (bv, tbv): (i64, i64) = conn
            .query_row(
                "SELECT business_volume, total_business_volume FROM member_period_totals WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![member, period],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            (bv, tbv),
            (100_000, 100_000),
            "live figures must not be zeroed"
        );

        let snapshot_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM monthly_snapshots WHERE period_id = ?1",
                [period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(snapshot_count, 0, "no orphaned snapshot row");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn confirm_backup_and_close_refuses_an_unknown_period() {
        let (conn, dir) = seeded_with_temp_db();
        let err = confirm_backup_and_close(
            &conn,
            &dir.join("console.db"),
            &dir,
            ConfirmBackupAndCloseInput {
                period_id: 999_999,
                external_medium_path: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn manual_backup_current_period_refuses_when_no_month_is_open() {
        let (conn, dir) = seeded_with_temp_db();
        let err = manual_backup_current_period(&conn, &dir.join("console.db"), &dir).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn manual_backup_current_period_writes_a_manual_backup_with_its_own_audit_cause() {
        let (conn, dir) = seeded_with_temp_db();
        let month = chrono::Local::now().format("%Y-%m").to_string();
        insert_period(&conn, &month, "open");

        let record = manual_backup_current_period(&conn, &dir.join("console.db"), &dir).unwrap();

        assert_eq!(record.kind, "manual");
        assert_eq!(record.period_id, None);
        let cause: String = conn
            .query_row(
                "SELECT cause FROM audit_log WHERE entity_id = ?1",
                [record.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cause, "manual_backup");
        std::fs::remove_dir_all(&dir).ok();
    }
}
