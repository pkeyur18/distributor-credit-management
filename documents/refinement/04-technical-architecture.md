# 04 — Technical Architecture

Twelve architecture decisions, nine modules, full DDL for ten entities, all 40 IPC contracts, state machines, security architecture, and both backup mechanisms. Build every Rust command and every table from this file directly.

---

## 1. Stack

| Layer | Choice |
|---|---|
| Application shell | **Tauri v2** — no client-server split, no network layer anywhere |
| Frontend | React + TypeScript, running in an OS-native WebView |
| Backend | Rust application core — the only thing that touches the database |
| UI library | shadcn/ui + Tailwind CSS. **Inter** font bundled locally — no CDN, no web fonts, ever |
| Database | SQLite via `rusqlite`, encrypted at rest with **SQLCipher** |
| Excel generation | `rust_xlsxwriter`, Rust-side only |
| Auth hashing | **Argon2id** |
| Packaging | Native Tauri bundler — Windows (`.msi`/`.exe`) + macOS (`.dmg`/`.app`), no auto-update, ~10–20MB installer |
| Precision | All volume/reward figures: fixed-point integers, `×100`, never floats |

There is **no server, no client-server split, and no network code anywhere.** The offline requirement is enforced structurally — no network capability is declared in the Tauri configuration — not by a promise not to use it.

### 1.1 Architectural drivers, in order

1. **Recursive calculation must recompute instantly** — Rule-26 — the differential/slab/royalty depend on the whole subtree beneath a member.
2. **Sensitive data behind one non-technical credential** — thousands of people's PII behind a single login.
3. **A closed month destroys its own live evidence unless backed up first** — the backup gate is the single most safety-critical piece of logic in the system.

Every major decision below traces to one of these three.

---

## 2. Architecture Decision Records

### ADR-001 — Single-process offline desktop application
**Decision:** One process, no HTTP/IPC-over-socket boundary anywhere, not even local.
**Rationale:** A server process — even bound to `localhost` — introduces a listener and an attack surface the offline requirement doesn't need and the security requirement argues against. Tauri's WebView↔Rust IPC is in-process message-passing, not a network boundary.
**Consequence:** No API layer to design, version, or secure. Cannot later be extended into a client-server model without real rework — accepted, member access is permanently out of scope (OS-1), not deferred.

