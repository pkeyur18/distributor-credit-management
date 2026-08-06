# API Specification (Tauri IPC Command Surface)

This is not a REST/GraphQL API — per `architecture.md` ADR-001/ADR-002, the system is a single-process Tauri v2 application with **no network layer at all**. "API" here means the **typed, allowlisted IPC command surface** between the React/TypeScript Presentation container and the Rust Application container (`architecture.md` §6, Appendix C). Each command below is invoked as `invoke('<command_name>', payload)` from the frontend; there is no HTTP method or URL — the "Command" column is the literal Tauri command name, which is the closest equivalent.

**Source of truth**: `architecture.md` Appendix C, validated against `requirement-spec.md`'s FR-1–9/Rules 1–41 and the approved prototype's actual screens. Two corrections applied against the raw architecture text, both explained inline below:
- **`reverse_entry` is removed.** No requirement document describes reversal as functionally distinct from editing — `client-requirements-validation.md` RQ-7 treats "edited or reversed" as synonymous, resolving to a single edit-in-place action. Confirmed dead by the architect this session (see [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md)). `edit_entry` is the sole correction mechanism, for both open and closed periods.
- **The prototype's "Closed month snapshot" export is mapped to `redownload_backup`**, not treated as a 5th, undocumented export command. This is the recommended resolution of HIGH-1 (see [01](01-implementation-readiness-assessment.md) and [11](11-open-questions-and-decisions.md)) — architect confirmation still pending, but this is the safe default and is applied throughout this document.

All commands run inside the security boundary described in `architecture.md` §11.3: the WebView has zero general filesystem/shell/network capability, only these named, typed commands. No general-purpose query or filesystem command is exposed anywhere (a deliberate absence, not an oversight).

---

## Module M1 — Member Directory

| API ID | Command | Purpose | Actor | Authorization | Request | Response | Validation | Business Rules | Success | Error(s) | Idempotency | Transaction | Audit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| API-01 | `create_root_member` | Create the single root member, once, at first-run setup | Admin | Authenticated (setup-mode) | name, phone, email?, address, consent | Member record | Name/phone/address required; consent required; only callable once (no root exists yet) | Rule-30 (no Reference ID for root) | 201-equivalent (member created) | Root already exists → refused | Not idempotent — second call fails by design | Single-row insert, no chain recalculation needed (no children yet) | audit_log entry, cause=`entry` |
| API-02 | `add_member` | Onboard a new non-root member | Admin | Authenticated | name, phone, email?, address, reference_id, consent | Member record incl. assigned 6-digit ID | Reference ID must resolve to an existing **active** member (Rule-30); phone unique active+inactive (Rule-34, offers reactivation instead of erroring); consent required (Rule-40); level-width/depth checks are warn-only, never block (Rule-1, Rule-32) | Rule-30, Rule-34, Rule-35, Rule-40, Rule-1, Rule-32 | 201-equivalent | Reference ID not found/inactive → refused; phone already active → refused; phone matches inactive → returns reactivation offer, not an error | Not idempotent (each call creates a new member) | Single transaction: ID allocation (random, excludes 100000, Rule-35) + insert | audit_log entry |
| API-03 | `edit_member` | Update name/phone/email/address | Admin | Authenticated | member_id, updated fields | Updated member record | Phone uniqueness re-checked (Rule-34); introducer field is never accepted as editable input (Rule-37 — locked at the API layer, not just the UI) | Rule-28, Rule-34, Rule-37 | 200-equivalent | Phone collision → refused | Idempotent (same input → same result) | Single-row update | audit_log entry per changed field, cause=`edit` |
| API-04 | `deactivate_member` | Mark a member inactive | Admin | Authenticated | member_id | Updated member record | Root member cannot be deactivated | Rule-28 | 200-equivalent | Target is root → refused | Idempotent | Single-row update; **no recalculation triggered** — inactive has zero calculation effect (Rule-28 corrected) | audit_log entry |
| API-05 | `reactivate_member` | Reactivate a previously-deactivated member, preserving ID/position/history | Admin | Authenticated | member_id (or phone, when triggered from the duplicate-check flow) | Updated member record | Member must currently be inactive | Rule-34, Rule-28 | 200-equivalent | Member already active → refused | Idempotent | Single-row update | audit_log entry |
| API-06 | `search_members` | Search by name or 6-digit ID | Admin | Authenticated | query string | List of matching members (name, ID, TBV, slab, status) | None (empty query → empty result, not an error) | FR-1 | 200-equivalent | — | Read-only, always idempotent | None (read) | Not audited (read-only) |

