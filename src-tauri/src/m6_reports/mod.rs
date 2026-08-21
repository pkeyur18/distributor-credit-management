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
/// Sort field/direction pair the operator picks in the Reports tab before
/// exporting — one dropdown per report card, always driving that report's
/// exported `.xlsx` row order. `MonthlySortField`/`YearlySortField` are
/// separate enums rather than one shared enum because the two reports
/// genuinely don't share a field set (a yearly-average row has no
/// `slab_pct`/`rewards` at all) — an invalid combination should be
/// unrepresentable, not a runtime validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MonthlySortField {
    Name,
    BusinessVolume,
    TotalBusinessVolume,
    SlabPct,
    Rewards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum YearlySortField {
    Name,
    AvgBusinessVolume,
    AvgTotalBusinessVolume,
}

/// Name compares case-insensitively (an operator sorting by name expects
/// "bob" and "Zoe" ordered by letter, not by ASCII case); every field then
/// ties on `id` so two equal values (or two same-named members) always
/// land in the same stable order run to run.
fn sort_monthly_rows(rows: &mut [MemberExportRow], field: MonthlySortField, dir: SortDirection) {
    rows.sort_by(|a, b| {
        let ord = match field {
            MonthlySortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            MonthlySortField::BusinessVolume => a.business_volume.cmp(&b.business_volume),
            MonthlySortField::TotalBusinessVolume => {
                a.total_business_volume.cmp(&b.total_business_volume)
            }
            MonthlySortField::SlabPct => a.slab_pct.cmp(&b.slab_pct),
            MonthlySortField::Rewards => a.rewards.cmp(&b.rewards),
        };
        let ord = if dir == SortDirection::Desc { ord.reverse() } else { ord };
        ord.then_with(|| a.id.cmp(&b.id))
    });
}

fn sort_yearly_rows(rows: &mut [YearlyAverageRow], field: YearlySortField, dir: SortDirection) {
    rows.sort_by(|a, b| {
        let ord = match field {
            YearlySortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            YearlySortField::AvgBusinessVolume => a
                .avg_business_volume
                .partial_cmp(&b.avg_business_volume)
                .unwrap_or(std::cmp::Ordering::Equal),
            YearlySortField::AvgTotalBusinessVolume => a
                .avg_total_business_volume
                .partial_cmp(&b.avg_total_business_volume)
                .unwrap_or(std::cmp::Ordering::Equal),
        };
        let ord = if dir == SortDirection::Desc { ord.reverse() } else { ord };
        ord.then_with(|| a.id.cmp(&b.id))
    });
}

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

