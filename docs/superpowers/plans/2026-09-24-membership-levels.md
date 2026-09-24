# Membership Levels Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn Rule-10's single royalty condition into a four-rung monthly membership ladder (Gold → Platinum → Diamond → Ace), each rung with its own editable qualifying count and royalty rate.

**Architecture:** The pure engine (`m3_calc/engine.rs`) derives a member's level (rank 0–4) from its direct legs' slabs and levels, bottom-up, exactly like slab. The rank is stored per period in `member_period_totals`/`monthly_snapshots` and zeroed at close. Settings hold four (count, rate) pairs. Level 1 reuses today's two royalty keys. The settings-change recompute and its preview switch to deepest-first order, because a child's level now feeds its parent's.

**Tech Stack:** Rust (rusqlite/SQLCipher, Tauri v2 commands), React 19 + TypeScript, Vitest, WebdriverIO E2E.

**Spec:** `docs/superpowers/specs/2026-09-24-membership-levels-design.md`

## Global Constraints

- No count, rate or threshold hardcoded outside tests and seed defaults. Every one comes from `settings`.
- Level names are drafts: `["Gold", "Platinum", "Diamond", "Ace"]`, rank 1..=4. They are defined once in Rust (`m3_calc::engine::MEMBERSHIP_LEVEL_NAMES`) and once in TypeScript (`src/lib/membership-levels.ts`). They are never stored and never a setting. Rank 0 renders as `—` (U+2014).
- The database stores the rank (`membership_tier INTEGER NOT NULL DEFAULT 0`), never a name.
- Levels are monthly. Nothing reads or copies a level across periods. Close snapshots the level and then zeroes it.
- Settings keys:
  - Level 1: `royalty_qualifying_count` / `royalty_rate_percent` (existing).
  - Levels 2–4: `royalty_tier_{2,3,4}_qualifying_count` / `royalty_tier_{2,3,4}_rate_percent`.
  - IPC camelCase: `royaltyTier{2,3,4}QualifyingCount` / `royaltyTier{2,3,4}RatePercent`.
- Restricted vocabulary: the words "subscription" and "tier" never appear in a user-visible string. "Membership" and the four level names are allowed.
- Commits: conventional-commit prefix (`feat:`, `fix:`, `test:`, `docs:`), **no `Co-Authored-By` trailer**, on `feature/membership-levels` only. Run `git branch --show-current` before every commit. Never commit to `develop`/`main`. Push only at the end; the user opens the PR.
- Rust commands run from `src-tauri/`: `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`. Frontend commands run from the repo root: `npm run test`, `npm run lint`, `npx tsc --noEmit`, `npm run vocab-grep`.
- Money is ×100 fixed point in the DB (ADR-004). Engine unit tests and golden scenarios use real units against a real-unit slab table. DB tests use ×100 against the seeded table (top slab 14% at threshold 1,000,000).

---

### Task 1: Schema migration and seeded settings

**Files:**
- Create: `src-tauri/src/db/migrations/0002_membership_tier.sql`
- Modify: `src-tauri/src/db/migrations.rs:3` (MIGRATIONS list) and its tests
- Modify: `src-tauri/src/db/seed.rs:16-83` (doc comment, rows, count assert) and its tests

**Interfaces:**
- Produces:
  - Column `membership_tier` on `member_period_totals` and `monthly_snapshots`.
  - Six settings keys present on both fresh and upgraded databases (22 rows total on a fresh install).

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `src-tauri/src/db/migrations.rs`, and change `is_idempotent_when_run_twice_on_the_same_database`'s expected count from `1` to `2`:

```rust
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
            conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(value("royalty_qualifying_count"), "4", "existing value untouched");
        assert_eq!(value("royalty_rate_percent"), "1.5", "existing value untouched");
        for rank in 2..=4 {
            assert_eq!(value(&format!("royalty_tier_{rank}_qualifying_count")), "3");
            assert_eq!(
                value(&format!("royalty_tier_{rank}_rate_percent")),
                "1.5",
                "upgrade copies the installation's current rate so no Rewards figure moves"
            );
        }
    }
```

In `src-tauri/src/db/seed.rs` tests:
- Rename `inserts_exactly_sixteen_settings_rows` to `inserts_exactly_twenty_two_settings_rows` and change its expectation to `22`.
- Change `seeding_twice_does_not_duplicate_rows`'s `assert_eq!(settings_count, 16)` to `22`.
- Add:

```rust
    #[test]
    fn seeds_the_three_later_membership_levels_with_count_3_and_rate_1() {
        let conn = seeded_db();
        for rank in 2..=4 {
            let count: String = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    [format!("royalty_tier_{rank}_qualifying_count")],
                    |r| r.get(0),
                )
                .unwrap();
            let rate: String = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    [format!("royalty_tier_{rank}_rate_percent")],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!((count.as_str(), rate.as_str()), ("3", "1"));
        }
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd src-tauri && cargo test --lib db::`
Expected: FAIL. `membership_tier_column_exists...` fails (column missing), the backfill test panics on a missing key, the seed count is 16 not 22, and the migration count is 1 not 2.

- [ ] **Step 3: Write the migration**

Create `src-tauri/src/db/migrations/0002_membership_tier.sql`:

```sql
-- Migration 0002 — membership levels (CR-7, Rule-47).
-- Rank only (0 = none, 1..=4 = Gold/Platinum/Diamond/Ace): names are
-- drafts that live in code, never in the database. Existing snapshots stay
-- at 0 — that is what those closed months showed.
ALTER TABLE member_period_totals ADD COLUMN membership_tier INTEGER NOT NULL DEFAULT 0;
ALTER TABLE monthly_snapshots    ADD COLUMN membership_tier INTEGER NOT NULL DEFAULT 0;

-- Levels 2-4's settings, on an already-seeded database only. db/mod.rs
-- runs migrations *before* seed::run, and seed_settings skips entirely
-- when settings is non-empty — inserting unconditionally here would
-- suppress every default setting on a brand-new install (seed.rs owns
-- those). Rates copy the installation's current royalty rate, so an
-- upgrade moves no Rewards figure until the client edits one.
INSERT INTO settings (key, value)
SELECT k, v FROM (
    SELECT 'royalty_tier_2_qualifying_count' AS k, '3' AS v
    UNION ALL SELECT 'royalty_tier_3_qualifying_count', '3'
    UNION ALL SELECT 'royalty_tier_4_qualifying_count', '3'
    UNION ALL SELECT 'royalty_tier_2_rate_percent',
                     (SELECT value FROM settings WHERE key = 'royalty_rate_percent')
    UNION ALL SELECT 'royalty_tier_3_rate_percent',
                     (SELECT value FROM settings WHERE key = 'royalty_rate_percent')
    UNION ALL SELECT 'royalty_tier_4_rate_percent',
                     (SELECT value FROM settings WHERE key = 'royalty_rate_percent')
)
WHERE EXISTS (SELECT 1 FROM settings);
```

In `src-tauri/src/db/migrations.rs` line 3:

```rust
const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("migrations/0001_initial.sql")),
    (2, include_str!("migrations/0002_membership_tier.sql")),
];
```

- [ ] **Step 4: Extend the seed**

In `src-tauri/src/db/seed.rs`:
- Change the doc comment on `run` from "16 settings rows" to "22 settings rows (16 original + CR-7's six membership-level rows)".
- Insert these six entries into `rows`, directly after `("royalty_rate_percent", "1"),`:

```rust
        // CR-7/Rule-47: membership levels 2-4 (level 1 is the two rows above).
        ("royalty_tier_2_qualifying_count", "3"),
        ("royalty_tier_2_rate_percent", "1"),
        ("royalty_tier_3_qualifying_count", "3"),
        ("royalty_tier_3_rate_percent", "1"),
        ("royalty_tier_4_qualifying_count", "3"),
        ("royalty_tier_4_rate_percent", "1"),
```

- Change the `debug_assert_eq!` to `rows.len(), 22, "settings inventory is 22 rows (conflict C1 + CR-7)"`.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd src-tauri && cargo test --lib db::`
Expected: PASS (all `db::migrations` and `db::seed` tests).

- [ ] **Step 6: Commit**

```bash
git branch --show-current   # must print feature/membership-levels
git add src-tauri/src/db
git commit -m "feat: add membership_tier column and level settings (migration 0002)"
```

---

### Task 2: Engine computes the level and pays the level's rate

**Files:**
- Modify: `src-tauri/src/m3_calc/engine.rs` (whole public surface and tests)
- Modify: `src-tauri/src/m3_calc/mod.rs`:
  - `direct_children_figures` (lines 72-93)
  - `upsert_totals` (126-158)
  - `walk_chain` (160-182)
  - `recalculate_chain` (203-240)
  - `recompute_open_period_rows` (253-275)
  - `preview_settings_impact` (419-497): signature change only in this task
  - `snapshot_children_figures` (515-538)
  - `insert_snapshot` (558-589)
  - `walk_chain_into_snapshot` (591-625)
  - `write_correction_snapshot` (637-676)
- Modify: `src-tauri/tests/golden_scenarios.rs`, `src-tauri/tests/differential_non_negativity.rs` (call sites)

**Interfaces:**
- Consumes: Task 1's `membership_tier` column and six settings keys.
- Produces (later tasks rely on these exact names):
  - `m3_calc::engine::MEMBERSHIP_LEVEL_NAMES: [&str; 4]`
  - `m3_calc::engine::membership_level_name(rank: i64) -> &'static str` (rank 0 or out of range returns `"\u{2014}"`)
  - `m3_calc::engine::RoyaltyTier { qualifying_count: i64, rate_percent: f64 }` (`Debug, Clone, Copy, PartialEq`)
  - `ChildFigures { total_business_volume, slab_pct, membership_tier: i64 }` (now `Debug, Clone, Copy`)
  - `NodeFigures { ..., membership_tier: i64 }`
  - `compute_node(own_business_volume: i64, children: &[ChildFigures], slabs: &[(i64, i64)], tiers: &[RoyaltyTier]) -> NodeFigures`
  - `m3_calc::ROYALTY_TIER_KEYS: [(&str, &str); MEMBERSHIP_LEVEL_NAMES.len()]` (`pub`)
  - `m3_calc::royalty_tiers(conn: &Connection) -> Result<Vec<RoyaltyTier>, AppError>` (`pub`)

- [ ] **Step 1: Write the failing engine tests**

In `engine.rs` `mod tests`:

(a) Add these constants below `SLABS`:

```rust
    // Every existing Rule-10 test assumed min 3 / 1% — the same rule, now
    // expressed as four identical rungs.
    const UNIFORM: [RoyaltyTier; 4] = [RoyaltyTier {
        qualifying_count: 3,
        rate_percent: 1.0,
    }; 4];
    // Distinct rates so "highest level's rate replaces" is observable.
    const LADDER: [RoyaltyTier; 4] = [
        RoyaltyTier { qualifying_count: 3, rate_percent: 1.0 },
        RoyaltyTier { qualifying_count: 3, rate_percent: 2.0 },
        RoyaltyTier { qualifying_count: 3, rate_percent: 3.0 },
        RoyaltyTier { qualifying_count: 3, rate_percent: 4.0 },
    ];
```

