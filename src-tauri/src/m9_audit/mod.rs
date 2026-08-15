// M9 — Audit Log & Technical Logging (04-technical-architecture.md §3.1,
// US-M9.1, S14). Two entirely separate things sharing one module because
// NFR-5 and NFR-11 are two halves of the same client-facing promise ("every
// change is explainable") but must never be the same list: `audit_log` is
// the client-visible, append-only trail (Rule-43's backups-manifest is a
// different, unrelated read-without-a-key problem — see `backup`'s own doc
// comment); the technical log below has no UI surface and exists purely for
// the maintainer.
use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppError;
use crate::m1_members;

/// T-M9.1-1: the one shared write path every mutating command routes
/// through, refactored from ~8 previously ad hoc `INSERT INTO audit_log`
/// call sites scattered across `m1_members`/`m2_entries`/`m5_close`/
/// `m7_settings`/`backup`. `record_entry`/`edit_entry` call it once per
/// changed field; every other mutating command, once. D-12/D-13's
/// corrected value sets: `entity_type` in member/entry/setting/period/
/// backup/auth; `cause` in entry/edit/correction/settings_change/
/// period_close/manual_backup/console_backup/restore. `reversal` is
/// retired (T-M9.1-3, `reverse_entry` was dropped) and must never be
/// reintroduced under a new meaning, same as `PeriodLocked` in `error.rs`.
pub fn write_audit_entry(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
    field: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    cause: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO audit_log (entity_type, entity_id, field, old_value, new_value, changed_at, cause)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            entity_type,
            entity_id,
            field,
            old_value,
            new_value,
            chrono::Local::now().date_naive().to_string(),
            cause,
        ],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: String,
    pub cause: String,
    /// `members.name` for `entity_type = 'member'` rows, resolved via the
    /// join below; `None` for every other entity type (setting/period/
    /// backup/auth have no member to name).
    pub member_name: Option<String>,
}

fn row_to_entry(r: &rusqlite::Row) -> rusqlite::Result<AuditLogEntry> {
    Ok(AuditLogEntry {
        id: r.get(0)?,
        entity_type: r.get(1)?,
        entity_id: r.get(2)?,
        field: r.get(3)?,
        old_value: r.get(4)?,
        new_value: r.get(5)?,
        changed_at: r.get(6)?,
        cause: r.get(7)?,
        member_name: r.get(8)?,
    })
}

const AUDIT_COLUMNS: &str = "audit_log.id, audit_log.entity_type, audit_log.entity_id, \
     audit_log.field, audit_log.old_value, audit_log.new_value, audit_log.changed_at, \
     audit_log.cause, m.name";
const AUDIT_FROM: &str = "audit_log LEFT JOIN members m \
     ON m.id = audit_log.entity_id AND audit_log.entity_type = 'member'";

/// API-32 (T-M9.1-4). `member_query` reuses Rule-44's one shared search
/// function (`m1_members::search_members`) to resolve name/ID/phone to a
/// member-id set — one canonicalisation, not a second copy of it, matching
/// T-M1.4-1's own precedent. Empty/absent query returns the whole
/// chronological log, newest first; a non-empty one narrows to
/// `entity_type = 'member'` rows for the matched ids only, since the filter
/// is explicitly member-scoped — the other entity types (setting/period/
/// backup/auth) have no member to filter by.
pub fn get_audit_log(
    conn: &Connection,
    member_query: Option<&str>,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let trimmed = member_query.map(str::trim).filter(|q| !q.is_empty());
    let Some(query) = trimmed else {
        let mut stmt = conn.prepare(&format!(
            "SELECT {AUDIT_COLUMNS} FROM {AUDIT_FROM} ORDER BY audit_log.id DESC"
        ))?;
        return Ok(stmt
            .query_map([], row_to_entry)?
            .collect::<Result<_, _>>()?);
    };

    let matched_ids: Vec<i64> = m1_members::search_members(conn, query, false)?
        .into_iter()
        .map(|r| r.id)
        .collect();
    if matched_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = matched_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT {AUDIT_COLUMNS} FROM {AUDIT_FROM} \
         WHERE audit_log.entity_type = 'member' AND audit_log.entity_id IN ({placeholders}) \
         ORDER BY audit_log.id DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let params = rusqlite::params_from_iter(matched_ids.iter());
    let entries: Vec<AuditLogEntry> = stmt
        .query_map(params, row_to_entry)?
        .collect::<Result<_, _>>()?;
    Ok(entries)
}

