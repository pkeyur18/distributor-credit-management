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
}
