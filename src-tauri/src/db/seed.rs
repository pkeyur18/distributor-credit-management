use rusqlite::{Connection, Result as SqlResult};
use serde_json::json;

// §4.3 default slab table. Thresholds stored ×100 (ADR-004); percentages are
// plain 0–100 integers, not fixed-point money.
const DEFAULT_SLABS: &[(i64, i64)] = &[
    (10_000, 2),
    (40_000, 4),
    (120_000, 6),
    (300_000, 8),
    (500_000, 10),
    (700_000, 12),
    (1_000_000, 14),
];

/// First-run seed: 7 slab rows + 16 settings rows (02-business-rules.md §4.3
/// / §6). Idempotent — only seeds a table that is currently empty, so a
/// second run (or a login on an already-seeded database) is a no-op.
pub fn run(conn: &Connection) -> SqlResult<()> {
    seed_slab_table(conn)?;
    seed_settings(conn)?;
    Ok(())
}

fn seed_slab_table(conn: &Connection) -> SqlResult<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
    for (i, (threshold, percentage)) in DEFAULT_SLABS.iter().enumerate() {
        conn.execute(
            "INSERT INTO slab_table (threshold, percentage, sort_order) VALUES (?1, ?2, ?3)",
            (threshold, percentage, (i + 1) as i64),
        )?;
    }
    Ok(())
}

fn seed_settings(conn: &Connection) -> SqlResult<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    // D-1 applied: five mandatory export columns, not four.
    let default_export_columns = json!([
        "name",
        "member_number",
        "phone",
        "business_volume",
        "total_business_volume"
    ])
    .to_string();
    let slab_thresholds =
        json!(DEFAULT_SLABS.iter().map(|(t, _)| t).collect::<Vec<_>>()).to_string();
    let slab_percentages =
        json!(DEFAULT_SLABS.iter().map(|(_, p)| p).collect::<Vec<_>>()).to_string();
    let yearly_cycle = json!({ "start": "01-01", "end": "12-31" }).to_string();

    let rows: &[(&str, &str)] = &[
        ("slab_thresholds", &slab_thresholds),
        ("slab_percentages", &slab_percentages),
        ("reference_unit_value", "500"),
        ("hierarchy_depth", "4"), // D-3
        ("level_2_width", "9"),
        ("level_3_width", "6"),
        ("level_4_width", "3"),
        ("royalty_qualifying_count", "3"),
        ("royalty_rate_percent", "1"),
        ("yearly_cycle", &yearly_cycle),
        ("low_contribution_threshold", "10000"), // ×100 (ADR-004): 100.00
        ("default_export_columns", &default_export_columns),
        ("session_timeout_minutes", "15"), // D-4
        ("console_backup_schedule", "off"),
        ("console_backup_retention_count", "10"),
        ("console_backup_folder", "backups"),
    ];
    debug_assert_eq!(
        rows.len(),
        16,
        "settings inventory is 16 rows (conflict C1)"
    );

    for (key, value) in rows {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            (key, value),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn seeded_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        super::super::migrations::run(&mut conn).unwrap();
        super::run(&conn).unwrap();
        conn
    }

    #[test]
    fn inserts_exactly_seven_slab_rows_matching_the_default_table() {
        let conn = seeded_db();

        let mut stmt = conn
            .prepare("SELECT threshold, percentage, sort_order FROM slab_table ORDER BY sort_order")
            .unwrap();
        let rows: Vec<(i64, i64, i64)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        // §4.3 defaults, thresholds stored ×100 (ADR-004).
        assert_eq!(
            rows,
            vec![
                (10_000, 2, 1),
                (40_000, 4, 2),
                (120_000, 6, 3),
                (300_000, 8, 4),
                (500_000, 10, 5),
                (700_000, 12, 6),
                (1_000_000, 14, 7),
            ]
        );
    }

    #[test]
    fn inserts_exactly_sixteen_settings_rows() {
        let conn = seeded_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            count, 16,
            "authoritative settings count is 16, not 13 (conflict C1)"
        );
    }

    #[test]
    fn default_export_columns_has_five_entries_per_d1() {
        let conn = seeded_db();
        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'default_export_columns'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let columns: Vec<String> = serde_json::from_str(&value).unwrap();
        assert_eq!(
            columns,
            vec![
                "name",
                "member_number",
                "phone",
                "business_volume",
                "total_business_volume"
            ],
            "D-1: five mandatory columns, not four"
        );
    }

    #[test]
    fn hierarchy_depth_and_session_timeout_match_d3_and_d4() {
        let conn = seeded_db();
        let depth: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'hierarchy_depth'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let timeout: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'session_timeout_minutes'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(depth, "4", "D-3");
        assert_eq!(timeout, "15", "D-4");
    }

    #[test]
    fn seeding_twice_does_not_duplicate_rows() {
        let mut conn = Connection::open_in_memory().unwrap();
        super::super::migrations::run(&mut conn).unwrap();
        super::run(&conn).unwrap();
        super::run(&conn).unwrap();

        let slab_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM slab_table", [], |r| r.get(0))
            .unwrap();
        let settings_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(slab_count, 7);
        assert_eq!(settings_count, 16);
    }
}
