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
**Indexes:** PK on `id`; unique index on `phone`; index on `introducer_member_id` (chain-upward traversal, Rule-26); index on `name` (search, FR-1).
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
| `period_month` | TEXT (YYYY-MM) | Yes | Fixed at creation, never changes | Rule-21 |
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

**Purpose:** The live cache of the current open period's figures per member — Business Volume, TBV, slab, differential, royalty, Rewards. Recomputed in place on every chain-upward recalculation (Rule-26).
**Lifecycle:** One row per member per **currently-open** period only. Zeroed and effectively reset to a fresh row set at each monthly close (Rule-38) — the closing period's final state is preserved separately in `monthly_snapshots` first.
**Retention:** Live/ephemeral — only the current period exists here; history lives in `monthly_snapshots`.
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

**Relationships:** Composite-keyed to `members` and `periods`. Read by M4 (member detail, chart) and written exclusively by M3 (calculation engine) as a side-effect of M2/M7 commands.
**Indexes:** Composite PK `(member_id, period_id)`.
**Related requirements:** Rule-5, Rule-6, Rule-7, Rule-8, Rule-9, Rule-10, Rule-11, Rule-12, Rule-13, Rule-26.

---

## Entity: `periods`

**Purpose:** One row per calendar month, tracking its lifecycle status.
**Lifecycle:** Created (implicitly, `status = open`) when the first entry of a new month is recorded, or explicitly at month-start; transitions `open → ended_locked` (when the calendar month elapses, Rule-36) `→ closed` (on successful `confirm_backup_and_close`, Rule-18).
**Retention:** Permanent.
**Security sensitivity:** Low.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK | |
| `period_month` | TEXT (YYYY-MM) | Yes | UNIQUE | Rule-21 |
| `status` | ENUM | Yes | `open` \| `ended_locked` \| `closed` | Rule-17, Rule-36, Rule-18 |
| `ended_at` | TIMESTAMP | No | Set when the calendar month elapses | Triggers Rule-20's alert and Rule-36's entry lock |
| `closed_at` | TIMESTAMP | No | Set on successful close | |

**Relationships:** One-to-many with `member_period_totals` (only while `open`), `monthly_snapshots`, `backups`.
**Indexes:** PK on `id`; unique index on `period_month`; index on `status` (powers `get_outstanding_periods`, oldest-first ordering).
**Related requirements:** Rule-17, Rule-18, Rule-20, Rule-21, Rule-36, Rule-38.
**Notable business consequence:** A calendar month that elapses with zero entries (possible because entry is locked while a prior reset is outstanding) produces **no row transition to a snapshot** at all — it is excluded from yearly averaging, per the empty-month rule (RQ-16, see [03-business-rules.md](03-business-rules.md)).

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

**Purpose:** Metadata for every generated backup file — the actual gate that Rule-18's close flow depends on, plus manual and versioned-correction backups.
**Lifecycle:** One row per backup event (`confirm_backup_and_close`, `manual_backup_current_period`, or a closed-month `edit_entry` correction). `is_original = true` only for a period's version-1 backup; never modified after creation (Rule-39).
**Retention:** Permanent, nothing auto-deleted (Rule-31).
**Security sensitivity:** Medium — file paths, not member data directly, but points at files containing personal data.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `id` | INTEGER | Yes | PK | |
| `period_id` | INTEGER | Yes | FK → `periods.id` | |
| `version` | INTEGER | Yes | — | Mirrors `monthly_snapshots.version` for the same close/correction event |
| `internal_retained_path` | TEXT | Yes | — | The actual gate for `confirm_backup_and_close` — write-verified before zeroing proceeds |
| `external_medium_path` | TEXT | No (nullable) | — | Prompted for at the same time; **failure does not block the close** — reminded, not enforced (Rule-31 extension, physically-separate-medium requirement) |
| `checksum` | TEXT | Yes | — | Verified on write, and re-verifiable on demand |
| `is_original` | BOOLEAN | Yes | True only for version 1 | Rule-39 — never modified once set |

**Relationships:** Many-to-one with `periods`.
**Indexes:** PK on `id`; index on `(period_id, version DESC)`.
**Related requirements:** Rule-18, Rule-31, Rule-39, LOW-3 (data recovery).
**Read pre-authentication.** This is the one table consulted before any session exists: the data-recovery screen (LOW-3) lists restore points and restores from them while the main database is unreadable, so no credential can be verified. `checksum` is what makes that safe — it is the difference between a backup being *available* and being *trustworthy*, and a restore must refuse on mismatch rather than overwrite one corrupt file with another. See API-34–36 in [04-api-specification.md](04-api-specification.md).

---

## Entity: `settings`

**Purpose:** Key/value store for the 13 configurable items in Appendix B.
**Lifecycle:** Seeded with defaults at first-run setup; updated freely via `update_settings`.
**Retention:** Live configuration; changes are audited (`audit_log`), but the table itself holds only current values.
**Security sensitivity:** Low.

| Attribute | Type | Required | Constraints | Notes |
|---|---|---|---|---|
| `key` | TEXT | Yes | PK | e.g. `royalty_min_children`, `royalty_rate`, `hierarchy_depth`, `level_widths` (JSON), `reference_unit_value`, `yearly_cycle_start`/`yearly_cycle_end`, `low_contribution_threshold`, `default_export_columns` (JSON) |
| `value` | TEXT/JSON | Yes | Type-checked per key at the application layer | |

**Related requirements:** FR-6, Rule-4, Rule-14, Rule-23, Rule-24, Rule-27, §7 Settings Inventory (all 13 items).

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

**Seed/reference data:** 7 default slab rows (Appendix B item 1–2), 13 default settings values (Appendix B), no member data (root is created interactively at first-run, not seeded).

**Data validation summary:** Enforced at both the DB layer (CHECK constraints: `business_volume_entries.amount > 0`, UNIQUE on `members.phone`/`slab_table.threshold`/`periods.period_month`) and the application layer (Rule-1/Rule-32 advisory warnings, which are UI/API-layer only — deliberately **not** DB constraints, since they must never block a save).