### ADR-002 — Tauri v2 over Electron, .NET, and Python
**Decision:** Tauri v2, React + TypeScript frontend, Rust core.
**Rationale:** Wins on the client's top priority — PII security — structurally: the WebView cannot reach the filesystem or database except through explicitly-exposed Rust commands. Smaller trust boundary than Electron's renderer. Ties on UI modernity and cross-platform; wins on install footprint (~10–20MB vs Electron's ~150–200MB) and keeps 90% of the surface in one language (TypeScript).
**Consequence:** The developer needs real Rust for the calculation engine, DB access, auth, export. Tauri's plugin ecosystem is younger than Electron's — anything uncovered gets written directly in Rust (TR-3).

### ADR-003 — SQLite + SQLCipher over OS-level encryption or embedded Postgres
**Decision:** SQLite + SQLCipher, application-managed key (ADR-008), not OS-level full-disk encryption.
**Rationale:** OS-level encryption protects the file only while the disk is powered off/locked — not while the OS is unlocked (most of the time), and not for a copied-out backup on a USB drive (mandatory per RQ-19). Application-level encryption travels correctly with the file wherever it ends up. Embedded Postgres would add a server process, port, and operational surface for zero benefit at this scale.
**Consequence:** The encryption key must be derived and managed by the application (ADR-008), not delegated to the OS.

### ADR-004 — Fixed-point integer arithmetic over floating point
**Decision:** Every Business Volume, TBV, differential, royalty, own-Business-Volume reward and Rewards figure is an `i64` integer representing hundredths of a unit (`×100`), end to end through storage and calculation. Conversion to a two-decimal display string happens only at the UI boundary.
**Rationale:** Floating point cannot exactly represent most decimal fractions; summing hundreds of terms across a deep hierarchy accumulates visible drift — exactly the "nearly right, which is worse than obviously wrong" failure UN-08 names. Fixed-point is exact under addition, subtraction, and the percentage multiplications this system performs.
**Consequence:** Every percentage operation is integer multiply-then-divide with an explicit rounding rule (round-half-up, applied once, at the point a term is finalised — never on an intermediate sum).

### ADR-005 — Chain-upward incremental recalculation
**Decision:** On a write against member X, walk only the direct path from X to the root, recomputing each ancestor's aggregate from its already-correct children.
**Rationale:** Full-tree recompute (25,000 members on every single entry) blows past the 2-second target as the network grows. Batched/deferred recomputation directly violates Rule-26/UN-14. Chain-upward is correct (proven §5.3) and its cost is bounded by tree depth × average width, not total member count — flat performance from 500 to 25,000 members.
**Consequence:** The engine must re-scan **all** of an ancestor's direct children when recomputing that ancestor's differential/royalty terms — not just the child that changed, because the ancestor's own slab may have shifted, changing every term against every child. Full detail: §5.2.

### ADR-006 — Append-only entries + versioned snapshots
**Decision:** Applied uniformly to `monthly_snapshots` and `backups`. Every correction produces a new version; prior versions retained forever.
**Rationale:** The client explicitly rejected overwriting a corrected record in place (RQ-7, RQ-20) — the original figure may already have been communicated to a member. Versioning is the only model that satisfies "corrected" and "provably what it originally said" simultaneously.
**Consequence:** Every permanent-record table carries a `version` column; reporting logic must read `MAX(version)` per `(member, period)`, never assume one row per period.

### ADR-007 — Rust-side Excel generation
**Decision:** All `.xlsx` generation (`rust_xlsxwriter`) happens in Rust, never in the WebView.
**Rationale:** Continues ADR-002's security boundary — the WebView never handles raw file content or paths, only asks "export this" and gets success/failure.
**Consequence:** Column-formatting, inactive-row colouring, and vocabulary-safe filename generation all live in Rust.

### ADR-008 — PIN/password + Argon2id + local recovery codes
**Decision:** Local credential (PIN and/or password), Argon2id hashing, local failed-attempt lockout, one-time local recovery codes generated at setup. No cloud auth, no biometric as the sole mechanism.
**Rationale:** Cloud/OAuth is architecturally impossible under the no-network constraint. Biometric APIs vary between Windows and macOS and were never requested — introducing one would be unrequested scope. Argon2id is memory-hard against offline brute force of a stolen encrypted file.
**Consequence:** Recovery codes are the **only** recovery path — no "forgot password" email flow, no vendor backdoor. Loss of both credential and codes is **permanently unrecoverable by design**. Must be communicated to the client plainly at setup, not buried in settings.

### ADR-009 — No software validation on slab-table monotonicity
**Decision:** No monotonicity check is built into the settings screen, now or ever, without the client re-raising it.
**Rationale:** A client decision, not a technical one, recorded here because a future developer might assume it was simply forgotten. It was not — the client was offered the cheap safeguard and explicitly declined it (RQ-1, 3 Aug 2026).
**Consequence:** If the slab table is ever edited to break monotonicity, the engine will compute and silently store a negative differential — nothing catches it, by design. **Do not add this unprompted** — see [06](06-decision-log-and-open-items.md) §6.

### ADR-010 — Settings-driven rule engine, no hardcoded business constants
**Decision:** All scheme parameters live in `settings` and `slab_table`, read at the point of use — never compiled into the binary, not even as defaults-with-override. Defaults are seed data inserted at first-run setup.
**Rationale:** Treating defaults as "seed data, then just data" avoids a whole bug class where code accidentally reads a compiled default instead of the client's current setting. Also makes Rule-27's row flexibility fall out naturally from the slab table being a real table, not a fixed-width struct.
**Consequence:** Every module touching a business parameter takes a `Settings` snapshot or queries the table — a discipline enforced by self-review, since there is no second developer.

### ADR-011 — Cross-platform bundling, no auto-update
**Decision:** Tauri's native bundler targets both Windows and macOS from one codebase. No auto-update mechanism.
**Rationale:** Auto-update would require exactly the network capability the offline requirement forbids. Updates are a new installer, run manually.
**Consequence:** Version upgrades are a deliberate manual action; the maintainer is responsible for telling the client an update exists, since the system never checks itself. Code signing is required on both platforms to avoid "unknown publisher" warnings that would confuse a low-technical user.

### ADR-012 — Whole-console backup generalizes the `backups` table
**Decision:** Rather than a second `console_backups` table, generalize the existing `backups` table: `period_id` becomes nullable, add a `kind` column (`period_close`/`scheduled`/`manual`/`pre_restore_safety`) and a `schedule_kind` column (`daily`/`weekly`/`monthly`, set only when `kind = scheduled`).
**Rationale:** Every kind of whole-console backup is a verified copy of the single SQLCipher file — the same artifact a month-close backup already is, just not scoped to one period. A bespoke export format was rejected outright — the file already contains everything (members, entries, snapshots, settings, slab table, audit log, and the `auth` row); reconstructing that completeness in a second format is real work for no benefit.
**Consequence:** Every query currently assuming `backups.period_id` is non-null (e.g. `list_backups`) must filter on `kind = 'period_close'` explicitly.

---

## 3. Container & module architecture

```
Presentation (React + TypeScript, WebView)
  Screens: Home/Search · Member Detail · Add/Edit Member · BV Entry ·
  Hierarchy Chart · Settings · Monthly Close · Reports/Exports · Auth/Lock
  No direct filesystem, DB, or network access — every action is a typed IPC call
          │  Tauri IPC — typed commands only, allowlisted
Application (Rust)
  M1 Member & Structure   M2 Business Volume Entry   M3 Calculation Engine
  M4 Search & Chart       M5 Monthly Close           M6 Reporting & Exports
  M7 Settings             M8 Access & Alerts         M9 Audit & Logging
  Shared: encryption/key management, error types, fixed-point helpers, date/period helpers
          │  rusqlite (SQLCipher build)
Data — one encrypted SQLite file + retained backup versions
          │  file dialog (user-directed)
External medium — user-chosen, physically separate from the install disk
```

**Where the security boundary sits.** Between Presentation and Application. Everything PII-related lives strictly below that line. The WebView is handed only structured, already-validated data across a typed command interface and can never reach the filesystem, database, or key material directly.

### 3.1 Modules

| Module | Responsibilities | Commands |
|---|---|---|
| **M1 — Member & Structure** | Root creation (once), add/edit/deactivate/reactivate, all structural validation | `create_root_member`, `add_member`, `edit_member`, `deactivate_member`, `reactivate_member`, `search_members` |
| **M2 — Business Volume Entry** | Record BV, edit/correct an entry in any period, decide which months are recordable (Rule-36 as amended) | `record_entry`, `edit_entry`, `get_period_lock_status` |
| **M3 — Calculation Engine** | Bottom-up rollup, slab lookup, differential, royalty, own-Business-Volume reward, Rewards. **Pure function set, no I/O.** | `preview_settings_impact` (the only command — see §3.2) |
| **M4 — Search & Chart** | Home search (name/ID/phone), member detail, hierarchy chart, **full hierarchy window**, inactive-member colour coding | `search_members` (shared), `get_member_detail`, `get_direct_children_chart` (both one-depth and `full_tree`) |
| **M5 — Monthly Close** | Alert lifecycle and entry eligibility, gated close flow, permanent snapshot writing, closed-month correction, on-demand backup | `get_outstanding_periods`, `begin_close`, `confirm_backup_and_close`, `manual_backup_current_period` |
| **M6 — Reporting & Exports** | Three extracts, re-download of any past backup, inactive-row colouring | `export_monthly`, `export_yearly_average`, `export_low_contribution`, `list_backups`, `redownload_backup` |
| **M7 — Settings** | All client-adjustable parameters, slab row add/remove, mid-period recalculation warning, console backup schedule/retention | `get_settings`, `update_settings`, `add_slab_row`, `remove_slab_row`, `update_slab_row`, `get_console_backup_settings`, `update_console_backup_settings` |
| **M8 — Access & Alerts** | Setup wizard, login, lockout, session lock, outstanding-month alert, pre-flight recovery, whole-console backup/restore | `setup_first_run`, `login`, `lock_session`, `unlock_session`, `use_recovery_code`, `get_outstanding_alert`, `check_data_readable`, `list_restore_points`, `restore_from_backup`, `run_console_backup_now`, `restore_from_backup_file` |
| **M9 — Audit & Technical Logging** *(architecture-introduced, cross-cutting)* | Client-visible audit log; separate never-visible technical/diagnostic log | `get_audit_log` |

### 3.2 M3 in detail — the pure calculation core

M3 is deliberately a pure function set with no I/O: it takes the affected chain's current state plus current settings/slab table, and returns recomputed figures for every node on that chain. The calling code (M2's `record_entry`, M5's correction path) loads the chain, invokes the engine, and persists the result in one DB transaction. This is what makes the engine unit-testable against the six worked scenarios without touching a database.

**No command triggers a calculation.** There is no "recalculate" button anywhere (Rule-26), so there is no command surface that could become one. The engine runs only as an internal consequence of a write in M2, M5, or M7.

**`preview_settings_impact` (API-33)** is the one exception: it asks what the engine *would* produce under candidate settings, without committing. It swaps candidate settings in, recomputes, and restores them in a `finally` block — a panic must never leave live settings holding uncommitted values. Because slab/royalty settings never feed the TBV rollup, the preview reuses the live Total Business Volume and re-runs only the Rewards computation.

**M1 dependency chain:** M8 → M1 → M2 → M3 ← M7. M3 → M4, M5. M5 → M6, M8 (unlocks the alert). Full picture: `architecture.md` §6.9 module dependency map, unchanged by this consolidation.

---

## 4. Data architecture

### 4.1 Conventions

- **Precision** (ADR-004): every volume/reward column is `INTEGER`, storing the value `×100`. `123456` means `1234.56`.
- **Identifiers:** member IDs are 6-digit integers in **100001–999999**, rejection-sampled against currently-used IDs, never reused.
- **Timestamps:** ISO-8601 UTC text; displayed in Indian date format at the UI boundary.
- **No hard deletes anywhere** (Rule-28, Rule-42) — every entity table is append-only or soft-deleted (`is_active`), never `DELETE`d.

### 4.2 Entity relationships

```
members ──(introducer_member_id, self-ref)── members
members ──< business_volume_entries
members ──< member_period_totals >── periods
members ──< monthly_snapshots >── periods
periods ──< backups
members ──< audit_log
slab_table, settings, auth — standalone, referenced by the calc engine at read time
```

### 4.3 Full schema — 10 entities

**`members`**

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | 6-digit, random, **100001–999999**, never reissued |
| `name` | TEXT NOT NULL | |
| `phone` | TEXT NOT NULL UNIQUE | Unique across active **and** inactive |
| `email` | TEXT NULL | Optional, validated format if present |
| `address` | TEXT NOT NULL | |
| `introducer_member_id` | INTEGER NULL, FK → members.id | NULL only for the one root; **immutable after insert** |
| `level` | INTEGER NOT NULL | Cached at creation, never recomputed — safe because Rule-37 makes it permanent |
| `is_active` | BOOLEAN NOT NULL DEFAULT true | **Display-only** — zero calculation effect (Rule-28 corrected) |
| `joining_date` | TEXT NOT NULL | Auto-captured, editable |
| `consent_given` | BOOLEAN NOT NULL | Mandatory (Rule-40) |
| `consent_date` | TEXT NOT NULL | Auto-captured |
| `created_at` | TEXT NOT NULL | |

Indexes: PK `id`; unique `phone` (uniqueness under Rule-34 **and** the lookup index for phone search under Rule-44); index `introducer_member_id` (chain traversal); index `name` (search).

**Note on phone search (Rule-44):** matching is a substring comparison on the canonical key (digits, then an international prefix or trunk zero dropped), so the unique index accelerates an exact or prefix match but a mid-number match is a scan. At the NFR-2 ceiling of 25,000 rows that scan is trivially inside the NFR-1 two-second budget, and no additional index or normalised column is warranted. If a future profile ever shows otherwise, the cheapest fix is a stored canonical-key shadow column — **do not add it speculatively.**

**`business_volume_entries`** — the append-only ledger

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `member_id` | INTEGER NOT NULL, FK → members.id | |
| `amount` | INTEGER NOT NULL | `×100`; `CHECK (amount > 0)` — Rule-16a |
| `entry_date` | TEXT NOT NULL | Editable only within `period_month`'s bounds |
| `period_month` | TEXT NOT NULL | `YYYY-MM`, **derived from `entry_date` at creation**, fixed thereafter, never changes. A figure belongs to the month its own date falls in — never to "the month being closed" (Rule-21) |
| `created_at` | TEXT NOT NULL | |
| `updated_at` | TEXT NULL | Set on `edit_entry` |

Index: `(member_id, period_month)` — hot path for entry display and monthly export.

**`slab_table`**

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `threshold` | INTEGER NOT NULL UNIQUE | `×100`; addable/removable (Rule-27) |
| `percentage` | INTEGER NOT NULL | 0–100. **No monotonicity check** (Rule-41) |
| `sort_order` | INTEGER NOT NULL | Lookup order |

**`member_period_totals`** *(live cache — one row set per **not-yet-closed** period)*

> **Amended 7 Aug 2026 (CR-2).** Previously *"current open period only"*. Because an ended-but-unclosed month still accepts entries (Rule-36 as amended), more than one period can hold live figures at the same time. The composite primary key already carries this without a schema change; what changes is the lifecycle statement, not the table. A close clears **only** the period being closed (Rule-38). In practice more than one live period is rare — it requires a month to have been left unclosed past the end of the next one.

| Column | Type | Notes |
|---|---|---|
| `member_id` | INTEGER NOT NULL, FK → members.id | PK (composite) |
| `period_id` | INTEGER NOT NULL, FK → periods.id | PK (composite) |
| `business_volume` | INTEGER NOT NULL | `×100` |
| `total_business_volume` | INTEGER NOT NULL | `×100`, Rule-6 |
| `slab_pct` | INTEGER NOT NULL | Rule-3/7 |
| `differential` | INTEGER NOT NULL | `×100`, Rule-8 |
| `royalty` | INTEGER NOT NULL | `×100`, Rule-10 |
| `own_reward` | INTEGER NOT NULL | `×100`, Rule-46 (added 8 Aug 2026, CR-4) |
| `rewards` | INTEGER NOT NULL | `×100`, Rule-12 = differential + royalty + own_reward |

**`periods`**

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `period_month` | TEXT NOT NULL UNIQUE | `YYYY-MM` |
| `status` | TEXT NOT NULL CHECK IN (`open`,`awaiting_close`,`closed`) | §7.1 state machine. **Renamed 7 Aug 2026 (CR-2): `ended_locked` → `awaiting_close`** — the period is ended and *still accepting entries*, so the old name stated the opposite of the truth. Documentation-only rename; no implementation exists yet |
| `ended_at` | TEXT NULL | Set when the calendar month elapses — raises Rule-20's alert and blocks the *current* month for entry, not this one |
| `closed_at` | TEXT NULL | Set on successful close |

**`monthly_snapshots`** — the permanent, versioned historical record

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `member_id` | INTEGER NOT NULL, FK → members.id | |
| `period_id` | INTEGER NOT NULL, FK → periods.id | |
| `version` | INTEGER NOT NULL | Starts at 1, incremented per correction |
| `business_volume` | INTEGER NOT NULL | |
| `total_business_volume` | INTEGER NOT NULL | |
| `slab_pct` | INTEGER NOT NULL | |
| `differential` | INTEGER NOT NULL | |
| `royalty` | INTEGER NOT NULL | |
| `own_reward` | INTEGER NOT NULL | Rule-46 (added 8 Aug 2026, CR-4) |
| `rewards` | INTEGER NOT NULL | |
| `is_active_status` | BOOLEAN NOT NULL | Snapshot of `is_active` at close time |
| `created_at` | TEXT NOT NULL | |

Unique `(member_id, period_id, version)`. Index `(period_id, version DESC)` for "latest version" reads. **All reporting reads `MAX(version)` per `(member_id, period_id)`** — never assume version 1 is current.

**`backups`** — generalized per ADR-012

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `period_id` | INTEGER NULL, FK → periods.id | NULL for every `kind` except `period_close` |
| `kind` | TEXT NOT NULL CHECK IN (`period_close`,`scheduled`,`manual`,`pre_restore_safety`) | |
| `schedule_kind` | TEXT NULL CHECK IN (`daily`,`weekly`,`monthly`) | Set only when `kind = scheduled` |
| `version` | INTEGER NOT NULL | Mirrors the snapshot version for `period_close`; `1` otherwise |
| `internal_retained_path` | TEXT NOT NULL | The actual gate for `period_close`; retained copy for others |
| `external_medium_path` | TEXT NULL | Prompted, but failure never blocks the close |
| `checksum` | TEXT NOT NULL | Verified on write and on restore |
| `is_original` | BOOLEAN NOT NULL | `true` only for version 1 of a `period_close` row; never modified after |
| `created_at` | TEXT NOT NULL | |

Index `(period_id, version DESC)`; index `(kind, created_at DESC)` for retention prune and restore-points listing.

**`settings`** — key/value, 16 rows

| Column | Type | Notes |
|---|---|---|
| `key` | TEXT PK | See [02](02-business-rules.md) §6 for the full 16-row inventory |
| `value` | TEXT NOT NULL | Serialised; typed at the application boundary |

**`auth`** — single row

| Column | Type | Notes |
|---|---|---|
| `pin_hash` | TEXT NULL | Argon2id; either this or `password_hash` (or both) set |
| `password_hash` | TEXT NULL | Argon2id |
| `failed_attempts` | INTEGER NOT NULL DEFAULT 0 | |
| `locked_until` | TEXT NULL | See [06](06-decision-log-and-open-items.md) O4 — full ladder undefined |
| `recovery_codes` | TEXT NOT NULL | JSON array of hashed, one-time-use codes |
| `session_timeout_minutes` | INTEGER NOT NULL | See O3 |

**`audit_log`**

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `entity_type` | TEXT NOT NULL | `member`/`entry`/`setting`/`period`/`backup`/`auth` (D-12/D-13's corrected set) |
| `entity_id` | INTEGER NOT NULL | |
| `field` | TEXT NOT NULL | |
| `old_value` | TEXT NULL | |
| `new_value` | TEXT NULL | |
| `changed_at` | TEXT NOT NULL | |
| `cause` | TEXT NOT NULL | `entry`/`edit`/`correction`/`settings_change`/`period_close`/`manual_backup`/`console_backup` — the closed seven (T-M9.1-3). `reversal` retired (`reverse_entry` dropped — see [06](06-decision-log-and-open-items.md) §6); `restore` was added in error during S10, never part of this list, and retired in S14 alongside it — see `backup::restore_from_backup`'s doc comment for why a restore's own record lives in the S14 backups-manifest instead (§9.5) |

Index `(entity_type, entity_id)`.

### 4.4 Full DDL

```sql
CREATE TABLE members (
    id                    INTEGER PRIMARY KEY,
    name                  TEXT NOT NULL,
    phone                 TEXT NOT NULL UNIQUE,
    email                 TEXT NULL,
    address               TEXT NOT NULL,
    introducer_member_id  INTEGER NULL REFERENCES members(id),
    level                 INTEGER NOT NULL,
    is_active             INTEGER NOT NULL DEFAULT 1,
    joining_date          TEXT NOT NULL,
    consent_given         INTEGER NOT NULL,
    consent_date          TEXT NOT NULL,
    created_at            TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_members_phone ON members(phone);
CREATE INDEX idx_members_introducer ON members(introducer_member_id);
CREATE INDEX idx_members_name ON members(name);

CREATE TABLE business_volume_entries (
    id            INTEGER PRIMARY KEY,
    member_id     INTEGER NOT NULL REFERENCES members(id),
    amount        INTEGER NOT NULL CHECK (amount > 0),
    entry_date    TEXT NOT NULL,
    period_month  TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NULL
);
CREATE INDEX idx_bve_member_period ON business_volume_entries(member_id, period_month);

CREATE TABLE slab_table (
    id          INTEGER PRIMARY KEY,
    threshold   INTEGER NOT NULL,
    percentage  INTEGER NOT NULL,
    sort_order  INTEGER NOT NULL
);
CREATE UNIQUE INDEX idx_slab_threshold ON slab_table(threshold);

CREATE TABLE periods (
    id            INTEGER PRIMARY KEY,
    period_month  TEXT NOT NULL UNIQUE,
    status        TEXT NOT NULL CHECK (status IN ('open','awaiting_close','closed')),
    ended_at      TEXT NULL,
    closed_at     TEXT NULL
);
CREATE INDEX idx_periods_status ON periods(status);

CREATE TABLE member_period_totals (
    member_id               INTEGER NOT NULL REFERENCES members(id),
    period_id               INTEGER NOT NULL REFERENCES periods(id),
    business_volume         INTEGER NOT NULL,
    total_business_volume   INTEGER NOT NULL,
    slab_pct                INTEGER NOT NULL,
    differential             INTEGER NOT NULL,
    royalty                  INTEGER NOT NULL,
    own_reward               INTEGER NOT NULL,
    rewards                  INTEGER NOT NULL,
    PRIMARY KEY (member_id, period_id)
);

CREATE TABLE monthly_snapshots (
    id                       INTEGER PRIMARY KEY,
    member_id                INTEGER NOT NULL REFERENCES members(id),
    period_id                INTEGER NOT NULL REFERENCES periods(id),
    version                  INTEGER NOT NULL,
    business_volume          INTEGER NOT NULL,
    total_business_volume    INTEGER NOT NULL,
    slab_pct                 INTEGER NOT NULL,
    differential              INTEGER NOT NULL,
    royalty                   INTEGER NOT NULL,
    own_reward                INTEGER NOT NULL,
    rewards                   INTEGER NOT NULL,
    is_active_status          INTEGER NOT NULL,
    created_at                TEXT NOT NULL,
    UNIQUE (member_id, period_id, version)
);
CREATE INDEX idx_snapshots_period_version ON monthly_snapshots(period_id, version DESC);

CREATE TABLE backups (
    id                       INTEGER PRIMARY KEY,
    period_id                INTEGER NULL REFERENCES periods(id),
    kind                      TEXT NOT NULL CHECK (kind IN ('period_close','scheduled','manual','pre_restore_safety')),
    schedule_kind             TEXT NULL CHECK (schedule_kind IN ('daily','weekly','monthly')),
    version                  INTEGER NOT NULL,
    internal_retained_path    TEXT NOT NULL,
    external_medium_path      TEXT NULL,
    checksum                  TEXT NOT NULL,
    is_original               INTEGER NOT NULL,
    created_at                TEXT NOT NULL
);
CREATE INDEX idx_backups_period_version ON backups(period_id, version DESC);
CREATE INDEX idx_backups_kind_created ON backups(kind, created_at DESC);

CREATE TABLE settings (
    key    TEXT PRIMARY KEY,
    value  TEXT NOT NULL
);

CREATE TABLE auth (
    id                        INTEGER PRIMARY KEY CHECK (id = 1),
    pin_hash                  TEXT NULL,
    password_hash             TEXT NULL,
    failed_attempts           INTEGER NOT NULL DEFAULT 0,
    locked_until              TEXT NULL,
    recovery_codes            TEXT NOT NULL,
    session_timeout_minutes   INTEGER NOT NULL
);

CREATE TABLE audit_log (
    id           INTEGER PRIMARY KEY,
    entity_type  TEXT NOT NULL,
    entity_id    INTEGER NOT NULL,
    field        TEXT NOT NULL,
    old_value    TEXT NULL,
    new_value    TEXT NULL,
    changed_at   TEXT NOT NULL,
    cause        TEXT NOT NULL
);
CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id);
```

### 4.5 Seed data at first-run

- 7 default slab rows (§4.3 above / [02](02-business-rules.md) §4.3)
- 16 default settings values ([02](02-business-rules.md) §6)
- No member data — root is created interactively at first-run setup, never seeded
- No migration tooling — the system starts empty (NFR-16)

---

## 5. Calculation engine deep-dive

### 5.1 Algorithm

On a Business Volume write against member X in the currently open period:

```
1. business_volume(X) ← SUM(business_volume_entries WHERE member_id = X AND period = current)

2. chain ← path from X to root, X first, root last

3. for each N in chain, in order (X first):
   a. total_business_volume(N) ← business_volume(N) + Σ total_business_volume(c) for every direct child c of N
      // children off the chain already hold correct cached figures from prior writes —
      // only the one child on the chain has just changed.

   b. slab_pct(N) ← highest slab_table.percentage where slab_table.threshold <= total_business_volume(N)
                     (0% if below the lowest threshold)

   c. differential(N) ← Σ over EVERY direct child c of N:
          (slab_pct(N) − slab_pct(c)) × total_business_volume(c)
      // ALL direct children re-scanned here, not only the one on the chain — see 5.2.

   d. qualifying ← direct children of N whose slab_pct == top_slab_pct
      if count(qualifying) >= settings.royalty_min_children:
          royalty(N) ← Σ settings.royalty_rate × total_business_volume(c) for c in qualifying
      else:
          royalty(N) ← 0

   e. own_reward(N) ← slab_pct(N) × business_volume(N)
      // Rule-46 (CR-4, 8 Aug 2026): N's own Business Volume earns at N's own slab.
      // Additive only — does not change differential(N) or royalty(N) above.

   f. rewards(N) ← differential(N) + royalty(N) + own_reward(N)

   g. persist total_business_volume(N), slab_pct(N), differential(N), royalty(N), own_reward(N), rewards(N)
      to member_period_totals — never to business_volume, which changes only via entries.

4. Steps 3a–3g run inside ONE database transaction — either the whole chain updates
   consistently or none of it does.
```

### 5.2 Why differential re-scans all of a node's children, not just the changed one

**This is the detail most likely to be implemented wrong.** When X's Business Volume changes, X's own TBV changes, which may change X's slab. Moving up to `parent(X)`, its TBV also changes (it sums X's new figure), which may change its slab too. But `differential(parent(X))` sums over **every** direct child of `parent(X)`; the term for each is `(slab_pct(parent(X)) − slab_pct(child)) × TBV(child)`. If `parent(X)`'s own slab just changed, **every one of those terms changed**, including for children never touched by this write. Re-scanning only the one child on the chain would silently leave every sibling's term stale.

This is bounded work — the number of direct children at any level is the configured level width (default 9/6/3, advisory but typically small), so re-scanning is cheap.

### 5.3 Worked trace — Scenario 3

Reproducing [02](02-business-rules.md) §5.3 against this algorithm end to end. `chain = [p1, D, A]`.

1. At `p1`: leaf, no children — assume unchanged for this trace.
2. At `D`: `TBV(D) = BV(D) + Σ TBV(children incl. p1..p3) = 1,250` → `slab_pct(D) = 6%`. `differential(D)` re-scans **all** of D's children (p1, p2, p3), not just p1.
3. At `A`: `TBV(A) = BV(A) + Σ TBV(B..G) = 500 + 6×1,250 = 8,000` → `12%`. `differential(A)` re-scans **all six** of A's direct children B–G, each contributing `(12%−6%)×1,250 = 75`, total `450`. Royalty: no direct child of A is on the top slab → `royalty(A) = 0`. `own_reward(A) = 12% × 500 = 60` (Rule-46, CR-4 — A's own Business Volume, not its TBV). `rewards(A) = 450 + 0 + 60 = 510` — matches the golden value exactly. ✅