(b) Change `leaf` to set `membership_tier: 0`, and add:

```rust
    fn leg(total_business_volume: i64, slab_pct: i64, membership_tier: i64) -> ChildFigures {
        ChildFigures {
            total_business_volume,
            slab_pct,
            membership_tier,
        }
    }
```

(c) In every existing test, change `compute_node(x, &children, SLABS, 3, 1.0)` to `compute_node(x, &children, SLABS, &UNIFORM)`. In `royalty_rate_supports_a_fractional_percent`, use `&[RoyaltyTier { qualifying_count: 3, rate_percent: 1.25 }; 4]` instead.

(d) Add the new tests:

```rust
    #[test]
    fn membership_level_name_maps_rank_to_the_draft_names() {
        assert_eq!(membership_level_name(0), "\u{2014}");
        assert_eq!(membership_level_name(1), "Gold");
        assert_eq!(membership_level_name(4), "Ace");
        assert_eq!(membership_level_name(5), "\u{2014}");
        assert_eq!(membership_level_name(-1), "\u{2014}");
    }

    #[test]
    fn no_level_below_the_gold_count() {
        let figures = compute_node(0, &[leg(10_000, 14, 0), leg(10_000, 14, 0)], SLABS, &LADDER);
        assert_eq!(figures.membership_tier, 0);
        assert_eq!(figures.royalty, 0);
    }

    #[test]
    fn gold_exactly_at_the_gold_count_pays_the_gold_rate() {
        let children = [leg(10_000, 14, 0), leg(10_000, 14, 0), leg(10_000, 14, 0)];
        let figures = compute_node(0, &children, SLABS, &LADDER);
        assert_eq!(figures.membership_tier, 1);
        assert_eq!(figures.royalty, 300, "1% of 10,000 on each of 3 top-slab legs");
    }

    #[test]
    fn platinum_needs_the_platinum_count_of_gold_legs() {
        let two_gold = [leg(30_000, 14, 1), leg(30_000, 14, 1), leg(10_000, 14, 0)];
        assert_eq!(compute_node(0, &two_gold, SLABS, &LADDER).membership_tier, 1);

        let three_gold = [leg(30_000, 14, 1), leg(30_000, 14, 1), leg(30_000, 14, 1)];
        let figures = compute_node(0, &three_gold, SLABS, &LADDER);
        assert_eq!(figures.membership_tier, 2);
        assert_eq!(
            figures.royalty, 1_800,
            "Platinum's 2% replaces Gold's 1%: 2% of 90,000"
        );
    }

    #[test]
    fn a_higher_level_leg_counts_toward_a_lower_rung() {
        // Client's own example (decision 5): 2 Platinum + 1 Gold = 3 legs at
        // Gold or higher -> Platinum; only 2 at Platinum or higher -> not Diamond.
        let children = [leg(90_000, 14, 2), leg(90_000, 14, 2), leg(30_000, 14, 1)];
        let figures = compute_node(0, &children, SLABS, &LADDER);
        assert_eq!(figures.membership_tier, 2);
        assert_eq!(figures.royalty, 4_200, "2% of 210,000");
    }

    #[test]
    fn three_diamond_legs_reach_ace_through_every_rung() {
        let children = [leg(270_000, 14, 3), leg(270_000, 14, 3), leg(270_000, 14, 3)];
        let figures = compute_node(0, &children, SLABS, &LADDER);
        assert_eq!(figures.membership_tier, 4);
        assert_eq!(figures.royalty, 32_400, "Ace's 4% of 810,000");
    }

    #[test]
    fn the_ladder_stops_at_the_first_unmet_rung() {
        // Platinum needs 5 here, so Diamond/Ace (count 1) are never reached
        // even though three Diamond legs would satisfy them on their own.
        let tiers = [
            RoyaltyTier { qualifying_count: 3, rate_percent: 1.0 },
            RoyaltyTier { qualifying_count: 5, rate_percent: 2.0 },
            RoyaltyTier { qualifying_count: 1, rate_percent: 3.0 },
            RoyaltyTier { qualifying_count: 1, rate_percent: 4.0 },
        ];
        let children = [leg(270_000, 14, 3), leg(270_000, 14, 3), leg(270_000, 14, 3)];
        assert_eq!(compute_node(0, &children, SLABS, &tiers).membership_tier, 1);
    }

    #[test]
    fn the_royalty_base_stays_top_slab_legs_only_at_every_level() {
        // A Platinum member's non-top-slab leg adds nothing to royalty
        // (Rule-10 base unchanged); it is paid through differential instead.
        let children = [
            leg(30_000, 14, 1),
            leg(30_000, 14, 1),
            leg(30_000, 14, 1),
            leg(5_000, 10, 0),
        ];
        let figures = compute_node(0, &children, SLABS, &LADDER);
        assert_eq!(figures.membership_tier, 2);
        assert_eq!(figures.royalty, 1_800);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd src-tauri && cargo test --lib m3_calc::engine`
Expected: compile errors (`RoyaltyTier`, `membership_level_name`, `membership_tier` not found; `compute_node` takes 5 arguments).

- [ ] **Step 3: Implement the engine**

In `engine.rs`:

(a) Add above `ChildFigures`:

```rust
/// Rule-47 (CR-7): the draft membership-level names, rank 1..=4 in order.
/// Display only — the database stores the rank, never a name, and names are
/// not a setting. Renaming a level is a change here and in
/// `src/lib/membership-levels.ts`, nowhere else.
pub const MEMBERSHIP_LEVEL_NAMES: [&str; 4] = ["Gold", "Platinum", "Diamond", "Ace"];

/// Rank 0 ("no level") and anything out of range render as an em dash.
pub fn membership_level_name(rank: i64) -> &'static str {
    usize::try_from(rank)
        .ok()
        .and_then(|r| r.checked_sub(1))
        .and_then(|i| MEMBERSHIP_LEVEL_NAMES.get(i))
        .copied()
        .unwrap_or("\u{2014}")
}

/// One membership level's settings (Rule-47), rank = index + 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoyaltyTier {
    pub qualifying_count: i64,
    pub rate_percent: f64,
}
```

(b) `ChildFigures`: add `#[derive(Debug, Clone, Copy)]` and the field `pub membership_tier: i64,`. Update its doc comment to mention the level.

(c) `NodeFigures`: add `pub membership_tier: i64,` after `slab_pct`.

(d) Add the rule function below `top_slab_percentage`:

```rust
/// Rule-47: rank 1 needs `tiers[0].qualifying_count` direct legs on the top
/// slab; each later rank k needs rank k-1 already held *and*
/// `tiers[k-1].qualifying_count` direct legs at rank k-1 or higher (a
/// higher-level leg has been through every rung below it). Stops at the
/// first unmet rung. Iterates `tiers` — never assumes there are four.
fn membership_tier(children: &[ChildFigures], top_slab_pct: i64, tiers: &[RoyaltyTier]) -> i64 {
    let mut rank = 0;
    for (i, tier) in tiers.iter().enumerate() {
        let qualifying = if i == 0 {
            children.iter().filter(|c| c.slab_pct == top_slab_pct).count()
        } else {
            children
                .iter()
                .filter(|c| c.membership_tier >= i as i64)
                .count()
        };
        if (qualifying as i64) < tier.qualifying_count {
            break;
        }
        rank = i as i64 + 1;
    }
    rank
}
```

(e) Replace `compute_node`'s signature, doc line and royalty block:

```rust
/// One post-order step (Rule-5): given a member's own Business Volume and
/// its direct children's current figures, compute that member's TBV
/// (Rule-6), slab (Rule-7), differential (Rule-8), membership level
/// (Rule-47), royalty (Rule-10 as amended by Rule-47, Rule-25),
/// own-Business-Volume reward (Rule-46), and Rewards (Rule-12).
pub fn compute_node(
    own_business_volume: i64,
    children: &[ChildFigures],
    slabs: &[(i64, i64)],
    tiers: &[RoyaltyTier],
) -> NodeFigures {
    // ... total_business_volume, slab_pct, differential unchanged ...

    // Rule-47: the level, then royalty at that level's rate only (decision
    // 2 — the highest level's rate replaces, never stacks). Base is
    // unchanged from Rule-10: only top-slab legs, both to count Gold and to pay.
    let top_slab_pct = top_slab_percentage(slabs);
    let membership_tier = membership_tier(children, top_slab_pct, tiers);
    let royalty: i64 = if membership_tier == 0 {
        0
    } else {
        let rate_percent = tiers[(membership_tier - 1) as usize].rate_percent;
        children
            .iter()
            .filter(|c| c.slab_pct == top_slab_pct)
            .map(|c| round_half_up_f64(rate_percent * c.total_business_volume as f64 / 100.0))
            .sum()
    };

    // ... own_reward, rewards unchanged ...

    NodeFigures {
        total_business_volume,
        slab_pct,
        membership_tier,
        differential,
        royalty,
        own_reward,
        rewards,
    }
}
```

Delete the old `qualifying` vector and its `if qualifying.len() as i64 >= royalty_min_children` block.

- [ ] **Step 4: Wire `mod.rs` so the crate compiles**

In `src-tauri/src/m3_calc/mod.rs`:

(a) Change the import to `use engine::{compute_node, ChildFigures, NodeFigures, RoyaltyTier, MEMBERSHIP_LEVEL_NAMES};`.

(b) Add below `setting_f64`:

```rust
/// Rule-47: (qualifying count, rate) settings keys per membership level,
/// rank 1 first. Rank 1 keeps Rule-10's original keys, so an upgraded
/// install needed no data migration for them. Sized off the names list, so
/// the names and the settings can never disagree on how many levels exist.
pub const ROYALTY_TIER_KEYS: [(&str, &str); MEMBERSHIP_LEVEL_NAMES.len()] = [
    ("royalty_qualifying_count", "royalty_rate_percent"),
    ("royalty_tier_2_qualifying_count", "royalty_tier_2_rate_percent"),
    ("royalty_tier_3_qualifying_count", "royalty_tier_3_rate_percent"),
    ("royalty_tier_4_qualifying_count", "royalty_tier_4_rate_percent"),
];

pub fn royalty_tiers(conn: &Connection) -> Result<Vec<RoyaltyTier>, AppError> {
    ROYALTY_TIER_KEYS
        .iter()
        .map(|(count_key, rate_key)| {
            Ok(RoyaltyTier {
                qualifying_count: setting_i64(conn, count_key)?,
                rate_percent: setting_f64(conn, rate_key)?,
            })
        })
        .collect()
}
```

