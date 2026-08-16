// M4 — Search & Chart (04-technical-architecture.md §3.1). `search_members`
// itself shipped inside `m1_members` (S5, US-M1.4); this module holds the
// two commands actually named for this epic — US-M4.1 (`get_member_detail`,
// API-10) and US-M4.2/US-M4.3 (`get_direct_children_chart`, API-11). Both
// are read-only: no audit entry, no recalculation, no write.
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use crate::error::AppError;
use crate::m1_members::Member;
use crate::m3_calc::engine::round_half_up_div100;
use crate::m5_close::get_period_lock_status;

pub mod pdf;

/// T-M2.5-3: figure screens default to the **oldest recordable** period,
/// never "whatever period_id happens to be highest." Before US-M2.5 every
/// query here picked its period independently per member (`ORDER BY
/// period_id DESC LIMIT 1` against that member's own rows), which is wrong
/// once CR-2 allows two periods to be simultaneously open/awaiting_close —
/// two members touched at different times could each resolve to a
/// different period, and the screen would show a blend of two months at
/// once. Resolving a single `period_id` up front and threading it through
/// every query is what makes the screen show one consistent month.
pub fn resolve_view_period_id(
    conn: &Connection,
    period_month: Option<&str>,
) -> Result<i64, AppError> {
    match period_month {
        // An explicit month names a real switcher option, so a miss here
        // is a genuine caller error worth surfacing.
        Some(month) => conn
            .query_row(
                "SELECT id FROM periods WHERE period_month = ?1",
                [month],
                |r| r.get(0),
            )
            .optional()?
            .ok_or_else(|| AppError::NotFound {
                message: "Period not found.".into(),
            }),
        // No month named: default to the oldest recordable one. Its
        // `periods` row may not exist yet on a fresh install or in a test
        // fixture that never ran the login catch-up (US-M5.5) — that's not
        // an error, it just means nothing has happened yet. `0` never
        // matches a real `period_id` (SQLite's INTEGER PRIMARY KEY starts
        // at 1), so every downstream LEFT JOIN against it degrades to the
        // same all-zero COALESCE result this code already produced before
        // period-awareness existed.
        None => {
            let status = get_period_lock_status(conn)?;
            let month = status
                .recordable_period_months
                .first()
                .cloned()
                .expect("get_period_lock_status always names at least the current month");
            Ok(conn
                .query_row(
                    "SELECT id FROM periods WHERE period_month = ?1",
                    [&month],
                    |r| r.get(0),
                )
                .optional()?
                .unwrap_or(0))
        }
    }
}

/// A member's `member_period_totals` row for one explicit period — the
/// same "read whatever's there, COALESCE to 0" convention `search_members`
/// already established (S5/S7), just scoped to a caller-resolved
/// `period_id` (`resolve_view_period_id`) rather than each member picking
/// their own latest row independently.
struct PeriodTotals {
    business_volume: i64,
    total_business_volume: i64,
    slab_pct: i64,
    royalty: i64,
    own_reward: i64,
    rewards: i64,
}

fn totals_for_period(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<PeriodTotals, AppError> {
    Ok(conn.query_row(
        "SELECT COALESCE(t.business_volume, 0), COALESCE(t.total_business_volume, 0),
                COALESCE(t.slab_pct, 0),
                COALESCE(t.royalty, 0), COALESCE(t.own_reward, 0), COALESCE(t.rewards, 0)
         FROM (SELECT 1) dummy
         LEFT JOIN member_period_totals t
                ON t.member_id = ?1 AND t.period_id = ?2",
        rusqlite::params![member_id, period_id],
        |r| {
            Ok(PeriodTotals {
                business_volume: r.get(0)?,
                total_business_volume: r.get(1)?,
                slab_pct: r.get(2)?,
                royalty: r.get(3)?,
                own_reward: r.get(4)?,
                rewards: r.get(5)?,
            })
        },
    )?)
}

fn member_exists(conn: &Connection, member_id: i64) -> Result<bool, AppError> {
    Ok(conn
        .query_row("SELECT 1 FROM members WHERE id = ?1", [member_id], |_| {
            Ok(())
        })
        .optional()?
        .is_some())
}

fn top_slab_percentage(conn: &Connection) -> Result<i64, AppError> {
    Ok(conn.query_row(
        "SELECT COALESCE(MAX(percentage), 0) FROM slab_table",
        [],
        |r| r.get(0),
    )?)
}

fn royalty_rate_percent(conn: &Connection) -> Result<i64, AppError> {
    let value: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'royalty_rate_percent'",
        [],
        |r| r.get(0),
    )?;
    value.parse().map_err(|_| AppError::Validation {
        field: "royalty_rate_percent".into(),
        message: "setting 'royalty_rate_percent' is not a valid integer".into(),
    })
}

