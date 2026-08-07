# Data Model Specification

Source: `architecture.md` §8 (data model, ER diagram, full DDL in Appendix A). All ten entities below already exist in the architecture document's DDL — this deliverable validates that DDL against the requirements/business-rules baseline (deliverable 03) and documents it in the task's required format, correcting the one field range that changed since the DDL was drafted (member ID lower bound).

Storage engine: SQLite via `rusqlite`, encrypted at rest with SQLCipher (ADR-003). All monetary/volume figures use **fixed-point integers, stored as the real value × 100** (ADR-004), never floats — this is what makes Rule-22's "no intermediate rounding" guarantee possible in practice.

---

## Entity: `members`

**Purpose:** One row per network member (root + all descendants). The hierarchy itself.
**Lifecycle:** Created once via `add_member`/`create_root_member`; **never removed, under any circumstance** (Rule-28, Rule-42) — and never omitted from an export either; can be deactivated/reactivated indefinitely, deactivation being display-only with no calculation effect; most fields editable via `edit_member`; `introducer_member_id` is immutable after insert (Rule-37). There is no delete path in the schema, the API, or the UI, by client requirement.
**Retention:** Permanent, no expiry.
**Security sensitivity:** High — contains name, phone, address, email (personal data under DPDP Act 2023). Encrypted at rest via SQLCipher; never written to filenames or logs in plaintext.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK, 100001–999999, randomly allocated, never released | **Corrected range** — Rule-35 |
| `name` | TEXT | Yes | — | |
| `phone` | TEXT | Yes | UNIQUE across active **and** inactive members | Rule-34 |
| `email` | TEXT | No | — | |
| `address` | TEXT | Yes | — | |
| `introducer_member_id` | INTEGER | Conditionally | FK → `members.id`, NULL only for the root; immutable after insert | Rule-30, Rule-37 |
| `level` | INTEGER | Yes | Cached at creation, never recomputed | Derived from introducer's level + 1; root = 1 |
| `is_active` | BOOLEAN | Yes | Default true | **Display-only flag** — zero effect on any calculation (Rule-28, corrected) |
| `joining_date` | DATE | Yes | Auto-set on creation | |
| `consent_given` | BOOLEAN | Yes | Must be true to save (Rule-40) | New field, DPDP-driven |
| `consent_date` | DATE | Yes (when consent_given) | Auto-captured on save | |
| `created_at` | TIMESTAMP | Yes | Auto-set | |

**Relationships:** Self-referencing (`introducer_member_id` → `members.id`) forms the tree. One-to-many with `business_volume_entries`, `member_period_totals`, `monthly_snapshots`, `audit_log`.
**Indexes:** PK on `id`; unique index on `phone` (uniqueness under Rule-34 **and** the lookup index for phone search under Rule-44); index on `introducer_member_id` (chain-upward traversal, Rule-26); index on `name` (search, FR-1).

**Phone as a search key (Rule-44, added 7 Aug 2026 — CR-1).** `phone` is now matched by `search_members`, not merely constrained. Matching is a substring comparison on the **canonical key** (digits, then an international prefix or trunk zero dropped) with a four-digit floor (V4.4), so the unique index accelerates an exact or prefix match while a mid-number match is a scan. At the NFR-2 ceiling of 25,000 rows that scan sits comfortably inside NFR-1's two-second budget. **Do not add a stored canonical-key shadow column speculatively** — if a real profile ever demands it, that is the cheapest fix, but it is not needed at this scale. The stored value keeps whatever formatting the administrator typed; normalisation happens only at comparison time.
**Related requirements:** FR-1, FR-2, FR-3, FR-4, Rule-1, Rule-2, Rule-28, Rule-30, Rule-32, Rule-34, Rule-35, Rule-37, Rule-40.

---

## Entity: `business_volume_entries`

