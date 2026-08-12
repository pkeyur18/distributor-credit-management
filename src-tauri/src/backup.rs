// Internal-retained backup copies (ADR-012's `backups` table). First needed
// by US-M2.2's closed-month correction (S7) — Rule-39/architecture §7.3
// require every closed-month edit to write a new `backups` version alongside
// the new `monthly_snapshots` version. S11's close and S14's console backup
// reuse `write_backup_copy` rather than each deriving their own copy step.
//
// The physical copy must run against a file that already reflects the
// change being backed up, so callers commit their data-writing transaction
// *before* calling this — copying mid-transaction would capture the file's
// pre-transaction bytes (SQLite's default rollback-journal mode keeps the
// main file unchanged until COMMIT). That means the backup row is written
// as a second step, not inside the same transaction as the correction it
// records; a crash between the two leaves the correction applied without
// its backup row, surfaced to the caller as an error rather than hidden.
use std::path::Path;

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::AppError;

pub(crate) fn sha256_hex(path: &Path) -> Result<String, AppError> {
    let bytes = std::fs::read(path)?;
    let digest = Sha256::digest(&bytes);
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Row 16 / Rule-43: the backups folder is a name the operator can change
/// (`console_backup_folder`, validated at write time in
/// `m7_settings::update_console_backup_settings`), not a fixed constant —
/// re-read from `conn` at every write/restore so a change takes effect on
/// the very next backup, with no restart and no re-derivation of `AppPaths`.
fn resolve_backups_dir(
    conn: &Connection,
    app_data_dir: &Path,
) -> Result<std::path::PathBuf, AppError> {
    let folder: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'console_backup_folder'",
        [],
        |r| r.get(0),
    )?;
    let dir = app_data_dir.join(folder);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn next_backup_version(conn: &Connection, period_id: i64) -> Result<i64, AppError> {
    let max: Option<i64> = conn
        .query_row(
            "SELECT MAX(version) FROM backups WHERE period_id = ?1",
            [period_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    Ok(max.unwrap_or(0) + 1)
}

/// Copies `db_path` into `backups_dir`, checksums the copy, and records it
/// as a new `backups` row for `period_id`. Returns the row's version.
/// `is_original` is set only for a period's very first backup (S11's close,
/// version 1) — every later version, including every correction, is not.
pub fn write_backup_copy(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    period_id: i64,
    kind: &str,
) -> Result<i64, AppError> {
    let backups_dir = resolve_backups_dir(conn, app_data_dir)?;
    let version = next_backup_version(conn, period_id)?;
    let internal_retained_path = backups_dir.join(format!("period-{period_id}-v{version}.db"));
    std::fs::copy(db_path, &internal_retained_path)?;
    let checksum = sha256_hex(&internal_retained_path)?;
    let is_original = version == 1;
    let created_at = chrono::Local::now().date_naive().to_string();

    conn.execute(
        "INSERT INTO backups
            (period_id, kind, schedule_kind, version, internal_retained_path,
             external_medium_path, checksum, is_original, created_at)
         VALUES (?1, ?2, NULL, ?3, ?4, NULL, ?5, ?6, ?7)",
        rusqlite::params![
            period_id,
            kind,
            version,
            internal_retained_path.to_string_lossy(),
            checksum,
            is_original,
            created_at,
        ],
    )?;
    Ok(version)
}

fn next_console_backup_version(conn: &Connection) -> Result<i64, AppError> {
    // `period_id IS NULL` throughout — `period_id = ?1` would never match a
    // NULL column (SQL's `NULL = NULL` is unknown, not true), which is
    // exactly why `write_backup_copy`'s version query can't serve these
    // rows: every whole-console backup would silently collide at version 1.
    let max: Option<i64> = conn
        .query_row(
            "SELECT MAX(version) FROM backups WHERE period_id IS NULL",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    Ok(max.unwrap_or(0) + 1)
}

/// Rule-43/ADR-012: a whole-console backup (`kind` in `scheduled`/`manual`/
/// `pre_restore_safety`, `period_id` always NULL). Same write-then-verify
/// shape as `write_backup_copy` — copy, checksum, record — just not scoped
/// to one period. Returns the new `backups` row's id (not a version — these
/// rows aren't addressed as a period's version sequence, they're picked
/// individually off the Restore card by id).
pub fn write_console_backup_copy(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    kind: &str,
    schedule_kind: Option<&str>,
) -> Result<i64, AppError> {
    let backups_dir = resolve_backups_dir(conn, app_data_dir)?;
    let version = next_console_backup_version(conn)?;
    let internal_retained_path = backups_dir.join(format!("console-{kind}-v{version}.db"));
    std::fs::copy(db_path, &internal_retained_path)?;
    let checksum = sha256_hex(&internal_retained_path)?;
    let created_at = chrono::Local::now().date_naive().to_string();

    conn.execute(
        "INSERT INTO backups
            (period_id, kind, schedule_kind, version, internal_retained_path,
             external_medium_path, checksum, is_original, created_at)
         VALUES (NULL, ?1, ?2, ?3, ?4, NULL, ?5, 0, ?6)",
        rusqlite::params![
            kind,
            schedule_kind,
            version,
            internal_retained_path.to_string_lossy(),
            checksum,
            created_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Rule-43: after every `scheduled`/`manual` write, rows of those two kinds
/// beyond `retention_count` are deleted, oldest first. `period_close` and
/// `pre_restore_safety` rows are never pruned by this (T-M8.5-3's own
/// wording, pulled forward — see `commands.rs`'s `run_console_backup_now`).
pub fn prune_console_backups(conn: &Connection, retention_count: i64) -> Result<(), AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, internal_retained_path FROM backups
         WHERE kind IN ('scheduled', 'manual')
         ORDER BY created_at DESC, id DESC
         LIMIT -1 OFFSET ?1",
    )?;
    let stale: Vec<(i64, String)> = stmt
        .query_map([retention_count], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    drop(stmt);
    for (id, path) in stale {
        let _ = std::fs::remove_file(&path);
        conn.execute("DELETE FROM backups WHERE id = ?1", [id])?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecord {
    pub id: i64,
    pub period_id: Option<i64>,
    pub kind: String,
    pub schedule_kind: Option<String>,
    pub version: i64,
    pub checksum: String,
    pub is_original: bool,
    pub created_at: String,
}

fn row_to_backup_record(r: &rusqlite::Row) -> rusqlite::Result<BackupRecord> {
    Ok(BackupRecord {
        id: r.get(0)?,
        period_id: r.get(1)?,
        kind: r.get(2)?,
        schedule_kind: r.get(3)?,
        version: r.get(4)?,
        checksum: r.get(5)?,
        is_original: r.get(6)?,
        created_at: r.get(7)?,
    })
}

const BACKUP_RECORD_COLUMNS: &str =
    "id, period_id, kind, schedule_kind, version, checksum, is_original, created_at";

pub fn get_backup_record(conn: &Connection, id: i64) -> Result<BackupRecord, AppError> {
    conn.query_row(
        &format!("SELECT {BACKUP_RECORD_COLUMNS} FROM backups WHERE id = ?1"),
        [id],
        row_to_backup_record,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "Backup not found.".into(),
    })
}

/// API-35: every `backups` kind, newest first — the Restore card's data
/// (T-M7.4-5). S10 only reaches this from the authenticated Settings
/// screen, where `conn` is already the live, open, decrypted connection —
/// `commands.rs`'s `list_restore_points` refuses cleanly if none is open.
/// Reading a *foreign* console's backup list before login, with no key at
/// all, is the "genuine corrupted-database detection" work `commands.rs`'s
/// own `check_data_readable` comment already defers to S14 — not solved
/// here.
pub fn list_restore_points(conn: &Connection) -> Result<Vec<BackupRecord>, AppError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {BACKUP_RECORD_COLUMNS} FROM backups ORDER BY created_at DESC, id DESC"
    ))?;
    let rows = stmt
        .query_map([], row_to_backup_record)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn console_backup_retention_count(conn: &Connection) -> Result<i64, AppError> {
    let value: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'console_backup_retention_count'",
        [],
        |r| r.get(0),
    )?;
    value.parse().map_err(|_| AppError::Validation {
        field: "consoleBackupRetentionCount".into(),
        message: "Stored retention count is not a valid number.".into(),
    })
}

fn write_console_backup_audit(
    conn: &Connection,
    backup_id: i64,
    kind: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO audit_log (entity_type, entity_id, field, old_value, new_value, changed_at, cause)
         VALUES ('backup', ?1, 'kind', NULL, ?2, ?3, 'console_backup')",
        rusqlite::params![backup_id, kind, chrono::Local::now().date_naive().to_string()],
    )?;
    Ok(())
}

/// API-39. `kind = "manual"` is T-M7.4-4's "Back up now" action, built this
/// sprint. `kind = "scheduled"` (the login-triggered catch-up, T-M8.5-2) is
/// still S14 — this function is agnostic to who calls it with which kind,
/// so S14 only needs to add the login-time call site, not a second
/// implementation. Prunes to retention immediately afterward (T-M7.4-3 —
/// a lowered retention count takes effect on the *next* prune, and every
/// call here is one).
pub fn run_console_backup_now(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    kind: &str,
    schedule_kind: Option<&str>,
) -> Result<BackupRecord, AppError> {
    let id = write_console_backup_copy(conn, db_path, app_data_dir, kind, schedule_kind)?;
    write_console_backup_audit(conn, id, kind)?;
    let retention = console_backup_retention_count(conn)?;
    prune_console_backups(conn, retention)?;
    get_backup_record(conn, id)
}

fn write_restore_audit(conn: &Connection, backup_id: i64, source: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO audit_log (entity_type, entity_id, field, old_value, new_value, changed_at, cause)
         VALUES ('backup', ?1, 'console_restored', NULL, ?2, ?3, 'restore')",
        rusqlite::params![backup_id, source, chrono::Local::now().date_naive().to_string()],
    )?;
    Ok(())
}

/// Every "verify a checksum" requirement in this system (§9.1, Rule-18,
/// `write_backup_copy`) means write-then-verify integrity, never a
/// signature or provenance check — the same meaning applies here: confirm
/// `source_path`'s bytes landed at `db_path` byte-for-byte, immediately
/// after the copy and before any further write touches either file.
///
/// Also takes a `pre_restore_safety` copy of whatever was live (Rule-43 —
/// every restore path takes one, regardless of entry point). Its physical
/// file is copied from the pre-overwrite bytes, but its `backups` row is
/// deliberately recorded *afterward*, through `conn`'s now-live
/// post-restore view: a row written any earlier would live inside the very
/// file this function is about to replace and be lost the moment that
/// happens — `conn` keeps writing to the same path throughout, so its
/// state after the overwrite is whatever the restored file now holds.
fn overwrite_live_database(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    source_path: &Path,
) -> Result<(), AppError> {
    let backups_dir = resolve_backups_dir(conn, app_data_dir)?;
    let safety_version = next_console_backup_version(conn)?;
    let safety_path = backups_dir.join(format!("console-pre_restore_safety-v{safety_version}.db"));
    std::fs::copy(db_path, &safety_path)?;
    let safety_checksum = sha256_hex(&safety_path)?;
    let safety_created_at = chrono::Local::now().date_naive().to_string();

    let source_checksum = sha256_hex(source_path)?;
    std::fs::copy(source_path, db_path)?;
    let dest_checksum = sha256_hex(db_path)?;
    if source_checksum != dest_checksum {
        return Err(AppError::Conflict {
            message: "The restored file's checksum did not verify after copying.".into(),
        });
    }

    conn.execute(
        "INSERT INTO backups
            (period_id, kind, schedule_kind, version, internal_retained_path,
             external_medium_path, checksum, is_original, created_at)
         VALUES (NULL, 'pre_restore_safety', NULL, ?1, ?2, NULL, ?3, 0, ?4)",
        rusqlite::params![
            safety_version,
            safety_path.to_string_lossy(),
            safety_checksum,
            safety_created_at,
        ],
    )?;
    Ok(())
}

/// API-36: restore one of this console's own retained backups by id.
/// Rule-43's "checksum does not verify" refusal — re-hash the retained
/// file now and compare against the checksum recorded when it was
/// written, catching on-disk degradation since that write.
pub fn restore_from_backup(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    backup_id: i64,
) -> Result<(), AppError> {
    let (path, stored_checksum): (String, String) = conn
        .query_row(
            "SELECT internal_retained_path, checksum FROM backups WHERE id = ?1",
            [backup_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound {
            message: "Backup not found.".into(),
        })?;
    let current_checksum = sha256_hex(Path::new(&path))?;
    if current_checksum != stored_checksum {
        return Err(AppError::Conflict {
            message: "This backup's checksum no longer matches — the file may have been altered or corrupted.".into(),
        });
    }
    overwrite_live_database(conn, db_path, app_data_dir, Path::new(&path))?;
    // After, not before: the post-restore database is what the Restore
    // card and any future audit read actually query — an entry written
    // into the pre-restore file above would be discarded the moment it's
    // overwritten, the same reasoning `overwrite_live_database` applies to
    // its own safety-copy row.
    write_restore_audit(conn, backup_id, "retained_backup")
}

/// API-40: restore from an admin-picked file — bringing a backup over from
/// another machine, or from external media (§9.5). There is no prior
/// checksum to compare against for a file this console has never
/// recorded, so verification here is the copy-integrity check
/// `overwrite_live_database` always performs.
pub fn restore_from_backup_file(
    conn: &Connection,
    db_path: &Path,
    app_data_dir: &Path,
    source_path: &Path,
) -> Result<(), AppError> {
    if !source_path.is_file() {
        return Err(AppError::NotFound {
            message: "The chosen file could not be found.".into(),
        });
    }
    overwrite_live_database(conn, db_path, app_data_dir, source_path)?;
    write_restore_audit(conn, 0, "external_file")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded_with_temp_db() -> (Connection, tempfile_dir::TempDir) {
        let dir = tempfile_dir::TempDir::new();
        let db_path = dir.path().join("console.db");
        let conn = db::open_encrypted(&db_path, "test-key").unwrap();
        (conn, dir)
    }

    // No `tempfile` crate in the workspace yet — a tiny local stand-in
    // rather than adding a dependency for one test module.
    mod tempfile_dir {
        use std::path::{Path, PathBuf};

        pub struct TempDir(PathBuf);
        impl TempDir {
            pub fn new() -> Self {
                // Nanos alone can collide between parallel test threads on
                // fast hardware; a process-wide counter makes it unique.
                static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let dir =
                    std::env::temp_dir().join(format!("bvconsole-backup-test-{nanos}-{unique}"));
                std::fs::create_dir_all(&dir).unwrap();
                Self(dir)
            }
            pub fn path(&self) -> &Path {
                &self.0
            }
        }
        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }

    fn insert_period(conn: &Connection, month: &str) -> i64 {
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, 'closed')",
            [month],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn first_backup_for_a_period_is_version_one_and_marked_original() {
        let (conn, dir) = seeded_with_temp_db();
        let period = insert_period(&conn, "2026-08");
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        let version =
            write_backup_copy(&conn, &db_path, app_data_dir, period, "period_close").unwrap();

        assert_eq!(version, 1);
        let (is_original, checksum): (bool, String) = conn
            .query_row(
                "SELECT is_original, checksum FROM backups WHERE period_id = ?1 AND version = 1",
                [period],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(is_original);
        assert!(!checksum.is_empty());
    }

    #[test]
    fn a_second_backup_for_the_same_period_increments_the_version_and_is_not_original() {
        let (conn, dir) = seeded_with_temp_db();
        let period = insert_period(&conn, "2026-08");
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        write_backup_copy(&conn, &db_path, app_data_dir, period, "period_close").unwrap();
        let second =
            write_backup_copy(&conn, &db_path, app_data_dir, period, "period_close").unwrap();

        assert_eq!(second, 2);
        let is_original: bool = conn
            .query_row(
                "SELECT is_original FROM backups WHERE period_id = ?1 AND version = 2",
                [period],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            !is_original,
            "only version 1 of a period's backups is original"
        );
    }

    #[test]
    fn the_copied_file_on_disk_matches_the_recorded_checksum() {
        let (conn, dir) = seeded_with_temp_db();
        let period = insert_period(&conn, "2026-08");
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        write_backup_copy(&conn, &db_path, app_data_dir, period, "period_close").unwrap();

        let (path, checksum): (String, String) = conn
            .query_row(
                "SELECT internal_retained_path, checksum FROM backups WHERE period_id = ?1",
                [period],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(sha256_hex(Path::new(&path)).unwrap(), checksum);
    }

    // --- write_console_backup_copy / prune_console_backups (US-M7.4, S10) ---

    #[test]
    fn console_backups_version_independently_of_any_periods_backups() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        let period = insert_period(&conn, "2026-08");
        write_backup_copy(&conn, &db_path, app_data_dir, period, "period_close").unwrap();

        let id = write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();

        let (kind, period_id, version): (String, Option<i64>, i64) = conn
            .query_row(
                "SELECT kind, period_id, version FROM backups WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(kind, "manual");
        assert_eq!(period_id, None);
        assert_eq!(
            version, 1,
            "the first console backup, independent of the period_close row above"
        );
    }

    #[test]
    fn a_second_console_backup_increments_its_own_version() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();
        let second_id =
            write_console_backup_copy(&conn, &db_path, app_data_dir, "scheduled", Some("daily"))
                .unwrap();

        let version: i64 = conn
            .query_row(
                "SELECT version FROM backups WHERE id = ?1",
                [second_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn pruning_keeps_only_the_retained_count_of_scheduled_and_manual_rows() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        for _ in 0..5 {
            write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();
        }

        prune_console_backups(&conn, 2).unwrap();

        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM backups WHERE kind = 'manual'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            remaining, 2,
            "only the retention count of manual rows must survive"
        );
    }

    #[test]
    fn pruning_never_touches_period_close_or_pre_restore_safety_rows() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        let period = insert_period(&conn, "2026-08");
        write_backup_copy(&conn, &db_path, app_data_dir, period, "period_close").unwrap();
        write_console_backup_copy(&conn, &db_path, app_data_dir, "pre_restore_safety", None)
            .unwrap();
        for _ in 0..3 {
            write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();
        }

        prune_console_backups(&conn, 0).unwrap();

        let protected: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM backups WHERE kind IN ('period_close', 'pre_restore_safety')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            protected, 2,
            "period_close and pre_restore_safety rows are never pruned"
        );
        let manual_remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM backups WHERE kind = 'manual'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            manual_remaining, 0,
            "retention 0 must prune every manual row"
        );
    }

    // --- run_console_backup_now ---

    #[test]
    fn run_console_backup_now_writes_a_row_and_an_audit_entry() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        let record = run_console_backup_now(&conn, &db_path, app_data_dir, "manual", None).unwrap();

        assert_eq!(record.kind, "manual");
        let audit_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE cause = 'console_backup'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(audit_count, 1);
    }

    #[test]
    fn run_console_backup_now_prunes_to_the_configured_retention_immediately() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        conn.execute(
            "UPDATE settings SET value = '2' WHERE key = 'console_backup_retention_count'",
            [],
        )
        .unwrap();

        for _ in 0..4 {
            run_console_backup_now(&conn, &db_path, app_data_dir, "manual", None).unwrap();
        }

        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM backups WHERE kind = 'manual'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            remaining, 2,
            "T-M7.4-3: retention takes effect on the very next prune"
        );
    }

    // --- list_restore_points / restore_from_backup / restore_from_backup_file ---

    #[test]
    fn list_restore_points_returns_every_kind_newest_first() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        let period = insert_period(&conn, "2026-08");
        write_backup_copy(&conn, &db_path, app_data_dir, period, "period_close").unwrap();
        write_console_backup_copy(&conn, &db_path, app_data_dir, "scheduled", Some("daily"))
            .unwrap();
        let last_id =
            write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();

        let points = list_restore_points(&conn).unwrap();

        assert_eq!(points.len(), 3);
        assert_eq!(
            points[0].id, last_id,
            "the most recently written backup must lead"
        );
    }

    #[test]
    fn restore_from_backup_refuses_an_unknown_id() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        let err = restore_from_backup(&conn, &db_path, app_data_dir, 999_999).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn restore_from_backup_refuses_a_degraded_file() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        let id = write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();
        let path: String = conn
            .query_row(
                "SELECT internal_retained_path FROM backups WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        std::fs::write(&path, b"corrupted after the fact").unwrap();

        let err = restore_from_backup(&conn, &db_path, app_data_dir, id).unwrap_err();
        assert!(matches!(err, AppError::Conflict { .. }));
    }

    #[test]
    fn restore_from_backup_overwrites_the_live_file_and_takes_a_safety_copy_first() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        // The manual backup's own file is captured *before* its row exists
        // (`write_console_backup_copy`'s own write-after-copy order) — so
        // restoring from it lands a database with zero `backups` rows,
        // which is what makes the post-restore row count below a clean
        // signal that the overwrite actually happened.
        let id = write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();

        restore_from_backup(&conn, &db_path, app_data_dir, id).unwrap();

        let kinds: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT kind FROM backups ORDER BY id")
                .unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };
        assert_eq!(
            kinds,
            vec!["pre_restore_safety".to_string()],
            "the restored file must have none of the pre-restore backup rows, plus exactly \
             one safety-copy row recorded afterward (Rule-43)"
        );

        let restore_audit_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE cause = 'restore'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(restore_audit_count, 1);
    }

    #[test]
    fn restore_from_backup_file_refuses_a_missing_file() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        let err = restore_from_backup_file(
            &conn,
            &db_path,
            app_data_dir,
            &dir.path().join("does-not-exist.db"),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn restore_from_backup_file_copies_an_external_file_over_the_live_database() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        // A structurally real encrypted database (same key/schema) — `conn`
        // must keep writing successfully into `db_path` after the overwrite
        // (the safety-copy row), which an arbitrary byte string couldn't
        // support.
        let source_path = dir.path().join("brought-from-another-machine.db");
        std::fs::copy(&db_path, &source_path).unwrap();

        restore_from_backup_file(&conn, &db_path, app_data_dir, &source_path).unwrap();

        let kinds: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT kind FROM backups ORDER BY id")
                .unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };
        assert_eq!(
            kinds,
            vec!["pre_restore_safety".to_string()],
            "restoring a foreign file must still take a safety copy of what was live (Rule-43)"
        );
    }

    // --- dynamic `console_backup_folder` resolution ---

    #[test]
    fn changing_the_backup_folder_setting_moves_where_the_next_backup_lands() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        let first_id =
            write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();
        conn.execute(
            "UPDATE settings SET value = 'custom-folder' WHERE key = 'console_backup_folder'",
            [],
        )
        .unwrap();
        let second_id =
            write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();

        let first_path: String = conn
            .query_row(
                "SELECT internal_retained_path FROM backups WHERE id = ?1",
                [first_id],
                |r| r.get(0),
            )
            .unwrap();
        let second_path: String = conn
            .query_row(
                "SELECT internal_retained_path FROM backups WHERE id = ?1",
                [second_id],
                |r| r.get(0),
            )
            .unwrap();

        assert!(first_path.contains("/backups/") || first_path.contains("\\backups\\"));
        assert!(second_path.contains("custom-folder"));
        assert!(dir.path().join("custom-folder").is_dir());
    }

    #[test]
    fn list_restore_points_still_resolves_backups_written_under_a_since_changed_folder() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();
        conn.execute(
            "UPDATE settings SET value = 'custom-folder' WHERE key = 'console_backup_folder'",
            [],
        )
        .unwrap();
        write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();

        let points = list_restore_points(&conn).unwrap();
        assert_eq!(points.len(), 2);
        for point in &points {
            let path: String = conn
                .query_row(
                    "SELECT internal_retained_path FROM backups WHERE id = ?1",
                    [point.id],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(
                Path::new(&path).is_file(),
                "every listed backup's file must still resolve on disk regardless of the \
                 folder setting at the time it was written"
            );
        }
    }
}