// ---------------------------------------------------------------------
// API-10 — get_member_detail (US-M4.1)
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnRewardLine {
    pub own_business_volume: i64,
    pub own_slab_pct: i64,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DifferentialLine {
    pub child_id: i64,
    pub child_name: String,
    pub child_total_business_volume: i64,
    pub child_slab_pct: i64,
    pub own_slab_pct: i64,
    pub differential_pct: i64,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoyaltyLine {
    pub qualifying_children: i64,
    pub rate_percent: i64,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardBreakdown {
    pub own_reward: OwnRewardLine,
    pub differentials: Vec<DifferentialLine>,
    pub royalty: Option<RoyaltyLine>,
    pub rewards_total: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberDetailChild {
    pub member_id: i64,
    pub name: String,
    pub total_business_volume: i64,
    pub slab_pct: i64,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberDetail {
    pub member: Member,
    pub total_business_volume: i64,
    pub slab_pct: i64,
    pub leg_count: i64,
    pub rewards: RewardBreakdown,
    pub direct_children: Vec<MemberDetailChild>,
}

fn direct_children_for_detail(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<Vec<MemberDetailChild>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.name, m.is_active,
                COALESCE(t.total_business_volume, 0), COALESCE(t.slab_pct, 0)
         FROM members m
         LEFT JOIN member_period_totals t
                ON t.member_id = m.id AND t.period_id = ?2
         WHERE m.introducer_member_id = ?1
         ORDER BY m.id",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![member_id, period_id], |r| {
            Ok(MemberDetailChild {
                member_id: r.get(0)?,
                name: r.get(1)?,
                is_active: r.get(2)?,
                total_business_volume: r.get(3)?,
                slab_pct: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// API-10 (T-M4.1-1/1-2). Own-Business-Volume reward first (Rule-46/CR-4),
/// then every direct leg's differential term (all of them, not just the
/// nonzero ones — Rule-8 re-scans every child), then a royalty summary line
/// when the member has at least one leg (Rule-10/Rule-25). Rule-11's note —
/// differential and royalty never both pay on the same leg — falls out of
/// the formulas themselves; nothing here special-cases a leg out of one
/// term because it qualified for the other.
pub fn get_member_detail(
    conn: &Connection,
    member_id: i64,
    period_month: Option<&str>,
) -> Result<MemberDetail, AppError> {
    if !member_exists(conn, member_id)? {
        return Err(AppError::NotFound {
            message: "Member not found.".into(),
        });
    }
    let period_id = resolve_view_period_id(conn, period_month)?;
    let member = conn.query_row("SELECT * FROM members WHERE id = ?1", [member_id], |r| {
        Member::from_row(r)
    })?;
    let totals = totals_for_period(conn, member_id, period_id)?;
    let children = direct_children_for_detail(conn, member_id, period_id)?;

    let differentials: Vec<DifferentialLine> = children
        .iter()
        .map(|c| {
            let differential_pct = totals.slab_pct - c.slab_pct;
            DifferentialLine {
                child_id: c.member_id,
                child_name: c.name.clone(),
                child_total_business_volume: c.total_business_volume,
                child_slab_pct: c.slab_pct,
                own_slab_pct: totals.slab_pct,
                differential_pct,
                amount: round_half_up_div100(differential_pct * c.total_business_volume),
            }
        })
        .collect();

    let royalty = if children.is_empty() {
        None
    } else {
        let top_slab = top_slab_percentage(conn)?;
        let qualifying_children = children.iter().filter(|c| c.slab_pct == top_slab).count() as i64;
        Some(RoyaltyLine {
            qualifying_children,
            rate_percent: royalty_rate_percent(conn)?,
            amount: totals.royalty,
        })
    };

    Ok(MemberDetail {
        total_business_volume: totals.total_business_volume,
        slab_pct: totals.slab_pct,
        leg_count: children.len() as i64,
        rewards: RewardBreakdown {
            own_reward: OwnRewardLine {
                own_business_volume: totals.business_volume,
                own_slab_pct: totals.slab_pct,
                amount: totals.own_reward,
            },
            differentials,
            royalty,
            rewards_total: totals.rewards,
        },
        direct_children: children,
        member,
    })
}

/// API-46 (CR-6, M4.8). Reuses `get_member_detail` unchanged — no new
/// calculation logic, same period resolution, same reward breakdown. The
/// period label shown on the document comes from whatever `period_month`
/// resolved to; `generated_at` is wall-clock time at export, not a stored
/// value (this document is a point-in-time snapshot, same spirit as the
/// full hierarchy window's own timestamp, Rule-45).
pub fn export_member_detail_pdf(
    conn: &Connection,
    member_id: i64,
    period_month: Option<&str>,
    output_path: &str,
) -> Result<crate::m6_reports::ExportResult, AppError> {
    let detail = get_member_detail(conn, member_id, period_month)?;
    // Mirrors resolve_view_period_id's own None-branch (used by
    // get_member_detail above) rather than looking the label back up by
    // `period_id` — that id can be the sentinel `0` when no `periods` row
    // exists yet (fresh install, or a test fixture with no activity), and
    // a lookup against a nonexistent id would fail where get_member_detail
    // itself does not.
    let period_label: String = match period_month {
        Some(month) => month.to_string(),
        None => {
            let status = get_period_lock_status(conn)?;
            status
                .recordable_period_months
                .first()
                .cloned()
                .expect("get_period_lock_status always names at least the current month")
        }
    };
    let generated_at = chrono::Local::now().format("%d %b %Y %H:%M").to_string();

    pdf::render_member_detail_pdf(&detail, &period_label, &generated_at, output_path)?;

    Ok(crate::m6_reports::ExportResult {
        file_path: output_path.to_string(),
    })
}

// ---------------------------------------------------------------------
// API-11 — get_direct_children_chart (US-M4.2 / US-M4.3)
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartNode {
    pub member_id: i64,
    pub name: String,
    pub own_business_volume: i64,
    pub is_active: bool,
    pub introducer_member_id: Option<i64>,
    pub slab_pct: i64,
    pub rewards: i64,
    /// Direct-child count. Needed so the Structure screen can render a
    /// node's leaf/expandable affordance (`StructureTreeNode`'s `legCount`)
    /// without first fetching that node's own children — the lazy,
    /// one-generation-per-fetch loading `full_tree: false` implies.
    pub leg_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlabTableRow {
    pub id: i64,
    pub threshold: i64,
    pub percentage: i64,
    pub sort_order: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectChildrenChartResult {
    /// The requested member first, then its descendants — bounded to one
    /// generation when `full_tree` is false (T-M4.2-1), unbounded when true
    /// (T-M4.3-1: same command, no new one; the field was always in the
    /// contract). US-M4.4's Home charts are what calls this with
    /// `full_tree: true` this sprint, ahead of the Full Hierarchy Window
    /// screen itself (US-M4.3, S9) which will reuse this same result.
    pub nodes: Vec<ChartNode>,
    /// The resolved Sprint 8 gap: `get_settings` (US-M7.1) isn't built
    /// until S10, so this is the only IPC path that can hand the frontend
    /// the configured slab rows — needed so Home's slab-distribution
    /// charts can draw one bar per row, including zero-count slabs.
    pub slab_table: Vec<SlabTableRow>,
}

fn slab_table_rows(conn: &Connection) -> Result<Vec<SlabTableRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, threshold, percentage, sort_order FROM slab_table ORDER BY sort_order",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SlabTableRow {
                id: r.get(0)?,
                threshold: r.get(1)?,
                percentage: r.get(2)?,
                sort_order: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// API-11. `max_depth` caps the recursive walk at 1 (member + direct
/// children) when `full_tree` is false, or effectively unbounded when
/// true — one query shape either way rather than two code paths.
fn chart_nodes(
    conn: &Connection,
    member_id: i64,
    max_depth: i64,
    period_id: i64,
) -> Result<Vec<ChartNode>, AppError> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE subtree(id, depth) AS (
            SELECT ?1, 0
            UNION ALL
            SELECT m.id, subtree.depth + 1
            FROM members m
            JOIN subtree ON m.introducer_member_id = subtree.id
            WHERE subtree.depth < ?2
         )
         SELECT m.id, m.name, m.is_active, m.introducer_member_id,
                COALESCE(t.business_volume, 0), COALESCE(t.slab_pct, 0), COALESCE(t.rewards, 0),
                (SELECT COUNT(*) FROM members c WHERE c.introducer_member_id = m.id)
         FROM subtree
         JOIN members m ON m.id = subtree.id
         LEFT JOIN member_period_totals t
                ON t.member_id = m.id AND t.period_id = ?3
         ORDER BY subtree.depth, m.id",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![member_id, max_depth, period_id], |r| {
            Ok(ChartNode {
                member_id: r.get(0)?,
                name: r.get(1)?,
                is_active: r.get(2)?,
                introducer_member_id: r.get(3)?,
                own_business_volume: r.get(4)?,
                slab_pct: r.get(5)?,
                rewards: r.get(6)?,
                leg_count: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------
// API-42 — get_ancestor_chain (Structure screen's breadcrumb trail)
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AncestorNode {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AncestorChainResult {
    /// Root-first, the requested member last — ancestorTrail()'s ordering
    /// in the prototype (ui-prototype-v2.html:626-630).
    pub chain: Vec<AncestorNode>,
}

fn introducer_of(conn: &Connection, member_id: i64) -> Result<Option<i64>, AppError> {
    conn.query_row(
        "SELECT introducer_member_id FROM members WHERE id = ?1",
        [member_id],
        |r| r.get(0),
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "Member not found.".into(),
    })
}

/// Leaf(the requested member)-to-root walk — same upward-loop idiom as
/// `m3_calc::chain_to_root`, duplicated locally rather than shared across
/// the module boundary (this module already keeps its own small
/// single-row-lookup helpers, e.g. `member_exists`, rather than reaching
/// into `m3_calc`'s private ones).
fn ancestor_chain_ids(conn: &Connection, member_id: i64) -> Result<Vec<i64>, AppError> {
    let mut chain = vec![member_id];
    let mut current = member_id;
    while let Some(parent) = introducer_of(conn, current)? {
        chain.push(parent);
        current = parent;
    }
    Ok(chain)
}

/// API-42. Cost scales with chain *depth* (indexed primary-key point
/// lookups, in-process SQLite), not with total member count — see the
/// design spec §2 for the worst-case analysis.
pub fn get_ancestor_chain(
    conn: &Connection,
    member_id: i64,
) -> Result<AncestorChainResult, AppError> {
    let ids = ancestor_chain_ids(conn, member_id)?;
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("SELECT id, name FROM members WHERE id IN ({placeholders})");
    let mut stmt = conn.prepare(&sql)?;
    let mut by_id: std::collections::HashMap<i64, String> = stmt
        .query_map(rusqlite::params_from_iter(ids.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<Result<_, _>>()?;
    let chain = ids
        .into_iter()
        .rev()
        .map(|id| AncestorNode {
            name: by_id
                .remove(&id)
                .expect("id came from the members table, so a row must exist"),
            id,
        })
        .collect();
    Ok(AncestorChainResult { chain })
}

fn root_member_id(conn: &Connection) -> Result<i64, AppError> {
    conn.query_row(
        "SELECT id FROM members WHERE introducer_member_id IS NULL LIMIT 1",
        [],
        |r| r.get(0),
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "No root member exists yet.".into(),
    })
}

/// `member_id: None` resolves to the root member (there is always at most
/// one — `create_root_member` refuses a second, AC-7). Needed by both
/// callers this sprint that have no member already in hand: the Structure
/// screen's default `/structure` route, and Home's whole-population fetch
/// for its slab-distribution charts (US-M4.4).
pub fn get_direct_children_chart(
    conn: &Connection,
    member_id: Option<i64>,
    full_tree: bool,
    period_month: Option<&str>,
) -> Result<DirectChildrenChartResult, AppError> {
    let member_id = match member_id {
        Some(id) => id,
        None => root_member_id(conn)?,
    };
    if !member_exists(conn, member_id)? {
        return Err(AppError::NotFound {
            message: "Member not found.".into(),
        });
    }
    let period_id = resolve_view_period_id(conn, period_month)?;
    let max_depth = if full_tree { i64::MAX } else { 1 };
    Ok(DirectChildrenChartResult {
        nodes: chart_nodes(conn, member_id, max_depth, period_id)?,
        slab_table: slab_table_rows(conn)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::m3_calc::recalculate_chain;

    fn seeded() -> Connection {
        db::open_seeded_in_memory().unwrap()
    }

    #[test]
    fn get_ancestor_chain_is_root_first_and_includes_the_member_itself() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let child = insert_member(&conn, "Child", Some(root));
        let grandchild = insert_member(&conn, "Grandchild", Some(child));

        let result = get_ancestor_chain(&conn, grandchild).unwrap();
        let names: Vec<&str> = result.chain.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, vec!["Root", "Child", "Grandchild"]);
        assert_eq!(result.chain.last().unwrap().id, grandchild);
    }

    #[test]
    fn get_ancestor_chain_for_the_root_member_is_a_single_entry() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let result = get_ancestor_chain(&conn, root).unwrap();
        assert_eq!(result.chain.len(), 1);
        assert_eq!(result.chain[0].id, root);
    }

    #[test]
    fn get_ancestor_chain_refuses_an_unknown_member() {
        let conn = seeded();
        let err = get_ancestor_chain(&conn, 999_999).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    // Rule-32: exceeding the configured max depth only warns, it never
    // blocks onboarding — the chain walk must not assume any bound.
    #[test]
    fn get_ancestor_chain_handles_a_chain_deeper_than_the_advisory_max_depth() {
        let conn = seeded();
        let mut parent = insert_member(&conn, "L0", None);
        for i in 1..=30 {
            parent = insert_member(&conn, &format!("L{i}"), Some(parent));
        }
        let result = get_ancestor_chain(&conn, parent).unwrap();
        assert_eq!(result.chain.len(), 31);
        assert_eq!(result.chain[0].name, "L0");
        assert_eq!(result.chain.last().unwrap().name, "L30");
    }

    fn insert_period(conn: &Connection, month: &str) -> i64 {
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
            [month],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_member(conn: &Connection, name: &str, introducer: Option<i64>) -> i64 {
        static NEXT_PHONE: std::sync::atomic::AtomicI64 =
            std::sync::atomic::AtomicI64::new(9_100_000_000);
        let phone = NEXT_PHONE
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string();
        conn.execute(
            "INSERT INTO members
                (name, phone, address, introducer_member_id, level, is_active,
                 joining_date, consent_given, consent_date, created_at)
             VALUES (?1, ?2, 'addr', ?3, 1, 1, '2026-01-01', 1, '2026-01-01', '2026-01-01')",
            rusqlite::params![name, phone, introducer],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_entry(conn: &Connection, member_id: i64, month: &str, amount: i64) {
        conn.execute(
            "INSERT INTO business_volume_entries
                (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                member_id,
                amount,
                format!("{month}-15"),
                month,
                format!("{month}-15")
            ],
        )
        .unwrap();
    }

    #[test]
    fn get_member_detail_refuses_an_unknown_member() {
        let conn = seeded();
        let err = get_member_detail(&conn, 999_999, None).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn get_member_detail_defaults_to_zero_with_no_activity_yet() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let detail = get_member_detail(&conn, root, None).unwrap();
        assert_eq!(detail.total_business_volume, 0);
        assert_eq!(detail.leg_count, 0);
        assert_eq!(detail.rewards.rewards_total, 0);
        assert!(detail.rewards.royalty.is_none(), "no legs, no royalty line");
        assert!(detail.rewards.differentials.is_empty());
    }

    // Rule-8's own worked example: D at 6% (own BV 500), children A (2%,
    // 300), B (0%, 50), C (4%, 1000) -> differential 35. Reused here to
    // prove get_member_detail's per-child breakdown reproduces the same
    // per-leg amounts the engine computed, not just the aggregate total.
    #[test]
    fn differential_breakdown_reproduces_rule_8s_worked_example() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let d = insert_member(&conn, "D", None);
        let a = insert_member(&conn, "A", Some(d));
        let b = insert_member(&conn, "B", Some(d));
        let c = insert_member(&conn, "C", Some(d));
        insert_entry(&conn, d, "2026-08", 50_000); // ×100: 500.00
        insert_entry(&conn, a, "2026-08", 30_000); // ×100: 300.00 -> 2%
        insert_entry(&conn, b, "2026-08", 5_000); // ×100: 50.00 -> 0%
        insert_entry(&conn, c, "2026-08", 100_000); // ×100: 1,000.00 -> 4%
        for m in [a, b, c, d] {
            recalculate_chain(&conn, m, period).unwrap();
        }

        // Explicit month, not the `None` default: the default resolves via
        // `get_period_lock_status`'s real-calendar-month fallback, which
        // this fixture's hardcoded "2026-08" period won't match once the
        // real date moves on — the test's premise is reading back this
        // specific period, regardless of what day it happens to run.
        let detail = get_member_detail(&conn, d, Some("2026-08")).unwrap();
        assert_eq!(detail.slab_pct, 6);
        assert_eq!(detail.leg_count, 3);
        let by_child: std::collections::HashMap<i64, i64> = detail
            .rewards
            .differentials
            .iter()
            .map(|line| (line.child_id, line.amount))
            .collect();
        assert_eq!(by_child.len(), 3);
        let total: i64 = by_child.values().sum();
        assert_eq!(total, 35_00, "sums to Rule-8's ×100 differential of 35.00");

        // Rule-12/13: the stored aggregate must equal the sum of the three
        // displayed lines — no term is silently dropped by the breakdown.
        let reconstructed = detail.rewards.own_reward.amount
            + detail
                .rewards
                .differentials
                .iter()
                .map(|l| l.amount)
                .sum::<i64>()
            + detail
                .rewards
                .royalty
                .as_ref()
                .map(|r| r.amount)
                .unwrap_or(0);
        assert_eq!(detail.rewards.rewards_total, reconstructed);
    }

    #[test]
    fn direct_children_chart_bounds_to_one_generation_by_default() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let child = insert_member(&conn, "Child", Some(root));
        let _grandchild = insert_member(&conn, "Grandchild", Some(child));

        let result = get_direct_children_chart(&conn, Some(root), false, None).unwrap();
        let ids: Vec<i64> = result.nodes.iter().map(|n| n.member_id).collect();
        assert_eq!(ids, vec![root, child], "root + direct children only");
        assert_eq!(result.nodes[0].leg_count, 1, "root has one direct leg");
        assert_eq!(
            result.nodes[1].leg_count, 1,
            "child's own leg count (the grandchild) is still reported, even though the grandchild itself isn't returned at depth 1"
        );
    }

    #[test]
    fn full_tree_reaches_every_descendant() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let child = insert_member(&conn, "Child", Some(root));
        let grandchild = insert_member(&conn, "Grandchild", Some(child));

        let result = get_direct_children_chart(&conn, Some(root), true, None).unwrap();
        let ids: std::collections::HashSet<i64> =
            result.nodes.iter().map(|n| n.member_id).collect();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&root) && ids.contains(&child) && ids.contains(&grandchild));
    }

    #[test]
    fn chart_result_carries_slab_table_ordered_by_sort_order() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let result = get_direct_children_chart(&conn, Some(root), false, None).unwrap();
        assert_eq!(result.slab_table.len(), 7, "the 7 default seeded rows");
        let orders: Vec<i64> = result.slab_table.iter().map(|s| s.sort_order).collect();
        let mut sorted = orders.clone();
        sorted.sort();
        assert_eq!(orders, sorted);
    }

    #[test]
    fn get_direct_children_chart_refuses_an_unknown_member() {
        let conn = seeded();
        let err = get_direct_children_chart(&conn, Some(999_999), false, None).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn get_direct_children_chart_resolves_the_root_when_no_member_id_is_given() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let child = insert_member(&conn, "Child", Some(root));

        let result = get_direct_children_chart(&conn, None, false, None).unwrap();
        let ids: Vec<i64> = result.nodes.iter().map(|n| n.member_id).collect();
        assert_eq!(ids, vec![root, child]);
    }

    #[test]
    fn view_period_defaults_to_the_oldest_outstanding_month_not_the_highest_period_id() {
        // T-M2.5-3 (Gap 2): once CR-2 allows two periods to sit
        // open/awaiting_close simultaneously, the figure screens must
        // default to the OLDER, still-outstanding one — never "whichever
        // period row happens to have the highest id," which before US-M2.5
        // is what every query here picked instead.
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let older = insert_period(&conn, "2026-06");
        conn.execute(
            "UPDATE periods SET status = 'awaiting_close' WHERE id = ?1",
            [older],
        )
        .unwrap();
        let newer = insert_period(&conn, "2026-07"); // higher period_id, status stays 'open'
        insert_entry(&conn, root, "2026-06", 10_000);
        insert_entry(&conn, root, "2026-07", 99_000);
        recalculate_chain(&conn, root, older).unwrap();
        recalculate_chain(&conn, root, newer).unwrap();

        let default_view = get_member_detail(&conn, root, None).unwrap();
        assert_eq!(
            default_view.total_business_volume, 10_000,
            "defaults to the older, outstanding month, not the newer higher-id one"
        );

        let explicit_newer = get_member_detail(&conn, root, Some("2026-07")).unwrap();
        assert_eq!(explicit_newer.total_business_volume, 99_000);

        let default_chart = get_direct_children_chart(&conn, Some(root), false, None).unwrap();
        assert_eq!(default_chart.nodes[0].own_business_volume, 10_000);
    }

    #[test]
    fn get_direct_children_chart_refuses_when_no_root_exists_yet() {
        let conn = seeded();
        let err = get_direct_children_chart(&conn, None, false, None).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn export_member_detail_pdf_writes_a_real_file() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let dir =
            std::env::temp_dir().join(format!("bvconsole-export-pdf-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let output_path = dir.join("member.pdf");

        let result =
            export_member_detail_pdf(&conn, root, None, output_path.to_str().unwrap()).unwrap();

        assert_eq!(result.file_path, output_path.to_string_lossy());
        assert!(output_path.exists());
        let bytes = std::fs::read(&output_path).unwrap();
        assert!(bytes.starts_with(b"%PDF"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn export_member_detail_pdf_refuses_an_unknown_member() {
        let conn = seeded();
        let result = export_member_detail_pdf(&conn, 999_999, None, "unused.pdf");
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }
}
