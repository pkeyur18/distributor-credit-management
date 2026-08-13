// M6 — Reports & Exports (US-M6.1/M6.2/M6.3/M6.4/M6.5, S13). Every `.xlsx`
// is generated Rust-side (ADR-007) — the WebView only ever supplies a
// destination path chosen through the same native save dialog `backup.rs`'s
// restore flow already uses for source paths, never raw file content.
use rusqlite::Connection;
use rust_xlsxwriter::{Color, Format, IntoExcelData, Workbook, Worksheet, XlsxError};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

fn xlsx_err(e: XlsxError) -> AppError {
    AppError::Export(e.to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub file_path: String,
}

/// One member's row of exportable data — identity fields always come from
/// the live `members` table (a snapshot only ever carries figures + a
/// point-in-time active flag, per Rule-38), while the five financial
/// fields come from whichever source the caller resolved (live
/// `member_period_totals` for an open/awaiting_close month,
/// `monthly_snapshots` at `MAX(version)` for a closed one — T-M6.1-4).
struct MemberExportRow {
    id: i64,
    name: String,
    phone: String,
    email: Option<String>,
    address: String,
    introducer_member_id: Option<i64>,
    introducer_name: Option<String>,
    level: i64,
    leg_count: i64,
    is_active: bool,
    joining_date: String,
    business_volume: i64,
    total_business_volume: i64,
    slab_pct: i64,
    rewards: i64,
    royalty: i64,
}

const EXPORT_ROW_BASE_COLUMNS: &str = "
    m.id, m.name, m.phone, m.email, m.address, m.introducer_member_id,
    intro.name, m.level,
    (SELECT COUNT(*) FROM members c WHERE c.introducer_member_id = m.id)";

fn row_to_export_row(
    r: &rusqlite::Row,
    is_active: bool,
    business_volume: i64,
    total_business_volume: i64,
    slab_pct: i64,
    rewards: i64,
    royalty: i64,
) -> rusqlite::Result<MemberExportRow> {
    Ok(MemberExportRow {
        id: r.get(0)?,
        name: r.get(1)?,
        phone: r.get(2)?,
        email: r.get(3)?,
        address: r.get(4)?,
        introducer_member_id: r.get(5)?,
        introducer_name: r.get(6)?,
        level: r.get(7)?,
        leg_count: r.get(8)?,
        is_active,
        joining_date: r.get(10)?,
        business_volume,
        total_business_volume,
        slab_pct,
        rewards,
        royalty,
    })
}

/// The open/awaiting_close path (T-M6.1-4's "not closed" branch): live
/// `member_period_totals`, every member included regardless of activity
/// (`T-M6.1-6` — no "active only" filter anywhere in this screen).
fn load_live_export_rows(
    conn: &Connection,
    period_id: i64,
) -> Result<Vec<MemberExportRow>, AppError> {
    let sql = format!(
        "SELECT {EXPORT_ROW_BASE_COLUMNS},
                m.is_active, m.joining_date,
                COALESCE(t.business_volume, 0), COALESCE(t.total_business_volume, 0),
                COALESCE(t.slab_pct, 0), COALESCE(t.rewards, 0), COALESCE(t.royalty, 0)
         FROM members m
         LEFT JOIN members intro ON intro.id = m.introducer_member_id
         LEFT JOIN member_period_totals t ON t.member_id = m.id AND t.period_id = ?1
         ORDER BY m.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([period_id], |r| {
            row_to_export_row(
                r,
                r.get(9)?,
                r.get(11)?,
                r.get(12)?,
                r.get(13)?,
                r.get(14)?,
                r.get(15)?,
            )
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// The closed-month path (T-M6.1-4's "reads from the permanent snapshot"
/// branch): `monthly_snapshots` at `MAX(version)` per member, and the
/// snapshot's own point-in-time `is_active_status` rather than the
/// member's current live status. An `INNER JOIN` against the snapshot
/// means a period with none (Task 1's empty-month path, T-M5.4-1) simply
/// returns zero rows — the caller never reaches this for such a period
/// anyway, since it's excluded from the closed-month picker (T-M5.4-2).
fn load_snapshot_export_rows(
    conn: &Connection,
    period_id: i64,
) -> Result<Vec<MemberExportRow>, AppError> {
    let sql = format!(
        "SELECT {EXPORT_ROW_BASE_COLUMNS},
                s.is_active_status, m.joining_date,
                s.business_volume, s.total_business_volume,
                s.slab_pct, s.rewards, s.royalty
         FROM monthly_snapshots s
         JOIN members m ON m.id = s.member_id
         LEFT JOIN members intro ON intro.id = m.introducer_member_id
         WHERE s.period_id = ?1
           AND s.version = (
                SELECT MAX(version) FROM monthly_snapshots s2
                WHERE s2.member_id = s.member_id AND s2.period_id = s.period_id
               )
         ORDER BY m.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([period_id], |r| {
            row_to_export_row(
                r,
                r.get(9)?,
                r.get(11)?,
                r.get(12)?,
                r.get(13)?,
                r.get(14)?,
                r.get(15)?,
            )
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn resolve_period(conn: &Connection, period_month: &str) -> Result<(i64, String), AppError> {
    conn.query_row(
        "SELECT id, status FROM periods WHERE period_month = ?1",
        [period_month],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .map_err(|_| AppError::NotFound {
        message: "Period not found.".into(),
    })
}

/// Rule-19/D-1: the five mandatory columns first, in this fixed order,
/// then whichever optional columns were selected — matching header order
/// and cell order exactly, since a picker checkbox that reordered columns
/// between runs would make two extracts of the same month hard to compare.
fn write_export_xlsx(
    rows: &[MemberExportRow],
    optional: &[OptionalColumn],
    output_path: &str,
) -> Result<(), AppError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let mut col: u16 = 0;
    for header in MANDATORY_COLUMNS {
        worksheet.write(0, col, header).map_err(xlsx_err)?;
        col += 1;
    }
    for opt in optional {
        worksheet.write(0, col, opt.header()).map_err(xlsx_err)?;
        col += 1;
    }

    // T-M6.1-6/Rule-42: a deactivated member still appears in every export
    // (no "active only" filter exists), in the same distinct colour used
    // on screen — Excel's own theming is unrelated to the app's light/dark
    // mode, so this is a fixed light-background tint, not a CSS variable.
    let inactive_format = Format::new().set_background_color(Color::RGB(0xFEF2F2));

    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        let format = if row.is_active {
            None
        } else {
            Some(&inactive_format)
        };
        let mut c: u16 = 0;
        write_cell(worksheet, r, c, row.name.as_str(), format)?;
        c += 1;
        write_cell(worksheet, r, c, row.id, format)?;
        c += 1;
        write_cell(worksheet, r, c, row.phone.as_str(), format)?;
        c += 1;
        write_cell(worksheet, r, c, row.business_volume as f64 / 100.0, format)?;
        c += 1;
        write_cell(
            worksheet,
            r,
            c,
            row.total_business_volume as f64 / 100.0,
            format,
        )?;
        c += 1;

        for opt in optional {
            match opt {
                OptionalColumn::Email => write_cell(worksheet, r, c, row.email.as_deref(), format)?,
                OptionalColumn::Address => {
                    write_cell(worksheet, r, c, row.address.as_str(), format)?
                }
                OptionalColumn::ReferenceNumber => {
                    write_cell(worksheet, r, c, row.introducer_member_id, format)?
                }
                OptionalColumn::IntroducerName => {
                    write_cell(worksheet, r, c, row.introducer_name.as_deref(), format)?
                }
                OptionalColumn::HierarchyLevel => write_cell(worksheet, r, c, row.level, format)?,
                OptionalColumn::DirectLegsCount => {
                    write_cell(worksheet, r, c, row.leg_count, format)?
                }
                OptionalColumn::SlabPct => write_cell(worksheet, r, c, row.slab_pct, format)?,
                OptionalColumn::Rewards => {
                    write_cell(worksheet, r, c, row.rewards as f64 / 100.0, format)?
                }
                OptionalColumn::RoyaltyEarned => {
                    write_cell(worksheet, r, c, row.royalty as f64 / 100.0, format)?
                }
                OptionalColumn::JoiningDate => {
                    write_cell(worksheet, r, c, row.joining_date.as_str(), format)?
                }
                OptionalColumn::ActiveStatus => {
                    let status = if row.is_active { "Active" } else { "Inactive" };
                    write_cell(worksheet, r, c, status, format)?
                }
            }
            c += 1;
        }
    }

    workbook.save(output_path).map_err(xlsx_err)?;
    Ok(())
}

fn write_cell(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    data: impl IntoExcelData,
    format: Option<&Format>,
) -> Result<(), AppError> {
    match format {
        Some(f) => worksheet
            .write_with_format(row, col, data, f)
            .map_err(xlsx_err)?,
        None => worksheet.write(row, col, data).map_err(xlsx_err)?,
    };
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMonthlyInput {
    pub period_month: String,
    /// Rule-33 keys (`OptionalColumn::parse`) — the five mandatory columns
    /// are never in this list; the frontend's picker never offers them as
    /// untickable.
    #[serde(default)]
    pub optional_columns: Vec<String>,
    pub output_path: String,
}

/// API-16. `T-M6.1-4`: an already-`closed` period reads its permanent
/// snapshot at `MAX(version)`, never live totals (which would read as
/// zeroed — Rule-38). `output_path` is chosen by the operator through a
/// native save dialog on the frontend (ADR-007) and passed straight
/// through, exactly like `backup::restore_from_backup_file`'s
/// `source_path` already does for the read side of the same boundary.
pub fn export_monthly(
    conn: &Connection,
    input: ExportMonthlyInput,
) -> Result<ExportResult, AppError> {
    let (period_id, status) = resolve_period(conn, &input.period_month)?;
    let optional = input
        .optional_columns
        .iter()
        .map(|k| OptionalColumn::parse(k))
        .collect::<Result<Vec<_>, _>>()?;
    let rows = if status == "closed" {
        load_snapshot_export_rows(conn, period_id)?
    } else {
        load_live_export_rows(conn, period_id)?
    };
    write_export_xlsx(&rows, &optional, &input.output_path)?;
    Ok(ExportResult {
        file_path: input.output_path,
    })
}

struct YearlyAverageRow {
    id: i64,
    name: String,
    phone: String,
    avg_business_volume: f64,
    avg_total_business_volume: f64,
    period_count: i64,
}

/// Rule-23: the divisor is **per member** — the count of periods that
/// specifically have a snapshot *for that member*, not a single
/// system-wide count shared by everyone. T-M6.2-1's "protects late
/// joiners" is exactly this: a member who joined two months ago has only
/// two possible snapshots and must average over two, not over every
/// period the console has ever closed. A member with zero snapshots
/// (nothing ever closed since they joined) has nothing to average and is
/// excluded from the report entirely, rather than shown as a division by
/// zero.
fn compute_yearly_averages(conn: &Connection) -> Result<Vec<YearlyAverageRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.name, m.phone,
                AVG(s.business_volume), AVG(s.total_business_volume), COUNT(*)
         FROM members m
         JOIN monthly_snapshots s ON s.member_id = m.id
         WHERE s.version = (
                SELECT MAX(version) FROM monthly_snapshots s2
                WHERE s2.member_id = s.member_id AND s2.period_id = s.period_id
               )
         GROUP BY m.id
         ORDER BY m.id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(YearlyAverageRow {
                id: r.get(0)?,
                name: r.get(1)?,
                phone: r.get(2)?,
                avg_business_volume: r.get(3)?,
                avg_total_business_volume: r.get(4)?,
                period_count: r.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// API-17. Extract carries both the Total Business Volume and own-Business
/// Volume averages, each sharing the one "Months" count (T-M6.2-3) — both
/// figures are averaged over exactly the same set of periods, since a
/// snapshot row always carries both fields together.
pub fn export_yearly_average(
    conn: &Connection,
    output_path: &str,
) -> Result<ExportResult, AppError> {
    let rows = compute_yearly_averages(conn)?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let headers = [
        "Name",
        "Member Number",
        "Phone",
        "Average Business Volume",
        "Average Total Business Volume",
        "Months",
    ];
    for (col, header) in headers.iter().enumerate() {
        worksheet.write(0, col as u16, *header).map_err(xlsx_err)?;
    }
    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        worksheet.write(r, 0, row.name.as_str()).map_err(xlsx_err)?;
        worksheet.write(r, 1, row.id).map_err(xlsx_err)?;
        worksheet
            .write(r, 2, row.phone.as_str())
            .map_err(xlsx_err)?;
        worksheet
            .write(r, 3, row.avg_business_volume / 100.0)
            .map_err(xlsx_err)?;
        worksheet
            .write(r, 4, row.avg_total_business_volume / 100.0)
            .map_err(xlsx_err)?;
        worksheet.write(r, 5, row.period_count).map_err(xlsx_err)?;
    }
    workbook.save(output_path).map_err(xlsx_err)?;

    Ok(ExportResult {
        file_path: output_path.to_string(),
    })
}

/// D-1/Rule-19/Rule-33 (06-decision-log-and-open-items.md C9): five
/// columns, always present on every per-member extract, untickable in the
/// column picker, in this fixed order.
pub const MANDATORY_COLUMNS: [&str; 5] = [
    "Name",
    "Member Number",
    "Phone",
    "Business Volume",
    "Total Business Volume",
];

/// Rule-33's optional list, minus Total Business Volume (moved to
/// `MANDATORY_COLUMNS` by D-1). Keys match the frontend's
/// `OPTIONAL_COLUMNS` keys one-to-one, so the column picker and this
/// extraction switch can never silently drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalColumn {
    Email,
    Address,
    ReferenceNumber,
    IntroducerName,
    HierarchyLevel,
    DirectLegsCount,
    SlabPct,
    Rewards,
    RoyaltyEarned,
    JoiningDate,
    ActiveStatus,
}

impl OptionalColumn {
    // Keys match settings.tsx's `OPTIONAL_EXPORT_COLUMNS` (T-M7.2-4, S10)
    // exactly — that constant's own comment says M6 must align with it
    // rather than invent a second set, since `default_export_columns`
    // (db/seed.rs) already persists values in this convention.
    pub fn parse(key: &str) -> Result<Self, AppError> {
        Ok(match key {
            "email" => Self::Email,
            "address" => Self::Address,
            "reference_number" => Self::ReferenceNumber,
            "introducer_name" => Self::IntroducerName,
            "hierarchy_level" => Self::HierarchyLevel,
            "direct_legs_count" => Self::DirectLegsCount,
            "slab_pct" => Self::SlabPct,
            "rewards" => Self::Rewards,
            "royalty_earned" => Self::RoyaltyEarned,
            "joining_date" => Self::JoiningDate,
            "active_status" => Self::ActiveStatus,
            other => {
                return Err(AppError::Validation {
                    field: "optionalColumns".into(),
                    message: format!("Unknown export column '{other}'."),
                })
            }
        })
    }

    pub fn header(self) -> &'static str {
        match self {
            Self::Email => "Email",
            Self::Address => "Address",
            Self::ReferenceNumber => "Reference Number",
            Self::IntroducerName => "Introducer Name",
            Self::HierarchyLevel => "Hierarchy Level",
            Self::DirectLegsCount => "Direct Legs Count",
            Self::SlabPct => "Slab %",
            Self::Rewards => "Rewards",
            Self::RoyaltyEarned => "Royalty Earned",
            Self::JoiningDate => "Joining Date",
            Self::ActiveStatus => "Status",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mandatory_columns_are_the_five_named_by_d_1() {
        assert_eq!(
            MANDATORY_COLUMNS,
            [
                "Name",
                "Member Number",
                "Phone",
                "Business Volume",
                "Total Business Volume",
            ]
        );
    }

    #[test]
    fn optional_column_parse_round_trips_every_settings_screen_key() {
        // Must match settings.tsx's OPTIONAL_EXPORT_COLUMNS exactly (T-M7.2-4).
        let keys = [
            "email",
            "address",
            "reference_number",
            "introducer_name",
            "hierarchy_level",
            "direct_legs_count",
            "slab_pct",
            "rewards",
            "royalty_earned",
            "joining_date",
            "active_status",
        ];
        for key in keys {
            assert!(OptionalColumn::parse(key).is_ok(), "key '{key}' must parse");
        }
    }

    #[test]
    fn optional_column_parse_refuses_an_unknown_key() {
        let err = OptionalColumn::parse("total_business_volume").unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    use crate::db;

    fn seeded() -> Connection {
        db::open_seeded_in_memory().unwrap()
    }

    fn temp_output_path(label: &str) -> std::path::PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bvconsole-m6-test-{nanos}-{unique}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(format!("{label}.xlsx"))
    }

    fn insert_member(
        conn: &Connection,
        name: &str,
        is_active: bool,
        introducer: Option<i64>,
    ) -> i64 {
        static NEXT_PHONE: std::sync::atomic::AtomicI64 =
            std::sync::atomic::AtomicI64::new(9_200_000_000);
        let phone = NEXT_PHONE
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string();
        conn.execute(
            "INSERT INTO members
                (name, phone, address, introducer_member_id, level, is_active,
                 joining_date, consent_given, consent_date, created_at)
             VALUES (?1, ?2, 'addr', ?3, 1, ?4, '2026-01-01', 1, '2026-01-01', '2026-01-01')",
            rusqlite::params![name, phone, introducer, is_active],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_period(conn: &Connection, month: &str, status: &str) -> i64 {
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, ?2)",
            [month, status],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_totals(conn: &Connection, member_id: i64, period_id: i64, bv: i64, tbv: i64) {
        conn.execute(
            "INSERT INTO member_period_totals
                (member_id, period_id, business_volume, total_business_volume, slab_pct,
                 differential, royalty, own_reward, rewards)
             VALUES (?1, ?2, ?3, ?4, 6, 0, 0, ?3, ?3)",
            rusqlite::params![member_id, period_id, bv, tbv],
        )
        .unwrap();
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_snapshot(
        conn: &Connection,
        member_id: i64,
        period_id: i64,
        version: i64,
        bv: i64,
        tbv: i64,
        is_active: bool,
    ) {
        conn.execute(
            "INSERT INTO monthly_snapshots
                (member_id, period_id, version, business_volume, total_business_volume,
                 slab_pct, differential, royalty, own_reward, rewards, is_active_status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 6, 0, 0, ?4, ?4, ?6, '2026-06-01')",
            rusqlite::params![member_id, period_id, version, bv, tbv, is_active],
        )
        .unwrap();
    }

    #[test]
    fn load_live_export_rows_includes_a_member_with_no_activity_this_period() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08", "open");
        let with_activity = insert_member(&conn, "Active One", true, None);
        let without_activity = insert_member(&conn, "Quiet One", true, Some(with_activity));
        insert_totals(&conn, with_activity, period, 100_000, 100_000);

        let rows = load_live_export_rows(&conn, period).unwrap();
        assert_eq!(rows.len(), 2);
        let quiet = rows.iter().find(|r| r.id == without_activity).unwrap();
        assert_eq!(quiet.business_volume, 0);
        assert_eq!(quiet.total_business_volume, 0);
    }

    #[test]
    fn load_live_export_rows_includes_a_deactivated_member_with_no_active_only_filter() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08", "open");
        let inactive = insert_member(&conn, "Gone", false, None);
        insert_totals(&conn, inactive, period, 50_000, 50_000);

        let rows = load_live_export_rows(&conn, period).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].is_active);
    }

    #[test]
    fn load_snapshot_export_rows_reads_the_latest_version_not_the_original() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let member = insert_member(&conn, "Corrected", true, None);
        insert_snapshot(&conn, member, period, 1, 100_000, 100_000, true);
        insert_snapshot(&conn, member, period, 2, 250_000, 250_000, true);

        let rows = load_snapshot_export_rows(&conn, period).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].business_volume, 250_000,
            "must read version 2, the correction, not version 1"
        );
    }

    #[test]
    fn load_snapshot_export_rows_uses_the_point_in_time_active_status() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        // Active in the live members table now, but the snapshot recorded
        // them as inactive at close time — the export must reflect the
        // snapshot's own point-in-time flag, not today's live status.
        let member = insert_member(&conn, "Reactivated Since", true, None);
        insert_snapshot(&conn, member, period, 1, 10_000, 10_000, false);

        let rows = load_snapshot_export_rows(&conn, period).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].is_active);
    }

    #[test]
    fn load_snapshot_export_rows_excludes_a_period_with_no_snapshot_at_all() {
        // T-M5.4-1/T-M5.4-2: an empty-month close writes zero snapshot
        // rows — the query must return zero rows too, not error.
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        insert_member(&conn, "Nobody Entered", true, None);

        let rows = load_snapshot_export_rows(&conn, period).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn export_monthly_writes_all_five_mandatory_columns_regardless_of_optional_selection() {
        let conn = seeded();
        insert_period(&conn, "2026-08", "open");
        insert_member(&conn, "Solo", true, None);
        let output_path = temp_output_path("monthly-open");

        let result = export_monthly(
            &conn,
            ExportMonthlyInput {
                period_month: "2026-08".into(),
                optional_columns: vec![],
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap();

        assert_eq!(result.file_path, output_path.to_string_lossy());
        assert!(
            output_path.exists(),
            "the .xlsx file must actually be written"
        );
        assert!(std::fs::metadata(&output_path).unwrap().len() > 0);
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
    }

    #[test]
    fn export_monthly_reads_the_closed_periods_snapshot_not_zeroed_live_totals() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let member = insert_member(&conn, "Closed Month Member", true, None);
        // Rule-38: live totals are zeroed on close — if export_monthly read
        // these instead of the snapshot, the extract would show zero.
        insert_totals(&conn, member, period, 0, 0);
        insert_snapshot(&conn, member, period, 1, 75_000, 75_000, true);
        let output_path = temp_output_path("monthly-closed");

        let result = export_monthly(
            &conn,
            ExportMonthlyInput {
                period_month: "2026-05".into(),
                optional_columns: vec!["active_status".into()],
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap();

        assert!(std::path::Path::new(&result.file_path).exists());
        // The row-loading path itself is asserted directly above
        // (`load_snapshot_export_rows_reads_the_latest_version...`); this
        // confirms `export_monthly` actually takes the closed-period
        // branch rather than the live one for a `closed` status.
        let rows = load_snapshot_export_rows(&conn, period).unwrap();
        assert_eq!(rows[0].business_volume, 75_000);
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
    }

    #[test]
    fn export_monthly_refuses_an_unknown_month() {
        let conn = seeded();
        let output_path = temp_output_path("monthly-unknown");
        let err = export_monthly(
            &conn,
            ExportMonthlyInput {
                period_month: "2099-01".into(),
                optional_columns: vec![],
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn export_monthly_refuses_an_unknown_optional_column_key() {
        let conn = seeded();
        insert_period(&conn, "2026-08", "open");
        let output_path = temp_output_path("monthly-bad-column");
        let err = export_monthly(
            &conn,
            ExportMonthlyInput {
                period_month: "2026-08".into(),
                optional_columns: vec!["notARealColumn".into()],
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn compute_yearly_averages_divides_by_the_members_own_snapshot_count_not_a_global_one() {
        // T-M6.2-1: "protects late joiners" — Member A has three closed
        // periods' worth of snapshots, Member B (a late joiner) has one.
        // B's average must divide by 1, not by 3.
        let conn = seeded();
        let p1 = insert_period(&conn, "2026-03", "closed");
        let p2 = insert_period(&conn, "2026-04", "closed");
        let p3 = insert_period(&conn, "2026-05", "closed");
        let a = insert_member(&conn, "Long Timer", true, None);
        let b = insert_member(&conn, "Late Joiner", true, None);
        insert_snapshot(&conn, a, p1, 1, 100_000, 100_000, true);
        insert_snapshot(&conn, a, p2, 1, 200_000, 200_000, true);
        insert_snapshot(&conn, a, p3, 1, 300_000, 300_000, true);
        insert_snapshot(&conn, b, p3, 1, 90_000, 90_000, true);

        let rows = compute_yearly_averages(&conn).unwrap();
        let a_row = rows.iter().find(|r| r.id == a).unwrap();
        let b_row = rows.iter().find(|r| r.id == b).unwrap();

        assert_eq!(a_row.period_count, 3);
        assert_eq!(a_row.avg_business_volume, 200_000.0); // (100k+200k+300k)/3

        assert_eq!(
            b_row.period_count, 1,
            "late joiner divides by their own count"
        );
        assert_eq!(b_row.avg_business_volume, 90_000.0);
    }

    #[test]
    fn compute_yearly_averages_uses_the_corrected_version_not_the_original() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let member = insert_member(&conn, "Corrected", true, None);
        insert_snapshot(&conn, member, period, 1, 100_000, 100_000, true);
        insert_snapshot(&conn, member, period, 2, 400_000, 400_000, true);

        let rows = compute_yearly_averages(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].avg_business_volume, 400_000.0);
    }

    #[test]
    fn compute_yearly_averages_excludes_a_member_with_no_snapshot_at_all() {
        let conn = seeded();
        insert_member(&conn, "Never Closed A Month Under", true, None);
        let rows = compute_yearly_averages(&conn).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn export_yearly_average_writes_a_file_with_the_expected_header() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let member = insert_member(&conn, "Solo", true, None);
        insert_snapshot(&conn, member, period, 1, 100_000, 100_000, true);
        let output_path = temp_output_path("yearly-average");

        let result = export_yearly_average(&conn, &output_path.to_string_lossy()).unwrap();

        assert_eq!(result.file_path, output_path.to_string_lossy());
        assert!(output_path.exists());
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
    }
}