## Module M2 — Business Volume Entry

| API ID | Command | Purpose | Actor | Authorization | Request | Response | Validation | Business Rules | Success | Error(s) | Idempotency | Transaction | Audit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| API-07 | `get_period_lock_status` | Check whether entry is currently locked | Admin | Authenticated | — | `{locked: bool, outstanding_months: [...]}` | — | Rule-36, Rule-20 | 200-equivalent | — | Read-only | None | Not audited |
| API-08 | `record_entry` | Record a new Business Volume entry against a member, in the current open period | Admin | Authenticated | member_id, amount, entry_date | Entry record + updated Rewards chain for that member's ancestors | amount > 0 (Rule-16a), ≤2 decimals (Rule-16), entry_date within current period bounds; **refused entirely if a reset is outstanding** (Rule-36) | Rule-15, Rule-16, Rule-16a, Rule-26, Rule-36 | 201-equivalent | Recording locked → refused with the name of the outstanding month; invalid amount → refused | Not idempotent (each call adds a new entry — this is the append-only ledger model) | One transaction: insert entry + chain-upward recalculation (Rule-26/ADR-005) of every ancestor up to root | audit_log entry, cause=`entry` |
| API-09 | `edit_entry` | Correct an existing entry — in the open period, or in **any closed month** | Admin | Authenticated | entry_id, corrected amount and/or date | Updated entry + recalculated chain (+ new snapshot version if the entry belongs to a closed month) | Same amount/date validation as `record_entry`, scoped to the entry's own period bounds | Rule-39 (extends Rule-38) | 200-equivalent | Invalid amount → refused | Idempotent (same correction reapplied → same result) | One transaction: update entry + chain-upward recalculation; if period is closed, additionally: new `monthly_snapshots` version + new `backups` version (original version untouched) | audit_log entry, cause=`edit` or `correction` |

## Module M3 — Calculation Engine

**No commands are exposed.** Deliberate, per `architecture.md` line 565–567 — there is no "recalculate" button anywhere in the product (Rule-26), so nothing external ever needs to trigger this module directly. It runs exclusively as an internal side-effect of `record_entry`, `edit_entry`, `update_settings`, `add_slab_row`, `remove_slab_row`, and `update_slab_row`.

## Module M4 — Member Detail & Hierarchy Chart

| API ID | Command | Purpose | Actor | Authorization | Request | Response | Validation | Business Rules | Success | Error(s) | Idempotency | Transaction | Audit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| API-10 | `get_member_detail` | Full detail view: contact info, Rewards breakdown, direct children, TBV, leg count | Admin | Authenticated | member_id | Member detail payload (FR-3 field set) | member_id must exist | FR-3 | 200-equivalent | Not found → refused | Read-only | None | Not audited |
| API-11 | `get_direct_children_chart` | Hierarchy chart node data for a member and its direct children | Admin | Authenticated | member_id, `full_tree: bool` | List of nodes: name, ID, **own** Business Volume only (Rule-7/FR-2 note) | member_id must exist | FR-2 | 200-equivalent | Not found → refused | Read-only | None | Not audited |

## Module M5 — Monthly Close

