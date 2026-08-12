// M2 — Business Volume Entry (04-technical-architecture.md §3.1, §6 API-08/
// API-09; 02-business-rules.md Rule-15/16/16a/39). US-M2.1/M2.2, S7.
//
// US-M2.3/M2.4 (Feature "M2.2" in the backlog — entry eligibility by
// period, S12) are **not** built yet: nothing here refuses a current-month
// entry while an earlier one is outstanding, or refuses a closed-month
// write via this path (that stays `edit_entry`'s job, Rule-39). Until
// US-M5.5 lands (S12) no code transitions a period between states either —
// `get_or_create_open_period` below is a narrow stand-in scoped to what
// `record_entry`/`edit_entry` need (a period row to write against), not a
// preview of M5.5's catch-up state machine.
use std::path::Path;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::backup;
use crate::error::AppError;
use crate::m3_calc;

fn today_iso() -> String {
    chrono::Local::now().date_naive().to_string()
}

/// Rule-16/Rule-16a's own derivation source: `entry_date`, never "the
/// period being closed" (T-M2.1-1). Also used by `edit_entry` to re-derive
/// what an edited date's period *would* be, to enforce T-M2.2-3.
fn period_month_of_date(date: &str) -> Result<String, AppError> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.format("%Y-%m").to_string())
        .map_err(|_| AppError::Validation {
            field: "entryDate".into(),
            message: "Date must be a valid calendar date (YYYY-MM-DD).".into(),
        })
}

fn validate_amount(amount: i64) -> Result<(), AppError> {
    if amount <= 0 {
        return Err(AppError::Validation {
            field: "amount".into(),
            message: "Business Volume must be greater than zero.".into(),
        });
    }
    Ok(())
}

fn member_exists(conn: &Connection, member_id: i64) -> Result<bool, AppError> {
    Ok(conn
        .query_row("SELECT 1 FROM members WHERE id = ?1", [member_id], |_| {
            Ok(true)
        })
        .optional()?
        .unwrap_or(false))
}

