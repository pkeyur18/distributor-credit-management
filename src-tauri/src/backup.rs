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

/// Rule-43/S14: `backups` rows live inside the SQLCipher file, so the
/// pre-auth commands (`check_data_readable`, `list_restore_points`,
/// `restore_from_backup`, `restore_from_backup_file` — Rule-29's closed set
/// of seven) have no key to read them with. This is an unencrypted mirror,
/// a JSON array at `AppPaths::backups_manifest_path`, holding exactly the
/// fields `08-security-authorization-matrix.md`/§8.6 already says these
/// commands may reveal pre-auth — "only that backups exist and roughly when
/// they were taken", never member data or figures. Every SQL write/prune
/// site in this module mirrors into it at the same call site, so the two
/// never drift independently. `list_restore_points` reads only this file
/// now, for both authenticated and unauthenticated callers — one function,
/// one behaviour, matching Rule-44's "one shared function" precedent rather
/// than two lists that could silently disagree.
pub mod manifest {
    use std::path::Path;

    use serde::{Deserialize, Serialize};

    use crate::error::AppError;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BackupManifestEntry {
        pub id: i64,
        pub period_id: Option<i64>,
        pub kind: String,
        pub schedule_kind: Option<String>,
        pub version: i64,
        pub internal_retained_path: String,
        pub checksum: String,
        pub is_original: bool,
        pub created_at: String,
    }

    fn read_all(manifest_path: &Path) -> Result<Vec<BackupManifestEntry>, AppError> {
        if !manifest_path.exists() {
            return Ok(Vec::new());
        }
        let bytes = std::fs::read(manifest_path)?;
        if bytes.is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }

    fn write_all(manifest_path: &Path, entries: &[BackupManifestEntry]) -> Result<(), AppError> {
        let bytes = serde_json::to_vec_pretty(entries)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        std::fs::write(manifest_path, bytes)?;
        Ok(())
    }

    /// Every kind, newest first — API-35's actual data source now.
    pub fn list(manifest_path: &Path) -> Result<Vec<BackupManifestEntry>, AppError> {
        let mut entries = read_all(manifest_path)?;
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
        Ok(entries)
    }

    pub fn find(manifest_path: &Path, id: i64) -> Result<Option<BackupManifestEntry>, AppError> {
        Ok(read_all(manifest_path)?.into_iter().find(|e| e.id == id))
    }

    /// `pre_restore_safety` entries written by an unauthenticated restore
    /// have no `backups` SQL row to borrow an id from (there may be no
    /// open, keyed connection at all) — and a safety copy of the
    /// about-to-be-replaced database is, semantically, never really "owned"
    /// by any one database's `backups` table anyway, old or new. Negative
    /// ids are allocated for these entries instead: SQLite's `INTEGER
    /// PRIMARY KEY` rowids are always positive, so the two id spaces can
    /// never collide.
    pub fn next_negative_id(manifest_path: &Path) -> Result<i64, AppError> {
        let min = read_all(manifest_path)?
            .into_iter()
            .map(|e| e.id)
            .filter(|id| *id < 0)
            .min();
        Ok(min.unwrap_or(0) - 1)
    }

    /// Upsert by id, not a blind push. `write_backup_copy`/
    /// `write_console_backup_copy` call this from inside a caller-owned SQL
    /// transaction that can still roll back afterward (e.g. a period-close
    /// that fails a later step) — `backups.id` has no `AUTOINCREMENT`, so a
    /// rolled-back insert's rowid can be reused by a later, successful one.
    /// Without the upsert, that reuse would leave two entries sharing one
    /// id (`find` and `list` would then disagree about which is current).
    /// The physical file from the failed attempt is left in place —
    /// harmless, since it's a real, checksummed, restorable backup even
    /// though its close never completed, not a corrupted one.
    pub fn append(manifest_path: &Path, entry: BackupManifestEntry) -> Result<(), AppError> {
        let mut entries = read_all(manifest_path)?;
        entries.retain(|e| e.id != entry.id);
        entries.push(entry);
        write_all(manifest_path, &entries)
    }

    /// Mirrors `prune_console_backups`' deletion set exactly — called with
    /// the same ids, right after the SQL deletion, from the same caller.
    pub fn remove(manifest_path: &Path, remove_ids: &[i64]) -> Result<(), AppError> {
        let mut entries = read_all(manifest_path)?;
        entries.retain(|e| !remove_ids.contains(&e.id));
        write_all(manifest_path, &entries)
    }
}

