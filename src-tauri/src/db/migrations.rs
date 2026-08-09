use rusqlite::{Connection, Result as SqlResult};

const MIGRATIONS: &[(u32, &str)] = &[(1, include_str!("migrations/0001_initial.sql"))];

pub fn run(conn: &mut Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     INTEGER PRIMARY KEY,
            applied_at  TEXT NOT NULL
        )",
    )?;

    for &(version, sql) in MIGRATIONS {
        let already_applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [version],
            |row| row.get(0),
        )?;
        if already_applied {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            [version],
        )?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn all_tables(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    #[test]
    fn creates_all_ten_entity_tables_on_a_fresh_database() {
        let mut conn = Connection::open_in_memory().unwrap();
        super::run(&mut conn).unwrap();

        let tables = all_tables(&conn);
        let expected = [
            "audit_log",
            "auth",
            "backups",
            "business_volume_entries",
            "member_period_totals",
            "members",
            "monthly_snapshots",
            "periods",
            "settings",
            "slab_table",
        ];
        for table in expected {
            assert!(tables.contains(&table.to_string()), "missing table {table}");
        }
    }

    #[test]
    fn is_idempotent_when_run_twice_on_the_same_database() {
        let mut conn = Connection::open_in_memory().unwrap();
        super::run(&mut conn).unwrap();
        super::run(&mut conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "each migration should be recorded exactly once");
    }

    #[test]
    fn auth_table_has_no_session_timeout_column() {
        // D-14: settings is the single source of truth for the session timeout.
        let mut conn = Connection::open_in_memory().unwrap();
        super::run(&mut conn).unwrap();

        let stmt = conn.prepare("SELECT * FROM auth").unwrap();
        let names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        assert!(
            !names.contains(&"session_timeout_minutes".to_string()),
            "auth table must not carry session_timeout_minutes (D-14)"
        );
    }

    #[test]
    fn periods_status_check_allows_awaiting_close_not_ended_locked() {
        let mut conn = Connection::open_in_memory().unwrap();
        super::run(&mut conn).unwrap();

        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES ('2026-08', 'awaiting_close')",
            [],
        )
        .expect("awaiting_close must be a valid status");

        let rejected = conn.execute(
            "INSERT INTO periods (period_month, status) VALUES ('2026-09', 'ended_locked')",
            [],
        );
        assert!(
            rejected.is_err(),
            "ended_locked is stale wording and must be rejected"
        );
    }
}
