// M3 — Calculation Engine (04-technical-architecture.md §3.2, §5, ADR-005).
// US-M3.1's pure core lives in `engine`; everything below is US-M3.2 — the
// only DB-touching part of this module, and the practical implementation
// of Rule-5's bottom-up order: walk the changed member's chain to the
// root, re-deriving each ancestor from its already-correct children.
//
// No Tauri command triggers this (Rule-26 — there is no "recalculate"
// button, so there is no command surface that could become one). The
// caller is M2's `record_entry`/`edit_entry` (S7) and M5's correction path
// (S12); this sprint builds the engine those will call, not the calling
// commands themselves.
pub mod engine;

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use engine::{compute_node, ChildFigures, NodeFigures};

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

/// Rule-5's path from the changed member to the root, member first — T-M3.2-1.
fn chain_to_root(conn: &Connection, member_id: i64) -> Result<Vec<i64>, AppError> {
    let mut chain = vec![member_id];
    let mut current = member_id;
    while let Some(parent) = introducer_of(conn, current)? {
        chain.push(parent);
        current = parent;
    }
    Ok(chain)
}

fn period_month_of(conn: &Connection, period_id: i64) -> Result<String, AppError> {
    conn.query_row(
        "SELECT period_month FROM periods WHERE id = ?1",
        [period_id],
        |r| r.get(0),
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "Period not found.".into(),
    })
}

/// Step 1 of the algorithm (§5.1): a member's own Business Volume for the
/// period is always re-summed from entries, never cached.
fn business_volume_of(
    conn: &Connection,
    member_id: i64,
    period_month: &str,
) -> Result<i64, AppError> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM business_volume_entries
         WHERE member_id = ?1 AND period_month = ?2",
        rusqlite::params![member_id, period_month],
        |r| r.get(0),
    )?)
}