/// 07-design-system.md's Colour-Plus-Label Rule (NFR-8, explicitly naming
/// M6.5): a deactivated row's distinct colour is never allowed to stand
/// alone without a text label. Rule-33 lists Active/inactive status as
/// merely optional, but that optional-ness only governs whether the
/// *picker* starts it ticked — this accessibility requirement means the
/// column itself is effectively always present once any row is
/// colour-coded, so it's forced in here rather than trusting every caller
/// to remember it.
fn with_forced_active_status(optional: &[OptionalColumn]) -> Vec<OptionalColumn> {
    let mut optional = optional.to_vec();
    if !optional.contains(&OptionalColumn::ActiveStatus) {
        optional.push(OptionalColumn::ActiveStatus);
    }
    optional
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
    let optional = with_forced_active_status(optional);
    let optional = optional.as_slice();

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
    pub sort_field: MonthlySortField,
    pub sort_direction: SortDirection,
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
    let mut rows = if status == "closed" {
        load_snapshot_export_rows(conn, period_id)?
    } else {
        load_live_export_rows(conn, period_id)?
    };
    sort_monthly_rows(&mut rows, input.sort_field, input.sort_direction);
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyPreviewRow {
    pub id: i64,
    pub name: String,
    pub business_volume: i64,
    pub total_business_volume: i64,
    pub slab_pct: i64,
}

/// API-43 — the Reports screen's on-screen "Monthly data" preview table
/// (prototype parity). Read-only: returns numbers already computed by the
/// same `load_live_export_rows`/`load_snapshot_export_rows` paths
/// `export_monthly` uses, never raw file content, so ADR-002/ADR-007's
/// WebView-never-touches-the-filesystem boundary is untouched — this is
/// the same shape as `get_member_detail` already returning computed
/// rewards data to the frontend.
///
/// Returned in member-id order — the Reports screen applies whichever sort
/// the operator picked client-side (reports.tsx), since every field this
/// preview can display (name/BV/Total BV/Slab %) is already present here.
/// `rewards` is deliberately not a field on `MonthlyPreviewRow` (it isn't a
/// column in this table), so a Rewards sort choice reorders only the
/// exported file, not this preview.
pub fn preview_monthly_data(
    conn: &Connection,
    period_month: &str,
) -> Result<Vec<MonthlyPreviewRow>, AppError> {
    let (period_id, status) = resolve_period(conn, period_month)?;
    let rows = if status == "closed" {
        load_snapshot_export_rows(conn, period_id)?
    } else {
        load_live_export_rows(conn, period_id)?
    };
    let preview: Vec<MonthlyPreviewRow> = rows
        .into_iter()
        .map(|r| MonthlyPreviewRow {
            id: r.id,
            name: r.name,
            business_volume: r.business_volume,
            total_business_volume: r.total_business_volume,
            slab_pct: r.slab_pct,
        })
        .collect();
    Ok(preview)
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YearlyAveragePreviewRow {
    pub id: i64,
    pub name: String,
    pub avg_business_volume: f64,
    pub avg_total_business_volume: f64,
    pub period_count: i64,
}

/// API-44 — the Reports screen's "Yearly average" preview table and the
/// "Low-contribution report" stat-card/table (prototype parity), both
/// sourced from this single list: the frontend filters it on the
/// threshold input locally, recomputing live as the operator types
/// (matching the prototype's `oninput` behaviour) rather than round-
/// tripping per keystroke. Reuses `compute_yearly_averages` — the
/// low-contribution "own BV, not Total BV" rule (Rule-24) stays the sole
/// responsibility of `export_low_contribution` when actually exporting;
/// this list is presentation only.
///
/// Returned in member-id order — the two cards that read this list (Yearly
/// Average, Low-Contribution) each apply their own sort choice client-side,
/// since they can disagree on ordering while sharing this one fetch.
pub fn preview_yearly_average(conn: &Connection) -> Result<Vec<YearlyAveragePreviewRow>, AppError> {
    let rows: Vec<YearlyAveragePreviewRow> = compute_yearly_averages(conn)?
        .into_iter()
        .map(|r| YearlyAveragePreviewRow {
            id: r.id,
            name: r.name,
            avg_business_volume: r.avg_business_volume,
            avg_total_business_volume: r.avg_total_business_volume,
            period_count: r.period_count,
        })
        .collect();
    Ok(rows)
}

/// API-17. Extract carries both the Total Business Volume and own-Business
/// Volume averages, each sharing the one "Months" count (T-M6.2-3) — both
/// figures are averaged over exactly the same set of periods, since a
/// snapshot row always carries both fields together.
pub fn export_yearly_average(
    conn: &Connection,
    output_path: &str,
    sort_field: YearlySortField,
    sort_direction: SortDirection,
) -> Result<ExportResult, AppError> {
    let mut rows = compute_yearly_averages(conn)?;
    sort_yearly_rows(&mut rows, sort_field, sort_direction);

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

fn low_contribution_threshold_setting(conn: &Connection) -> Result<i64, AppError> {
    let value: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'low_contribution_threshold'",
        [],
        |r| r.get(0),
    )?;
    value.parse().map_err(|_| AppError::Validation {
        field: "lowContributionThreshold".into(),
        message: "Stored low-contribution threshold is not a valid number.".into(),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLowContributionInput {
    /// Cents (ADR-004). `None` reads `settings.low_contribution_threshold`
    /// (default 100.00) — T-M6.3-2's "overridable per run" without
    /// changing the stored setting.
    pub threshold: Option<i64>,
    pub sort_field: YearlySortField,
    pub sort_direction: SortDirection,
    pub output_path: String,
}

/// API-18/Rule-24: filters on the yearly average of **own** Business
/// Volume, never Total Business Volume — the client's answer differed
/// from the architect's original recommendation and was deliberately
/// re-confirmed (Rule-24's own source note). Reuses
/// `compute_yearly_averages`'s per-member denominator (T-M6.2-1), so a
/// late joiner is filtered on their own average, not one diluted by
/// months before they existed.
pub fn export_low_contribution(
    conn: &Connection,
    input: ExportLowContributionInput,
) -> Result<ExportResult, AppError> {
    let threshold = match input.threshold {
        Some(t) => t,
        None => low_contribution_threshold_setting(conn)?,
    };
    let mut rows: Vec<YearlyAverageRow> = compute_yearly_averages(conn)?
        .into_iter()
        .filter(|r| r.avg_business_volume < threshold as f64)
        .collect();
    sort_yearly_rows(&mut rows, input.sort_field, input.sort_direction);

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let headers = [
        "Name",
        "Member Number",
        "Phone",
        "Average Business Volume",
        "Months",
    ];
    for (col, header) in headers.iter().enumerate() {
        worksheet.write(0, col as u16, *header).map_err(xlsx_err)?;
    }
    // T-M6.3-3: an empty result is still a valid, successfully-written
    // extract — zero data rows, not an error.
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
        worksheet.write(r, 4, row.period_count).map_err(xlsx_err)?;
    }
    workbook.save(&input.output_path).map_err(xlsx_err)?;

    Ok(ExportResult {
        file_path: input.output_path,
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedMonthBackup {
    pub period_id: i64,
    pub period_month: String,
    pub closed_at: Option<String>,
    pub latest_version: i64,
    pub is_corrected: bool,
}

/// API-19: closed periods that have a snapshot — the `INNER JOIN` against
/// `monthly_snapshots` excludes an empty-month close (T-M5.4-1's zero-row
/// path) automatically, matching T-M5.4-2's "not offered as a closed-month
/// export option." This is a different listing from `backup::list_restore_points`
/// (API-35, S14): that one lists every whole-console backup by row for the
/// Settings restore card; this one lists closed *periods* that have
/// exportable snapshot data, for the Reports screen's re-download card.
pub fn list_backups(conn: &Connection) -> Result<Vec<ClosedMonthBackup>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.period_month, p.closed_at, MAX(s.version)
         FROM periods p
         JOIN monthly_snapshots s ON s.period_id = p.id
         WHERE p.status = 'closed'
         GROUP BY p.id
         ORDER BY p.period_month DESC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            let latest_version: i64 = r.get(3)?;
            Ok(ClosedMonthBackup {
                period_id: r.get(0)?,
                period_month: r.get(1)?,
                closed_at: r.get(2)?,
                latest_version,
                is_corrected: latest_version > 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// API-20/HIGH-1: the command behind the prototype's "Closed month
/// snapshot" card — no separate export command exists. Always the latest
/// version (T-M6.4-2): a correction's original stays in the audit trail
/// only, never re-enters an export. Fixed column set, matching the
/// prototype's `exportClosedSnapshot()` exactly — no picker on this card.
pub fn redownload_backup(
    conn: &Connection,
    period_id: i64,
    output_path: &str,
) -> Result<ExportResult, AppError> {
    let rows = load_snapshot_export_rows(conn, period_id)?;
    if rows.is_empty() {
        return Err(AppError::NotFound {
            message: "No backup found for that period.".into(),
        });
    }
    let fixed_columns = [
        OptionalColumn::SlabPct,
        OptionalColumn::Rewards,
        OptionalColumn::RoyaltyEarned,
        OptionalColumn::ActiveStatus,
    ];
    write_export_xlsx(&rows, &fixed_columns, output_path)?;
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

    fn export_row(id: i64, name: &str, bv: i64, tbv: i64, slab_pct: i64, rewards: i64) -> MemberExportRow {
        MemberExportRow {
            id,
            name: name.into(),
            phone: "0".into(),
            email: None,
            address: "addr".into(),
            introducer_member_id: None,
            introducer_name: None,
            level: 1,
            leg_count: 0,
            is_active: true,
            joining_date: "2026-01-01".into(),
            business_volume: bv,
            total_business_volume: tbv,
            slab_pct,
            rewards,
            royalty: 0,
        }
    }

    #[test]
    fn sort_monthly_rows_sorts_by_the_chosen_field_and_direction() {
        let mut rows = vec![
            export_row(1, "Bob", 300, 300, 8, 50),
            export_row(2, "Alice", 100, 100, 12, 10),
        ];
        sort_monthly_rows(&mut rows, MonthlySortField::BusinessVolume, SortDirection::Asc);
        assert_eq!(rows.iter().map(|r| r.id).collect::<Vec<_>>(), vec![2, 1]);

        sort_monthly_rows(&mut rows, MonthlySortField::Rewards, SortDirection::Desc);
        assert_eq!(rows.iter().map(|r| r.id).collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn sort_monthly_rows_name_is_case_insensitive() {
        let mut rows = vec![export_row(1, "bob", 0, 0, 0, 0), export_row(2, "Alice", 0, 0, 0, 0)];
        sort_monthly_rows(&mut rows, MonthlySortField::Name, SortDirection::Asc);
        assert_eq!(rows.iter().map(|r| r.id).collect::<Vec<_>>(), vec![2, 1]);
    }

    #[test]
    fn sort_monthly_rows_ties_break_on_id() {
        let mut rows = vec![export_row(2, "Same", 50, 50, 0, 0), export_row(1, "Same", 50, 50, 0, 0)];
        sort_monthly_rows(&mut rows, MonthlySortField::BusinessVolume, SortDirection::Asc);
        assert_eq!(
            rows.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![1, 2],
            "equal business volume must still resolve to a stable, id-ordered result"
        );
    }

    fn yearly_row(id: i64, name: &str, avg_bv: f64, avg_tbv: f64) -> YearlyAverageRow {
        YearlyAverageRow {
            id,
            name: name.into(),
            phone: "0".into(),
            avg_business_volume: avg_bv,
            avg_total_business_volume: avg_tbv,
            period_count: 1,
        }
    }

    #[test]
    fn sort_yearly_rows_sorts_by_the_chosen_field_and_direction() {
        let mut rows = vec![yearly_row(1, "Bob", 300.0, 300.0), yearly_row(2, "Alice", 100.0, 100.0)];
        sort_yearly_rows(
            &mut rows,
            YearlySortField::AvgTotalBusinessVolume,
            SortDirection::Desc,
        );
        assert_eq!(rows.iter().map(|r| r.id).collect::<Vec<_>>(), vec![1, 2]);

        sort_yearly_rows(&mut rows, YearlySortField::Name, SortDirection::Asc);
        assert_eq!(rows.iter().map(|r| r.id).collect::<Vec<_>>(), vec![2, 1]);
    }

    #[test]
    fn export_monthly_writes_rows_in_the_requested_sort_order() {
        // The written .xlsx's cells can't be read back by this crate (see
        // other export tests' own comments) — assert on the same
        // sort_monthly_rows call export_monthly makes internally, applied to
        // the same rows load_live_export_rows produces for this period.
        let conn = seeded();
        let period = insert_period(&conn, "2026-08", "open");
        let low = insert_member(&conn, "Low BV", true, None);
        let high = insert_member(&conn, "High BV", true, None);
        insert_totals(&conn, low, period, 10_000, 10_000);
        insert_totals(&conn, high, period, 90_000, 90_000);
        let output_path = temp_output_path("monthly-sorted");

        export_monthly(
            &conn,
            ExportMonthlyInput {
                period_month: "2026-08".into(),
                optional_columns: vec![],
                sort_field: MonthlySortField::BusinessVolume,
                sort_direction: SortDirection::Desc,
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap();

        let mut rows = load_live_export_rows(&conn, period).unwrap();
        sort_monthly_rows(&mut rows, MonthlySortField::BusinessVolume, SortDirection::Desc);
        assert_eq!(rows[0].id, high, "highest Business Volume first");
        assert_eq!(rows[1].id, low);
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
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
    fn with_forced_active_status_adds_it_when_the_caller_never_selected_it() {
        // NFR-8/07-design-system.md's Colour-Plus-Label Rule: the inactive
        // row's tint is never allowed to appear with no text label, so this
        // must hold even when the picker's checkbox was left unticked.
        let result = with_forced_active_status(&[OptionalColumn::Email]);
        assert_eq!(
            result,
            vec![OptionalColumn::Email, OptionalColumn::ActiveStatus]
        );
    }

    #[test]
    fn with_forced_active_status_does_not_duplicate_an_explicit_selection() {
        let result =
            with_forced_active_status(&[OptionalColumn::ActiveStatus, OptionalColumn::Email]);
        assert_eq!(
            result,
            vec![OptionalColumn::ActiveStatus, OptionalColumn::Email]
        );
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
                sort_field: MonthlySortField::Name,
                sort_direction: SortDirection::Asc,
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
                sort_field: MonthlySortField::Name,
                sort_direction: SortDirection::Asc,
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
                sort_field: MonthlySortField::Name,
                sort_direction: SortDirection::Asc,
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
                sort_field: MonthlySortField::Name,
                sort_direction: SortDirection::Asc,
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn preview_monthly_data_returns_member_id_order_leaving_sort_to_the_caller() {
        // Sorting for this preview now happens client-side (reports.tsx) so
        // the operator's chosen field/direction can drive it without a
        // round-trip; this only pins the underlying order still being the
        // plain `ORDER BY m.id` the SQL itself uses.
        let conn = seeded();
        let period = insert_period(&conn, "2026-08", "open");
        let first = insert_member(&conn, "Zeta", true, None);
        let second = insert_member(&conn, "Alpha", true, None);
        insert_totals(&conn, first, period, 10_000, 10_000);
        insert_totals(&conn, second, period, 90_000, 90_000);

        let rows = preview_monthly_data(&conn, "2026-08").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, first, "member-id order, not by any BV field");
        assert_eq!(rows[1].id, second);
    }

    #[test]
    fn preview_monthly_data_reads_the_closed_periods_snapshot_not_zeroed_live_totals() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let member = insert_member(&conn, "Closed Month Member", true, None);
        insert_totals(&conn, member, period, 0, 0);
        insert_snapshot(&conn, member, period, 1, 75_000, 75_000, true);

        let rows = preview_monthly_data(&conn, "2026-05").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].total_business_volume, 75_000);
    }

    #[test]
    fn preview_monthly_data_refuses_an_unknown_month() {
        let conn = seeded();
        let err = preview_monthly_data(&conn, "2099-01").unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn preview_yearly_average_returns_member_id_order_leaving_sort_to_the_caller() {
        // Both the Yearly Average and Low-Contribution cards read this same
        // list and can pick different sorts client-side (reports.tsx) — this
        // only pins the underlying order still being plain member-id order.
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let first = insert_member(&conn, "Zeta", true, None);
        let second = insert_member(&conn, "Alpha", true, None);
        insert_snapshot(&conn, first, period, 1, 10_000, 10_000, true);
        insert_snapshot(&conn, second, period, 1, 90_000, 90_000, true);

        let rows = preview_yearly_average(&conn).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, first, "member-id order, not by any average");
        assert_eq!(rows[1].id, second);
    }

    #[test]
    fn preview_yearly_average_matches_compute_yearly_averages() {
        // Same divisor rule (Rule-23) as the export path — this preview is
        // just compute_yearly_averages reshaped, not a second calculation.
        let conn = seeded();
        let period = insert_period(&conn, "2026-04", "closed");
        let member = insert_member(&conn, "Late Joiner", true, None);
        insert_snapshot(&conn, member, period, 1, 90_000, 90_000, true);

        let rows = preview_yearly_average(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].period_count, 1,
            "late joiner divides by their own count"
        );
        assert_eq!(rows[0].avg_business_volume, 90_000.0);
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

        let result = export_yearly_average(
            &conn,
            &output_path.to_string_lossy(),
            YearlySortField::Name,
            SortDirection::Asc,
        )
        .unwrap();

        assert_eq!(result.file_path, output_path.to_string_lossy());
        assert!(output_path.exists());
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
    }

    #[test]
    fn export_low_contribution_filters_on_own_bv_not_total_bv() {
        // Rule-24: this member's own Business Volume average (50.00) is
        // below a 100.00 threshold, but their Total Business Volume
        // average (5,000.00, inflated by a downline) is nowhere near it.
        // Filtering on TBV instead would wrongly exclude them.
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let low_own_bv = insert_member(&conn, "Low Own BV, High TBV", true, None);
        insert_snapshot(&conn, low_own_bv, period, 1, 5_000, 500_000, true);
        let high_own_bv = insert_member(&conn, "High Own BV", true, None);
        insert_snapshot(&conn, high_own_bv, period, 1, 200_000, 200_000, true);
        let output_path = temp_output_path("low-contribution-own-bv");

        export_low_contribution(
            &conn,
            ExportLowContributionInput {
                threshold: Some(10_000), // 100.00
                sort_field: YearlySortField::Name,
                sort_direction: SortDirection::Asc,
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap();

        // The row-loading path is what actually decides inclusion — assert
        // it directly, since the written file's cells can't be read back
        // by this crate.
        let rows: Vec<YearlyAverageRow> = compute_yearly_averages(&conn)
            .unwrap()
            .into_iter()
            .filter(|r| r.avg_business_volume < 10_000.0)
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, low_own_bv);
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
    }

    #[test]
    fn export_low_contribution_defaults_to_the_settings_threshold() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        // Seeded low_contribution_threshold is 10_000 (100.00) — this
        // member's 50.00 average must be included with no threshold
        // override at all.
        let member = insert_member(&conn, "Below Default Threshold", true, None);
        insert_snapshot(&conn, member, period, 1, 5_000, 5_000, true);
        let output_path = temp_output_path("low-contribution-default-threshold");

        let threshold = low_contribution_threshold_setting(&conn).unwrap();
        assert_eq!(threshold, 10_000);

        export_low_contribution(
            &conn,
            ExportLowContributionInput {
                threshold: None,
                sort_field: YearlySortField::Name,
                sort_direction: SortDirection::Asc,
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap();
        assert!(output_path.exists());
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
    }

    #[test]
    fn export_low_contribution_writes_successfully_with_an_empty_result() {
        // T-M6.3-3: nobody below threshold is a valid outcome, not an error.
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let member = insert_member(&conn, "Well Above Threshold", true, None);
        insert_snapshot(&conn, member, period, 1, 1_000_000, 1_000_000, true);
        let output_path = temp_output_path("low-contribution-empty");

        let result = export_low_contribution(
            &conn,
            ExportLowContributionInput {
                threshold: Some(100),
                sort_field: YearlySortField::Name,
                sort_direction: SortDirection::Asc,
                output_path: output_path.to_string_lossy().into_owned(),
            },
        )
        .unwrap();

        assert_eq!(result.file_path, output_path.to_string_lossy());
        assert!(output_path.exists());
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
    }

    #[test]
    fn list_backups_reports_the_latest_version_and_marks_a_correction() {
        let conn = seeded();
        let corrected = insert_period(&conn, "2026-04", "closed");
        let plain = insert_period(&conn, "2026-05", "closed");
        let member = insert_member(&conn, "Someone", true, None);
        insert_snapshot(&conn, member, corrected, 1, 100_000, 100_000, true);
        insert_snapshot(&conn, member, corrected, 2, 150_000, 150_000, true);
        insert_snapshot(&conn, member, plain, 1, 50_000, 50_000, true);

        let backups = list_backups(&conn).unwrap();
        assert_eq!(backups.len(), 2);
        assert_eq!(backups[0].period_month, "2026-05", "newest first");

        let corrected_row = backups.iter().find(|b| b.period_id == corrected).unwrap();
        assert_eq!(corrected_row.latest_version, 2);
        assert!(corrected_row.is_corrected);

        let plain_row = backups.iter().find(|b| b.period_id == plain).unwrap();
        assert_eq!(plain_row.latest_version, 1);
        assert!(!plain_row.is_corrected);
    }

    #[test]
    fn list_backups_excludes_an_empty_month_close() {
        // T-M5.4-2: an empty-month close has zero monthly_snapshots rows —
        // it must not appear in the closed-month re-download list at all.
        let conn = seeded();
        insert_period(&conn, "2026-05", "closed");
        let backups = list_backups(&conn).unwrap();
        assert!(backups.is_empty());
    }

    #[test]
    fn list_backups_excludes_a_period_that_is_not_yet_closed() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08", "awaiting_close");
        let member = insert_member(&conn, "Someone", true, None);
        insert_snapshot(&conn, member, period, 1, 10_000, 10_000, true);
        let backups = list_backups(&conn).unwrap();
        assert!(backups.is_empty(), "a snapshot only exists on closed periods in real use, but this still shouldn't list a non-closed one");
    }

    #[test]
    fn redownload_backup_always_reads_the_latest_version() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let member = insert_member(&conn, "Corrected", true, None);
        insert_snapshot(&conn, member, period, 1, 100_000, 100_000, true);
        insert_snapshot(&conn, member, period, 2, 999_000, 999_000, true);
        let output_path = temp_output_path("redownload-corrected");

        let result = redownload_backup(&conn, period, &output_path.to_string_lossy()).unwrap();

        assert_eq!(result.file_path, output_path.to_string_lossy());
        assert!(output_path.exists());
        let rows = load_snapshot_export_rows(&conn, period).unwrap();
        assert_eq!(
            rows[0].business_volume, 999_000,
            "always the latest version"
        );
        std::fs::remove_dir_all(output_path.parent().unwrap()).ok();
    }

    #[test]
    fn redownload_backup_refuses_a_period_with_no_snapshot() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-05", "closed");
        let output_path = temp_output_path("redownload-empty");
        let err = redownload_backup(&conn, period, &output_path.to_string_lossy()).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }
}
