# Membership Levels — Design

**Date:** 2026-09-24
**Status:** Approved for planning
**Requirement refs:** CR-7 (new), extends Rule-10 and Rule-25, new Rule-47 (see
`documents/implementation-readiness/03-business-rules.md`)

## 1. Problem

Today a member earns royalty when at least `royalty_qualifying_count` (default 3) of
their direct legs are on the top slab (Rule-10). The client wants that condition to
become the first rung of a ladder of membership levels, each with its own qualifying
count and its own royalty rate, all editable from the Settings screen:

| Rank | Level (draft name) | Qualifies when |
|---|---|---|
| 0 | — (none) | below Gold |
| 1 | Gold | ≥ N₁ direct legs on the top slab (today's royalty condition) |
| 2 | Platinum | holds Gold **and** ≥ N₂ direct legs at Gold or higher |
| 3 | Diamond | holds Platinum **and** ≥ N₃ direct legs at Platinum or higher |
| 4 | Ace | holds Diamond **and** ≥ N₄ direct legs at Diamond or higher |

Level names are **drafts**, not settings. They are not editable from the Settings
screen; renaming is a code change (one constant per language, §4.4). The number of
levels (four) is fixed by the requirement. Every count and rate is a setting — no
count, rate or threshold is hardcoded (Product Principle 4).

## 2. Confirmed decisions (client, 2026-09-24)

1. **Monthly, not permanent.** A member's level is derived per period from that
   period's figures, exactly like slab. A member can be Gold in August and hold no
   level in September. Closed months keep the level their snapshot recorded.
2. **Highest level's rate replaces.** A member earns royalty at the rate of their
   highest level only — never a sum of rates. The base is unchanged from Rule-10: the
   Total Business Volume of each direct leg on the top slab.
3. **One qualifying count per level.** N₁…N₄ are four separate settings.
4. **Ladder is sequential.** Rule 3 of the original request ("Diamond if 3+ Diamond
   legs") was a typo for Platinum legs.
5. **"At the required level or higher" counts.** A Platinum leg has been through Gold,
   so it counts as a Gold leg. Example: 2 Platinum legs + 1 Gold leg = 3 legs at Gold
   or higher → the member is Platinum (with N₂ = 3).

Carried over unchanged:
- Inactive legs count (Rule-28 — every direct child, active or not).
- "Top slab" is the highest-percentage row in the slab table (Rule-10/Rule-27).
- Royalty still stacks up a chain (Rule-25): every member is assessed independently
  against their own direct legs.
- No monotonicity check across level rates, same accepted-risk stance as slab
  percentages (Rule-41/ADR-009).

## 3. Business rule (new Rule-47, amends Rule-10)

```
top(x)        = direct legs c of x with slab%(c) = top slab percentage
Level(x)      = 0
if |top(x)| ≥ N₁:                                 Level(x) = 1
for k in 2..=4:
    if Level(x) = k-1 and |{c : Level(c) ≥ k-1}| ≥ Nₖ:  Level(x) = k
Royalty(x)    = 0                                   if Level(x) = 0
              = Σ rateₗₑᵥₑₗ₍ₓ₎ × TBV(c) for c in top(x)  otherwise
```

- Rule-10 with Level 1's rate is today's behaviour exactly: when every level's rate
  equals the current `royalty_rate_percent`, no Rewards figure changes (§6.1 test).
- Rule-11 (royalty and differential never double-pay) is untouched — the base is
  still top-slab legs only.
- `Level(x)` depends on children's levels, so it has the same bottom-up dependency
  shape as TBV (Rule-5). This matters for the settings-change recompute (§4.3).

## 4. Architecture

### 4.1 Data model — migration `0002_membership_tier.sql`

```sql
ALTER TABLE member_period_totals ADD COLUMN membership_tier INTEGER NOT NULL DEFAULT 0;
ALTER TABLE monthly_snapshots    ADD COLUMN membership_tier INTEGER NOT NULL DEFAULT 0;

-- Only on an already-seeded database. db/mod.rs runs migrations *before*
-- seed::run, and seed_settings skips entirely when settings is non-empty —
-- an unconditional insert here would suppress all 16 default settings on a
-- brand-new install.
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

- Stored value is the **rank** (0–4), never a name — renaming a level never needs a
  migration.
- Existing closed snapshots keep `membership_tier = 0` and display "—": that is what
  those months showed. No backfill.
- Upgrade defaults copy the installation's current royalty rate into levels 2–4, so
  an upgrade changes no Rewards figure until the client edits a rate.

### 4.2 Settings keys

| Level | Qualifying count key | Rate key |
|---|---|---|
| 1 Gold | `royalty_qualifying_count` (existing) | `royalty_rate_percent` (existing) |
| 2 Platinum | `royalty_tier_2_qualifying_count` | `royalty_tier_2_rate_percent` |
| 3 Diamond | `royalty_tier_3_qualifying_count` | `royalty_tier_3_rate_percent` |
| 4 Ace | `royalty_tier_4_qualifying_count` | `royalty_tier_4_rate_percent` |

- Level 1 keeps the existing keys — no data migration, no rename.
- Numbered keys follow the sibling `level_2_width` convention; no draft name appears
  in any key.
- `seed.rs` gains the six keys (defaults: count 3, rate 1 — matching the existing
  Level 1 seed); inventory becomes 22 rows.
- IPC DTOs (`Settings`, `SettingsPatch`, `CandidateSettings`) gain camelCase fields
  `royaltyTier{2,3,4}QualifyingCount` / `royaltyTier{2,3,4}RatePercent`, same shape as
  `level2Width`.
- Validation (V7.4 extended): every qualifying count is a positive whole number; every
  rate is ≥ 0. Each write is audited through the existing `write_audit`.
- Any change to these eight keys triggers the existing recalc-warning + open-period
  recompute path, same as a royalty change today.

### 4.3 Calculation engine (`m3_calc`)

`engine.rs` stays pure:

```rust
pub struct ChildFigures {
    pub total_business_volume: i64,
    pub slab_pct: i64,
    pub membership_tier: i64,        // new
}

pub struct RoyaltyTier {             // one per level, rank = index + 1
    pub qualifying_count: i64,
    pub rate_percent: f64,
}

pub struct NodeFigures { /* existing fields */ pub membership_tier: i64 }

pub fn compute_node(
    own_business_volume: i64,
    children: &[ChildFigures],
    slabs: &[(i64, i64)],
    tiers: &[RoyaltyTier],           // replaces royalty_min_children + royalty_rate_percent
) -> NodeFigures
```

The engine iterates `tiers` by index — it never assumes there are four, and never
reads a name.

`mod.rs` changes:
- `royalty_tiers(conn)` reads the eight keys once per recompute (replaces the two
  `setting_*` calls at every call site).
- `direct_children_figures` also selects `COALESCE(t.membership_tier, 0)`.
- `upsert_totals` and every snapshot insert write `membership_tier`.
- **Ordering fix 1 — `recompute_open_period_rows`.** Its current comment says order
  doesn't matter because slab settings never affect TBV. That no longer holds: a
  child's level feeds its parent's level. Rows are recomputed deepest first
  (`ORDER BY members.level DESC`; `level` is fixed at creation and introducers never
  change, Rule-37), so every parent reads its children's already-updated level.
- **Ordering fix 2 — `preview_settings_impact`.** It currently computes each member
  from children's *live* figures. It must instead walk deepest first and substitute
  each child's *predicted* level (held in an in-memory map) — otherwise the preview
  diverges from what the save actually writes (T-M7.3-6). `MemberImpact` gains
  `membership_tier_before/after`; `SettingsImpactPreview` gains per-level member
  counts before/after.
- `recalculate_chain` and the closed-month correction walk already go member → root,
  so each parent already reads a fresh child; they only take the new `tiers` argument.

### 4.4 Level names

- Rust: `pub const MEMBERSHIP_LEVEL_NAMES: [&str; 4] = ["Gold", "Platinum", "Diamond", "Ace"];`
  in `m3_calc/engine.rs` — used by the PDF and the monthly extract.
- TypeScript: the same list in one constant in `src/lib/` — used by screens.
- Rank 0 renders as "—".
- A Rust test asserts `MEMBERSHIP_LEVEL_NAMES.len()` equals the number of tier setting
  pairs, so the list and the settings can't drift apart.

### 4.5 Screens and outputs

| Surface | Change |
|---|---|
| Settings → Royalty card (`settings.tsx`) | Becomes a 4-row table: level name (read-only), qualifying count, rate %. One save button, one recalc-warning dialog. Helper text states the ladder rule. |
| Recalc warning dialog (`recalc-warning-dialog.tsx`) | For a royalty change, adds "Members per membership level: before → after" and a level column in the affected-members table. |
| Member detail (`member-detail.tsx`) | Membership pill beside slab; royalty line reads "Royalty — {level}, {rate}% — {n} of {m} legs on top slab". |
| Member detail PDF (`m4_search/pdf.rs`) | Membership line in the figures block; royalty line carries the level name. |
| Monthly extract (`m6_reports`) | New optional column `membership_level` / "Membership" (`export-columns.ts` + `OptionalColumn`). |

**Deliberately unchanged:**
- Structure tree and full hierarchy window — nodes show exactly name, number and own
  Business Volume (Rule-45/FR-2).
- Yearly-average and low-contribution extracts — an average of a monthly level has no
  meaning.
- Home screen, Business Volume entry, month close flow, backup/restore, auth.

### 4.6 Vocabulary

"Membership" and the four draft level names are client-supplied and used as-is. The
words "subscription" and "tier" never appear in any visible string (settings key
names are internal). `scripts/vocabulary-grep.mjs` needs no change.

## 5. Files affected

**Rust:** `m3_calc/engine.rs`, `m3_calc/mod.rs`, `db/migrations.rs`,
`db/migrations/0002_membership_tier.sql` (new), `db/seed.rs`, `m7_settings/mod.rs`,
`m5_close/mod.rs`, `m2_entries/mod.rs`, `m1_members/mod.rs`, `m6_reports/mod.rs`,
`m4_search/mod.rs`, `m4_search/pdf.rs`, `tests/fixtures/mod.rs`,
`tests/golden_scenarios.rs`, `tests/differential_non_negativity.rs`, `tests/contract.rs`.

**Frontend:** `screens/settings.tsx`, `screens/member-detail.tsx`,
`components/recalc-warning-dialog.tsx`, `lib/export-columns.ts`,
`lib/ipc/entities.ts`, `lib/ipc/m3-calc.ts`, `lib/ipc/m4-search.ts`,
`lib/ipc/m7-settings.ts`, one new constant in `src/lib/`, plus their tests.

**Docs:** `documents/implementation-readiness/03-business-rules.md` (Rule-10 amended,
Rule-47 new, Scenario 7), `PRODUCT.md` (capabilities line).

## 6. Testing

### 6.1 Engine (pure, `engine.rs`)
- No level below N₁ top-slab legs; Gold exactly at N₁.
- Each rung exactly at Nₖ and at Nₖ − 1.
- "Or higher" counting: 2 Platinum + 1 Gold legs → Platinum (N₂ = 3).
- Sequential ladder: 3 Diamond legs with N₁…N₄ = 3 → Ace.
- Highest rate replaces: Platinum member paid at rate₂ only.
- Regression: with all four rates equal to the old single rate, every existing
  royalty test and all six golden scenarios produce identical figures.
- Settings-driven only: tests pass counts/rates as data; no literal in `engine.rs`
  outside tests.

### 6.2 Golden Scenario 7 (fixtures + `golden_scenarios.rs`)
A four-generation tree built so its root reaches Ace, with hand-worked level and
royalty at every node, reconciled through the real engine.

### 6.3 Database (`m3_calc/mod.rs`, `db/`)
- Migration on an already-seeded DB adds six keys copying the current rate and does
  not touch existing values; fresh install still seeds all 22 rows (guards the
  migration/seed ordering trap).
- Raising N₂ in Settings drops a parent **and** its parent's level in one save
  (ordering fix 1).
- `preview_settings_impact` output equals what the save writes, including levels
  (ordering fix 2).
- Month close and correction snapshots carry `membership_tier`.

### 6.4 Contract / frontend
- `tests/contract.rs`: new settings fields round-trip.
- Vitest: Settings royalty table saves all eight values and shows the dialog; member
  detail renders the pill and royalty line; "—" for rank 0.
- E2E: settings spec covers editing a level row. A four-level tree is single-month, so
  it is seedable through the UI, but deep — added only if seeding time stays within
  the existing suite's timeout.

## 7. Out of scope

- Permanent / highest-ever level (rejected, decision 1).
- Stacked rates (rejected, decision 2).
- Editable level names or a configurable number of levels.
- Showing the level on tree nodes, home screen, or the yearly/low-contribution extracts.
- Backfilling levels into snapshots closed before this change.