/// Rule-28: every direct child, active or not — no filtering here. A child
/// with no row yet for this period (nothing in its subtree has been
/// entered this period) defaults to zero, which is its correct TBV.
fn direct_children_figures(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<Vec<ChildFigures>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(t.total_business_volume, 0), COALESCE(t.slab_pct, 0)
         FROM members m
         LEFT JOIN member_period_totals t ON t.member_id = m.id AND t.period_id = ?2
         WHERE m.introducer_member_id = ?1",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![member_id, period_id], |r| {
            Ok(ChildFigures {
                total_business_volume: r.get(0)?,
                slab_pct: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn slab_table(conn: &Connection) -> Result<Vec<(i64, i64)>, AppError> {
    let mut stmt = conn.prepare("SELECT threshold, percentage FROM slab_table")?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn setting_i64(conn: &Connection, key: &str) -> Result<i64, AppError> {
    let value: String =
        conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get(0)
        })?;
    value.parse().map_err(|_| AppError::Validation {
        field: key.into(),
        message: format!("setting '{key}' is not a valid integer"),
    })
}

fn upsert_totals(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
    business_volume: i64,
    figures: &NodeFigures,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO member_period_totals
            (member_id, period_id, business_volume, total_business_volume, slab_pct,
             differential, royalty, own_reward, rewards)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT (member_id, period_id) DO UPDATE SET
            business_volume = excluded.business_volume,
            total_business_volume = excluded.total_business_volume,
            slab_pct = excluded.slab_pct,
            differential = excluded.differential,
            royalty = excluded.royalty,
            own_reward = excluded.own_reward,
            rewards = excluded.rewards",
        rusqlite::params![
            member_id,
            period_id,
            business_volume,
            figures.total_business_volume,
            figures.slab_pct,
            figures.differential,
            figures.royalty,
            figures.own_reward,
            figures.rewards,
        ],
    )?;
    Ok(())
}

fn walk_chain(
    conn: &Connection,
    chain: &[i64],
    period_month: &str,
    period_id: i64,
    slabs: &[(i64, i64)],
    royalty_min_children: i64,
    royalty_rate_percent: i64,
) -> Result<(), AppError> {
    for &id in chain {
        let business_volume = business_volume_of(conn, id, period_month)?;
        let children = direct_children_figures(conn, id, period_id)?;
        let figures = compute_node(
            business_volume,
            &children,
            slabs,
            royalty_min_children,
            royalty_rate_percent,
        );
        upsert_totals(conn, id, period_id, business_volume, &figures)?;
    }
    Ok(())
}

/// ADR-005: on a Business Volume write against `member_id` within
/// `period_id`, recompute only the chain from that member to the root —
/// never the full tree (T-M3.2-1). Each node is written before its parent
/// is computed, so the parent's read of that one changed child is already
/// fresh; every other child is read as its already-correct cached figure
/// from a prior write (T-M3.2-2 — still re-scanned in full at each
/// ancestor, since the ancestor's own slab may have moved). Confined to
/// `period_id` throughout (T-M3.2-4) — a member can hold rows for more
/// than one not-yet-closed period, and this never reads or writes any
/// other one.
///
/// The whole walk is atomic (T-M3.2-3). API-08 requires "insert entry +
/// recalc" to be *one* transaction, which means the future caller (M2's
/// `record_entry`/`edit_entry`, S7; M5's correction path, S12) opens that
/// transaction and calls this from inside it — so this must never nest a
/// second `BEGIN` of its own (SQLite refuses that outright). Called
/// standalone, with no transaction already open, it owns one itself so
/// the walk is still atomic on its own.
pub fn recalculate_chain(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<(), AppError> {
    let chain = chain_to_root(conn, member_id)?;
    let period_month = period_month_of(conn, period_id)?;
    let slabs = slab_table(conn)?;
    let royalty_min_children = setting_i64(conn, "royalty_qualifying_count")?;
    let royalty_rate_percent = setting_i64(conn, "royalty_rate_percent")?;

    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        walk_chain(
            &tx,
            &chain,
            &period_month,
            period_id,
            &slabs,
            royalty_min_children,
            royalty_rate_percent,
        )?;
        tx.commit()?;
    } else {
        walk_chain(
            conn,
            &chain,
            &period_month,
            period_id,
            &slabs,
            royalty_min_children,
            royalty_rate_percent,
        )?;
    }
    Ok(())
}

/// `pub(crate)`: `m5_close::manual_backup_current_period` (API-15) reuses
/// this rather than re-deriving "the in-progress month" a second way.
pub(crate) fn current_open_period_id(conn: &Connection) -> Result<Option<i64>, AppError> {
    let period_month = chrono::Local::now().format("%Y-%m").to_string();
    Ok(conn
        .query_row(
            "SELECT id FROM periods WHERE period_month = ?1 AND status = 'open'",
            [period_month],
            |r| r.get(0),
        )
        .optional()?)
}

fn open_period_member_ids(conn: &Connection, period_id: i64) -> Result<Vec<i64>, AppError> {
    let mut stmt =
        conn.prepare("SELECT member_id FROM member_period_totals WHERE period_id = ?1")?;
    let rows = stmt
        .query_map([period_id], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn recompute_open_period_rows(conn: &Connection, period_id: i64) -> Result<(), AppError> {
    let period_month = period_month_of(conn, period_id)?;
    let slabs = slab_table(conn)?;
    let royalty_min_children = setting_i64(conn, "royalty_qualifying_count")?;
    let royalty_rate_percent = setting_i64(conn, "royalty_rate_percent")?;

    for id in open_period_member_ids(conn, period_id)? {
        let business_volume = business_volume_of(conn, id, &period_month)?;
        let children = direct_children_figures(conn, id, period_id)?;
        let figures = compute_node(
            business_volume,
            &children,
            &slabs,
            royalty_min_children,
            royalty_rate_percent,
        );
        upsert_totals(conn, id, period_id, business_volume, &figures)?;
    }
    Ok(())
}

/// T-M7.1-1/T-M7.2-2: a slab-table or royalty-setting change affects every
/// member's slab-driven figures, not one ancestor chain, so
/// `recalculate_chain` doesn't fit — this recomputes every row already
/// present in the currently open calendar-month period. Order doesn't
/// matter here the way it does for `recalculate_chain`: Rule-6's TBV
/// formula never depends on the slab table or royalty settings, only on
/// each member's own Business Volume (re-summed from entries, untouched by
/// a settings edit) and its children's already-stored TBV — so every row
/// can be recomputed independently from what's already on disk. If no
/// period is currently open (nothing entered yet this calendar month),
/// there is nothing to recompute — a no-op, not an error. Composes inside
/// a caller-owned transaction exactly like `recalculate_chain`, since
/// API-22 requires "write setting(s) + recalculate" as one transaction.
pub fn recalculate_open_period(conn: &Connection) -> Result<(), AppError> {
    let Some(period_id) = current_open_period_id(conn)? else {
        return Ok(());
    };

    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        recompute_open_period_rows(&tx, period_id)?;
        tx.commit()?;
    } else {
        recompute_open_period_rows(conn, period_id)?;
    }
    Ok(())
}

// --- preview_settings_impact (API-33, US-M7.3) ---

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateSettings {
    pub slab_thresholds: Option<Vec<i64>>,
    pub slab_percentages: Option<Vec<i64>>,
    pub royalty_qualifying_count: Option<i64>,
    pub royalty_rate_percent: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberImpact {
    pub member_id: i64,
    pub member_name: String,
    pub rewards_before: i64,
    pub rewards_after: i64,
    pub slab_pct_before: i64,
    pub slab_pct_after: i64,
    pub royalty_before: i64,
    pub royalty_after: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsImpactPreview {
    pub rewards_before: i64,
    pub rewards_after: i64,
    pub affected_members: Vec<MemberImpact>,
}

/// Merges `candidate`'s slab table with the live one — both fields are
/// required together (a threshold needs its matching percentage) or both
/// absent (keep the live table unchanged).
fn resolve_candidate_slabs(
    conn: &Connection,
    candidate: &CandidateSettings,
) -> Result<Vec<(i64, i64)>, AppError> {
    match (&candidate.slab_thresholds, &candidate.slab_percentages) {
        (Some(thresholds), Some(percentages)) => {
            if thresholds.len() != percentages.len() {
                return Err(AppError::Validation {
                    field: "slabPercentages".into(),
                    message: "Slab thresholds and percentages must have the same length.".into(),
                });
            }
            Ok(thresholds
                .iter()
                .zip(percentages.iter())
                .map(|(&t, &p)| (t, p))
                .collect())
        }
        (None, None) => slab_table(conn),
        _ => Err(AppError::Validation {
            field: "slabThresholds".into(),
            message: "Slab thresholds and percentages must be provided together.".into(),
        }),
    }
}

struct LiveFigures {
    member_id: i64,
    member_name: String,
    slab_pct: i64,
    royalty: i64,
    rewards: i64,
}

fn live_figures_for_open_period(
    conn: &Connection,
    period_id: i64,
) -> Result<Vec<LiveFigures>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.name, t.slab_pct, t.royalty, t.rewards
         FROM member_period_totals t
         JOIN members m ON m.id = t.member_id
         WHERE t.period_id = ?1",
    )?;
    let rows = stmt
        .query_map([period_id], |r| {
            Ok(LiveFigures {
                member_id: r.get(0)?,
                member_name: r.get(1)?,
                slab_pct: r.get(2)?,
                royalty: r.get(3)?,
                rewards: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// API-33: what the open period's figures would become under candidate
/// slab/royalty settings — **no I/O beyond reads**, matching M3's pure
/// nature (§3.2). Rather than the architecture doc's literal "write
/// candidate settings, recompute, restore in a `finally`" description, this
/// calls the same pure `compute_node` the real save path uses, fed the
/// candidate values directly — nothing is ever written, so a panic can't
/// leave anything uncommitted to restore. Both paths sharing `compute_node`
/// is also what guarantees the preview equals what actually lands
/// (T-M7.3-6): given the same live Business Volume and children figures
/// (Rule-6 — TBV never depends on slab/royalty settings) and the same
/// candidate values, the two calls are identical function applications.
pub fn preview_settings_impact(
    conn: &Connection,
    candidate: CandidateSettings,
) -> Result<SettingsImpactPreview, AppError> {
    let Some(period_id) = current_open_period_id(conn)? else {
        return Ok(SettingsImpactPreview {
            rewards_before: 0,
            rewards_after: 0,
            affected_members: Vec::new(),
        });
    };
    let period_month = period_month_of(conn, period_id)?;
    let slabs = resolve_candidate_slabs(conn, &candidate)?;
    let royalty_min_children = candidate
        .royalty_qualifying_count
        .map(Ok)
        .unwrap_or_else(|| setting_i64(conn, "royalty_qualifying_count"))?;
    let royalty_rate_percent = candidate
        .royalty_rate_percent
        .map(Ok)
        .unwrap_or_else(|| setting_i64(conn, "royalty_rate_percent"))?;

    let mut rewards_before_total = 0;
    let mut rewards_after_total = 0;
    let mut affected = Vec::new();
    for live in live_figures_for_open_period(conn, period_id)? {
        let business_volume = business_volume_of(conn, live.member_id, &period_month)?;
        let children = direct_children_figures(conn, live.member_id, period_id)?;
        let after = compute_node(
            business_volume,
            &children,
            &slabs,
            royalty_min_children,
            royalty_rate_percent,
        );

        rewards_before_total += live.rewards;
        rewards_after_total += after.rewards;

        if live.rewards != after.rewards
            || live.slab_pct != after.slab_pct
            || (live.royalty > 0) != (after.royalty > 0)
        {
            affected.push(MemberImpact {
                member_id: live.member_id,
                member_name: live.member_name,
                rewards_before: live.rewards,
                rewards_after: after.rewards,
                slab_pct_before: live.slab_pct,
                slab_pct_after: after.slab_pct,
                royalty_before: live.royalty,
                royalty_after: after.royalty,
            });
        }
    }
    affected.sort_by(|a, b| {
        let da = (a.rewards_after - a.rewards_before).abs();
        let db = (b.rewards_after - b.rewards_before).abs();
        db.cmp(&da).then_with(|| a.member_name.cmp(&b.member_name))
    });

    Ok(SettingsImpactPreview {
        rewards_before: rewards_before_total,
        rewards_after: rewards_after_total,
        affected_members: affected,
    })
}

fn is_active_of(conn: &Connection, member_id: i64) -> Result<bool, AppError> {
    conn.query_row(
        "SELECT is_active FROM members WHERE id = ?1",
        [member_id],
        |r| r.get(0),
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound {
        message: "Member not found.".into(),
    })
}

/// Rule-39/§7.3's closed-period source of truth: a child's current figures
/// come from its own latest `monthly_snapshots` version, never from
/// `member_period_totals` — that table holds the closed period's zeroed
/// live figures (Rule-38), not its historical record.
fn snapshot_children_figures(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<Vec<ChildFigures>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(t.total_business_volume, 0), COALESCE(t.slab_pct, 0)
         FROM members m
         LEFT JOIN monthly_snapshots t ON t.member_id = m.id AND t.period_id = ?2
            AND t.version = (
                SELECT MAX(version) FROM monthly_snapshots
                WHERE member_id = m.id AND period_id = ?2
            )
         WHERE m.introducer_member_id = ?1",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![member_id, period_id], |r| {
            Ok(ChildFigures {
                total_business_volume: r.get(0)?,
                slab_pct: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn next_snapshot_version(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<i64, AppError> {
    let max: Option<i64> = conn
        .query_row(
            "SELECT MAX(version) FROM monthly_snapshots WHERE member_id = ?1 AND period_id = ?2",
            rusqlite::params![member_id, period_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    Ok(max.unwrap_or(0) + 1)
}

#[allow(clippy::too_many_arguments)]
fn insert_snapshot(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
    version: i64,
    business_volume: i64,
    figures: &NodeFigures,
    is_active_status: bool,
    created_at: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO monthly_snapshots
            (member_id, period_id, version, business_volume, total_business_volume,
             slab_pct, differential, royalty, own_reward, rewards, is_active_status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            member_id,
            period_id,
            version,
            business_volume,
            figures.total_business_volume,
            figures.slab_pct,
            figures.differential,
            figures.royalty,
            figures.own_reward,
            figures.rewards,
            is_active_status,
            created_at,
        ],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn walk_chain_into_snapshot(
    conn: &Connection,
    chain: &[i64],
    period_month: &str,
    period_id: i64,
    slabs: &[(i64, i64)],
    royalty_min_children: i64,
    royalty_rate_percent: i64,
    created_at: &str,
) -> Result<(), AppError> {
    for &id in chain {
        let business_volume = business_volume_of(conn, id, period_month)?;
        let children = snapshot_children_figures(conn, id, period_id)?;
        let figures = compute_node(
            business_volume,
            &children,
            slabs,
            royalty_min_children,
            royalty_rate_percent,
        );
        let version = next_snapshot_version(conn, id, period_id)?;
        let is_active = is_active_of(conn, id)?;
        insert_snapshot(
            conn,
            id,
            period_id,
            version,
            business_volume,
            &figures,
            is_active,
            created_at,
        )?;
    }
    Ok(())
}

/// Rule-39/ADR-006, §7.3: a closed-period correction recomputes the changed
/// member's chain "in isolation" — reading every child from its latest
/// `monthly_snapshots` version rather than the live `member_period_totals`
/// table — and writes each ancestor's result as a **new** snapshot version,
/// one per member on the chain, at that member's own next version number.
/// `member_period_totals` is never touched by this path; a closed period's
/// live totals stay zeroed (Rule-38) regardless of how many corrections
/// follow. The caller (M2's `edit_entry`, S7) opens the transaction, the
/// same composability contract as `recalculate_chain`.
pub fn write_correction_snapshot(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<(), AppError> {
    let chain = chain_to_root(conn, member_id)?;
    let period_month = period_month_of(conn, period_id)?;
    let slabs = slab_table(conn)?;
    let royalty_min_children = setting_i64(conn, "royalty_qualifying_count")?;
    let royalty_rate_percent = setting_i64(conn, "royalty_rate_percent")?;
    let created_at = chrono::Local::now().date_naive().to_string();

    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        walk_chain_into_snapshot(
            &tx,
            &chain,
            &period_month,
            period_id,
            &slabs,
            royalty_min_children,
            royalty_rate_percent,
            &created_at,
        )?;
        tx.commit()?;
    } else {
        walk_chain_into_snapshot(
            conn,
            &chain,
            &period_month,
            period_id,
            &slabs,
            royalty_min_children,
            royalty_rate_percent,
            &created_at,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded() -> Connection {
        db::open_seeded_in_memory().unwrap()
    }

    fn insert_period(conn: &Connection, month: &str) -> i64 {
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
            [month],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_member(conn: &Connection, introducer: Option<i64>) -> i64 {
        static NEXT_PHONE: std::sync::atomic::AtomicI64 =
            std::sync::atomic::AtomicI64::new(9_000_000_000);
        let phone = NEXT_PHONE
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string();
        conn.execute(
            "INSERT INTO members
                (name, phone, address, introducer_member_id, level, is_active,
                 joining_date, consent_given, consent_date, created_at)
             VALUES ('T', ?1, 'addr', ?2, 1, 1, '2026-01-01', 1, '2026-01-01', '2026-01-01')",
            rusqlite::params![phone, introducer],
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

    fn totals(conn: &Connection, member_id: i64, period_id: i64) -> (i64, i64, i64) {
        conn.query_row(
            "SELECT total_business_volume, slab_pct, rewards FROM member_period_totals
             WHERE member_id = ?1 AND period_id = ?2",
            rusqlite::params![member_id, period_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
    }

    #[test]
    fn a_leaf_write_recalculates_its_own_row() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let root = insert_member(&conn, None);
        insert_entry(&conn, root, "2026-08", 100_000); // ×100: 1,000.00

        recalculate_chain(&conn, root, period).unwrap();

        let (tbv, slab, _) = totals(&conn, root, period);
        assert_eq!(tbv, 100_000);
        assert_eq!(slab, 4); // ×100 thresholds: 40,000 <= 100,000 < 120,000
    }

    #[test]
    fn a_deep_write_recalculates_every_ancestor_in_one_transaction() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let root = insert_member(&conn, None);
        let mid = insert_member(&conn, Some(root));
        let leaf = insert_member(&conn, Some(mid));
        insert_entry(&conn, leaf, "2026-08", 10_000); // ×100: 100.00

        recalculate_chain(&conn, leaf, period).unwrap();

        for id in [leaf, mid, root] {
            let (tbv, _, _) = totals(&conn, id, period);
            assert_eq!(
                tbv, 10_000,
                "member {id}'s TBV must include the leaf's write"
            );
        }
    }

    #[test]
    fn a_siblings_differential_changes_when_the_parents_slab_shifts() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let parent = insert_member(&conn, None);
        let a = insert_member(&conn, Some(parent));
        let b = insert_member(&conn, Some(parent));

        insert_entry(&conn, a, "2026-08", 30_000); // ×100: 300.00
        recalculate_chain(&conn, a, period).unwrap();
        let (_, _, parent_rewards_before) = totals(&conn, parent, period);

        // B's own write must re-scan A too — A's differential term depends
        // on the parent's slab, which B's write may have just moved.
        insert_entry(&conn, b, "2026-08", 500_000); // ×100: 5,000.00
        recalculate_chain(&conn, b, period).unwrap();
        let (_, _, parent_rewards_after) = totals(&conn, parent, period);

        assert_ne!(
            parent_rewards_before, parent_rewards_after,
            "parent's differential over both A and B must move once B's write shifts the parent's slab"
        );
    }

    #[test]
    fn two_live_periods_stay_isolated() {
        let conn = seeded();
        let older = insert_period(&conn, "2026-07");
        let newer = insert_period(&conn, "2026-08");
        let root = insert_member(&conn, None);

        insert_entry(&conn, root, "2026-07", 50_000); // ×100: 500.00
        recalculate_chain(&conn, root, older).unwrap();
        let before = totals(&conn, root, older);

        insert_entry(&conn, root, "2026-08", 900_000); // ×100: 9,000.00
        recalculate_chain(&conn, root, newer).unwrap();
        let after = totals(&conn, root, older);

        assert_eq!(
            before, after,
            "writing into the newer period must leave the older period's row byte-identical"
        );
    }

    #[test]
    fn an_inactive_members_business_volume_still_rolls_up() {
        // Rule-28/T-M3.2-5: is_active has zero calculation effect.
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let root = insert_member(&conn, None);
        let child = insert_member(&conn, Some(root));
        conn.execute("UPDATE members SET is_active = 0 WHERE id = ?1", [child])
            .unwrap();
        insert_entry(&conn, child, "2026-08", 20_000); // ×100: 200.00

        recalculate_chain(&conn, child, period).unwrap();

        let (tbv, _, _) = totals(&conn, root, period);
        assert_eq!(
            tbv, 20_000,
            "an inactive member's BV must still reach the root"
        );
    }

    #[test]
    fn recalculating_touches_exactly_one_row_per_chain_member_not_per_descendant() {
        // T-M3.2-7: cost is O(depth), independent of how large the tree is
        // off-chain. Build a wide root with many extra children, then
        // write against one deep leaf and count exactly which rows moved.
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let root = insert_member(&conn, None);
        for _ in 0..50 {
            insert_member(&conn, Some(root));
        }
        let mid = insert_member(&conn, Some(root));
        let leaf = insert_member(&conn, Some(mid));
        insert_entry(&conn, leaf, "2026-08", 1_000); // ×100: 10.00

        recalculate_chain(&conn, leaf, period).unwrap();

        let rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM member_period_totals WHERE period_id = ?1",
                [period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(rows, 3, "only leaf, mid and root — the chain — get a row");
    }

    #[test]
    fn recalculating_an_unknown_member_is_refused() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let err = recalculate_chain(&conn, 999_999, period).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn composes_inside_a_callers_already_open_transaction() {
        // API-08: "insert entry + recalc" must be one transaction — the
        // future caller (record_entry, S7) opens it and calls this from
        // inside it. Must not try to nest a second BEGIN (SQLite refuses
        // that), and rolling back the caller's transaction must discard
        // this function's writes too.
        let mut conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let root = insert_member(&conn, None);

        let tx = conn.transaction().unwrap();
        tx.execute(
            "INSERT INTO business_volume_entries
                (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, 10000, '2026-08-15', '2026-08', '2026-08-15')",
            [root],
        )
        .unwrap();
        recalculate_chain(&tx, root, period).unwrap();
        tx.rollback().unwrap();

        let row_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM member_period_totals WHERE member_id = ?1)",
                [root],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            !row_exists,
            "rolling back the caller's outer transaction must discard the recalculation too"
        );
    }

    // --- recalculate_open_period (US-M7.1/M7.2, S10) ---

    fn this_month() -> String {
        chrono::Local::now().format("%Y-%m").to_string()
    }

    #[test]
    fn no_open_period_is_a_silent_no_op() {
        let conn = seeded();
        recalculate_open_period(&conn).unwrap();
    }

    #[test]
    fn a_slab_table_edit_is_reflected_on_the_next_recalculation() {
        let conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let root = insert_member(&conn, None);
        insert_entry(&conn, root, &month, 100_000); // ×100: 1,000.00
        recalculate_chain(&conn, root, period).unwrap();
        let (_, slab_before, _) = totals(&conn, root, period);
        assert_eq!(slab_before, 4); // 40,000 <= 100,000 < 120,000

        // Simulate T-M7.1-1's update_slab_row: move the 4% row's threshold
        // above the member's TBV, same as the client's own worked example
        // (moving 6% to 1,000 / 2% to 200).
        conn.execute(
            "UPDATE slab_table SET threshold = 200_000 WHERE percentage = 4",
            [],
        )
        .unwrap();

        recalculate_open_period(&conn).unwrap();

        let (_, slab_after, _) = totals(&conn, root, period);
        assert_eq!(
            slab_after, 2,
            "the member's slab must move with the edited table"
        );
    }

    #[test]
    fn a_royalty_setting_change_is_reflected_without_a_new_entry() {
        let conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let parent = insert_member(&conn, None);
        for _ in 0..3 {
            let child = insert_member(&conn, Some(parent));
            insert_entry(&conn, child, &month, 1_000_000); // ×100: 10,000.00 — top slab
            recalculate_chain(&conn, child, period).unwrap();
        }
        let (_, _, rewards_before) = totals(&conn, parent, period);
        assert!(
            rewards_before > 0,
            "3 qualifying children must already earn royalty"
        );

        conn.execute(
            "UPDATE settings SET value = '10' WHERE key = 'royalty_qualifying_count'",
            [],
        )
        .unwrap();

        recalculate_open_period(&conn).unwrap();

        let (_, _, rewards_after) = totals(&conn, parent, period);
        assert_eq!(
            rewards_after, 0,
            "raising the qualifying count above 3 must drop royalty on the next recalculation"
        );
    }

    #[test]
    fn only_the_open_periods_rows_are_touched() {
        let conn = seeded();
        let closed_month = "2026-01";
        let open_month = this_month();
        let closed_period = insert_period(&conn, closed_month);
        conn.execute(
            "UPDATE periods SET status = 'closed' WHERE id = ?1",
            [closed_period],
        )
        .unwrap();
        let open_period = insert_period(&conn, &open_month);
        let root = insert_member(&conn, None);
        insert_entry(&conn, root, closed_month, 50_000);
        recalculate_chain(&conn, root, closed_period).unwrap();
        let before = totals(&conn, root, closed_period);

        insert_entry(&conn, root, &open_month, 900_000);
        recalculate_chain(&conn, root, open_period).unwrap();
        conn.execute("UPDATE slab_table SET threshold = threshold + 1", [])
            .unwrap();

        recalculate_open_period(&conn).unwrap();

        let after = totals(&conn, root, closed_period);
        assert_eq!(
            before, after,
            "recalculating the open period must never touch a closed period's row"
        );
    }

    #[test]
    fn recalculate_open_period_composes_inside_a_callers_already_open_transaction() {
        let mut conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let root = insert_member(&conn, None);
        insert_entry(&conn, root, &month, 100_000);
        recalculate_chain(&conn, root, period).unwrap();

        let tx = conn.transaction().unwrap();
        tx.execute(
            "UPDATE slab_table SET threshold = 1 WHERE percentage = 14",
            [],
        )
        .unwrap();
        recalculate_open_period(&tx).unwrap();
        tx.rollback().unwrap();

        let (_, slab_after_rollback, _) = totals(&conn, root, period);
        assert_eq!(
            slab_after_rollback, 4,
            "rolling back the caller's transaction must discard the recalculation too"
        );
    }

    // --- write_correction_snapshot (US-M2.2, S7) ---

    fn insert_snapshot_row(
        conn: &Connection,
        member_id: i64,
        period_id: i64,
        version: i64,
        total_business_volume: i64,
        slab_pct: i64,
    ) {
        conn.execute(
            "INSERT INTO monthly_snapshots
                (member_id, period_id, version, business_volume, total_business_volume,
                 slab_pct, differential, royalty, own_reward, rewards, is_active_status, created_at)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, 0, 0, 0, 0, 1, '2026-08-01')",
            rusqlite::params![
                member_id,
                period_id,
                version,
                total_business_volume,
                slab_pct
            ],
        )
        .unwrap();
    }

    fn snapshot_row(conn: &Connection, member_id: i64, period_id: i64, version: i64) -> (i64, i64) {
        conn.query_row(
            "SELECT total_business_volume, slab_pct FROM monthly_snapshots
             WHERE member_id = ?1 AND period_id = ?2 AND version = ?3",
            rusqlite::params![member_id, period_id, version],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    }

    fn max_snapshot_version(conn: &Connection, member_id: i64, period_id: i64) -> i64 {
        conn.query_row(
            "SELECT MAX(version) FROM monthly_snapshots WHERE member_id = ?1 AND period_id = ?2",
            rusqlite::params![member_id, period_id],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn a_correction_reads_children_from_their_latest_snapshot_not_live_totals() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let parent = insert_member(&conn, None);
        let child = insert_member(&conn, Some(parent));
        // Simulates a prior close (S11): v1 snapshots exist, live totals do
        // not (Rule-38 zeroes them).
        insert_snapshot_row(&conn, child, period, 1, 10_000, 4);
        insert_snapshot_row(&conn, parent, period, 1, 10_000, 4);

        // The correction: child's entry is the source of its BV now, not
        // the pre-seeded snapshot figure above — business_volume_of always
        // re-sums from business_volume_entries (§5.1's own step 1).
        insert_entry(&conn, child, "2026-08", 40_000); // ×100: 400.00
        write_correction_snapshot(&conn, child, period).unwrap();

        let (child_tbv, child_slab) = snapshot_row(&conn, child, period, 2);
        assert_eq!(child_tbv, 40_000);
        assert_eq!(child_slab, 4); // exactly at the 400*100 threshold

        let (parent_tbv, _) = snapshot_row(&conn, parent, period, 2);
        assert_eq!(
            parent_tbv, 40_000,
            "parent must roll up the child's corrected snapshot, not a stale live total"
        );

        let live_total_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM member_period_totals WHERE period_id = ?1)",
                [period],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            !live_total_exists,
            "a closed period's member_period_totals must never be written by a correction"
        );
    }

    #[test]
    fn each_chain_member_advances_its_own_version_independently() {
        // A member corrected twice already sits at v2; an ancestor never
        // touched by a prior correction still sits at v1. One further
        // correction must give each its own next version, not a shared one.
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let parent = insert_member(&conn, None);
        let child = insert_member(&conn, Some(parent));
        insert_snapshot_row(&conn, child, period, 1, 10_000, 4);
        insert_snapshot_row(&conn, child, period, 2, 15_000, 4);
        insert_snapshot_row(&conn, parent, period, 1, 15_000, 4);

        insert_entry(&conn, child, "2026-08", 5_000); // ×100: 50.00
        write_correction_snapshot(&conn, child, period).unwrap();

        assert_eq!(max_snapshot_version(&conn, child, period), 3);
        assert_eq!(max_snapshot_version(&conn, parent, period), 2);
    }

    #[test]
    fn correction_snapshot_composes_inside_a_callers_already_open_transaction() {
        let mut conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let root = insert_member(&conn, None);
        insert_snapshot_row(&conn, root, period, 1, 5_000, 2);

        let tx = conn.transaction().unwrap();
        tx.execute(
            "INSERT INTO business_volume_entries
                (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, 5000, '2026-08-15', '2026-08', '2026-08-15')",
            [root],
        )
        .unwrap();
        write_correction_snapshot(&tx, root, period).unwrap();
        tx.rollback().unwrap();

        assert_eq!(
            max_snapshot_version(&conn, root, period),
            1,
            "rolling back the caller's outer transaction must discard the correction too"
        );
    }

    // --- preview_settings_impact (API-33, US-M7.3, S11) ---

    #[test]
    fn no_open_period_previews_as_an_empty_no_op() {
        let conn = seeded();
        let preview = preview_settings_impact(&conn, CandidateSettings::default()).unwrap();
        assert_eq!(preview.rewards_before, 0);
        assert_eq!(preview.rewards_after, 0);
        assert!(preview.affected_members.is_empty());
    }

    #[test]
    fn an_empty_candidate_leaves_every_member_unaffected() {
        let conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let root = insert_member(&conn, None);
        insert_entry(&conn, root, &month, 100_000);
        recalculate_chain(&conn, root, period).unwrap();

        let preview = preview_settings_impact(&conn, CandidateSettings::default()).unwrap();

        assert!(preview.affected_members.is_empty());
        assert_eq!(preview.rewards_before, preview.rewards_after);
    }

    #[test]
    fn a_candidate_slab_table_moves_the_affected_members_slab_and_nothing_else() {
        let conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let root = insert_member(&conn, None);
        insert_entry(&conn, root, &month, 100_000); // ×100: 1,000.00
        recalculate_chain(&conn, root, period).unwrap();
        let (_, slab_before, _) = totals(&conn, root, period);
        assert_eq!(slab_before, 4);

        // Same worked move as `a_slab_table_edit_is_reflected_on_the_next_recalculation`:
        // the default 7-row table (seed.rs's DEFAULT_SLABS) with the 4% row's
        // threshold moved from 40,000 above the member's own TBV.
        let candidate = CandidateSettings {
            slab_thresholds: Some(vec![
                10_000, 200_000, 120_000, 300_000, 500_000, 700_000, 1_000_000,
            ]),
            slab_percentages: Some(vec![2, 4, 6, 8, 10, 12, 14]),
            ..Default::default()
        };
        let preview = preview_settings_impact(&conn, candidate).unwrap();

        assert_eq!(preview.affected_members.len(), 1);
        let m = &preview.affected_members[0];
        assert_eq!(m.member_id, root);
        assert_eq!(m.slab_pct_before, 4);
        assert_eq!(m.slab_pct_after, 2);

        // Nothing actually written — the live row is untouched.
        let (_, slab_still, _) = totals(&conn, root, period);
        assert_eq!(
            slab_still, 4,
            "a preview must never write to member_period_totals"
        );
    }

    #[test]
    fn preview_matches_exactly_what_a_real_save_produces() {
        let conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let parent = insert_member(&conn, None);
        for _ in 0..3 {
            let child = insert_member(&conn, Some(parent));
            insert_entry(&conn, child, &month, 1_000_000); // top slab
            recalculate_chain(&conn, child, period).unwrap();
        }

        let candidate = CandidateSettings {
            royalty_qualifying_count: Some(10),
            ..Default::default()
        };
        let preview = preview_settings_impact(&conn, candidate).unwrap();
        let predicted_parent = preview
            .affected_members
            .iter()
            .find(|m| m.member_id == parent)
            .expect("raising the qualifying count above 3 must stop the parent's royalty");
        assert_eq!(predicted_parent.royalty_after, 0);

        conn.execute(
            "UPDATE settings SET value = '10' WHERE key = 'royalty_qualifying_count'",
            [],
        )
        .unwrap();
        recalculate_open_period(&conn).unwrap();
        let (_, _, settled_rewards) = totals(&conn, parent, period);

        assert_eq!(
            settled_rewards, predicted_parent.rewards_after,
            "T-M7.3-6: the preview must equal what the real save actually settles at"
        );
    }
}