### 5.4 Complexity and the 2-second target

Each write touches `O(depth × average_width)` rows — bounded by tree depth (typically 4–5 levels) times children re-scanned per ancestor. At 25,000 members with single-digit-to-low-teens level widths, this is a few dozen row reads/writes per entry — **independent of total member count.** This is what makes the 2-second target (NFR-1) hold flat from 500 to 25,000 members.

---

## 6. API specification — 40 Tauri IPC commands

**40 commands total** — API-01 to API-40, no gaps. See [06](06-decision-log-and-open-items.md) C2. `reverse_entry` was removed (dead, confirmed) from the original 26-command count.

**No delete command exists anywhere, for any entity.** Members, entries, snapshots, backups — none are ever removed (Rule-28, Rule-42, Rule-31).

**Unauthenticated commands — the complete, closed list of seven** (C3): `login`, `setup_first_run`, `use_recovery_code`, `check_data_readable`, `list_restore_points`, `restore_from_backup`, `restore_from_backup_file`. Every other command requires an authenticated session. This list must never grow without revisiting [06](06-decision-log-and-open-items.md) C3 and the security matrix (§8 below).

Every mutating command runs inside exactly one DB transaction and produces exactly one `audit_log` entry (or one per changed field for `record_entry`/`edit_entry`). Read-only commands are never audited.

