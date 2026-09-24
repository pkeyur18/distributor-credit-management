use rusqlite::{Connection, Result as SqlResult};

const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("migrations/0001_initial.sql")),
    (2, include_str!("migrations/0002_membership_tier.sql")),
];

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
    fn creates_all_nine_entity_tables_on_a_fresh_database() {
        // Nine, not the published DDL's ten — no `auth` table; see
        // 0001_initial.sql's header comment for why.
        let mut conn = Connection::open_in_memory().unwrap();
        super::run(&mut conn).unwrap();

        let tables = all_tables(&conn);
        let expected = [
            "audit_log",
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
        assert_eq!(count, 2, "each migration should be recorded exactly once");
    }

    fn column_names(conn: &Connection, table: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    #[test]
    fn membership_tier_column_exists_on_live_totals_and_snapshots() {
        let mut conn = Connection::open_in_memory().unwrap();
        super::run(&mut conn).unwrap();
        for table in ["member_period_totals", "monthly_snapshots"] {
            assert!(
                column_names(&conn, table).contains(&"membership_tier".to_string()),
                "{table} is missing membership_tier"
            );
        }
    }

    #[test]
    fn migration_0002_inserts_no_settings_on_an_empty_database() {
        // db/mod.rs runs migrations *before* seed::run, and seed_settings
        // skips entirely when settings is non-empty — so 0002 must not
        // insert anything into a brand-new database.
        let mut conn = Connection::open_in_memory().unwrap();
        super::run(&mut conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migration_0002_backfills_level_settings_on_an_already_seeded_database() {
        // Simulates an existing install: 0001 applied and seeded, 0002 not yet.
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)",
        )
        .unwrap();
        conn.execute_batch(include_str!("migrations/0001_initial.sql"))
            .unwrap();
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute_batch(
            "INSERT INTO settings (key, value) VALUES
                ('royalty_qualifying_count', '4'),
                ('royalty_rate_percent', '1.5')",
        )
        .unwrap();

        super::run(&mut conn).unwrap();

        let value = |key: &str| -> String {
            conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
                r.get(0)
            })
            .unwrap()
        };
        assert_eq!(
            value("royalty_qualifying_count"),
            "4",
            "existing value untouched"
        );
        assert_eq!(
            value("royalty_rate_percent"),
            "1.5",
            "existing value untouched"
        );
        for rank in 2..=4 {
            assert_eq!(value(&format!("royalty_tier_{rank}_qualifying_count")), "3");
            assert_eq!(
                value(&format!("royalty_tier_{rank}_rate_percent")),
                "1.5",
                "upgrade copies the installation's current rate so no Rewards figure moves"
            );
        }
    }

    #[test]
    fn no_auth_table_exists() {
        // Credential/lockout state lives in an unencrypted sidecar file
        // (m8_auth::store) — see 0001_initial.sql's header comment for why
        // an in-database table is unreadable in principle.
        let mut conn = Connection::open_in_memory().unwrap();
        super::run(&mut conn).unwrap();

        assert!(!all_tables(&conn).contains(&"auth".to_string()));
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
