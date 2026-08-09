// Built ahead of its caller: `open_encrypted` takes an already-derived key
// and knows nothing about where it comes from (§3.2's module boundary). The
// real caller — Argon2id key derivation from the login PIN/password — is
// M8, Sprint 5. Exercised by this module's own tests until then.
#![allow(dead_code)]

mod migrations;
mod seed;

use rusqlite::Connection;
use std::path::Path;

use crate::error::AppError;

pub fn open_encrypted(path: &Path, key: &str) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    // Touch the database so SQLCipher validates the key / initializes the
    // file header now, rather than deferring the failure to the first query
    // a caller happens to run.
    conn.execute_batch("SELECT count(*) FROM sqlite_master")?;
    migrations::run(&mut conn)?;
    seed::run(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDbPath(std::path::PathBuf);

    impl TempDbPath {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("bvconsole-test-{label}-{nanos}.db"));
            Self(path)
        }
    }

    impl Drop for TempDbPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn fresh_launch_creates_file_with_all_tables_and_both_seed_sets() {
        let path = TempDbPath::new("fresh");
        let conn = open_encrypted(&path.0, "correct horse battery staple").unwrap();

        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name != 'schema_migrations'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 10);

        let slab_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))
            .unwrap();
        let settings_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(slab_count, 7);
        assert_eq!(settings_count, 16);
    }

    #[test]
    fn a_plain_unkeyed_connection_cannot_read_the_file() {
        let path = TempDbPath::new("unkeyed");
        open_encrypted(&path.0, "correct horse battery staple").unwrap();

        let plain = Connection::open(&path.0).unwrap();
        let result = plain.query_row("SELECT COUNT(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        });
        assert!(
            result.is_err(),
            "an unkeyed connection must not be able to read an encrypted file"
        );
    }

    #[test]
    fn reopening_with_the_correct_key_reads_the_same_data() {
        let path = TempDbPath::new("reopen");
        open_encrypted(&path.0, "correct horse battery staple").unwrap();

        let conn = open_encrypted(&path.0, "correct horse battery staple").unwrap();
        let slab_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))
            .unwrap();
        assert_eq!(slab_count, 7, "seed must not duplicate on a second open");
    }

    #[test]
    fn reopening_with_the_wrong_key_fails_to_read() {
        let path = TempDbPath::new("wrongkey");
        open_encrypted(&path.0, "correct horse battery staple").unwrap();

        let wrong = Connection::open(&path.0).unwrap();
        wrong
            .pragma_update(None, "key", "not the right passphrase")
            .unwrap();
        let result = wrong.query_row("SELECT COUNT(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        });
        assert!(result.is_err());
    }
}