### Module M1 — Member Directory

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-01 | `create_root_member` | Create the single root member, once, at first-run setup | Auth (setup-mode) | Name/phone/address/consent required; only callable once | Single-row insert, no chain recalc | `entry` |
| API-02 | `add_member` | Onboard a non-root member | Auth | Reference ID → existing active member (Rule-30); phone unique active+inactive, offers reactivation (Rule-34); consent required (Rule-40); level/depth checks warn-only, never block | ID allocation (random, excludes 100000) + insert, one transaction | `entry` |
| API-03 | `edit_member` | Update name/phone/email/address | Auth | Phone uniqueness re-checked; **introducer field never accepted as editable input** (Rule-37, locked at the API layer) | Single-row update | Per changed field, `edit` |
| API-04 | `deactivate_member` | Mark inactive | Auth | Root cannot be deactivated | Single-row update; **no recalculation triggered** — zero calculation effect | `edit` |
| API-05 | `reactivate_member` | Reactivate, preserving ID/position/history | Auth | Member must currently be inactive | Single-row update | `edit` |
| API-06 | `search_members` | Search by name, ID **or phone** (Rule-44) | Auth | None — empty query → empty result, not an error. Phone clause engages only at ≥4 digits (V4.4); both sides reduced to a canonical key before comparison — digits only, then an international prefix or trunk zero dropped (see Rule-44) | Read-only | Not audited. Response includes `phone` so results can display it |