(c) `direct_children_figures`: select `COALESCE(t.total_business_volume, 0), COALESCE(t.slab_pct, 0), COALESCE(t.membership_tier, 0)` and map `membership_tier: r.get(2)?`. Do the same in `snapshot_children_figures`, with `t.membership_tier` read from the snapshot alias.

(d) `upsert_totals`: add `membership_tier` to the column list, the `VALUES` list (as `?10`), the `DO UPDATE SET` list (`membership_tier = excluded.membership_tier`), and `figures.membership_tier` as the last param.

(e) `insert_snapshot`: add `membership_tier` after `rewards` in the column list, renumber `VALUES (?1 … ?13)`, and add `figures.membership_tier` after `figures.rewards` in the params.

(f) In `walk_chain`, `walk_chain_into_snapshot`, `recalculate_chain`, `recompute_open_period_rows` and `write_correction_snapshot`:
- Replace every `royalty_min_children: i64, royalty_rate_percent: f64` parameter pair with `tiers: &[RoyaltyTier]`.
- Replace every pair of `setting_i64(conn, "royalty_qualifying_count")?` / `setting_f64(conn, "royalty_rate_percent")?` loads with `let tiers = royalty_tiers(conn)?;`.
- Replace every `compute_node(bv, &children, slabs, royalty_min_children, royalty_rate_percent)` with `compute_node(bv, &children, slabs, &tiers)`, or `tiers` where it is already a slice parameter.

(g) `preview_settings_impact`: leave the candidate logic for Task 3. For now replace its two royalty loads with:

```rust
    let mut tiers = royalty_tiers(conn)?;
    if let Some(count) = candidate.royalty_qualifying_count {
        tiers[0].qualifying_count = count;
    }
    if let Some(rate) = candidate.royalty_rate_percent {
        tiers[0].rate_percent = rate;
    }
```

and call `compute_node(business_volume, &children, &slabs, &tiers)`.

- [ ] **Step 5: Update the integration-test call sites**

`src-tauri/tests/golden_scenarios.rs`:
- Change the import to `use bvconsole_lib::m3_calc::engine::{compute_node, ChildFigures, RoyaltyTier};`.
- Replace the two `ROYALTY_*` constants with:

```rust
// Rule-10's original min 3 / 1%, as four identical Rule-47 rungs — the six
// client scenarios predate membership levels and must reproduce unchanged.
const TIERS: [RoyaltyTier; 4] = [RoyaltyTier {
    qualifying_count: 3,
    rate_percent: 1.0,
}; 4];
```

- In `evaluate`, add `membership_tier: figures.membership_tier,` to the `ChildFigures` literal and call `compute_node(tree.own_bv, &children, SLABS, &TIERS)`.

`src-tauri/tests/differential_non_negativity.rs`:
- Change the import to include `RoyaltyTier`.
- Add `membership_tier: 0,` to the `ChildFigures` literal.
- Replace `compute_node(own_bv, &children, &table, 3, 1.0)` with `compute_node(own_bv, &children, &table, &[RoyaltyTier { qualifying_count: 3, rate_percent: 1.0 }; 4])`.

- [ ] **Step 6: Add a DB-level test that the chain walk stores the level**

Add to `m3_calc/mod.rs` `mod tests` (after the `totals` helper):

```rust
    fn tier(conn: &Connection, member_id: i64, period_id: i64) -> i64 {
        conn.query_row(
            "SELECT membership_tier FROM member_period_totals
             WHERE member_id = ?1 AND period_id = ?2",
            rusqlite::params![member_id, period_id],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn a_chain_write_stores_the_members_membership_level() {
        let conn = seeded();
        let period = insert_period(&conn, "2026-08");
        let parent = insert_member(&conn, None);
        for _ in 0..3 {
            let child = insert_member(&conn, Some(parent));
            insert_entry(&conn, child, "2026-08", 1_000_000); // ×100: top slab
            recalculate_chain(&conn, child, period).unwrap();
        }
        assert_eq!(tier(&conn, parent, period), 1, "3 top-slab legs at the seeded count of 3 is Gold");
    }

    #[test]
    fn a_correction_snapshot_records_the_level_from_childrens_snapshots() {
        let conn = seeded();
        conn.execute(
            "UPDATE settings SET value = '1' WHERE key = 'royalty_qualifying_count'",
            [],
        )
        .unwrap();
        let period = insert_period(&conn, "2026-08");
        let parent = insert_member(&conn, None);
        let child = insert_member(&conn, Some(parent));
        insert_snapshot_row(&conn, child, period, 1, 0, 0);
        insert_snapshot_row(&conn, parent, period, 1, 0, 0);
        insert_entry(&conn, child, "2026-08", 1_000_000); // top slab

        write_correction_snapshot(&conn, child, period).unwrap();

        let parent_tier: i64 = conn
            .query_row(
                "SELECT membership_tier FROM monthly_snapshots
                 WHERE member_id = ?1 AND period_id = ?2 AND version = 2",
                rusqlite::params![parent, period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(parent_tier, 1, "one top-slab leg at a Gold count of 1");
    }
```

- [ ] **Step 7: Run all Rust tests**

Run: `cd src-tauri && cargo test`
Expected: PASS. This includes `all_six_golden_scenarios_reproduce_exactly_through_the_real_engine` (the regression proof that four identical rungs equal the old rule) and the proptest suites.

- [ ] **Step 8: Commit**

```bash
git branch --show-current
git add src-tauri/src/m3_calc src-tauri/tests/golden_scenarios.rs src-tauri/tests/differential_non_negativity.rs
git commit -m "feat: compute membership level and pay its royalty rate"
```

---

### Task 3: Deepest-first settings recompute and preview (fixes the stale-child-slab bug)

**Files:**
- Modify: `src-tauri/src/m3_calc/mod.rs`:
  - `direct_children_figures`
  - `open_period_member_ids` → `open_period_member_ids_deepest_first`
  - `recompute_open_period_rows` doc comment
  - `CandidateSettings`, `MemberImpact`, `LiveFigures`, `live_figures_for_open_period`, `preview_settings_impact`

**Interfaces:**
- Consumes: Task 2's `royalty_tiers`, `RoyaltyTier`, `ChildFigures: Copy`.
- Produces:
  - `CandidateSettings` gains `royalty_tier2_qualifying_count: Option<i64>`, `royalty_tier2_rate_percent: Option<f64>`, and the same pair for 3 and 4 (serde camelCase: `royaltyTier2QualifyingCount` etc.).
  - `MemberImpact` gains `membership_tier_before: i64`, `membership_tier_after: i64`.

- [ ] **Step 1: Write the failing tests**

Add to `m3_calc/mod.rs` `mod tests`:

