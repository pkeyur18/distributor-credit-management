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
use sha2::{Digest, Sha256};

use crate::error::AppError;

fn sha256_hex(path: &Path) -> Result<String, AppError> {
    let bytes = std::fs::read(path)?;
    let digest = Sha256::digest(&bytes);
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
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
    backups_dir: &Path,
    period_id: i64,
    kind: &str,
) -> Result<i64, AppError> {
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
        let backups_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backups_dir).unwrap();

        let version =
            write_backup_copy(&conn, &db_path, &backups_dir, period, "period_close").unwrap();

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
        let backups_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backups_dir).unwrap();

        write_backup_copy(&conn, &db_path, &backups_dir, period, "period_close").unwrap();
        let second =
            write_backup_copy(&conn, &db_path, &backups_dir, period, "period_close").unwrap();

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
        let backups_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backups_dir).unwrap();

        write_backup_copy(&conn, &db_path, &backups_dir, period, "period_close").unwrap();

        let (path, checksum): (String, String) = conn
            .query_row(
                "SELECT internal_retained_path, checksum FROM backups WHERE period_id = ?1",
                [period],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(sha256_hex(Path::new(&path)).unwrap(), checksum);
    }
}