pub(crate) fn sha256_hex(path: &Path) -> Result<String, AppError> {
    let bytes = std::fs::read(path)?;
    let digest = Sha256::digest(&bytes);
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// The seeded default for `console_backup_folder` (`db/seed.rs`) — the
/// fallback `resolve_backups_dir` uses when there's no connection to read
/// the real setting from.
const DEFAULT_BACKUPS_FOLDER: &str = "backups";

/// Row 16 / Rule-43: the backups folder is a name the operator can change
/// (`console_backup_folder`, validated at write time in
/// `m7_settings::update_console_backup_settings`), not a fixed constant —
/// re-read from `conn` at every write/restore so a change takes effect on
/// the very next backup, with no restart and no re-derivation of `AppPaths`.
///
/// `conn: None` is the pre-auth restore path (`overwrite_live_database`
/// called with no open session) — that setting lives inside the very file
/// we may not have a key for, so it falls back to the seeded default
/// instead. Accepted limit: an admin who both customized this folder *and*
/// lost their database gets their `pre_restore_safety` copy written to the
/// default location instead of their custom one. Nothing else depends on
/// this fallback — every backup actually being restored *from* is located
/// via its manifest-recorded absolute path, never re-derived here.
fn resolve_backups_dir(
    conn: Option<&Connection>,
    app_data_dir: &Path,
) -> Result<std::path::PathBuf, AppError> {
    let folder = match conn {
        Some(conn) => conn.query_row(
            "SELECT value FROM settings WHERE key = 'console_backup_folder'",
            [],
            |r| r.get(0),
        )?,
        None => DEFAULT_BACKUPS_FOLDER.to_string(),
    };
    let dir = app_data_dir.join(folder);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// The manifest's fixed sidecar location, mirroring `AppPaths::resolve`'s
/// own `app_data_dir.join(BACKUPS_MANIFEST_FILE_NAME)` — a shared constant
/// rather than a threaded parameter, since every existing caller of the
/// functions below already has `app_data_dir` and none of them otherwise
/// need the rest of `AppPaths`.
fn manifest_path_for(app_data_dir: &Path) -> std::path::PathBuf {
    app_data_dir.join(crate::paths::BACKUPS_MANIFEST_FILE_NAME)
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
    let backups_dir = resolve_backups_dir(Some(conn), app_data_dir)?;
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
    manifest::append(
        &manifest_path_for(app_data_dir),
        manifest::BackupManifestEntry {
            id: conn.last_insert_rowid(),
            period_id: Some(period_id),
            kind: kind.to_string(),
            schedule_kind: None,
            version,
            internal_retained_path: internal_retained_path.to_string_lossy().into_owned(),
            checksum,
            is_original,
            created_at,
        },
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
    let backups_dir = resolve_backups_dir(Some(conn), app_data_dir)?;
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
    let id = conn.last_insert_rowid();
    manifest::append(
        &manifest_path_for(app_data_dir),
        manifest::BackupManifestEntry {
            id,
            period_id: None,
            kind: kind.to_string(),
            schedule_kind: schedule_kind.map(str::to_string),
            version,
            internal_retained_path: internal_retained_path.to_string_lossy().into_owned(),
            checksum,
            is_original: false,
            created_at,
        },
    )?;
    Ok(id)
}

/// Rule-43: after every `scheduled`/`manual` write, rows of those two kinds
/// beyond `retention_count` are deleted, oldest first. `period_close` and
/// `pre_restore_safety` rows are never pruned by this (T-M8.5-3's own
/// wording, pulled forward — see `commands.rs`'s `run_console_backup_now`).
pub fn prune_console_backups(
    conn: &Connection,
    app_data_dir: &Path,
    retention_count: i64,
) -> Result<(), AppError> {
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
    let stale_ids: Vec<i64> = stale.iter().map(|(id, _)| *id).collect();
    for (id, path) in stale {
        let _ = std::fs::remove_file(&path);
        conn.execute("DELETE FROM backups WHERE id = ?1", [id])?;
    }
    manifest::remove(&manifest_path_for(app_data_dir), &stale_ids)?;
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

impl From<manifest::BackupManifestEntry> for BackupRecord {
    fn from(e: manifest::BackupManifestEntry) -> Self {
        BackupRecord {
            id: e.id,
            period_id: e.period_id,
            kind: e.kind,
            schedule_kind: e.schedule_kind,
            version: e.version,
            checksum: e.checksum,
            is_original: e.is_original,
            created_at: e.created_at,
        }
    }
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
/// (T-M7.4-5) *and* the data-recovery screen's (T-M8.6-5). Reads the
/// manifest exclusively, not `backups` — the same call whether or not a
/// session is open, since a genuinely pre-login caller (no key, possibly no
/// database at all) has no other way to answer "what backups exist" (see
/// this module's own top-of-file doc comment).
pub fn list_restore_points(app_data_dir: &Path) -> Result<Vec<BackupRecord>, AppError> {
    Ok(manifest::list(&manifest_path_for(app_data_dir))?
        .into_iter()
        .map(BackupRecord::from)
        .collect())
}

/// T-M8.5-2: is a scheduled backup due right now? `Ok(None)` when the
/// schedule is `off` or nothing is due yet. No dedicated "last scheduled
/// run" setting exists — reusing the most recent `scheduled`-kind row's own
/// `created_at` is simpler and can't drift out of sync with reality the way
/// a separately-tracked timestamp could.
pub fn scheduled_backup_due(conn: &Connection) -> Result<Option<String>, AppError> {
    let schedule: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'console_backup_schedule'",
        [],
        |r| r.get(0),
    )?;
    if schedule == "off" {
        return Ok(None);
    }
    let last_run: Option<String> = conn
        .query_row(
            "SELECT MAX(created_at) FROM backups WHERE kind = 'scheduled'",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    let due = match last_run {
        None => true,
        Some(last_run) => {
            let last_run =
                chrono::NaiveDate::parse_from_str(&last_run, "%Y-%m-%d").map_err(|e| {
                    AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })?;
            let elapsed_days = (chrono::Local::now().date_naive() - last_run).num_days();
            let interval_days = match schedule.as_str() {
                "daily" => 1,
                "weekly" => 7,
                "monthly" => 28,
                _ => return Ok(None), // an unrecognized value is treated as off, not a crash
            };
            elapsed_days >= interval_days
        }
    };
    Ok(due.then_some(schedule))
}

/// `pub(crate)`: `m5_close::manual_backup_current_period` (API-15) prunes
/// against the same retention setting rather than re-deriving it.
pub(crate) fn console_backup_retention_count(conn: &Connection) -> Result<i64, AppError> {
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
    crate::m9_audit::write_audit_entry(
        conn,
        "backup",
        backup_id,
        "kind",
        None,
        Some(kind),
        "console_backup",
    )
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
    prune_console_backups(conn, app_data_dir, retention)?;
    get_backup_record(conn, id)
}

/// Every "verify a checksum" requirement in this system (§9.1, Rule-18,
/// `write_backup_copy`) means write-then-verify integrity, never a
/// signature or provenance check — the same meaning applies here: confirm
/// `source_path`'s bytes landed at `db_path` byte-for-byte, immediately
/// after the copy and before any further write touches either file.
///
/// Also takes a `pre_restore_safety` copy of whatever was live (Rule-43 —
/// every restore path takes one, regardless of entry point). **No `conn`
/// use of any kind happens after the physical overwrite below** — that was
/// a real bug in the S10-era version of this function: it wrote the safety
/// row through the pre-restore `Connection` *after* replacing the file on
/// disk, whose SQLCipher key context still belonged to the file that used
/// to be there. Harmless only by accident (same-key test fixtures, and the
/// only reachable caller always tore the session down right after), and
/// exactly the class of bug AC-38 (a different credential on the restored
/// file) would have exposed for real. The safety entry is written to the
/// manifest — never `backups` SQL, since a safety copy of the
/// about-to-be-replaced database doesn't semantically belong to any single
/// database's table, old or new — and it's written *before* the overwrite,
/// while `db_path` is still the file it was resolved against. `conn` is
/// `None` for every unauthenticated caller (Rule-29's closed set of seven)
/// and `Some` only for the authenticated Settings restore card, where it's
/// used solely to resolve the real `console_backup_folder` setting — a
/// read, before anything is mutated.
fn overwrite_live_database(
    conn: Option<&Connection>,
    db_path: &Path,
    app_data_dir: &Path,
    source_path: &Path,
) -> Result<(), AppError> {
    let backups_dir = resolve_backups_dir(conn, app_data_dir)?;
    let manifest_path = manifest_path_for(app_data_dir);
    let safety_id = manifest::next_negative_id(&manifest_path)?;
    let safety_path = backups_dir.join(format!("console-pre_restore_safety-{}.db", -safety_id));
    std::fs::copy(db_path, &safety_path)?;
    let safety_checksum = sha256_hex(&safety_path)?;
    let safety_created_at = chrono::Local::now().date_naive().to_string();
    manifest::append(
        &manifest_path,
        manifest::BackupManifestEntry {
            id: safety_id,
            period_id: None,
            kind: "pre_restore_safety".to_string(),
            schedule_kind: None,
            // Not addressed as a version sequence (single global entries,
            // same as every other console-wide row) — 1 throughout.
            version: 1,
            internal_retained_path: safety_path.to_string_lossy().into_owned(),
            checksum: safety_checksum,
            is_original: false,
            created_at: safety_created_at,
        },
    )?;

    let source_checksum = sha256_hex(source_path)?;
    std::fs::copy(source_path, db_path)?;
    let dest_checksum = sha256_hex(db_path)?;
    if source_checksum != dest_checksum {
        return Err(AppError::Conflict {
            message: "The restored file's checksum did not verify after copying.".into(),
        });
    }
    Ok(())
}

/// API-36: restore one of this console's own retained backups by id.
/// Rule-43's "checksum does not verify" refusal — re-hash the retained
/// file now and compare against the checksum recorded when it was
/// written, catching on-disk degradation since that write. Looked up via
/// the manifest, not SQL — this must work with `conn: None` (Rule-29's
/// closed set of seven; see the module doc comment).
///
/// No `audit_log` entry for the restore itself: the S10-era code wrote one
/// with `cause = 'restore'`, but that value was never actually part of the
/// documented seven-value taxonomy (T-M9.1-3, `02-business-rules.md`/[04]
/// §5.3) — the same class of drift 'reversal' already needed retiring for.
/// It also can't be made to work safely in general: writing it *before* the
/// overwrite (as this function used to, briefly) gets silently discarded
/// whenever the restored content predates that write, which is every
/// restore-to-an-older-backup by definition; writing it *after* requires a
/// connection to whatever database is now live, which may need a
/// completely different credential this process never had. The manifest's
/// own `pre_restore_safety` entry, with its `created_at`, already *is* the
/// durable, always-writable record that a restore happened.
pub fn restore_from_backup(
    conn: Option<&Connection>,
    db_path: &Path,
    app_data_dir: &Path,
    backup_id: i64,
) -> Result<(), AppError> {
    let manifest_path = manifest_path_for(app_data_dir);
    let entry = manifest::find(&manifest_path, backup_id)?.ok_or_else(|| AppError::NotFound {
        message: "Backup not found.".into(),
    })?;
    let current_checksum = sha256_hex(Path::new(&entry.internal_retained_path))?;
    if current_checksum != entry.checksum {
        return Err(AppError::Conflict {
            message: "This backup's checksum no longer matches — the file may have been altered or corrupted.".into(),
        });
    }
    overwrite_live_database(
        conn,
        db_path,
        app_data_dir,
        Path::new(&entry.internal_retained_path),
    )
}

/// API-40: restore from an admin-picked file — bringing a backup over from
/// another machine, or from external media (§9.5). There is no prior
/// checksum to compare against for a file this console has never
/// recorded, so verification here is the copy-integrity check
/// `overwrite_live_database` always performs. `conn: None` for every
/// unauthenticated caller, same as `restore_from_backup` — see that
/// function's doc comment for why neither writes an `audit_log` entry.
pub fn restore_from_backup_file(
    conn: Option<&Connection>,
    db_path: &Path,
    app_data_dir: &Path,
    source_path: &Path,
) -> Result<(), AppError> {
    if !source_path.is_file() {
        return Err(AppError::NotFound {
            message: "The chosen file could not be found.".into(),
        });
    }
    overwrite_live_database(conn, db_path, app_data_dir, source_path)
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

        prune_console_backups(&conn, app_data_dir, 2).unwrap();

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

        prune_console_backups(&conn, app_data_dir, 0).unwrap();

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

        let points = list_restore_points(app_data_dir).unwrap();

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

        let err = restore_from_backup(Some(&conn), &db_path, app_data_dir, 999_999).unwrap_err();
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

        let err = restore_from_backup(Some(&conn), &db_path, app_data_dir, id).unwrap_err();
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

        restore_from_backup(Some(&conn), &db_path, app_data_dir, id).unwrap();

        let sql_kind_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM backups", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            sql_kind_count, 0,
            "the restored file must have none of the pre-restore backup rows"
        );
        let manifest_entries = manifest::list(&manifest_path_for(app_data_dir)).unwrap();
        assert_eq!(
            manifest_entries
                .iter()
                .filter(|e| e.kind == "pre_restore_safety")
                .count(),
            1,
            "the safety copy — the durable record that a restore happened — lives in the \
             manifest, never `backups` SQL (Rule-43)"
        );
    }

    #[test]
    fn restore_from_backup_works_with_no_connection_at_all() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        let id = write_console_backup_copy(&conn, &db_path, app_data_dir, "manual", None).unwrap();
        drop(conn); // the genuine pre-auth shape: no key, no open connection

        let result = restore_from_backup(None, &db_path, app_data_dir, id);
        assert!(result.is_ok(), "{result:?}");
        let manifest_entries = manifest::list(&manifest_path_for(app_data_dir)).unwrap();
        assert!(manifest_entries
            .iter()
            .any(|e| e.kind == "pre_restore_safety"));
    }

    #[test]
    fn restore_from_backup_file_refuses_a_missing_file() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();

        let err = restore_from_backup_file(
            Some(&conn),
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

        restore_from_backup_file(Some(&conn), &db_path, app_data_dir, &source_path).unwrap();

        let sql_kind_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM backups", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sql_kind_count, 0);
        let manifest_entries = manifest::list(&manifest_path_for(app_data_dir)).unwrap();
        assert_eq!(
            manifest_entries
                .iter()
                .filter(|e| e.kind == "pre_restore_safety")
                .count(),
            1,
            "restoring a foreign file must still take a safety copy of what was live (Rule-43)"
        );
    }

    /// AC-38's actual shape at this module's level: restoring a file
    /// encrypted with a genuinely different key must not corrupt anything —
    /// the regression this module's own S10-era version would have hit,
    /// since every other test here restores between same-key fixtures.
    #[test]
    fn restore_from_backup_file_succeeds_across_a_different_key() {
        let (conn, dir) = seeded_with_temp_db();
        let db_path = dir.path().join("console.db");
        let app_data_dir = dir.path();
        let source_path = dir.path().join("other-machine.db");
        db::open_encrypted(&source_path, "a totally different key").unwrap();

        restore_from_backup_file(Some(&conn), &db_path, app_data_dir, &source_path).unwrap();

        assert!(db::open_encrypted(&db_path, "a totally different key").is_ok());
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

        let points = list_restore_points(app_data_dir).unwrap();
        assert_eq!(points.len(), 2);
        for point in &points {
            let entry = manifest::find(&manifest_path_for(app_data_dir), point.id)
                .unwrap()
                .unwrap();
            let path = entry.internal_retained_path;
            assert!(
                Path::new(&path).is_file(),
                "every listed backup's file must still resolve on disk regardless of the \
                 folder setting at the time it was written"
            );
        }
    }

    /// Final-check catch: `backups.id` has no `AUTOINCREMENT`, so a rolled-
    /// back SQL transaction's rowid can be reused by a later, successful
    /// insert — `write_backup_copy`/`write_console_backup_copy` call
    /// `manifest::append` from inside exactly that kind of caller-owned
    /// transaction (`confirm_backup_and_close`'s single close transaction).
    /// A blind push would leave two entries sharing one id; the upsert
    /// must replace the stale one instead.
    #[test]
    fn manifest_append_upserts_by_id_rather_than_duplicating() {
        let dir = tempfile_dir::TempDir::new();
        let manifest_path = dir.path().join("backups-manifest.json");
        let stale = manifest::BackupManifestEntry {
            id: 7,
            period_id: None,
            kind: "manual".to_string(),
            schedule_kind: None,
            version: 1,
            internal_retained_path: "stale-path.db".to_string(),
            checksum: "stale-checksum".to_string(),
            is_original: false,
            created_at: "2026-01-01".to_string(),
        };
        manifest::append(&manifest_path, stale).unwrap();

        let fresh = manifest::BackupManifestEntry {
            id: 7,
            period_id: None,
            kind: "manual".to_string(),
            schedule_kind: None,
            version: 2,
            internal_retained_path: "fresh-path.db".to_string(),
            checksum: "fresh-checksum".to_string(),
            is_original: false,
            created_at: "2026-02-01".to_string(),
        };
        manifest::append(&manifest_path, fresh).unwrap();

        let entries = manifest::list(&manifest_path).unwrap();
        assert_eq!(
            entries.len(),
            1,
            "the stale entry must be replaced, not duplicated"
        );
        assert_eq!(entries[0].checksum, "fresh-checksum");
    }
}