```rust
    fn differential(conn: &Connection, member_id: i64, period_id: i64) -> i64 {
        conn.query_row(
            "SELECT differential FROM member_period_totals WHERE member_id = ?1 AND period_id = ?2",
            rusqlite::params![member_id, period_id],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn a_slab_edit_recomputes_children_before_their_parent() {
        // Regression: the parent's row is inserted first (lower rowid), so
        // member-id/rowid order visited it before its child and computed its
        // differential against the child's *old* slab.
        let conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let parent = insert_member(&conn, None);
        let child = insert_member(&conn, Some(parent));
        insert_entry(&conn, parent, &month, 200_000);
        recalculate_chain(&conn, parent, period).unwrap();
        insert_entry(&conn, child, &month, 100_000); // 4% (40,000 <= 100,000)
        recalculate_chain(&conn, child, period).unwrap();
        // Parent TBV 300,000 -> 8%; (8 - 4) × 100,000 / 100.
        assert_eq!(differential(&conn, parent, period), 4_000);

        // Child drops to 2%; parent stays at 8%.
        conn.execute(
            "UPDATE slab_table SET threshold = 200000 WHERE percentage = 4",
            [],
        )
        .unwrap();
        recalculate_open_period(&conn).unwrap();

        assert_eq!(
            differential(&conn, parent, period),
            6_000,
            "(8 - 2) × 100,000 / 100 — the child's *new* slab"
        );
    }

    /// root -> mid -> low -> leaf with counts 1/2/1/1: leaf is top slab and
    /// low, mid and root are all Gold (Platinum needs 2 Gold legs; each has
    /// one). Lowering Platinum's count to 1 then makes mid Platinum and root
    /// Diamond — but root only reaches Diamond by reading mid's *new* level,
    /// so a parent-before-child walk leaves root stuck at Platinum. Root's
    /// own entry is recorded first so its row is visited first by any
    /// rowid/member-id order.
    fn four_generation_chain(conn: &Connection, month: &str, period: i64) -> [i64; 4] {
        conn.execute(
            "UPDATE settings SET value = '1' WHERE key IN (
                'royalty_qualifying_count', 'royalty_tier_3_qualifying_count',
                'royalty_tier_4_qualifying_count')",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE settings SET value = '2' WHERE key = 'royalty_tier_2_qualifying_count'",
            [],
        )
        .unwrap();
        let root = insert_member(conn, None);
        let mid = insert_member(conn, Some(root));
        let low = insert_member(conn, Some(mid));
        let leaf = insert_member(conn, Some(low));
        insert_entry(conn, root, month, 1_000);
        recalculate_chain(conn, root, period).unwrap();
        insert_entry(conn, leaf, month, 1_000_000); // top slab
        recalculate_chain(conn, leaf, period).unwrap();
        [root, mid, low, leaf]
    }

    #[test]
    fn lowering_the_platinum_count_moves_parent_and_grandparent_in_one_save() {
        let conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let [root, mid, low, _] = four_generation_chain(&conn, &month, period);
        assert_eq!(
            (tier(&conn, root, period), tier(&conn, mid, period), tier(&conn, low, period)),
            (1, 1, 1)
        );

        conn.execute(
            "UPDATE settings SET value = '1' WHERE key = 'royalty_tier_2_qualifying_count'",
            [],
        )
        .unwrap();
        recalculate_open_period(&conn).unwrap();

        assert_eq!(tier(&conn, low, period), 1, "low's only leg holds no level: still Gold");
        assert_eq!(tier(&conn, mid, period), 2, "one Gold leg now meets Platinum's 1");
        assert_eq!(
            tier(&conn, root, period),
            3,
            "root reaches Diamond only by reading mid's *new* Platinum in the same save"
        );
    }

    #[test]
    fn a_level_count_preview_matches_what_the_save_settles_at() {
        let conn = seeded();
        let month = this_month();
        let period = insert_period(&conn, &month);
        let [root, mid, _, _] = four_generation_chain(&conn, &month, period);

        let preview = preview_settings_impact(
            &conn,
            CandidateSettings {
                royalty_tier2_qualifying_count: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
        let predicted = |id: i64| {
            preview
                .affected_members
                .iter()
                .find(|m| m.member_id == id)
                .unwrap_or_else(|| panic!("member {id} must be listed as affected"))
        };
        assert_eq!(predicted(mid).membership_tier_before, 1);
        assert_eq!(predicted(mid).membership_tier_after, 2);
        assert_eq!(predicted(root).membership_tier_before, 1);
        assert_eq!(
            predicted(root).membership_tier_after,
            3,
            "the preview must use mid's *predicted* level, not its live one"
        );

        conn.execute(
            "UPDATE settings SET value = '1' WHERE key = 'royalty_tier_2_qualifying_count'",
            [],
        )
        .unwrap();
        recalculate_open_period(&conn).unwrap();
        for id in [root, mid] {
            let (_, _, settled_rewards) = totals(&conn, id, period);
            assert_eq!(settled_rewards, predicted(id).rewards_after, "T-M7.3-6 for member {id}");
            assert_eq!(tier(&conn, id, period), predicted(id).membership_tier_after);
        }
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd src-tauri && cargo test --lib m3_calc::tests`
Expected: compile error (`royalty_tier2_qualifying_count` and `membership_tier_before` don't exist). After Step 3 adds only the struct fields, the three tests fail on the assertions: 4,000 vs 6,000, root stuck at Platinum (2) instead of Diamond (3), and the preview's root prediction 2 instead of 3.

- [ ] **Step 3: Implement**

(a) Add a shared depth CTE near the top of `mod.rs`, after `chain_to_root`:

```rust
/// Every member's depth below its root, from the real introducer links — not
/// the stored `members.level`, so ordering can never drift from the tree.
/// Rule-37 (introducers never change) keeps this stable within a recompute.
const MEMBER_DEPTH_CTE: &str = "WITH RECURSIVE depth(id, d) AS (
        SELECT id, 0 FROM members WHERE introducer_member_id IS NULL
        UNION ALL
        SELECT m.id, depth.d + 1 FROM members m JOIN depth ON m.introducer_member_id = depth.id
     )";
```

(b) Split `direct_children_figures` so the preview can know each child's id:

```rust
fn direct_children_with_ids(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<Vec<(i64, ChildFigures)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT m.id, COALESCE(t.total_business_volume, 0), COALESCE(t.slab_pct, 0),
                COALESCE(t.membership_tier, 0)
         FROM members m
         LEFT JOIN member_period_totals t ON t.member_id = m.id AND t.period_id = ?2
         WHERE m.introducer_member_id = ?1",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![member_id, period_id], |r| {
            Ok((
                r.get(0)?,
                ChildFigures {
                    total_business_volume: r.get(1)?,
                    slab_pct: r.get(2)?,
                    membership_tier: r.get(3)?,
                },
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Rule-28: every direct child, active or not — no filtering here. A child
/// with no row yet for this period (nothing in its subtree has been
/// entered this period) defaults to zero, which is its correct TBV.
fn direct_children_figures(
    conn: &Connection,
    member_id: i64,
    period_id: i64,
) -> Result<Vec<ChildFigures>, AppError> {
    Ok(direct_children_with_ids(conn, member_id, period_id)?
        .into_iter()
        .map(|(_, figures)| figures)
        .collect())
}
```

(c) Replace `open_period_member_ids` with:

```rust
/// Deepest member first: a child's slab feeds its parent's differential
/// (Rule-8) and a child's level feeds its parent's level (Rule-47), so every
/// parent must read children already recomputed under the new settings.
fn open_period_member_ids_deepest_first(
    conn: &Connection,
    period_id: i64,
) -> Result<Vec<i64>, AppError> {
    let mut stmt = conn.prepare(&format!(
        "{MEMBER_DEPTH_CTE}
         SELECT t.member_id FROM member_period_totals t
         JOIN depth ON depth.id = t.member_id
         WHERE t.period_id = ?1
         ORDER BY depth.d DESC, t.member_id"
    ))?;
    let rows = stmt
        .query_map([period_id], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
```

Call it from `recompute_open_period_rows`. Replace the "Order doesn't matter here…" sentences in `recalculate_open_period`'s doc comment with: "Rows are recomputed deepest first (`open_period_member_ids_deepest_first`) — a child's slab and level both feed its parent's figures, so a parent must never read a child still holding pre-edit values."

(d) `CandidateSettings`: add the six fields after `royalty_rate_percent`:

```rust
    pub royalty_tier2_qualifying_count: Option<i64>,
    pub royalty_tier2_rate_percent: Option<f64>,
    pub royalty_tier3_qualifying_count: Option<i64>,
    pub royalty_tier3_rate_percent: Option<f64>,
    pub royalty_tier4_qualifying_count: Option<i64>,
    pub royalty_tier4_rate_percent: Option<f64>,
```

and a resolver below `resolve_candidate_slabs`. It replaces Task 2's temporary level-1-only override block:

```rust
/// Each level's candidate count/rate where given, the live setting otherwise.
fn resolve_candidate_tiers(
    conn: &Connection,
    candidate: &CandidateSettings,
) -> Result<Vec<RoyaltyTier>, AppError> {
    let overrides = [
        (candidate.royalty_qualifying_count, candidate.royalty_rate_percent),
        (candidate.royalty_tier2_qualifying_count, candidate.royalty_tier2_rate_percent),
        (candidate.royalty_tier3_qualifying_count, candidate.royalty_tier3_rate_percent),
        (candidate.royalty_tier4_qualifying_count, candidate.royalty_tier4_rate_percent),
    ];
    Ok(royalty_tiers(conn)?
        .into_iter()
        .zip(overrides)
        .map(|(live, (count, rate))| RoyaltyTier {
            qualifying_count: count.unwrap_or(live.qualifying_count),
            rate_percent: rate.unwrap_or(live.rate_percent),
        })
        .collect())
}
```

(e) `MemberImpact`: add `pub membership_tier_before: i64, pub membership_tier_after: i64,` after `royalty_after`. `LiveFigures`: add `membership_tier: i64`. Change `live_figures_for_open_period`'s SQL to select `t.membership_tier` in the sixth position (map `membership_tier: r.get(5)?`) and to order deepest first:

```rust
    let mut stmt = conn.prepare(&format!(
        "{MEMBER_DEPTH_CTE}
         SELECT m.id, m.name, t.slab_pct, t.royalty, t.rewards, t.membership_tier
         FROM member_period_totals t
         JOIN members m ON m.id = t.member_id
         JOIN depth ON depth.id = t.member_id
         WHERE t.period_id = ?1
         ORDER BY depth.d DESC, t.member_id"
    ))?;
```

(f) `preview_settings_impact`:
- Replace the tiers setup with `let tiers = resolve_candidate_tiers(conn, &candidate)?;`.
- Replace the loop body's `children` and the `affected` check as below.
- Add `use std::collections::HashMap;` at the top of the file.
- Update the doc comment's T-M7.3-6 sentence to say both paths now walk deepest first, and each parent reads its children's *predicted* figures.

```rust
    // Deepest first (live_figures_for_open_period's order), substituting each
    // already-predicted child — the real save does the same walk against
    // rows it has just rewritten, so the two stay identical (T-M7.3-6).
    let mut predicted: HashMap<i64, ChildFigures> = HashMap::new();
    for live in live_figures_for_open_period(conn, period_id)? {
        let business_volume = business_volume_of(conn, live.member_id, &period_month)?;
        let children: Vec<ChildFigures> =
            direct_children_with_ids(conn, live.member_id, period_id)?
                .into_iter()
                .map(|(id, figures)| predicted.get(&id).copied().unwrap_or(figures))
                .collect();
        let after = compute_node(business_volume, &children, &slabs, &tiers);
        predicted.insert(
            live.member_id,
            ChildFigures {
                total_business_volume: after.total_business_volume,
                slab_pct: after.slab_pct,
                membership_tier: after.membership_tier,
            },
        );

        // ... rewards/royalty-earner tallies unchanged ...

        if live.rewards != after.rewards
            || live.slab_pct != after.slab_pct
            || live.membership_tier != after.membership_tier
            || (live.royalty > 0) != (after.royalty > 0)
        {
            affected.push(MemberImpact {
                // ... existing fields ...
                membership_tier_before: live.membership_tier,
                membership_tier_after: after.membership_tier,
            });
        }
    }
```

- [ ] **Step 4: Run all Rust tests**

Run: `cd src-tauri && cargo test`
Expected: PASS. This includes the pre-existing `preview_matches_exactly_what_a_real_save_produces` and `a_slab_table_edit_is_reflected_on_the_next_recalculation`.

- [ ] **Step 5: Commit**

```bash
git branch --show-current
git add src-tauri/src/m3_calc/mod.rs
git commit -m "fix: recompute and preview settings changes deepest member first"
```

---

### Task 4: Month close snapshots the level, then resets it

**Files:**
- Modify: `src-tauri/src/m5_close/mod.rs:142-211` (`write_period_close_snapshots`, `zero_period_totals`) and its tests

**Interfaces:**
- Consumes: Task 1's column.
- Produces: none new.

- [ ] **Step 1: Write the failing test**

Add to `m5_close/mod.rs` `mod tests`, using the existing `seeded`, `insert_member(conn, introducer)`, `insert_period(conn, month, status)` and `insert_totals(conn, member_id, period_id, bv, slab_pct)` helpers:

```rust
    #[test]
    fn close_snapshots_the_membership_level_then_resets_it_for_next_month() {
        // Decision 1: monthly, reset at close, never carried forward.
        let conn = seeded();
        let member = insert_member(&conn, None);
        let period = insert_period(&conn, "2026-08", "awaiting_close");
        insert_totals(&conn, member, period, 100_000, 14);
        conn.execute(
            "UPDATE member_period_totals SET membership_tier = 2 WHERE member_id = ?1",
            [member],
        )
        .unwrap();

        write_period_close_snapshots(&conn, period, "2026-08-31").unwrap();
        zero_period_totals(&conn, period).unwrap();

        let snapshot_tier: i64 = conn
            .query_row(
                "SELECT membership_tier FROM monthly_snapshots WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![member, period],
                |r| r.get(0),
            )
            .unwrap();
        let live_tier: i64 = conn
            .query_row(
                "SELECT membership_tier FROM member_period_totals WHERE member_id = ?1 AND period_id = ?2",
                rusqlite::params![member, period],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(snapshot_tier, 2, "the closed month keeps the level it showed");
        assert_eq!(live_tier, 0, "nothing is left live to carry forward");
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test --lib m5_close::tests::close_snapshots_the_membership_level`
Expected: FAIL. `snapshot_tier` is 0 (not copied) and `live_tier` is 2 (not reset).

- [ ] **Step 3: Implement**

`write_period_close_snapshots`:
- Add `COALESCE(t.membership_tier, 0)` as the tenth selected column.
- Widen the tuple type to 10 `i64`/`bool` elements (`(i64, bool, i64, i64, i64, i64, i64, i64, i64, i64)`) and add `r.get(9)?`.
- Destructure `membership_tier` last.
- Add `membership_tier` after `rewards` in the `INSERT` column list, renumber `VALUES (?1, ?2, 1, ?3 … ?12)`, and pass `membership_tier` after `rewards` in the params.

`zero_period_totals`: add `membership_tier = 0` to the `SET` list. Extend its doc comment: "Volume, TBV, Rewards, royalty, membership level (Rule-47 — levels never carry into the next month)."

- [ ] **Step 4: Run all Rust tests**

Run: `cd src-tauri && cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git branch --show-current
git add src-tauri/src/m5_close/mod.rs
git commit -m "feat: snapshot membership level at close and reset it"
```

---

### Task 5: Settings read, validate and save all four levels

**Files:**
- Modify: `src-tauri/src/m7_settings/mod.rs`:
  - `Settings` (205-224)
  - `get_settings` (~284-306)
  - `SettingsPatch` (~315-328)
  - `apply_settings_patch` (329-393)
  - `update_settings` (~403-421)
  - tests
- Modify: `src-tauri/tests/contract.rs` (settings round-trip test)
- Modify: `src-tauri/src/commands.rs:475-477` (doc comment only)

**Interfaces:**
- Consumes: `m3_calc::ROYALTY_TIER_KEYS`, `m3_calc::recalculate_open_period`.
- Produces: `Settings` and `SettingsPatch` gain `royalty_tier{2,3,4}_qualifying_count` (`i64` / `Option<i64>`) and `royalty_tier{2,3,4}_rate_percent` (`f64` / `Option<f64>`). Serialized as `royaltyTier2QualifyingCount` etc.

- [ ] **Step 1: Write the failing tests**

Add to `m7_settings/mod.rs` `mod tests`:

```rust
    #[test]
    fn get_settings_returns_every_membership_levels_seeded_values() {
        let settings = get_settings(&seeded()).unwrap();
        assert_eq!(
            (settings.royalty_tier2_qualifying_count, settings.royalty_tier2_rate_percent),
            (3, 1.0)
        );
        assert_eq!(
            (settings.royalty_tier3_qualifying_count, settings.royalty_tier3_rate_percent),
            (3, 1.0)
        );
        assert_eq!(
            (settings.royalty_tier4_qualifying_count, settings.royalty_tier4_rate_percent),
            (3, 1.0)
        );
    }

    #[test]
    fn update_settings_saves_and_audits_a_later_levels_count_and_rate() {
        let conn = seeded();
        let updated = update_settings(
            &conn,
            SettingsPatch {
                royalty_tier3_qualifying_count: Some(5),
                royalty_tier3_rate_percent: Some(2.5),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(updated.royalty_tier3_qualifying_count, 5);
        assert_eq!(updated.royalty_tier3_rate_percent, 2.5);
        let audited: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE field IN
                    ('royalty_tier_3_qualifying_count', 'royalty_tier_3_rate_percent')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(audited, 2);
    }

    #[test]
    fn update_settings_refuses_a_non_positive_count_on_any_level() {
        let err = update_settings(
            &seeded(),
            SettingsPatch {
                royalty_tier4_qualifying_count: Some(0),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn update_settings_refuses_a_negative_rate_on_any_level() {
        let err = update_settings(
            &seeded(),
            SettingsPatch {
                royalty_tier2_rate_percent: Some(-0.5),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn a_later_levels_rate_change_recalculates_the_open_period() {
        let conn = seeded();
        let month = chrono::Local::now().format("%Y-%m").to_string();
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
            [&month],
        )
        .unwrap();
        let period: i64 = conn.last_insert_rowid();
        conn.execute(
            "UPDATE settings SET value = '1' WHERE key IN (
                'royalty_qualifying_count', 'royalty_tier_2_qualifying_count')",
            [],
        )
        .unwrap();
        // root -> mid -> leaf(top slab): mid is Gold, root is Platinum.
        let root = insert_member(&conn, None);
        let mid = insert_member(&conn, Some(root));
        let leaf = insert_member(&conn, Some(mid));
        conn.execute(
            "INSERT INTO business_volume_entries
                (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, 1000000, ?2 || '-15', ?2, ?2 || '-15')",
            rusqlite::params![leaf, month],
        )
        .unwrap();
        m3_calc::recalculate_chain(&conn, leaf, period).unwrap();
        let royalty_of = |id: i64| -> i64 {
            conn.query_row(
                "SELECT royalty FROM member_period_totals WHERE member_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap()
        };
        let before = royalty_of(root);

        update_settings(
            &conn,
            SettingsPatch {
                royalty_tier2_rate_percent: Some(3.0),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(before, 10_000, "1% of 1,000,000 at Platinum's seeded 1%");
        assert_eq!(royalty_of(root), 30_000, "3% of 1,000,000 once Platinum's rate is 3%");
    }
```

Check the existing `insert_member` helper at `m7_settings/mod.rs:734`. It takes `(conn, introducer: Option<i64>)`. If it hardcodes `level`, that is fine, because ordering never reads `level`.

In `src-tauri/tests/contract.rs`, find the `get_settings` round-trip test (search for `commands::get_settings`). Add after its first `get_settings` call:

```rust
    assert_eq!(before.royalty_tier2_qualifying_count, 3);
    assert_eq!(before.royalty_tier4_rate_percent, before.royalty_rate_percent);
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd src-tauri && cargo test --lib m7_settings && cargo test --test contract`
Expected: compile errors (`royalty_tier2_qualifying_count` etc. unknown).

- [ ] **Step 3: Implement**

(a) `Settings`: add after `royalty_rate_percent`:

```rust
    pub royalty_tier2_qualifying_count: i64,
    pub royalty_tier2_rate_percent: f64,
    pub royalty_tier3_qualifying_count: i64,
    pub royalty_tier3_rate_percent: f64,
    pub royalty_tier4_qualifying_count: i64,
    pub royalty_tier4_rate_percent: f64,
```

`get_settings`: add after the `royalty_rate_percent` line:

```rust
        royalty_tier2_qualifying_count: setting_i64(conn, "royalty_tier_2_qualifying_count")?,
        royalty_tier2_rate_percent: setting_f64(conn, "royalty_tier_2_rate_percent")?,
        royalty_tier3_qualifying_count: setting_i64(conn, "royalty_tier_3_qualifying_count")?,
        royalty_tier3_rate_percent: setting_f64(conn, "royalty_tier_3_rate_percent")?,
        royalty_tier4_qualifying_count: setting_i64(conn, "royalty_tier_4_qualifying_count")?,
        royalty_tier4_rate_percent: setting_f64(conn, "royalty_tier_4_rate_percent")?,
```

(b) `SettingsPatch`: add the same six names as `Option<i64>` / `Option<f64>` after `royalty_rate_percent`.

(c) Add above `apply_settings_patch`:

```rust
/// Rule-47: the patch's (count, rate) per membership level, rank 1 first —
/// same order as `m3_calc::ROYALTY_TIER_KEYS`.
fn royalty_tier_patches(patch: &SettingsPatch) -> [(Option<i64>, Option<f64>); 4] {
    [
        (patch.royalty_qualifying_count, patch.royalty_rate_percent),
        (patch.royalty_tier2_qualifying_count, patch.royalty_tier2_rate_percent),
        (patch.royalty_tier3_qualifying_count, patch.royalty_tier3_rate_percent),
        (patch.royalty_tier4_qualifying_count, patch.royalty_tier4_rate_percent),
    ]
}
```

(d) In `apply_settings_patch`, replace the V7.4 block with:

```rust
    // V7.4 (extended by Rule-47): every level's qualifying count is a
    // positive whole number and every rate is zero or more.
    for (i, (count, rate)) in royalty_tier_patches(patch).into_iter().enumerate() {
        let prefix = if i == 0 {
            "royalty".to_string()
        } else {
            format!("royaltyTier{}", i + 1)
        };
        if count.is_some_and(|c| c <= 0) {
            return Err(AppError::Validation {
                field: format!("{prefix}QualifyingCount"),
                message: "The royalty qualifying count must be a positive whole number.".into(),
            });
        }
        if rate.is_some_and(|r| r.is_nan() || r < 0.0) {
            return Err(AppError::Validation {
                field: format!("{prefix}RatePercent"),
                message: "The royalty rate must be zero or more.".into(),
            });
        }
    }
```

Replace the two `royalty_qualifying_count` / `royalty_rate_percent` write blocks, in the same position, with:

```rust
    for ((count, rate), (count_key, rate_key)) in royalty_tier_patches(patch)
        .into_iter()
        .zip(m3_calc::ROYALTY_TIER_KEYS)
    {
        if let Some(v) = count {
            write_setting(conn, count_key, &v.to_string())?;
            write_audit(conn, 0, count_key, &v.to_string())?;
        }
        if let Some(v) = rate {
            write_setting(conn, rate_key, &v.to_string())?;
            write_audit(conn, 0, rate_key, &v.to_string())?;
        }
    }
```

(e) `update_settings`:

```rust
    let recalculates = royalty_tier_patches(&patch)
        .iter()
        .any(|(count, rate)| count.is_some() || rate.is_some());
```

Update its doc comment ("Only a royalty qualifying-count or rate change, at any membership level, recalculates…"). Do the same in `commands.rs:475-477`.

- [ ] **Step 4: Run all Rust tests**

Run: `cd src-tauri && cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git branch --show-current
git add src-tauri/src/m7_settings/mod.rs src-tauri/src/commands.rs src-tauri/tests/contract.rs
git commit -m "feat: read, validate and save royalty settings for every membership level"
```

---

### Task 6: Member detail and PDF show the level

**Files:**
- Modify: `src-tauri/src/m4_search/mod.rs`:
  - `PeriodTotals`, `totals_for_period` (74-107)
  - delete `royalty_rate_percent` (126-137)
  - `RoyaltyLine` (164-169), `MemberDetail` (191-198), `get_member_detail` (235-300)
  - tests
- Modify: `src-tauri/src/m4_search/pdf.rs`:
  - `rewards_detail_rows` (255-296)
  - every `RoyaltyLine { .. }` literal in its tests (lines ~758, ~918, ~971)

**Interfaces:**
- Consumes: `m3_calc::royalty_tiers`, `m3_calc::engine::membership_level_name`.
- Produces:
  - `MemberDetail.membership_tier: i64` (JSON `membershipTier`).
  - `RoyaltyLine.membership_tier: i64` (JSON `membershipTier`).
  - `RoyaltyLine.rate_percent` is now the member's level's rate, or level 1's rate when rank 0.

- [ ] **Step 1: Write the failing tests**

`m4_search/mod.rs` `mod tests`:

```rust
    #[test]
    fn member_detail_reports_the_level_and_that_levels_royalty_rate() {
        let conn = seeded();
        conn.execute(
            "UPDATE settings SET value = '1' WHERE key IN (
                'royalty_qualifying_count', 'royalty_tier_2_qualifying_count')",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE settings SET value = '2.5' WHERE key = 'royalty_tier_2_rate_percent'",
            [],
        )
        .unwrap();
        let period = insert_period(&conn, "2026-08");
        let root = insert_member(&conn, "Root", None);
        let mid = insert_member(&conn, "Mid", Some(root));
        let leaf = insert_member(&conn, "Leaf", Some(mid));
        insert_entry(&conn, leaf, "2026-08", 1_000_000); // top slab
        recalculate_chain(&conn, leaf, period).unwrap();

        let detail = get_member_detail(&conn, root, Some("2026-08")).unwrap();
        assert_eq!(detail.membership_tier, 2, "Mid is Gold, so Root is Platinum");
        let royalty = detail.rewards.royalty.expect("root has a leg");
        assert_eq!(royalty.membership_tier, 2);
        assert_eq!(royalty.rate_percent, 2.5);
    }
```

`m4_search/pdf.rs` `mod tests`:
- Add `membership_tier: 0,` to every existing `RoyaltyLine { .. }` literal. The existing expectation `"Royalty \u{2014} 1 of 2 legs qualifying"` must keep passing.
- Add:

```rust
    #[test]
    fn rewards_detail_rows_names_the_level_and_its_rate_on_the_royalty_row() {
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine {
                own_business_volume: 0,
                own_slab_pct: 14,
                amount: 0,
            },
            differentials: vec![
                differential_line(1, "A", 3_000_000, 14, 14, 0),
                differential_line(2, "B", 3_000_000, 14, 14, 0),
                differential_line(3, "C", 3_000_000, 14, 14, 0),
            ],
            royalty: Some(RoyaltyLine {
                qualifying_children: 3,
                membership_tier: 2,
                rate_percent: 2.0,
                amount: 180_000,
            }),
            rewards_total: 180_000,
        };
        let rows = rewards_detail_rows(&rewards);
        assert_eq!(
            rows[4].description,
            "Royalty \u{2014} Platinum at 2% \u{2014} 3 of 3 legs qualifying"
        );
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd src-tauri && cargo test --lib m4_search`
Expected: compile errors (no `membership_tier` field on `MemberDetail` / `RoyaltyLine`).

- [ ] **Step 3: Implement `m4_search/mod.rs`**

- `PeriodTotals`: add `membership_tier: i64`. In `totals_for_period`, select `COALESCE(t.membership_tier, 0)` as the seventh column and map `membership_tier: r.get(6)?`.
- Delete the `royalty_rate_percent` function.
- `RoyaltyLine`: add `pub membership_tier: i64,` after `qualifying_children`.
- `MemberDetail`: add `pub membership_tier: i64,` after `slab_pct`.
- In `get_member_detail`, replace the royalty block and set the new field:

```rust
    let royalty = if children.is_empty() {
        None
    } else {
        let top_slab = top_slab_percentage(conn)?;
        let qualifying_children = children.iter().filter(|c| c.slab_pct == top_slab).count() as i64;
        // Rule-47: the rate of the level actually held; below Gold, show
        // Gold's rate — the one the member would earn on qualifying.
        let tiers = crate::m3_calc::royalty_tiers(conn)?;
        let rate_index = (totals.membership_tier.max(1) - 1) as usize;
        Some(RoyaltyLine {
            qualifying_children,
            membership_tier: totals.membership_tier,
            rate_percent: tiers[rate_index].rate_percent,
            amount: totals.royalty,
        })
    };
```

and add `membership_tier: totals.membership_tier,` to the `MemberDetail { .. }` literal.

- [ ] **Step 4: Implement `pdf.rs`**

Change the royalty row in `rewards_detail_rows`:

```rust
    if let Some(royalty) = &rewards.royalty {
        // Rule-47: name the level and its rate once one is held.
        let level = if royalty.membership_tier > 0 {
            format!(
                "{} at {}% \u{2014} ",
                membership_level_name(royalty.membership_tier),
                royalty.rate_percent
            )
        } else {
            String::new()
        };
        rows.push(RewardRow {
            description: format!(
                "Royalty \u{2014} {level}{} of {} legs qualifying",
                royalty.qualifying_children,
                rewards.differentials.len()
            ),
            business_volume: None,
            amount: format_amount(royalty.amount),
            emphasized: true,
        });
    }
```

Add `use crate::m3_calc::engine::membership_level_name;` to `pdf.rs`'s imports.

- [ ] **Step 5: Run all Rust tests**

Run: `cd src-tauri && cargo test`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git branch --show-current
git add src-tauri/src/m4_search
git commit -m "feat: show membership level on member detail and its PDF"
```

---

### Task 7: Monthly extract "Membership" column

**Files:**
- Modify: `src-tauri/src/m6_reports/mod.rs`:
  - `MemberExportRow` (105-122)
  - `row_to_export_row` (129-156)
  - `load_live_export_rows` (161-190)
  - `load_snapshot_export_rows` (199-233)
  - `write_export_xlsx` match (318-349)
  - `OptionalColumn` enum/parse/header (756-812)
  - test `export_row` helper, and `optional_column_parse_round_trips_every_settings_screen_key`
- Modify: `src/lib/export-columns.ts`

**Interfaces:**
- Consumes: `m3_calc::engine::membership_level_name`.
- Produces: optional export column key `"membership_level"`, header `"Membership"`.

- [ ] **Step 1: Write the failing tests**

In `m6_reports/mod.rs` tests:
- Add `"membership_level",` to the `keys` array in `optional_column_parse_round_trips_every_settings_screen_key`.
- Add `membership_tier: 0,` to the `export_row` helper's literal.
- Add:

```rust
    #[test]
    fn membership_level_column_has_its_own_header() {
        assert_eq!(
            OptionalColumn::parse("membership_level").unwrap().header(),
            "Membership"
        );
    }

    #[test]
    fn export_rows_carry_the_membership_level_from_live_totals_and_snapshots() {
        let conn = seeded();
        let open = insert_period(&conn, "2026-08", "open");
        let closed = insert_period(&conn, "2026-07", "closed");
        let member = insert_member(&conn, "Levelled", true, None);
        insert_totals(&conn, member, open, 100_000, 100_000);
        conn.execute(
            "UPDATE member_period_totals SET membership_tier = 3 WHERE member_id = ?1",
            [member],
        )
        .unwrap();
        insert_snapshot(&conn, member, closed, 1, 100_000, 100_000, true);
        conn.execute(
            "UPDATE monthly_snapshots SET membership_tier = 1 WHERE member_id = ?1",
            [member],
        )
        .unwrap();

        assert_eq!(load_live_export_rows(&conn, open).unwrap()[0].membership_tier, 3);
        assert_eq!(load_snapshot_export_rows(&conn, closed).unwrap()[0].membership_tier, 1);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd src-tauri && cargo test --lib m6_reports`
Expected: compile error (`membership_tier` not a field) and a parse failure for `membership_level`.

- [ ] **Step 3: Implement**

- `MemberExportRow`: add `membership_tier: i64` after `royalty`. `row_to_export_row`: add a `membership_tier: i64` parameter last and set the field. It now has 8 parameters, so add `#[allow(clippy::too_many_arguments)]` above it (CI runs clippy with `-D warnings`).
- `load_live_export_rows`: append `, COALESCE(t.membership_tier, 0)` to the selected columns and pass `r.get(16)?` as the new last argument.
- `load_snapshot_export_rows`: append `, s.membership_tier` and pass `r.get(16)?`.
- `OptionalColumn`: add a `MembershipLevel` variant after `RoyaltyEarned`, `"membership_level" => Self::MembershipLevel,` to `parse`, and `Self::MembershipLevel => "Membership",` to `header`.
- `write_export_xlsx` match:

```rust
                OptionalColumn::MembershipLevel => {
                    write_cell(worksheet, r, c, membership_level_name(row.membership_tier), format)?
                }
```

- Add `use crate::m3_calc::engine::membership_level_name;` to the module imports.

Do **not** add it to `redownload_backup`'s fixed column set. That set mirrors the prototype.

`src/lib/export-columns.ts`: insert after the `royalty_earned` entry:

```ts
  { key: "membership_level", label: "Membership" },
```

- [ ] **Step 4: Run the tests**

Run: `cd src-tauri && cargo test && cd .. && npm run test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git branch --show-current
git add src-tauri/src/m6_reports/mod.rs src/lib/export-columns.ts
git commit -m "feat: add Membership as an optional monthly extract column"
```

---

### Task 8: Golden Scenario 7 — the ladder to Ace

**Files:**
- Modify: `src-tauri/tests/golden_scenarios.rs` (append)

**Interfaces:**
- Consumes: `fixtures::MemberFixture`, `evaluate`-style recursion, `RoyaltyTier`.

Hand-worked figures (real units, default slab table, counts 3/3/3/3, rates 1/2/3/4%). Every node's own Business Volume is 0 except the top-slab leaves at 10,000:

| Node | Legs | TBV | Slab | Level | Royalty |
|---|---|---|---|---|---|
| Top leaf | — | 10,000 | 14% | — | 0 |
| Gold | 3 top leaves | 30,000 | 14% | Gold | 1% × 30,000 = 300 |
| Platinum | 3 Gold | 90,000 | 14% | Platinum | 2% × 90,000 = 1,800 |
| Diamond | 3 Platinum | 270,000 | 14% | Diamond | 3% × 270,000 = 8,100 |
| Ace | 3 Diamond | 810,000 | 14% | Ace | 4% × 810,000 = 32,400 |

Every differential is 0: each parent shares its legs' 14% slab (Rule-11).

- [ ] **Step 1: Write the test**

Append to `src-tauri/tests/golden_scenarios.rs`:

```rust
// --- Scenario 7 (CR-7/Rule-47): the membership ladder, top leaf to Ace. ---
// Constructed, not client-supplied — awaiting client confirmation of the
// figures (03-business-rules.md Rule-47). Kept out of `golden_scenarios()`,
// which holds the client's own six.

const S7_TOP: MemberFixture = MemberFixture {
    name: "top",
    own_bv: 10_000,
    children: &[],
};
const S7_GOLD: MemberFixture = MemberFixture {
    name: "gold",
    own_bv: 0,
    children: &[S7_TOP, S7_TOP, S7_TOP],
};
const S7_PLATINUM: MemberFixture = MemberFixture {
    name: "platinum",
    own_bv: 0,
    children: &[S7_GOLD, S7_GOLD, S7_GOLD],
};
const S7_DIAMOND: MemberFixture = MemberFixture {
    name: "diamond",
    own_bv: 0,
    children: &[S7_PLATINUM, S7_PLATINUM, S7_PLATINUM],
};
const S7_ACE: MemberFixture = MemberFixture {
    name: "ace",
    own_bv: 0,
    children: &[S7_DIAMOND, S7_DIAMOND, S7_DIAMOND],
};
const S7_TIERS: [RoyaltyTier; 4] = [
    RoyaltyTier { qualifying_count: 3, rate_percent: 1.0 },
    RoyaltyTier { qualifying_count: 3, rate_percent: 2.0 },
    RoyaltyTier { qualifying_count: 3, rate_percent: 3.0 },
    RoyaltyTier { qualifying_count: 3, rate_percent: 4.0 },
];

fn evaluate_with(
    tree: &MemberFixture,
    tiers: &[RoyaltyTier],
) -> bvconsole_lib::m3_calc::engine::NodeFigures {
    let children: Vec<ChildFigures> = tree
        .children
        .iter()
        .map(|child| {
            let f = evaluate_with(child, tiers);
            ChildFigures {
                total_business_volume: f.total_business_volume,
                slab_pct: f.slab_pct,
                membership_tier: f.membership_tier,
            }
        })
        .collect();
    compute_node(tree.own_bv, &children, SLABS, tiers)
}

#[test]
fn scenario_7_membership_ladder_reaches_ace_at_every_rungs_own_rate() {
    let expected = [
        (&S7_TOP, 10_000, 0, 0),
        (&S7_GOLD, 30_000, 1, 300),
        (&S7_PLATINUM, 90_000, 2, 1_800),
        (&S7_DIAMOND, 270_000, 3, 8_100),
        (&S7_ACE, 810_000, 4, 32_400),
    ];
    for (tree, tbv, level, royalty) in expected {
        let f = evaluate_with(tree, &S7_TIERS);
        assert_eq!(
            (f.total_business_volume, f.slab_pct, f.membership_tier, f.royalty, f.differential),
            (tbv, 14, level, royalty, 0),
            "node '{}'",
            tree.name
        );
    }
}
```

`MemberFixture`'s `name` field is `#[allow(dead_code)]` in `fixtures/mod.rs`. Reading it here is fine and needs no change there.

- [ ] **Step 2: Run the test**

Run: `cd src-tauri && cargo test --test golden_scenarios`
Expected: PASS (the engine landed in Task 2; this is the reconciliation proof). If it fails, the hand-worked table above is the reference. Fix the engine, not the numbers.

- [ ] **Step 3: Commit**

```bash
git branch --show-current
git add src-tauri/tests/golden_scenarios.rs
git commit -m "test: add golden scenario 7 for the membership ladder"
```

---

### Task 9: Frontend — level names, IPC types, Settings royalty table, recalc dialog

**Files:**
- Create: `src/lib/membership-levels.ts`, `src/lib/membership-levels.test.ts`
- Create: `src/components/recalc-warning-dialog.test.tsx`
- Modify: `src/lib/ipc/entities.ts:163-181` (`Settings`)
- Modify: `src/lib/ipc/m3-calc.ts` (`CandidateSettings`, `MemberImpact`)
- Modify: `src/screens/settings.tsx:323-395` (`RoyaltyCard`)
- Modify: `src/components/recalc-warning-dialog.tsx:79-89`
- Modify: `e2e/specs/settings.e2e.js` (one extra test)

**Interfaces:**
- Consumes: Task 3/5 JSON shapes.
- Produces:
  - `MEMBERSHIP_LEVEL_NAMES: readonly ["Gold","Platinum","Diamond","Ace"]`
  - `membershipLevelName(rank: number): string`

- [ ] **Step 1: Write the failing tests**

`src/lib/membership-levels.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { MEMBERSHIP_LEVEL_NAMES, membershipLevelName } from "./membership-levels";

describe("membershipLevelName", () => {
  it("maps rank 1..4 to the draft names and anything else to an em dash", () => {
    expect(MEMBERSHIP_LEVEL_NAMES).toEqual(["Gold", "Platinum", "Diamond", "Ace"]);
    expect(membershipLevelName(0)).toBe("—");
    expect(membershipLevelName(1)).toBe("Gold");
    expect(membershipLevelName(4)).toBe("Ace");
    expect(membershipLevelName(5)).toBe("—");
  });
});
```

`src/components/recalc-warning-dialog.test.tsx`:

```tsx
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { RecalcWarningDialog } from "./recalc-warning-dialog";
import type { MemberImpact, SettingsImpactPreview } from "@/lib/ipc/m3-calc";

function impact(overrides: Partial<MemberImpact>): MemberImpact {
  return {
    memberId: 1,
    memberName: "Asha",
    rewardsBefore: 0,
    rewardsAfter: 0,
    slabPctBefore: 14,
    slabPctAfter: 14,
    royaltyBefore: 0,
    royaltyAfter: 0,
    membershipTierBefore: 0,
    membershipTierAfter: 0,
    ...overrides,
  };
}

function preview(members: MemberImpact[]): SettingsImpactPreview {
  return {
    rewardsBefore: 0,
    rewardsAfter: 1,
    royaltyEarnerCountBefore: 1,
    royaltyEarnerCountAfter: 1,
    affectedMembers: members,
  };
}

describe("RecalcWarningDialog — royalty changes", () => {
  it("shows a level move by name", () => {
    render(
      <RecalcWarningDialog
        open
        onOpenChange={() => {}}
        kind="royalty"
        monthName="September 2026"
        preview={preview([impact({ membershipTierBefore: 2, membershipTierAfter: 1 })])}
        onConfirm={() => {}}
      />,
    );
    expect(screen.getByText("Platinum → Gold")).toBeInTheDocument();
  });

  it("shows royalty before → after when only a rate moved", () => {
    render(
      <RecalcWarningDialog
        open
        onOpenChange={() => {}}
        kind="royalty"
        monthName="September 2026"
        preview={preview([
          impact({
            membershipTierBefore: 2,
            membershipTierAfter: 2,
            royaltyBefore: 180000,
            royaltyAfter: 270000,
          }),
        ])}
        onConfirm={() => {}}
      />,
    );
    expect(screen.getByText("1800.00 → 2700.00")).toBeInTheDocument();
  });
});
```

(`centsToDisplay` formats 180000 as `1800.00`. This matches member-detail.test.tsx's `/1000\.00 at 14%/` for 100000.)

- [ ] **Step 2: Run the tests to verify they fail**

Run: `npx vitest run src/lib/membership-levels.test.ts src/components/recalc-warning-dialog.test.tsx`
Expected: FAIL (module not found; type errors on `membershipTierBefore`; "Starts" rendered instead).

- [ ] **Step 3: Implement the constant**

`src/lib/membership-levels.ts`:

```ts
// Rule-47 (CR-7): draft membership-level names, rank 1..4. Display only —
// the backend stores and sends the rank, and names are not a setting.
// Must match MEMBERSHIP_LEVEL_NAMES in src-tauri/src/m3_calc/engine.rs.
export const MEMBERSHIP_LEVEL_NAMES = ["Gold", "Platinum", "Diamond", "Ace"] as const;

export function membershipLevelName(rank: number): string {
  return MEMBERSHIP_LEVEL_NAMES[rank - 1] ?? "—";
}
```

- [ ] **Step 4: Extend the IPC types**

`src/lib/ipc/entities.ts`, in `Settings` after `royaltyRatePercent: number;`:

```ts
  royaltyTier2QualifyingCount: number;
  royaltyTier2RatePercent: number;
  royaltyTier3QualifyingCount: number;
  royaltyTier3RatePercent: number;
  royaltyTier4QualifyingCount: number;
  royaltyTier4RatePercent: number;
```

`src/lib/ipc/m3-calc.ts`:
- Add the same six fields as optional (`?:`) to `CandidateSettings`.
- Add to `MemberImpact`:

```ts
  membershipTierBefore: number;
  membershipTierAfter: number;
```

- [ ] **Step 5: Update the recalc dialog**

In `recalc-warning-dialog.tsx`:
- Add `import { membershipLevelName } from "@/lib/membership-levels";`.
- Replace the royalty branch of the cell (lines 83-87):

```tsx
                          {kind === "slab"
                            ? `${m.slabPctBefore}% → ${m.slabPctAfter}%`
                            : m.membershipTierBefore !== m.membershipTierAfter
                              ? `${membershipLevelName(m.membershipTierBefore)} → ${membershipLevelName(m.membershipTierAfter)}`
                              : `${centsToDisplay(m.royaltyBefore)} → ${centsToDisplay(m.royaltyAfter)}`}
```

- Change the header cell to `{kind === "slab" ? "Slab" : "Membership / royalty"}`.
- Update the file's top comment: the royalty column now shows a level move, or the royalty figure when only a rate moved.

- [ ] **Step 6: Rewrite `RoyaltyCard` in `settings.tsx`**

Add `import { membershipLevelName } from "@/lib/membership-levels";` to the imports. Replace `RoyaltyCard` (lines 325-395) with:

```tsx
// Rule-47: one row per membership level, rank 1 first. Rank 1 keeps Rule-10's
// original keys — and the `royalty-min`/`royalty-rate` ids the E2E suite uses.
const ROYALTY_TIER_FIELDS = [
  { count: "royaltyQualifyingCount", rate: "royaltyRatePercent" },
  { count: "royaltyTier2QualifyingCount", rate: "royaltyTier2RatePercent" },
  { count: "royaltyTier3QualifyingCount", rate: "royaltyTier3RatePercent" },
  { count: "royaltyTier4QualifyingCount", rate: "royaltyTier4RatePercent" },
] as const;

function RoyaltyCard({
  settings,
  onSettingsChange,
}: {
  settings: SettingsData;
  onSettingsChange: (next: SettingsData) => void;
}) {
  const toast = useToast();
  const recalcWarning = useRecalcWarning();
  const [rows, setRows] = useState(() =>
    ROYALTY_TIER_FIELDS.map((f) => ({
      count: String(settings[f.count]),
      rate: String(settings[f.rate]),
    })),
  );
  const [saving, setSaving] = useState(false);

  function setRow(index: number, patch: Partial<{ count: string; rate: string }>) {
    setRows((prev) => prev.map((row, i) => (i === index ? { ...row, ...patch } : row)));
  }

  async function save() {
    const patch: CandidateSettings = {};
    for (const [i, field] of ROYALTY_TIER_FIELDS.entries()) {
      const count = Number(rows[i].count);
      const rate = Number(rows[i].rate);
      if (!Number.isFinite(count) || !Number.isFinite(rate)) {
        toast.add({ title: "Enter valid numbers", type: "danger" });
        return;
      }
      patch[field.count] = count;
      patch[field.rate] = rate;
    }
    await recalcWarning.request("royalty", patch, async () => {
      setSaving(true);
      try {
        const updated = await updateSettings(patch);
        onSettingsChange(updated);
        toast.add({ title: "Royalty settings saved", type: "success" });
      } finally {
        setSaving(false);
      }
    });
  }

  return (
    <SectionCard
      id="settings-card-royalty"
      title="Royalty"
      description="Membership levels, and the royalty rate each one earns"
    >
      <div className="grid grid-cols-[1fr_1fr_1fr] items-center gap-x-3 gap-y-2">
        <span className="text-label">Membership</span>
        <span className="text-label">Minimum qualifying legs</span>
        <span className="text-label">Royalty rate (%)</span>
        {ROYALTY_TIER_FIELDS.map((_, i) => {
          const name = membershipLevelName(i + 1);
          const suffix = i === 0 ? "" : `-${i + 1}`;
          return (
            <div key={name} className="contents">
              <span className="text-body font-[650]">{name}</span>
              <Input
                id={`royalty-min${suffix}`}
                aria-label={`${name} minimum qualifying legs`}
                value={rows[i].count}
                onChange={(e) => setRow(i, { count: e.target.value })}
              />
              <Input
                id={`royalty-rate${suffix}`}
                aria-label={`${name} royalty rate (%)`}
                value={rows[i].rate}
                onChange={(e) => setRow(i, { rate: e.target.value })}
              />
            </div>
          );
        })}
      </div>
      <InputHint className="mt-2">
        {membershipLevelName(1)} needs that many direct legs on the top slab. Each later level
        needs that many direct legs at the level before it or higher. A member earns only their
        highest level&apos;s rate, on each top-slab leg&apos;s Total Business Volume. Levels are
        worked out afresh every month.
      </InputHint>
      <Button className="mt-3.5" disabled={saving} onClick={save}>
        Save royalty settings
      </Button>
      {recalcWarning.dialog}
    </SectionCard>
  );
}
```

`CandidateSettings` is already imported in `settings.tsx` (used by `useRecalcWarning`). If `tsc` says otherwise, add it to the `@/lib/ipc/m3-calc` import. `updateSettings(patch)` type-checks because every `CandidateSettings` field is a same-typed `Settings` field.

- [ ] **Step 7: Add the E2E step**

Append inside `describe("Settings", …)` in `e2e/specs/settings.e2e.js`, after the existing royalty test:

```js
  it("saves a later membership level's rate", async () => {
    await navigateTo("Settings");
    await $("#royalty-rate-2").waitForExist({ timeout: 3000 });
    await $("#royalty-rate-2").setValue("2");
    await $("button=Save royalty settings").click();
    const dialog = $('div[role="dialog"]');
    await dialog.waitForExist({ timeout: 3000 });
    const confirmButton = dialog.$("button*=Save and re-work");
    await confirmButton.waitForEnabled({ timeout: 3000 });
    await confirmButton.click();
    await $("h2*=Royalty settings saved").waitForExist({ timeout: 3000 });
  });
```

- [ ] **Step 8: Run the frontend checks**

Run: `npm run test && npx tsc --noEmit && npm run lint && npm run vocab-grep`
Expected: all pass. E2E runs in CI only (`tauri-driver` isn't installed locally), so note that it hasn't run locally.

- [ ] **Step 9: Commit**

```bash
git branch --show-current
git add src/lib/membership-levels.ts src/lib/membership-levels.test.ts src/lib/ipc/entities.ts src/lib/ipc/m3-calc.ts src/components/recalc-warning-dialog.tsx src/components/recalc-warning-dialog.test.tsx src/screens/settings.tsx e2e/specs/settings.e2e.js
git commit -m "feat: edit every membership level's royalty settings"
```

---

### Task 10: Frontend — member detail shows the level

**Files:**
- Modify: `src/lib/ipc/m4-search.ts:16,32-39`
- Modify: `src/screens/member-detail.tsx:184-194` (stat cards), `:270-284` (royalty row)
- Modify: `src/screens/member-detail.test.tsx`

**Interfaces:**
- Consumes: Task 6's `membershipTier` on `MemberDetail` and on `rewards.royalty`, and Task 9's `membershipLevelName`.

- [ ] **Step 1: Write the failing tests**

In `member-detail.test.tsx`'s `detailFor`:
- Add `membershipTier: 0,` after `slabPct: 14,`.
- Change the royalty to `royalty: { qualifyingChildren: 3, membershipTier: 0, ratePercent: 5, amount: 5000 },`.

The existing assertion `/3 of 1 legs qualifying/` stays valid for rank 0. Add inside `describe("MemberDetail — rewards breakdown", …)`:

```tsx
  it("names the membership level in its stat card and on the royalty row", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    const base = detailFor(CHILD_MEMBER);
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(
      detailFor(CHILD_MEMBER, {
        membershipTier: 2,
        rewards: {
          ...base.rewards,
          royalty: { qualifyingChildren: 3, membershipTier: 2, ratePercent: 2, amount: 5000 },
        },
      }),
    );
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    expect(screen.getByText("Membership")).toBeInTheDocument();
    expect(screen.getByText("Platinum")).toBeInTheDocument();
    expect(screen.getByText(/Platinum at 2% — 3 of 1 legs qualifying/)).toBeInTheDocument();
  });

  it("shows an em dash when no level is held", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(detailFor(CHILD_MEMBER));
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    expect(screen.getByText("—")).toBeInTheDocument();
  });
```

If `getByText("—")` matches more than one node (for example an existing placeholder elsewhere), switch to `within(screen.getByText("Membership").parentElement!).getByText("—")` and import `within` from `@testing-library/react`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `npx vitest run src/screens/member-detail.test.tsx`
Expected: type error / FAIL (no `membershipTier` on the type; no Membership card).

- [ ] **Step 3: Implement**

`src/lib/ipc/m4-search.ts`:
- Change the `royalty` line to:

```ts
  royalty: {
    qualifyingChildren: number;
    membershipTier: number;
    ratePercent: number;
    amount: number;
  } | null;
```

- Add `membershipTier: number;` after `slabPct: number;` in `MemberDetail`.

`src/screens/member-detail.tsx`:
- Add `import { membershipLevelName } from "@/lib/membership-levels";`.
- Change the stat grid class to `"mt-4 grid grid-cols-2 gap-3 sm:grid-cols-5"`, and insert after the Slab card:

```tsx
        <StatCard label="Membership" value={membershipLevelName(detail.membershipTier)} />
```

- Change the royalty row's muted span:

```tsx
                          <span className="font-normal text-muted-text">
                            —{" "}
                            {rewards.royalty.membershipTier > 0 &&
                              `${membershipLevelName(rewards.royalty.membershipTier)} at ${rewards.royalty.ratePercent}% — `}
                            {rewards.royalty.qualifyingChildren} of {rewards.differentials.length}{" "}
                            legs qualifying (top slab)
                          </span>
```

- [ ] **Step 4: Run the frontend checks**

Run: `npm run test && npx tsc --noEmit && npm run lint && npm run vocab-grep`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git branch --show-current
git add src/lib/ipc/m4-search.ts src/screens/member-detail.tsx src/screens/member-detail.test.tsx
git commit -m "feat: show membership level on the member detail screen"
```

---

### Task 11: Business-rules and product docs

**Files:**
- Modify: `documents/implementation-readiness/03-business-rules.md` (Rule-10 at line 81; append Rule-47 after Rule-46, ~line 350)
- Modify: `PRODUCT.md` (Capabilities "In scope" paragraph; Operating Context bullet on calculation)

- [ ] **Step 1: Amend Rule-10**

Change its heading to `### Rule-10 — Royalty qualification **[AMENDED 24 Sep 2026, CR-7 — see Rule-47]**`. Append this line to its body:

`**Amended:** qualifying under this rule is now membership level 1 (Gold). The paid rate is the rate of the member's highest level (Rule-47), not a single rate; the base (top-slab direct legs' TBV) is unchanged.`

- [ ] **Step 2: Add Rule-47** after Rule-46's block:

```markdown
### Rule-47 — Membership levels **[NEW — client requirement, CR-7]**
**Rule:** Each period, every member holds a membership level, rank 0–4 (none / Gold / Platinum / Diamond / Ace — draft names, not settings). Gold: ≥ N₁ direct legs on the top slab. Each later level k: holds level k−1 **and** ≥ Nₖ direct legs at level k−1 or higher. `Royalty(x)` = 0 at rank 0, otherwise `rate(level(x)) × Σ TBV(c)` over x's top-slab direct legs — the highest level's rate replaces the lower ones, never stacks.
**Source:** Client change request **CR-7**, 24 September 2026 — decisions: monthly (reset at close, never carried forward); highest rate replaces; one count and one rate per level (N₁…N₄, rate₁…rate₄, all in Settings); sequential ladder; a higher-level leg counts toward a lower rung.
**Applies to:** M3, M5 (snapshot + reset at close), M7 (settings), M4/M6 (display).
**Implementation impact:** `member_period_totals`/`monthly_snapshots` gain `membership_tier` (rank). Level 1 keeps `royalty_qualifying_count`/`royalty_rate_percent`; levels 2–4 add `royalty_tier_{2,3,4}_qualifying_count`/`_rate_percent`. A level depends on children's levels, so a settings-change recompute runs deepest member first.
**Test requirement:** Scenario 7 (constructed — **awaiting client confirmation of the figures**): with counts 3/3/3/3 and rates 1/2/3/4%, top leaves of 10,000 → Gold 300, Platinum 1,800, Diamond 8,100, Ace 32,400 royalty; every differential 0. Plus the client's own example: 2 Platinum legs + 1 Gold leg → Platinum.
```

- [ ] **Step 3: Update `PRODUCT.md`**

- In the Operating Context calculation bullet, append: "Royalty is paid by monthly membership level (Gold, Platinum, Diamond, Ace — added 24 Sep 2026, CR-7): each level needs a configured number of direct legs at the level below, and pays its own configured rate."
- In "In scope", change "royalty rate/qualifying count" to "royalty qualifying count and rate for each membership level".

- [ ] **Step 4: Check vocabulary**

Run: `grep -n -i -E "subscription|\btier\b" PRODUCT.md documents/implementation-readiness/03-business-rules.md`
Expected: no matches in user-facing prose. The key names inside backticks contain `tier`; that is fine, because they are internal identifiers.

- [ ] **Step 5: Commit**

```bash
git branch --show-current
git add documents/implementation-readiness/03-business-rules.md PRODUCT.md
git commit -m "docs: record Rule-47 membership levels (CR-7)"
```

---

### Task 12: Full verification and push

- [ ] **Step 1: Run the whole CI surface locally**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd .. && npm run lint && npm run test && npm run build
```

Expected: every command exits 0. `npm run build` runs `vocab-grep` and `tsc` as well. If `cargo fmt --check` fails, run `cargo fmt`, re-run, and commit as `style: cargo fmt`.

- [ ] **Step 2: Manual smoke test (optional, requires the app)**

`npm run tauri dev`, then:
1. Settings → Royalty shows four rows (Gold/Platinum/Diamond/Ace).
2. Edit Platinum's rate; the dialog appears; save.
3. Open a member with legs; the Membership card and the royalty row show the level.

- [ ] **Step 3: Push the branch; do not open a PR**

```bash
git branch --show-current   # feature/membership-levels
git push -u origin feature/membership-levels
```

Stop here. The user opens the PR to `develop` manually.