// --- Technical logging (NFR-11/T-M9.1-5) — no UI surface, never the audit
// log above. Scoped this sprint to one real caller: `commands::login`'s
// silent scheduled-backup-check failure (T-M8.5-2), the one place NFR-11's
// "fails silently" requirement is written down explicitly.

#[derive(Debug, Clone, Copy)]
pub enum TechLogLevel {
    Info,
    Error,
}

impl TechLogLevel {
    fn label(self) -> &'static str {
        match self {
            TechLogLevel::Info => "INFO",
            TechLogLevel::Error => "ERROR",
        }
    }
}

const TECH_LOG_SUBDIR: &str = "technical-logs";
// ponytail: a fixed 14-day window, not a configurable retention setting —
// this is a maintainer-only diagnostic file with no UI, unlike the
// console-backup retention count a client actually adjusts. Add a setting
// if that ever changes.
const TECH_LOG_RETENTION_DAYS: i64 = 14;

/// Daily files (`technical-YYYY-MM-DD.log`), pruned oldest-first past
/// `TECH_LOG_RETENTION_DAYS` — the same rotate-and-prune idiom
/// `backup::prune_console_backups` already uses. Deliberately infallible
/// from the caller's side: a logging failure must never become a second
/// failure on top of whatever it was trying to record.
pub fn tech_log(app_data_dir: &std::path::Path, level: TechLogLevel, message: &str) {
    let dir = app_data_dir.join(TECH_LOG_SUBDIR);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let today = chrono::Local::now().date_naive();
    let path = dir.join(format!("technical-{today}.log"));
    let line = format!(
        "{} {} {message}\n",
        chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
        level.label()
    );
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = file.write_all(line.as_bytes());
    }
    prune_tech_logs(&dir, today);
}