**Purpose:** The append-only ledger of every Business Volume figure ever recorded. This is the source of truth `member_period_totals` and `monthly_snapshots` are derived from.
**Lifecycle:** Created via `record_entry`; corrected in place via `edit_entry` (Rule-39) with every change captured in `audit_log` — never hard-deleted.
**Retention:** Permanent (this is the record that makes the "figure can always be explained" promise possible).
**Security sensitivity:** Medium — no direct personal data beyond the member link, but sensitive as the basis of every financial-equivalent figure in the system.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK | |
| `member_id` | INTEGER | Yes | FK → `members.id` | |
| `amount` | INTEGER | Yes | CHECK `> 0` (Rule-16a), stored ×100 | Zero and negative both refused at entry |
| `entry_date` | DATE | Yes | Must fall within `period_month`'s bounds | |
| `period_month` | TEXT (YYYY-MM) | Yes | **Derived from `entry_date`** at creation, fixed thereafter, never changes | Rule-21. A figure belongs to the month its own date falls in — **never** to "the month being closed". This is the trap Rule-21's struck-through third bullet describes, and CR-2 made it reachable again |
| `created_at` | TIMESTAMP | Yes | Auto-set | |
| `updated_at` | TIMESTAMP | No | Set on `edit_entry` | |

**Relationships:** Many-to-one with `members`. Aggregated (via chain-upward recalculation, Rule-26) into `member_period_totals`.
**Indexes:** PK on `id`; index on `(member_id, period_month)` — the hot path for both `record_entry` display and monthly export.
**Related requirements:** FR-5, Rule-15, Rule-16, Rule-16a, Rule-22, Rule-26, Rule-39.

---

## Entity: `slab_table`