/// Rule-21: one period row per calendar month — safe under
/// `periods.period_month`'s own UNIQUE constraint. Creates as `open`
/// (04-technical-architecture.md §7.1: "a period row for the current month
/// is created as open as soon as the calendar month begins") and never
/// transitions an existing row — that stays US-M5.5's job entirely.
fn get_or_create_open_period(conn: &Connection, period_month: &str) -> Result<i64, AppError> {
    if let Some(id) = conn
        .query_row(
            "SELECT id FROM periods WHERE period_month = ?1",
            [period_month],
            |r| r.get(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    conn.execute(
        "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
        [period_month],
    )?;
    Ok(conn.last_insert_rowid())
}

fn period_status(conn: &Connection, period_id: i64) -> Result<String, AppError> {
    conn.query_row(
        "SELECT status FROM periods WHERE id = ?1",
        [period_id],
        |r| r.get(0),
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "Period not found.".into(),
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessVolumeEntry {
    pub id: i64,
    pub member_id: i64,
    pub amount: i64,
    pub entry_date: String,
    pub period_month: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl BusinessVolumeEntry {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            member_id: row.get("member_id")?,
            amount: row.get("amount")?,
            entry_date: row.get("entry_date")?,
            period_month: row.get("period_month")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

fn load_entry(conn: &Connection, id: i64) -> Result<BusinessVolumeEntry, AppError> {
    conn.query_row(
        "SELECT id, member_id, amount, entry_date, period_month, created_at, updated_at
         FROM business_volume_entries WHERE id = ?1",
        [id],
        BusinessVolumeEntry::from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "Entry not found.".into(),
    })
}

fn write_audit(
    conn: &Connection,
    entry_id: i64,
    old_amount: Option<i64>,
    new_amount: i64,
    cause: &str,
) -> Result<(), AppError> {
    conn.execute(
        // D-12: `entity_type` is `member | entry | setting | period | backup | auth` —
        // 'entry' is the value for a business_volume_entries row.
        "INSERT INTO audit_log (entity_type, entity_id, field, old_value, new_value, changed_at, cause)
         VALUES ('entry', ?1, 'amount', ?2, ?3, ?4, ?5)",
        rusqlite::params![
            entry_id,
            old_amount.map(|v| v.to_string()),
            new_amount.to_string(),
            today_iso(),
            cause,
        ],
    )?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordEntryInput {
    pub member_id: i64,
    pub amount: i64,
    pub entry_date: String,
}

/// API-08: insert the entry and trigger the chain-upward recalculation in
/// one transaction (T-M2.1-1) — `m3_calc::recalculate_chain` is written to
/// compose inside exactly this kind of caller-owned transaction rather than
/// open its own.
pub fn record_entry(
    conn: &Connection,
    input: RecordEntryInput,
) -> Result<BusinessVolumeEntry, AppError> {
    validate_amount(input.amount)?;
    if !member_exists(conn, input.member_id)? {
        return Err(AppError::NotFound {
            message: "Member not found.".into(),
        });
    }
    let period_month = period_month_of_date(&input.entry_date)?;
    let created_at = today_iso();

    // `unchecked_transaction` (not the safe `&mut self` API) so this keeps
    // working with the shared `&Connection` every command handler already
    // holds via `locked_conn` — the same choice `m3_calc::recalculate_chain`
    // makes, for the same reason.
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO business_volume_entries (member_id, amount, entry_date, period_month, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![input.member_id, input.amount, input.entry_date, period_month, created_at],
    )?;
    let entry_id = tx.last_insert_rowid();
    let period_id = get_or_create_open_period(&tx, &period_month)?;
    m3_calc::recalculate_chain(&tx, input.member_id, period_id)?;
    write_audit(&tx, entry_id, None, input.amount, "entry")?;
    tx.commit()?;

    load_entry(conn, entry_id)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditEntryInput {
    pub id: i64,
    pub amount: i64,
    pub entry_date: String,
}

/// API-09 — the sole correction mechanism (Rule-39). An open/awaiting_close
/// period's edit recalculates the same way `record_entry` does (live
/// `member_period_totals`); a closed period's edit recomputes the chain "in
/// isolation" and writes a new `monthly_snapshots`/`backups` version
/// instead, per architecture §7.3 — live totals are never touched once a
/// period has closed (Rule-38).
pub fn edit_entry(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    input: EditEntryInput,
) -> Result<BusinessVolumeEntry, AppError> {
    validate_amount(input.amount)?;
    let existing = load_entry(conn, input.id)?;

    let new_period_month = period_month_of_date(&input.entry_date)?;
    if new_period_month != existing.period_month {
        return Err(AppError::Validation {
            field: "entryDate".into(),
            message: "An entry's date can only be changed within its own month (RQ-21).".into(),
        });
    }

    let period_id: i64 = conn
        .query_row(
            "SELECT id FROM periods WHERE period_month = ?1",
            [&existing.period_month],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound {
            message: "Period not found.".into(),
        })?;
    let closed = period_status(conn, period_id)? == "closed";
    let updated_at = today_iso();

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE business_volume_entries SET amount = ?1, entry_date = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![input.amount, input.entry_date, updated_at, input.id],
    )?;
    if closed {
        m3_calc::write_correction_snapshot(&tx, existing.member_id, period_id)?;
        write_audit(
            &tx,
            input.id,
            Some(existing.amount),
            input.amount,
            "correction",
        )?;
    } else {
        m3_calc::recalculate_chain(&tx, existing.member_id, period_id)?;
        write_audit(&tx, input.id, Some(existing.amount), input.amount, "edit")?;
    }
    tx.commit()?;

    if closed {
        // Post-commit by necessity — the copy must capture the file as it
        // exists *after* the correction above, and SQLite's rollback
        // journal only makes that true once the transaction has committed
        // (see backup.rs's module doc for the full reasoning).
        backup::write_backup_copy(conn, db_path, app_data_dir, period_id, "period_close")?;
    }

    load_entry(conn, input.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    // A real temp-file-backed database, not `open_seeded_in_memory` — the
    // closed-period path exercises `backup::write_backup_copy`, which
    // copies an actual file on disk.
    struct TempDb {
        conn: Connection,
        dir: std::path::PathBuf,
    }
    impl TempDb {
        fn new() -> Self {
            // Nanos alone can collide between two tests on a fast machine
            // running the suite in parallel — clock resolution isn't
            // guaranteed down to the nanosecond. A process-wide counter
            // alongside it makes every instance unique regardless.
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("bvconsole-m2-test-{nanos}-{unique}"));
            std::fs::create_dir_all(&dir).unwrap();
            let db_path = dir.join("console.db");
            let conn = db::open_encrypted(&db_path, "test-key").unwrap();
            Self { conn, dir }
        }
        fn db_path(&self) -> std::path::PathBuf {
            self.dir.join("console.db")
        }
        fn app_data_dir(&self) -> std::path::PathBuf {
            self.dir.clone()
        }
    }
    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
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

    fn member_period_total(conn: &Connection, member_id: i64, period_id: i64) -> Option<i64> {
        conn.query_row(
            "SELECT total_business_volume FROM member_period_totals
             WHERE member_id = ?1 AND period_id = ?2",
            rusqlite::params![member_id, period_id],
            |r| r.get(0),
        )
        .optional()
        .unwrap()
    }

    #[test]
    fn record_entry_creates_the_period_and_recalculates_the_chain() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);

        let entry = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 100_000,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap();

        assert_eq!(entry.period_month, "2026-08");
        let period_id: i64 = db
            .conn
            .query_row(
                "SELECT id FROM periods WHERE period_month = '2026-08'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let status: String = db
            .conn
            .query_row(
                "SELECT status FROM periods WHERE id = ?1",
                [period_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "open");
        assert_eq!(
            member_period_total(&db.conn, root, period_id),
            Some(100_000)
        );
    }

    #[test]
    fn record_entry_refuses_a_non_positive_amount() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let err = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 0,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn record_entry_refuses_an_unknown_member() {
        let db = TempDb::new();
        let err = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: 999_999,
                amount: 100,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn a_second_entry_the_same_month_reuses_the_same_period_row() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 1_000,
                entry_date: "2026-08-05".into(),
            },
        )
        .unwrap();
        record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 2_000,
                entry_date: "2026-08-20".into(),
            },
        )
        .unwrap();

        let period_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            period_count, 1,
            "Rule-21: one period row per calendar month"
        );
    }

    #[test]
    fn edit_entry_in_an_open_period_recalculates_live_totals() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let entry = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 100_000,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap();

        let updated = edit_entry(
            &db.conn,
            &db.db_path(),
            &db.app_data_dir(),
            EditEntryInput {
                id: entry.id,
                amount: 250_000,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap();
        assert_eq!(updated.amount, 250_000);
        assert!(updated.updated_at.is_some());

        let period_id: i64 = db
            .conn
            .query_row(
                "SELECT id FROM periods WHERE period_month = '2026-08'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            member_period_total(&db.conn, root, period_id),
            Some(250_000)
        );

        let cause: String = db
            .conn
            .query_row(
                "SELECT cause FROM audit_log WHERE entity_id = ?1 ORDER BY id DESC LIMIT 1",
                [entry.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cause, "edit");
    }

    #[test]
    fn edit_entry_refuses_moving_the_date_to_a_different_month() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let entry = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 1_000,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap();

        let err = edit_entry(
            &db.conn,
            &db.db_path(),
            &db.app_data_dir(),
            EditEntryInput {
                id: entry.id,
                amount: 1_000,
                entry_date: "2026-09-01".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn edit_entry_refuses_an_unknown_entry() {
        let db = TempDb::new();
        let err = edit_entry(
            &db.conn,
            &db.db_path(),
            &db.app_data_dir(),
            EditEntryInput {
                id: 999_999,
                amount: 1_000,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn edit_entry_in_a_closed_period_writes_a_new_snapshot_and_backup_leaving_version_one_untouched(
    ) {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let entry = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 100_000,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap();
        let period_id: i64 = db
            .conn
            .query_row(
                "SELECT id FROM periods WHERE period_month = '2026-08'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        // Simulates S11's close: live totals become a v1 snapshot, the
        // period is marked closed, and (per Rule-38) the live row is gone.
        db.conn
            .execute(
                "INSERT INTO monthly_snapshots
                    (member_id, period_id, version, business_volume, total_business_volume,
                     slab_pct, differential, royalty, own_reward, rewards, is_active_status, created_at)
                 SELECT member_id, period_id, 1, business_volume, total_business_volume,
                        slab_pct, differential, royalty, own_reward, rewards, 1, '2026-08-31'
                 FROM member_period_totals WHERE period_id = ?1",
                [period_id],
            )
            .unwrap();
        db.conn
            .execute(
                "DELETE FROM member_period_totals WHERE period_id = ?1",
                [period_id],
            )
            .unwrap();
        db.conn
            .execute(
                "UPDATE periods SET status = 'closed' WHERE id = ?1",
                [period_id],
            )
            .unwrap();
        let v1_before: (i64, i64) = db
            .conn
            .query_row(
                "SELECT total_business_volume, slab_pct FROM monthly_snapshots
                 WHERE member_id = ?1 AND period_id = ?2 AND version = 1",
                rusqlite::params![root, period_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        let updated = edit_entry(
            &db.conn,
            &db.db_path(),
            &db.app_data_dir(),
            EditEntryInput {
                id: entry.id,
                amount: 400_000,
                entry_date: "2026-08-15".into(),
            },
        )
        .unwrap();
        assert_eq!(updated.amount, 400_000);

        let v1_after: (i64, i64) = db
            .conn
            .query_row(
                "SELECT total_business_volume, slab_pct FROM monthly_snapshots
                 WHERE member_id = ?1 AND period_id = ?2 AND version = 1",
                rusqlite::params![root, period_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(v1_before, v1_after, "version 1 must stay byte-identical");

        let v2: i64 = db
            .conn
            .query_row(
                "SELECT total_business_volume FROM monthly_snapshots
                 WHERE member_id = ?1 AND period_id = ?2 AND version = 2",
                rusqlite::params![root, period_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(v2, 400_000);

        assert_eq!(
            member_period_total(&db.conn, root, period_id),
            None,
            "a closed period's live totals must never be written by a correction"
        );

        let backup_count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM backups WHERE period_id = ?1",
                [period_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(backup_count, 1);

        let cause: String = db
            .conn
            .query_row(
                "SELECT cause FROM audit_log WHERE entity_id = ?1 ORDER BY id DESC LIMIT 1",
                [entry.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cause, "correction");
    }
}