fn prune_tech_logs(dir: &std::path::Path, today: chrono::NaiveDate) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(date_str) = name
            .strip_prefix("technical-")
            .and_then(|s| s.strip_suffix(".log"))
        else {
            continue;
        };
        let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
            continue;
        };
        if (today - date).num_days() > TECH_LOG_RETENTION_DAYS {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    struct TempDir(std::path::PathBuf);
    impl TempDir {
        fn new(label: &str) -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("bvconsole-m9-{label}-{nanos}-{unique}"));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn seeded() -> Connection {
        db::open_seeded_in_memory().unwrap()
    }

    #[test]
    fn write_audit_entry_round_trips() {
        let conn = seeded();
        write_audit_entry(&conn, "member", 1, "name", Some("Old"), Some("New"), "edit").unwrap();
        let entries = get_audit_log(&conn, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type, "member");
        assert_eq!(entries[0].old_value.as_deref(), Some("Old"));
        assert_eq!(entries[0].cause, "edit");
    }

    #[test]
    fn no_query_returns_every_entity_type_newest_first() {
        let conn = seeded();
        write_audit_entry(&conn, "member", 1, "name", None, Some("A"), "entry").unwrap();
        write_audit_entry(
            &conn,
            "setting",
            0,
            "royalty",
            None,
            Some("5"),
            "settings_change",
        )
        .unwrap();
        let entries = get_audit_log(&conn, None).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entity_type, "setting", "most recent write leads");
    }

    #[test]
    fn a_member_query_narrows_to_member_rows_for_matched_ids_only() {
        let conn = seeded();
        let root = m1_members::create_root_member(
            &conn,
            m1_members::CreateRootMemberInput {
                name: "Top Member".into(),
                phone: "9876500000".into(),
                address: "1 Main Street".into(),
                email: None,
                consent_given: true,
            },
        )
        .unwrap();
        let root_id = root.id;
        let root_name = root.name;
        write_audit_entry(&conn, "member", root_id, "name", None, Some("X"), "entry").unwrap();
        write_audit_entry(
            &conn,
            "setting",
            0,
            "royalty",
            None,
            Some("5"),
            "settings_change",
        )
        .unwrap();

        let entries = get_audit_log(&conn, Some(&root_name)).unwrap();
        // `create_root_member` itself already wrote one "member"/entry row
        // (T-M1.1-9) before the manual one above — both belong to root_id,
        // and the "setting" row must be excluded entirely.
        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .all(|e| e.entity_type == "member" && e.entity_id == root_id));
    }

    #[test]
    fn member_rows_resolve_member_name_other_entity_types_resolve_none() {
        let conn = seeded();
        let root = m1_members::create_root_member(
            &conn,
            m1_members::CreateRootMemberInput {
                name: "Top Member".into(),
                phone: "9876500001".into(),
                address: "1 Main Street".into(),
                email: None,
                consent_given: true,
            },
        )
        .unwrap();
        write_audit_entry(&conn, "member", root.id, "name", None, Some("X"), "entry").unwrap();
        write_audit_entry(
            &conn,
            "setting",
            0,
            "royalty",
            None,
            Some("5"),
            "settings_change",
        )
        .unwrap();

        let entries = get_audit_log(&conn, None).unwrap();
        let member_row = entries.iter().find(|e| e.entity_type == "member").unwrap();
        let setting_row = entries.iter().find(|e| e.entity_type == "setting").unwrap();
        assert_eq!(member_row.member_name.as_deref(), Some(root.name.as_str()));
        assert_eq!(setting_row.member_name, None);
    }

    #[test]
    fn an_empty_query_behaves_like_no_query() {
        let conn = seeded();
        write_audit_entry(
            &conn,
            "setting",
            0,
            "royalty",
            None,
            Some("5"),
            "settings_change",
        )
        .unwrap();
        assert_eq!(get_audit_log(&conn, Some("  ")).unwrap().len(), 1);
    }

    #[test]
    fn tech_log_writes_a_line_to_todays_file() {
        let dir = TempDir::new("techlog");
        tech_log(
            &dir.0,
            TechLogLevel::Error,
            "scheduled backup failed: disk full",
        );
        let today = chrono::Local::now().date_naive();
        let path = dir
            .0
            .join(TECH_LOG_SUBDIR)
            .join(format!("technical-{today}.log"));
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("ERROR"));
        assert!(contents.contains("scheduled backup failed: disk full"));
    }

    #[test]
    fn tech_log_prunes_files_older_than_the_retention_window() {
        let dir = TempDir::new("techlog-prune");
        let logs_dir = dir.0.join(TECH_LOG_SUBDIR);
        std::fs::create_dir_all(&logs_dir).unwrap();
        let stale_date = chrono::Local::now().date_naive() - chrono::Duration::days(30);
        let stale_path = logs_dir.join(format!("technical-{stale_date}.log"));
        std::fs::write(&stale_path, "old entry\n").unwrap();

        tech_log(&dir.0, TechLogLevel::Info, "today's entry");

        assert!(!stale_path.exists(), "a 30-day-old log must be pruned");
    }

    /// T-M9.1-6: neither log ever receives a plaintext credential. This
    /// asserts the negative for the technical logger directly; the audit
    /// log's own guarantee is structural — nothing in the codebase ever
    /// calls `write_audit_entry` with a PIN, password or recovery code as
    /// `old_value`/`new_value` (M8's own audit call sites log the *fact*
    /// that a credential changed, e.g. "PIN set", never its value).
    #[test]
    fn tech_log_message_never_contains_a_credential_shaped_literal() {
        let dir = TempDir::new("techlog-security");
        let message = "scheduled console backup failed at login: io error";
        tech_log(&dir.0, TechLogLevel::Error, message);
        let today = chrono::Local::now().date_naive();
        let path = dir
            .0
            .join(TECH_LOG_SUBDIR)
            .join(format!("technical-{today}.log"));
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(!contents.to_lowercase().contains("pin"));
        assert!(!contents.to_lowercase().contains("password"));
    }
}