### Module M2 — Business Volume Entry

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-07 | `get_period_lock_status` | Report **which months accept entries**, oldest first, and which month is blocked and by what | Auth | — | Read-only | Not audited. Returns a list of recordable periods plus the blocking month, **not a boolean** (amended 7 Aug 2026, CR-2 — the name is retained for continuity; the semantics are now "entry eligibility", not "locked yes/no") |
| API-08 | `record_entry` | Record BV against a member, into the period its `entry_date` falls in | Auth | `amount > 0` (Rule-16a), ≤2 decimals (Rule-16), `entry_date` within its own period's bounds (V2.6). **Refused when that period is `closed`** (use API-09 instead, Rule-39), **and when it is the current month while any earlier period is `awaiting_close`** (V2.7, Rule-36) | Insert entry + chain-upward recalc (ADR-005) **within that entry's own period**, one transaction | `entry` |
| API-09 | `edit_entry` | Correct an entry — open period **or any closed month** | Auth | Same amount/date validation, scoped to the entry's own period bounds | Update entry + chain recalc; if period closed, additionally new `monthly_snapshots`/`backups` version | `edit` or `correction` |

### Module M3 — Calculation Engine *(no exposed commands except the preview)*

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-33 | `preview_settings_impact` | Dry run: what would the open period's figures become under candidate slab/royalty settings | Auth | Same shape checks as the settings commands; nothing persisted | **None — must not write.** Swap candidate in, recompute, restore in `finally`; a panic must never leave live settings holding uncommitted values | Not audited — nothing changed |

### Module M4 — Member Detail & Hierarchy Chart

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-10 | `get_member_detail` | Full detail: contact, Rewards breakdown (own-Business-Volume reward first, then per-leg differential, then royalty — Rule-46), direct children, TBV, leg count. Request: `member_id`, `period_month: string \| null` | Auth | member_id must exist | Read-only | Not audited |
| API-11 | `get_direct_children_chart` | Chart node data. Request: `member_id`, `full_tree: bool`, `period_month: string \| null`. With `full_tree: false` — the member and its direct children (FR-2). With `full_tree: true` — **the entire subtree**, which is what the full hierarchy window draws (FR-10, Rule-45) | Auth | member_id must exist | Read-only | Not audited |

**On API-11's `full_tree` flag.** The parameter was always in the command's contract; it is now put to work by FR-10 and no new command is introduced for the full hierarchy view. Either value returns the same node shape — name, ID, own Business Volume, active flag, introducer link — so FR-2's three-field constraint holds identically in both modes. The main window calls it once to obtain the count for the size gate (V4.5); the full hierarchy window calls it once more to draw. Both are cheap local reads against SQLite; the cost of the full view is in *rendering*, not in fetching, which is exactly why the render happens in a separate window.

**On API-10/API-11's `period_month` parameter (added S13, T-M2.5-3).** `null` resolves to the oldest recordable period (`get_period_lock_status`'s own ordering) — never "whichever period has the highest `period_id`," which is what both commands did before this parameter existed and is wrong once Rule-36/CR-2 allows two periods to be simultaneously `open`/`awaiting_close`. An explicit value selects any other recordable month via the month switcher (US-M2.5).

### Module M5 — Monthly Close

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-12 | `get_outstanding_periods` | List months awaiting close, oldest first | Auth | — | Read-only | Not audited |
| API-13 | `begin_close` | Start the close wizard for the oldest outstanding month | Auth | Only the oldest outstanding period may begin | None (prepare step) | Not audited |
| API-14 | `confirm_backup_and_close` | Generate + verify backup, then write snapshot and zero all live figures | Auth | Backup write must be verified (exists, checksum, readable) **before** any zeroing occurs | One transaction, backup-gated: write+verify backup → write snapshots v1 → zero `member_period_totals` → mark period closed. **Verify failure never begins the zeroing phase.** | `period_close` |
| API-15 | `manual_backup_current_period` | On-demand backup of the in-progress month, no zeroing | Auth | Same write-verify mechanism as API-14 | Single transaction: write + verify backup row | `manual_backup` |

### Module M6 — Reports & Exports

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-16 | `export_monthly` | Export current/selected month's data | Auth | Always includes 5 mandatory columns (Rule-19, [06](06-decision-log-and-open-items.md) C9/D-1) regardless of selection | Read-only | Not audited |
| API-17 | `export_yearly_average` | Export yearly average with snapshot-count denominator | Auth | — | Read-only | Not audited |
| API-18 | `export_low_contribution` | Export members below own-BV yearly-average threshold | Auth | — | Read-only | Not audited |
| API-19 | `list_backups` | List closed *periods* that have a snapshot, for the Reports screen's re-download card — not the same listing as API-35's whole-console backup rows | Auth | — | Read-only | Not audited |
| API-20 | `redownload_backup` | Re-download a past backup, always latest version | Auth | period_id must have ≥1 backup | Read-only | Not audited. This is the command backing "Closed month snapshot" — see [06](06-decision-log-and-open-items.md) C-history HIGH-1 |