**Purpose:** The editable, addable/removable percentage-band table driving every slab lookup.
**Lifecycle:** Seeded with 7 default rows at first-run setup (per Appendix B defaults); rows added/removed/edited freely via M7 commands.
**Retention:** Live configuration, no historical versioning of its own (a slab-table change's effect is captured downstream, in `member_period_totals`/`monthly_snapshots`, not by versioning this table itself).
**Security sensitivity:** Low.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK | |
| `threshold` | INTEGER | Yes | UNIQUE, stored ×100 | Rule-3, Rule-4 |
| `percentage` | INTEGER | Yes | — | No monotonicity check vs. threshold (Rule-41, accepted risk) |
| `sort_order` | INTEGER | Yes | — | Determines lookup order, not necessarily equal to threshold order if misconfigured |

**Relationships:** Referenced by the calculation engine (M3) at read time; not FK-linked to any other table.
**Indexes:** PK on `id`; unique index on `threshold`.
**Related requirements:** Rule-3, Rule-4, Rule-7, Rule-10, Rule-27, Rule-41.

---

## Entity: `member_period_totals`

**Purpose:** The live cache of each not-yet-closed period's figures per member — Business Volume, TBV, slab, differential, royalty, Rewards. Recomputed in place on every chain-upward recalculation (Rule-26).
**Lifecycle:** One row per member per **not-yet-closed** period — that is, any period whose status is `open` or `awaiting_close`. Zeroed and cleared for **the closing period only** at each monthly close (Rule-38); other live periods are untouched. The closing period's final state is preserved in `monthly_snapshots` first.
**Retention:** Live/ephemeral — only not-yet-closed periods exist here; history lives in `monthly_snapshots`.

> **Amended 7 August 2026 (CR-2).** Previously *"one row per member per currently-open period only"*. Because a month that has ended but is not closed still accepts entries (Rule-36 as amended), more than one period can hold live figures simultaneously. **The composite primary key `(member_id, period_id)` already supports this — there is no schema change**, only a change to what the table is allowed to contain. A recalculation is confined to the period the triggering entry belongs to and must never touch another period's rows. In practice more than one live period is rare: it requires a month to have been left unclosed past the end of the next one. Where figures are shown without a period being chosen, the **oldest** not-yet-closed period is the one displayed.
**Security sensitivity:** Medium.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `member_id` | INTEGER | Yes | PK (composite with `period_id`), FK → `members.id` | |
| `period_id` | INTEGER | Yes | PK (composite), FK → `periods.id` | |
| `business_volume` | INTEGER | Yes | Stored ×100 | Sum of this member's own entries this period |
| `total_business_volume` | INTEGER | Yes | Stored ×100 | Rule-6 |
| `slab_pct` | INTEGER | Yes | — | Rule-7 |
| `differential` | INTEGER | Yes | Stored ×100 | Rule-8, never negative (Rule-9) |
| `royalty` | INTEGER | Yes | Stored ×100 | Rule-10 |
| `rewards` | INTEGER | Yes | Stored ×100 | Rule-12 = differential + royalty |

**Relationships:** Composite-keyed to `members` and `periods` — one row set per not-yet-closed period. Read by M4 (member detail, chart) and written exclusively by M3 (calculation engine) as a side-effect of M2/M7 commands.
**Indexes:** Composite PK `(member_id, period_id)`.
**Related requirements:** Rule-5, Rule-6, Rule-7, Rule-8, Rule-9, Rule-10, Rule-11, Rule-12, Rule-13, Rule-26.

---

## Entity: `periods`

**Purpose:** One row per calendar month, tracking its lifecycle status.
**Lifecycle:** Created (implicitly, `status = open`) when the first entry of a new month is recorded, or explicitly at month-start; transitions `open → awaiting_close` (when the calendar month elapses, Rule-36) `→ closed` (on successful `confirm_backup_and_close`, Rule-18).

> **Status renamed 7 August 2026 (CR-2): `ended_locked` → `awaiting_close`.** The period is ended and **still accepting entries**, so the old name stated the opposite of the behaviour. Documentation-only rename — no implementation exists yet. A period that is `awaiting_close` accepts entries; a period that is `open` accepts entries only when no earlier period is `awaiting_close`.
**Retention:** Permanent.
**Security sensitivity:** Low.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK | |
| `period_month` | TEXT (YYYY-MM) | Yes | UNIQUE | Rule-21 |
| `status` | ENUM | Yes | `open` \| `awaiting_close` \| `closed` | Rule-17, Rule-36 (amended), Rule-18 |
| `ended_at` | TIMESTAMP | No | Set when the calendar month elapses | Triggers Rule-20's alert and blocks the **current** month for entry — not this one, which stays recordable until closed |
| `closed_at` | TIMESTAMP | No | Set on successful close | |

**Relationships:** One-to-many with `member_period_totals` (while `open` **or** `awaiting_close`), `monthly_snapshots`, `backups`.
**Indexes:** PK on `id`; unique index on `period_month`; index on `status` (powers `get_outstanding_periods`, oldest-first ordering).
**Related requirements:** Rule-17, Rule-18, Rule-20, Rule-21, Rule-36, Rule-38.
**Notable business consequence:** A calendar month that elapses with zero entries produces **no row transition to a snapshot** at all — it is excluded from yearly averaging, per the empty-month rule (RQ-16, see [03-business-rules.md](03-business-rules.md)). **Amended 7 Aug 2026 (CR-2):** this used to be a likely outcome, since entry was locked entirely while a prior reset was outstanding. It is now unlikely — an outstanding month keeps accepting entries throughout — but the rule is unchanged and must still be built. A genuinely empty month remains possible whenever nothing was recorded, for any reason.

---

## Entity: `monthly_snapshots`

**Purpose:** The permanent, versioned historical record of every closed period, per member. This is what all yearly reporting reads — never live values (Rule-38).
**Lifecycle:** Version 1 written atomically as part of `confirm_backup_and_close` (Rule-18/38). A new version is appended (never replacing an existing one) whenever a closed-month entry is corrected via `edit_entry` (Rule-39).
**Retention:** Permanent, all versions retained forever.
**Security sensitivity:** High — the permanent historical record of every member's figures.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK | |
| `member_id` | INTEGER | Yes | FK → `members.id` | |
| `period_id` | INTEGER | Yes | FK → `periods.id` | |
| `version` | INTEGER | Yes | UNIQUE with `(member_id, period_id)` | Starts at 1, increments on each correction (Rule-39) |
| `business_volume` | INTEGER | Yes | Stored ×100 | |
| `total_business_volume` | INTEGER | Yes | Stored ×100 | |
| `slab_pct` | INTEGER | Yes | — | |
| `rewards` | INTEGER | Yes | Stored ×100 | |
| `royalty` | INTEGER | Yes | Stored ×100 | |
| `is_active_status` | BOOLEAN | Yes | Snapshot of `members.is_active` at close time | So reports reflect who was live that month (Rule-38's field list) |

**Relationships:** Many-to-one with `members` and `periods`.
**Indexes:** PK on `id`; unique index on `(member_id, period_id, version)`; index on `(period_id, version DESC)` for "latest version" reads (Rule-39, `redownload_backup`).
**Related requirements:** Rule-23, Rule-24, Rule-38, Rule-39.
**Query convention:** All reporting (`export_yearly_average`, `export_low_contribution`, `redownload_backup`) must read `MAX(version)` per `(member_id, period_id)` — never assume version 1 is current.

---

## Entity: `backups`

**Purpose:** Metadata for every generated backup file. Originally the actual gate that Rule-18's close flow depends on, plus manual and versioned-correction backups; **generalized 7 August 2026 (Rule-43, ADR-012)** to also carry whole-console backups (scheduled, on-demand, or taken automatically as a safety net immediately before a restore), rather than adding a second table for them.
**Lifecycle:** One row per backup event — `confirm_backup_and_close`, `manual_backup_current_period`, or a closed-month `edit_entry` correction (all `kind = period_close`); or `run_console_backup_now` (`kind = scheduled` on a login-triggered due backup, `kind = manual` on demand); or an automatic write immediately before any restore (`kind = pre_restore_safety`). `is_original = true` only for a `period_close` row's version-1 backup; never modified after creation (Rule-39).
**Retention:** `period_close` rows are permanent, nothing auto-deleted (Rule-31). `scheduled`/`manual` rows are pruned to `settings.console_backup_retention_count` (default 10), oldest first, after every new one of either kind is written (Rule-43). `pre_restore_safety` rows are never auto-pruned.
**Security sensitivity:** Medium — file paths, not member data directly, but points at files containing personal data.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK | |
| `period_id` | INTEGER | No (nullable) | FK → `periods.id` | NULL for every `kind` except `period_close` (Rule-43) |
| `kind` | TEXT | Yes | One of `period_close`/`scheduled`/`manual`/`pre_restore_safety` | New 7 August 2026 (Rule-43) — see Lifecycle above |
| `schedule_kind` | TEXT | No (nullable) | One of `daily`/`weekly`/`monthly` | Set only when `kind = scheduled` |
| `version` | INTEGER | Yes | — | Mirrors `monthly_snapshots.version` for the same close/correction event when `kind = period_close`; `1` for every other kind |
| `internal_retained_path` | TEXT | Yes | — | The actual gate for `confirm_backup_and_close` when `kind = period_close` — write-verified before zeroing proceeds; the retained copy for every other kind |
| `external_medium_path` | TEXT | No (nullable) | — | Prompted for at the same time; **failure does not block the close** — reminded, not enforced (Rule-31 extension, physically-separate-medium requirement) |
| `checksum` | TEXT | Yes | — | Verified on write, and re-verifiable on demand |
| `is_original` | BOOLEAN | Yes | True only for version 1 of a `period_close` row | Rule-39 — never modified once set |

**Relationships:** Many-to-one with `periods` (nullable — only meaningful for `kind = period_close`).
**Indexes:** PK on `id`; index on `(period_id, version DESC)`; index on `(kind, created_at DESC)` for the retention prune and the widened restore-points listing.
**Related requirements:** Rule-18, Rule-31, Rule-39, Rule-43, LOW-3 (data recovery).
**Read pre-authentication.** This is the one table consulted before any session exists: the data-recovery screen (LOW-3) — reached automatically when the database can't be opened, or voluntarily via a plain link on the ordinary first-run setup screen, same screen either way with reworded copy — lists restore points and restores from them without a credential to verify against. `checksum` is what makes that safe — it is the difference between a backup being *available* and being *trustworthy*, and a restore must refuse on mismatch rather than overwrite one corrupt file with another. See API-34–36 and API-39–40 in [04-api-specification.md](04-api-specification.md).

---

## Entity: `settings`

**Purpose:** Key/value store for the 13 configurable items in Appendix B.
**Lifecycle:** Seeded with defaults at first-run setup; updated freely via `update_settings`.
**Retention:** Live configuration; changes are audited (`audit_log`), but the table itself holds only current values.
**Security sensitivity:** Low.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `key` | TEXT | Yes | PK | e.g. `royalty_min_children`, `royalty_rate`, `hierarchy_depth`, `level_widths` (JSON), `reference_unit_value`, `yearly_cycle_start`/`yearly_cycle_end`, `low_contribution_threshold`, `default_export_columns` (JSON), `console_backup_schedule`, `console_backup_retention_count`, `console_backup_folder` |
| `value` | TEXT/JSON | Yes | Type-checked per key at the application layer | |

**Related requirements:** FR-6, Rule-4, Rule-14, Rule-23, Rule-24, Rule-27, Rule-43, §7 Settings Inventory (originally 13 items, +3 for whole-console backup, 7 August 2026).

---

## Entity: `auth`

**Purpose:** Single-row table holding the one administrator's credentials and lockout state.
**Lifecycle:** Row created once at `setup_first_run`; updated on credential change or lockout-state transitions.
**Retention:** Permanent (one admin, for the application's lifetime).
**Security sensitivity:** Critical — this table alone gates access to every member's personal data. Hashed with Argon2id, never stored in plaintext, never logged.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK, single row (id=1) | |
| `pin_hash` | TEXT | No (nullable) | Argon2id | Nullable because password-only is allowed |
| `password_hash` | TEXT | No (nullable) | Argon2id | Nullable because PIN-only is allowed. At least one of `pin_hash`/`password_hash` must be non-null (Rule-29) |
| `failed_attempts` | INTEGER | Yes | Default 0 | |
| `locked_until` | TIMESTAMP | No (nullable) | — | Exponential backoff per attempt |
| `recovery_codes` | JSON | Yes | Array of hashed, single-use codes | Sole recovery path — no email/cloud (offline constraint) |
| `session_timeout_minutes` | INTEGER | Yes | From settings, duplicated/cached here or read from `settings` | Inactivity auto-lock |

**Related requirements:** FR-9, Rule-29, NFR-4.

---

## Entity: `audit_log`

**Purpose:** The append-only, who-changed-what record backing NFR-5 and every "figure can always be explained" promise in the source documents.
**Lifecycle:** Insert-only. Never edited, never deleted (mirrors `ui-theme.md`/prototype copy: "no entry is ever edited or removed").
**Retention:** Permanent.
**Security sensitivity:** Medium-High — records what personal data changed and when, itself a record subject to the same DPDP considerations as `members`.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK | |
| `entity_type` | ENUM | Yes | `member` \| `entry` \| `setting` \| `period` | |
| `entity_id` | INTEGER | Yes | — | |
| `field` | TEXT | Yes | — | |
| `old_value` | TEXT | No | — | |
| `new_value` | TEXT | No | — | |
| `changed_at` | TIMESTAMP | Yes | Auto-set | |
| `cause` | ENUM | Yes | `entry` \| `edit` \| `reversal`⁺ \| `correction` \| `settings_change` \| `period_close` \| `manual_backup` | ⁺`reversal` as a cause label may remain even though `reverse_entry` as a distinct command is dropped — an edit to a past entry can still be tagged `correction` rather than `reversal`; recommend `edit`/`correction` only, retire `reversal` as an unused enum value unless the client specifically wants that word preserved in the log |

**Related requirements:** NFR-5, Rule-39 (correction audit trail).

---

## Cross-cutting notes

**Transactions/concurrency:** Single-user, single-machine, single-session by design (OC-1) — SQLite's own file locking is sufficient; no distributed-transaction or multi-writer concern exists anywhere in this model (`architecture.md` §9.4, confirmed, no optimistic/pessimistic locking scheme needed beyond what SQLite provides).

**Migrations:** None required at launch — the system starts empty (NFR-16, OC-11), no import/migration tooling is in scope.

**Seed/reference data:** 7 default slab rows (Appendix B item 1–2), 16 default settings values (Appendix B — 13 original + 3 for whole-console backup, 7 August 2026), no member data (root is created interactively at first-run, not seeded).

**Data validation summary:** Enforced at both the DB layer (CHECK constraints: `business_volume_entries.amount > 0`, UNIQUE on `members.phone`/`slab_table.threshold`/`periods.period_month`, `backups.kind IN (...)`/`backups.schedule_kind IN (...)`) and the application layer (Rule-1/Rule-32 advisory warnings, which are UI/API-layer only — deliberately **not** DB constraints, since they must never block a save).