| API ID | Command | Purpose | Actor | Authorization | Request | Response | Validation | Business Rules | Success | Error(s) | Idempotency | Transaction | Audit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| API-12 | `get_outstanding_periods` | List months awaiting close, oldest first | Admin | Authenticated | — | Ordered list of periods with status | — | Rule-20, Rule-21 | 200-equivalent | — | Read-only | None | Not audited |
| API-13 | `begin_close` | Start the close wizard for the oldest outstanding month | Admin | Authenticated | period_id (must be the oldest outstanding) | Wizard session/state | Only the oldest outstanding period may begin | Rule-20 | 200-equivalent | Attempt to close a non-oldest period → refused | Idempotent (re-entering the wizard for the same period is safe) | None (read/prepare step, no data mutation yet) | Not audited (no state change) |
| API-14 | `confirm_backup_and_close` | Generate + verify backup, then write snapshot and zero all live figures | Admin | Authenticated | period_id | Close result: snapshot summary, backup record | Backup write must be verified (exists, checksum, readable) **before** any zeroing occurs (Rule-18) | Rule-18, Rule-38, Rule-31 | 200-equivalent, period now `closed` | Backup generation/verification fails → **abort entirely**, no data touched, alert stays up | Not idempotent in the sense of re-closing, but safe to retry after a failed attempt (nothing was mutated) | One transaction, backup-gated: write+verify backup → write monthly_snapshots (version 1) → zero member_period_totals → mark period closed. If the verify step fails, the transaction never begins the zeroing phase. | audit_log entry, cause=`period_close` |
| API-15 | `manual_backup_current_period` | On-demand backup of the in-progress (still-open) month, no zeroing | Admin | Authenticated | — | Backup record | Same write-verify mechanism as API-14, distinct file, no state change | Rule-31 (M5.8) | 200-equivalent | Write failure → refused, retry available | Idempotent (repeatable, produces a new timestamped backup each time) | Single transaction: write + verify backup row | audit_log entry, cause=`manual_backup` |

## Module M6 — Reports & Exports

| API ID | Command | Purpose | Actor | Authorization | Request | Response | Validation | Business Rules | Success | Error(s) | Idempotency | Transaction | Audit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| API-16 | `export_monthly` | Export current/selected month's data | Admin | Authenticated | period_id, selected optional columns | `.xlsx` file | Always includes the 4 mandatory columns (Rule-19) regardless of selection | Rule-19, Rule-33 | 200-equivalent | — | Idempotent (read-only export) | None (read) | Not audited (read-only export; the underlying data is already audited) |
| API-17 | `export_yearly_average` | Export yearly average per member, with snapshot-count denominator | Admin | Authenticated | yearly_cycle bounds (from settings) | `.xlsx` file | — | Rule-23 | 200-equivalent | — | Idempotent | None (read) | Not audited |
| API-18 | `export_low_contribution` | Export members below the own-BV yearly-average threshold | Admin | Authenticated | threshold override? (defaults to settings) | `.xlsx` file | — | Rule-24 | 200-equivalent | — | Idempotent | None (read) | Not audited |
| API-19 | `list_backups` | List all retained backups (regular monthly + manual + versioned corrections) | Admin | Authenticated | — | List of backup records with version, date, `is_original` flag | — | Rule-31, Rule-39 | 200-equivalent | — | Read-only | None | Not audited |
| API-20 | `redownload_backup` | Re-download any past backup, **always the latest version** for a corrected month | Admin | Authenticated | period_id (+ optional explicit version) | `.xlsx` / backup file | period_id must have at least one backup | Rule-31, Rule-39 | 200-equivalent | Not found → refused | Idempotent | None (read) | Not audited | This is the command backing the prototype's "Closed month snapshot" export card — see the HIGH-1 note at the top of this document. |

## Module M7 — Settings

| API ID | Command | Purpose | Actor | Authorization | Request | Response | Validation | Business Rules | Success | Error(s) | Idempotency | Transaction | Audit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| API-21 | `get_settings` | Fetch all current settings | Admin | Authenticated | — | Full settings payload (13 items, Appendix B) | — | FR-6 | 200-equivalent | — | Read-only | None | Not audited |
| API-22 | `update_settings` | Update one or more non-slab-table settings | Admin | Authenticated | changed key/value pairs | Updated settings | Type/range checks per field | FR-6 | 200-equivalent | Invalid value → refused | Idempotent | One transaction: write setting(s) + **recalculate the current open period only** if the change affects live figures (royalty rate/min-children — see the settings-mid-period-warning gap in [11](11-open-questions-and-decisions.md)) | audit_log entry, cause=`settings_change` |
| API-23 | `add_slab_row` | Add a new slab threshold/percentage row | Admin | Authenticated | threshold, percentage | Updated slab table | Duplicate threshold rejected; **no monotonicity check** (Rule-41, deliberate) | Rule-27 | 200-equivalent | Duplicate threshold → refused | Idempotent | One transaction: insert row + recalculate current open period | audit_log entry |
| API-24 | `remove_slab_row` | Remove a slab row | Admin | Authenticated | row_id | Updated slab table | Cannot remove the last remaining row (system must always have at least one slab, the implicit 0% base is separate/hardcoded) | Rule-27 | 200-equivalent | Would leave zero rows → refused | Idempotent | One transaction: delete row + recalculate current open period | audit_log entry |
| API-25 | `update_slab_row` | Edit a slab row's threshold/percentage | Admin | Authenticated | row_id, new threshold/percentage | Updated slab table | Duplicate threshold rejected; no monotonicity check (Rule-41) | Rule-4, Rule-41 | 200-equivalent | Duplicate threshold → refused | Idempotent | One transaction: update row + recalculate current open period | audit_log entry |