### Module M7 — Settings

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-21 | `get_settings` | Fetch all current settings | Auth | — | Read-only | Not audited |
| API-22 | `update_settings` | Update one or more non-slab-table settings | Auth | Type/range checks per field | One transaction: write setting(s) + recalculate current open period only if royalty rate/min-children changed. **Caller must show the pre-save warning first**, sourced from API-33 | `settings_change` |
| API-23 | `add_slab_row` | Add a threshold/percentage row | Auth | Duplicate threshold rejected; **no monotonicity check** (Rule-41) | Insert row + recalculate current open period | logged |
| API-24 | `remove_slab_row` | Remove a slab row | Auth | Cannot remove the last remaining row | Delete row + recalculate current open period | logged |
| API-25 | `update_slab_row` | Edit a row's threshold/percentage | Auth | Duplicate threshold rejected; no monotonicity check | Update row + recalculate current open period | logged |
| API-37 | `get_console_backup_settings` | Fetch whole-console backup schedule and retention | Auth | — | Read-only | Not audited |
| API-38 | `update_console_backup_settings` | Change schedule/retention count | Auth | Schedule ∈ {off,daily,weekly,monthly}; retention ≥ 1 | Write setting(s) | `settings_change` |

### Module M8 — Authentication

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-26 | `setup_first_run` | First-run wizard: set PIN and/or password, generate recovery codes | **Unauthenticated** (only when no `auth` row exists) | PIN 6 numeric digits; password ≥8 chars, letter+number; ≥1 credential required | Hash + store credential(s) (Argon2id), generate + hash recovery codes | `entity_type = 'auth'`, cause `entry`, one entry per credential configured, naming only *that* a PIN/password was set ("PIN set") — never the value (T-M9.1-6). Implemented S14 — couldn't audit before then, since `audit_log` lives inside the database this call is what creates |
| API-27 | `login` | Authenticate with PIN or password | **Unauthenticated** (entry point) | Credential must match stored hash | Verify + update failed_attempts/locked_until | ⚠️ **Gap, not yet built:** "only on failed-lockout transitions" was never realizable as written — lockout state (`auth.json`) is written before any database connection exists, the same reason `use_recovery_code` below can't audit either. Left open at the end of S14; needs either a documented exception (no `audit_log` entry for lockout, ever) or a design that doesn't require one |
| API-28 | `lock_session` | Manually lock the session | Auth | — | Idempotent | Not audited |
| API-29 | `unlock_session` | Resume a locked session | Locked-session state | Same as `login` | Same as `login` | Same gap as `login` above |
| API-30 | `use_recovery_code` | Reset credential(s) via a one-time recovery code | **Unauthenticated** | Code must match an unused hashed code | Verify code, invalidate all old codes, set new credential, generate new codes | ⚠️ **Gap, not yet built:** genuinely unauthenticated (Rule-29's closed set of seven) — no database connection exists to write into, the same limitation restores had before S14's manifest. Unlike restores, there's no equivalent "record it in a file instead" answer here without writing something credential-adjacent to an unencrypted sidecar, which is a real new security question (T-M9.1-6), not a S14-sized fix |
| API-31 | `get_outstanding_alert` | Fetch current outstanding-month alert state | Auth | — | Read-only | Not audited |
| API-39 | `run_console_backup_now` | Take an immediate whole-console backup; also the internal call the schedule check makes at login | Auth | — (`kind` inferred: `manual` user-triggered, `scheduled` login-triggered) | Copy + checksum the live DB file; prune `scheduled`/`manual` rows beyond retention, oldest first | `console_backup` |

### Pre-flight / Data Recovery *(unauthenticated of necessity)*

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-34 | `check_data_readable` | Can the encrypted database be opened? Decides sign-in vs recovery screen | **Unauthenticated** | Two-layer (S14): sidecar exists and parses; database file exists and is page-aligned. A structurally-plausible-but-actually-corrupted file still passes — `login`'s own Argon2-succeeds-but-SQLCipher-open-fails ordering is the real backstop, reported as `AppError::DataUnreadable` | Read-only | Not audited |
| API-35 | `list_restore_points` | Retained backups, every `kind`, newest first | **Unauthenticated** | — | Read-only, reads the S14 backups-manifest (§9.5), never `backups` SQL — same call for this screen and the authenticated Settings Restore card | Not audited |
| API-36 | `restore_from_backup` | Replace the unreadable database with a retained backup | **Unauthenticated** | **Must verify checksum before overwriting** — mismatch → refused, nothing overwritten. Writes a `pre_restore_safety` backup of the current file first, to the manifest | One transaction: safety-backup → verify → restore → drop any session → leave app at sign-in | No `audit_log` entry (S14 revision — see §9.5's own explanation); the manifest's `pre_restore_safety` entry is the durable record instead |
| API-40 | `restore_from_backup_file` | Replace the current/not-yet-existing database with an admin-picked file | **Unauthenticated** | Same checksum-verify-first requirement. Backs both the first-run "Restore instead" link and Settings' "Restore from a file…" (frontend gates the latter behind its own checklist-confirm modal) | Same as API-36 | Same as API-36 |

### Module M9 — Audit Log

| ID | Command | Purpose | Auth | Key validation | Transaction | Audit |
|---|---|---|---|---|---|---|
| API-32 | `get_audit_log` | Retrieve the change log, filterable by member | Auth | — | Read-only | This command reads the log; it does not produce an entry |

### 6.1 Command index by ID

`API-01`–`API-06` M1 · `API-07`–`API-09` M2 · `API-10`–`API-11` M4 · `API-12`–`API-15` M5 · `API-16`–`API-20` M6 · `API-21`–`API-25`, `API-37`–`API-38` M7 · `API-26`–`API-31`, `API-39` M8 · `API-32` M9 · `API-33` M3 · `API-34`–`API-36`, `API-40` pre-flight/recovery.

---

## 7. State machines

### 7.1 Period lifecycle

```
[start] → open  (system start / previous period closed)
open → awaiting_close  (calendar month elapses — Rule-36)
awaiting_close → closed  (backup confirmed + snapshot written — Rule-18/38)
awaiting_close → awaiting_close  (backup fails/cancelled — abort, alert stays)
closed → [end]
```

**Entry eligibility by state (Rule-36 as amended, 7 Aug 2026 — CR-2):**

| State | Accepts new entries? |
|---|---|
| `awaiting_close` | ✅ Yes — for as long as it stays unclosed. This is the whole point of the amendment |
| `open` (the current month) | ✅ Only when **no** earlier period is `awaiting_close`; otherwise refused, naming that period |
| `closed` | ❌ Never via `record_entry` — corrections only, through API-09 (Rule-39) |

A period row for the current month is created as `open` as soon as the calendar month begins, whether or not it can yet be written to; its writability is a function of what sits behind it, not of its own state. Multiple periods can sit at `awaiting_close` simultaneously (Rule-20's queue) and **each of them accepts entries**; only the oldest is closable, and each closes through its own instance of this state machine. More than one live period at once is expected to be rare — it requires a month to be left unclosed past the end of the next one.

**Renamed 7 Aug 2026:** `ended_locked` → `awaiting_close`. The old name described a total entry lock that no longer exists; keeping it would have made the schema state the opposite of the behaviour. Documentation-only — no implementation exists yet.

### 7.2 Member lifecycle

```
[start] → active  (created: root via setup once; others via add_member)
active → inactive  (deactivate — display flag only, RQ-2)
inactive → active  (reactivate — contact-number match, Rule-34)
active → [end]     NEVER (no hard delete — Rule-28)
inactive → [end]   NEVER (no hard delete — Rule-28)
```

No "moved" or "transferred" state exists — Rule-37 makes `introducer_member_id` immutable from creation. The loop-prevention check (V1.8) is retained as a safeguard but, per Rule-30, can structurally never fire in normal operation once transfers are prohibited.

### 7.3 Correction/versioning flow — editing a closed period's entry

```
Edit entry in a CLOSED period
  → explicit on-screen warning naming the closed month
  → recompute that period's chain in isolation (not live totals)
  → insert monthly_snapshots rows at version = MAX(version)+1
  → insert new backups row, version incremented
  → write audit_log entry (before/after, cause=correction)
  → original backup row (is_original=true) untouched — never modified
```

---

## 8. Security architecture

### 8.1 Encryption at rest

SQLCipher-encrypted database file (ADR-003). The key is derived via **Argon2id** from the PIN and/or password at login (ADR-008), held only in Rust process memory for the session, never persisted in any form. Backup files — internal-retained and external-medium alike — inherit the same encryption.

### 8.2 Authentication & session

PIN and/or password, either authenticates if both are set. Argon2id hashing. Mandatory failed-attempt lockout regardless of credential type — 5 failed attempts triggers it (full ladder beyond that: [06](06-decision-log-and-open-items.md) O4). One-time recovery codes, generated once at first-run, shown once, hashed at rest. Inactivity timer (configurable, default 15 min per O3) locks the session and **drops the derived key from memory** — genuinely inaccessible, not merely UI-hidden.

### 8.3 Application boundary — Tauri capability allowlist

The WebView is granted **no** general filesystem, shell, or network capability. Every capability is one of the 40 named commands in §6. No network capability is declared in the Tauri configuration at all — a structural, not policy, enforcement of the offline requirement.

### 8.4 PII handling in exports

Export filenames and every visible string are drawn from the restricted vocabulary and never embed PII in the filename itself — a monthly extract filename identifies the period, not any member. Extract *contents* naturally carry PII, accepted under the same retention/consent basis as the live system.

### 8.5 Roles

Exactly one role: **Administrator**. Full access to everything — every member, every setting, every export, every backup. Network members have zero system access, no login, no screen, ever — a hard architectural boundary, not a configuration default.

| Resource | Operation | Permission |
|---|---|---|
| Members | Create, Read, Update, Deactivate/Reactivate | Full |
| Members | Hard delete | **Denied — not offered anywhere** |
| Members | Change introducer | **Denied — not offered anywhere** |
| Business Volume entries | Create, Read, Edit (incl. closed-month) | Full |
| Business Volume entries | Delete | **Denied — correction is always an edit that preserves history via versioning** |
| Settings / slab table | Read, Update, Add/Remove slab rows | Full |
| Monthly close | Trigger, view status | Full, constrained to oldest-outstanding-first |
| Backups | Generate, list, re-download | Full |
| Audit log | Read | Full, filterable |
| Own credentials | Set, change, recover | Full, self-service only |
| *(none)* | Members (as data subjects) — any operation | **No access of any kind** |

### 8.6 Pre-authentication surface — the exposure, stated plainly

`check_data_readable` and `list_restore_points` reveal only *that* backups exist and roughly when they were taken — no member data, no figures.

`restore_from_backup` and `restore_from_backup_file` are the **only destructive unauthenticated commands in the system**. Someone with physical access to an unlocked machine, or possession of a backup file, could roll data back to an earlier point, or bring an unrelated console's backup onto this machine. Accepted because: it destroys no backup (every version retained), reveals nothing, physical device access is already out of scope (§8.8), and every restore writes a `pre_restore_safety` backup of whatever was live first — one step back from irreversible either way. Both must verify the target's checksum before overwriting.

The restored database is still encrypted and still requires the credential to open. Restoring grants access to nothing — which is exactly why any authenticated session is dropped immediately after any restore.

### 8.7 Data protection summary

| Concern | Mechanism |
|---|---|
| Encryption at rest | SQLCipher, key derived via Argon2id at login |
| Encryption in transit | **Ruled inapplicable** — no network exists |
| No PII in filenames | Backup/export filenames never embed a member name/phone/ID |
| Phone on the landing screen (Rule-44) | Accepted. Search results display the phone number so the administrator can confirm the right person. It is personal data under the DPDP Act 2023, but visible only to the single administrator role, which already sees it on Member Detail and in every export (Rule-33). No new party gains access and no new surface is created — see §8.9 |
| Filesystem isolation | WebView has zero general filesystem/shell/network capability — only the 40 allowlisted commands |
| Vocabulary constraint | No excluded word in any user-visible string, including error messages, tooltips, filenames |

### 8.8 Threat model — what is and is not defended

| Threat | Defended? | How |
|---|---|---|
| Theft of the encrypted DB file or a backup, machine off/locked | **Yes** | SQLCipher; key never stored, only derivable from the credential |
| Brute-forcing the credential against a stolen encrypted file | **Yes, materially slowed** | Argon2id is memory-hard |
| Online brute-force against the running application | **Yes** | Mandatory failed-attempt lockout |
| A compromised dependency exfiltrating PII over the network | **Yes, structurally** | No network capability exists in the built application at all |
| Device stolen or accessed while unlocked mid-session | **No — explicitly out of scope** | Client's own physical-security responsibility, bounded only by the inactivity timer |
| Loss of both credential and all recovery codes | **No — accepted by design** | The direct cost of "nobody but the client can ever get in," no vendor backdoor |
| Slab-table misconfiguration producing a negative differential | **No — explicitly declined by the client** | Rule-41/ADR-009. Not a security issue; included because it's the one place a stated guarantee is not defended in code, by choice |
| A future attempt to add member-facing or networked access | **N/A — structurally prevented** | ADR-001's single-process design has no socket to attach it to |

### 8.9 Compliance — India's DPDP Act 2023

| Item | Status |
|---|---|
| Consent capture at collection | **Implemented** — mandatory checkbox + auto-captured date (Rule-40) |
| Purpose limitation | Implicit — data used only for hierarchy/reward calculation, never shared (no network exists to share it over) |
| Retention | **Permanent and complete, by explicit client requirement** — members never removed, all data persists including in exports |
| Data-subject correction | Handled — `edit_member`/`edit_entry` support correction of any field, fully audited |
| Data-subject erasure | **Out of scope by client requirement** — no erasure path exists and none is to be built. See [06](06-decision-log-and-open-items.md) §6 |
| Audit obligation | Covered by `audit_log` (M9) |

---

## 9. Backup, retention & disaster recovery

### 9.1 The close-time backup gate

Per Rule-18: the **internal retained copy** is the actual gate — write verified (exists, checksum, readable) **before** the close transaction proceeds to write snapshots and zero live figures. The **external-medium copy** is prompted for at the same time but is a convenience layer on top of the gate — a failed external write does **not** block the close, but the system re-prompts and reminds until an off-machine copy exists.

### 9.2 Versioning on correction

Per Rule-39/ADR-006: correcting a closed month never modifies the `backups` row where `is_original = true`. A new `backups` row is inserted at an incremented `version`, alongside a new `monthly_snapshots` version. Reporting and future restoration always resolve to `MAX(version)`.

### 9.3 What this design does not protect against

If the client never takes the external-medium backup, the internal copy and the live database sit on the same physical disk — a single hardware failure, theft, or loss destroys both. The system prompts and periodically reminds, but cannot force an external medium to be present. **This is a process discipline the client must maintain, stated plainly rather than implied solved** — TR-4.

### 9.4 On-demand backup

`manual_backup_current_period` (API-15) uses the same write-and-verify mechanism as the close-time backup, but writes to a distinct, clearly-labelled file and does not affect `periods.status` or trigger any zeroing.

### 9.5 Full-console backup & cross-device restore — Rule-43, ADR-012

**A second, orthogonal mechanism to §9.1–9.4's month-close gate — not a replacement.**

**What's backed up.** The single encrypted SQLCipher file, in full — every table, the credential row included. A restored console needs no re-setup: the same credential that unlocked the original machine unlocks the restored one, because it's the same file.

**Schedule.** `settings.console_backup_schedule` (off/daily/weekly/monthly) is checked once, at every successful `login` — the only moment the process is reliably running, since there is no background service while closed. A due backup runs via `run_console_backup_now` (`kind = scheduled`) before the UI takes over. A day the client never opens the console catches up at the next login. `run_console_backup_now` is also callable directly for on-demand backup (`kind = manual`).

**Retention.** After every `scheduled`/`manual` write, rows of those two kinds beyond `settings.console_backup_retention_count` (default 10) are deleted, oldest first. `period_close` rows (permanent) and `pre_restore_safety` rows are never pruned by this.

**Restore, and its safety net.** `restore_from_backup_file` is one mechanism behind three surfaces: a plain "Restore from a backup file instead" link on the ordinary first-run setup screen (a brand-new machine has no console to log into and no local backups to choose from — skips straight to a file picker); the same recovery screen the db-error path uses (reworded, not duplicated); and the authenticated Settings "Restore" card (gated behind the frontend's own checklist-confirm modal before the command is called). Every restore path writes one `pre_restore_safety` backup of whatever is currently live **before** overwriting it, and drops any live session immediately after — the restored file may hold a different credential.

**The backups manifest (S14, new).** `backups` rows live *inside* the SQLCipher file — unreadable without a key, and there is no key before login. API-34/35/36/40 are nonetheless unauthenticated of necessity (Rule-29's closed set of seven), so an unencrypted mirror, `backups-manifest.json` (sibling to the existing `auth.json` sidecar), carries exactly the fields §8.6 already says these commands may reveal pre-auth — id/kind/version/checksum/path/created_at, never member data or figures. Every `backups`-row write in `backup.rs` mirrors into it at the same call site, so the two can't drift independently. `list_restore_points` reads the manifest exclusively now — one function, one list, for both the authenticated Restore card and the unauthenticated data-recovery screen, rather than two lists that could disagree. This extends ADR-012's reasoning rather than reversing it: ADR-012 rejected a *second SQL table*; the manifest is a second location for the same reason a disk-encryption tool's keyslot header sits outside the volume it protects — the thing that answers "what's here" can't itself require the key to read.

A `pre_restore_safety` entry is manifest-only, never a `backups` SQL row, and is written *before* the physical overwrite while the live file is still the one it was resolved against — the S10-era version wrote it (and a `cause = 'restore'` `audit_log` row) *after* the overwrite, through the pre-restore `Connection`, whose key context no longer matched the file underneath it. Harmless only by accident (every existing test restored between same-key fixtures); AC-38's actual shape — a different credential on the restored file — is exactly what would have exposed it. No `audit_log` entry is written for a restore at all as of S14: writing one before the overwrite is silently discarded whenever the restored content predates that write (every restore-to-an-older-backup, by definition), and writing one after requires a connection to whatever database is now live, which may need a credential this process never had. The manifest's `pre_restore_safety` entry, with its own `created_at`, is the durable record instead.

**What this doesn't change.** The month-close backup gate, correction versioning, and the single-machine caveat (§9.3) all continue to apply to `period_close` rows exactly as before.

---

## 10. Deployment & packaging

- **Targets:** Windows (`.msi`/`.exe`) and macOS (`.dmg`/`.app`), one Tauri codebase.
- **Code signing:** required on both platforms to avoid "unknown publisher" warnings that would confuse a low-technical user.
- **No auto-update:** consistent with the no-network constraint. Version upgrades are a new installer, run manually. The maintainer proactively notifies the client of an available update.
- **First-run setup:** on first launch, no existing encrypted database → setup wizard runs unconditionally (create PIN/password, generate and display recovery codes once, create the root member, review default settings). A plain "Restore from a backup file instead" link is the only addition beyond the original design.
- **Install footprint:** ~10–20MB installer, no bundled browser runtime.

---

## 11. Technical risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| **TR-1** | SQLCipher's bundled build (vendored OpenSSL) is slow to compile and occasionally fragile across OS/toolchain updates | Medium | Low (build-time only) | Pin exact dependency versions; document the known-good toolchain |
| **TR-2** | Argon2id cost parameters tuned for a fast dev machine feel sluggish on the client's actual hardware | Low | Medium (UX friction at login) | Tune against a deliberately modest baseline machine before handover |
| **TR-3** | Tauri v2's plugin ecosystem is younger than Electron's; a needed capability may require a Rust command from scratch | Medium | Low (more dev time, not a design flaw) | Accepted trade-off of ADR-002; budget for it |
| **TR-4** | Single-machine data loss if the client never takes the external-medium backup | Medium | Critical | Prompts and reminds at every close; ultimately a client process discipline, stated plainly |
| **TR-5** | Fixed-point arithmetic bugs (an off-by-one in a ×100 conversion) are subtle and could silently misstate every downstream figure | Low (if the six-scenario suite is followed) | Critical | The six-scenario unit test suite exists specifically to catch this class of bug before any UI is built on top |
| **TR-6** | Solo maintainer, no second reviewer — a design flaw or security gap could ship unnoticed | Medium | Medium–High | This document set itself is the primary mitigation — decisions recorded with rationale for later re-examination |
| **TR-7** | The full hierarchy view (FR-10, Rule-45) is a **top-down** chart, whose width grows with the number of leaves rather than with depth. At the NFR-2 ceiling of 25,000 members the canvas is tens of thousands of pixels wide; a print spans many pages, and the first draw takes noticeably longer than a normal screen | Medium (only at large network sizes) | Medium (usability of one view; no data risk) | **Chosen deliberately by the client on 7 Aug 2026** over a width-stable indented outline, because it matches the Structure screen's visual language. Mitigations agreed at the same time: a 10% zoom floor (far below the main chart's range), fit-width, search-and-scroll inside the window, and the >60-descendant confirmation naming the exact count before anything is drawn. Isolation in a separate window means the cost never lands on the main console. If the client later finds it unusable at scale, the fallback is the indented-outline layout, not a rewrite of the data path |

---

## 12. Project & folder structure

```
management_system/
├── documents/
│   ├── final/               # this document set — the build reference
│   ├── business/             # user-needs-document.md, client-requirements-validation.md
│   ├── draft/                 # requirement-draft.md, requirement-spec.md, open-questions-checklist.md
│   ├── design/                 # architecture.md, ui-prototype-v2.html, ui-theme.md
│   └── implementation-readiness/  # 01–12, superseded by documents/final/ as of this consolidation
├── src/                       # React + TypeScript frontend
│   ├── screens/               # Home/Search, MemberDetail, AddEditMember, BVEntry,
│   │                          # HierarchyChart, Settings, MonthlyClose, Reports, Auth
│   ├── windows/               # FullHierarchy — the separate-window entry point (FR-10).
│   │                          # Its own root, its own render; shares only the node component
│   │                          # and design tokens with the main app, never live state
│   ├── components/            # shadcn/ui-based shared components
│   └── lib/                   # typed IPC command wrappers, formatting helpers
└── src-tauri/
    ├── src/
    │   ├── m1_members/         # Member & Structure
    │   ├── m2_entries/         # Business Volume Entry
    │   ├── m3_calc/            # Calculation Engine (pure, no I/O)
    │   ├── m4_search/          # Search & Chart
    │   ├── m5_close/           # Monthly Close & Permanent Record
    │   ├── m6_reports/         # Reporting & Exports
    │   ├── m7_settings/        # Settings
    │   ├── m8_auth/            # Access & Alerts
    │   ├── m9_audit/           # Audit & Technical Logging
    │   ├── db/                 # SQLCipher connection, migrations, schema
    │   └── error.rs            # Shared AppError type
    └── capabilities/            # Tauri command allowlist
```

Module boundaries in `src-tauri/src/` mirror §3.1 exactly, so [02](02-business-rules.md) §7's rule→module map also serves as a map of where to find each rule's implementation in code.