## Module M8 — Authentication

| API ID | Command | Purpose | Actor | Authorization | Request | Response | Validation | Business Rules | Success | Error(s) | Idempotency | Transaction | Audit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| API-26 | `setup_first_run` | First-run wizard: set PIN and/or password, generate recovery codes | Admin | Unauthenticated (only callable when no `auth` row exists) | pin? and/or password | Recovery codes (shown once) | PIN: 6 numeric digits. Password: ≥8 chars, letter+number. At least one credential required. | Rule-29 | 201-equivalent | Auth already configured → refused | Not idempotent (single first-run action) | One transaction: hash + store credential(s) (Argon2id), generate + hash recovery codes | audit_log entry (auth setup) |
| API-27 | `login` | Authenticate with PIN or password | Admin | None (this is the auth entry point) | pin or password | Session token/state | Credential must match a stored hash | Rule-29, lockout rule | 200-equivalent | Wrong credential → generic "incorrect" error, increments failed_attempts; too many failures → locked-out response with countdown | N/A (each attempt is a distinct event) | Single transaction: verify + update failed_attempts/locked_until | audit_log entry only on failed-lockout transitions, not on every attempt (avoid flooding the log) |
| API-28 | `lock_session` | Manually lock the session (or triggered by inactivity timeout) | Admin | Authenticated | — | Locked state | — | NFR-4 | 200-equivalent | — | Idempotent | None | Not audited |
| API-29 | `unlock_session` | Resume a locked session with the same credential | Admin | Locked-session state | pin or password | Resumed session | Same as `login` | NFR-4, lockout rule | 200-equivalent | Same as `login` | N/A | Single transaction, same as `login` | Same as `login` |
| API-30 | `use_recovery_code` | Reset credential(s) using a one-time recovery code | Admin | Unauthenticated (recovery flow) | recovery_code, new pin/password | New recovery codes (old ones invalidated) | Recovery code must match an unused hashed code | Rule-29 | 200-equivalent | Invalid/used code → refused | Not idempotent (code is single-use, consumed on success) | One transaction: verify code, invalidate all old codes, set new credential, generate new codes | audit_log entry (credential recovery) |
| API-31 | `get_outstanding_alert` | Fetch the current outstanding-month alert state for the persistent banner | Admin | Authenticated | — | `{months: [...]}` or empty | — | Rule-20 | 200-equivalent | — | Read-only | None | Not audited |

## Module M9 — Audit Log

| API ID | Command | Purpose | Actor | Authorization | Request | Response | Validation | Business Rules | Success | Error(s) | Idempotency | Transaction | Audit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| API-32 | `get_audit_log` | Retrieve the change log, filterable by member | Admin | Authenticated | filter (member name/id)? | List of audit entries (date, member, field, before, after, cause) | — | NFR-5 | 200-equivalent | — | Read-only | None | This command reads the audit log; it does not itself produce an entry. |

---

## Command surface summary

- **32 commands total** across 9 modules (M3 exposes none, by design).
- **`reverse_entry` removed** from the 26-command count in the raw architecture text (was double-counted with `edit_entry` for the same use case) — see the note at the top of this document.
- Every command that mutates data runs inside exactly one DB transaction and produces exactly one `audit_log` entry (or, for `record_entry`/`edit_entry`, one entry per changed field), consistent with NFR-5/M9.
- Read-only commands (`search_members`, `get_member_detail`, `get_direct_children_chart`, export commands, `get_settings`, `list_backups`, `get_audit_log`, `get_outstanding_*`) are never audited — auditing tracks changes, not reads.
