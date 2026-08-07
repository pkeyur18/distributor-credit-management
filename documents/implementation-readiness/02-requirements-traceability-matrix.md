# Requirements Traceability Matrix

Base requirement IDs reuse the project's own existing numbering — `Rule-##` from `requirement-spec.md` (the stable, everywhere-referenced business-rule IDs), `FR-#` for the nine functional-requirement sections, and `NFR-#` for the sixteen non-functional items in `client-requirements-validation.md` §11 / `architecture.md` §12. No parallel `REQ-###` scheme is invented — it would only create a second ID space to keep in sync with the first.

**Status legend**: **Fully traced** — requirement has a clean design component, UI screen, API command and DB entity. **Partially traced** — traced, but see the note (usually: original spec wording superseded by a later, higher-precedence correction). **Conflicting (resolved)** — `requirement-spec.md`'s original text conflicts with `client-requirements-validation.md`; the conflict is real, but already resolved by the tier-1 document, resolution recorded in the Notes column. **Missing** — no design/UI/DB counterpart found. **Needs clarification** — genuinely open, see [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md).

Design components are cited by module (M1–M9 per `architecture.md` §6/§7). API commands per Appendix C (as corrected — `reverse_entry` removed, see [04-api-specification.md](04-api-specification.md)). DB entities per Appendix A DDL (see [05-data-model-specification.md](05-data-model-specification.md)).

---

## FR-1 / FR-2 / FR-3 — Search, Hierarchy Chart, Member Detail (Module M4, plus M1 identity rules)

| ID | Description | User Need | Design Component | UI Screen | API Command | DB Entity | Test Scenario | Status | Notes |
|---|---|---|---|---|---|---|---|---|---|
| FR-1 | Search by name, 6-digit ID **or phone number**, opens detail with 1-depth children | UN-15, UN-29 | M1/M4 | Home/Search | `search_members` | members | TEST-FR1 | Fully traced | Phone matching added 7 Aug 2026 by CR-1 — see Rule-44 below. |
| FR-2 | Hierarchy chart: name, ID, **own** Business Volume only (not TBV) | UN-16 | M4 | Structure | `get_direct_children_chart` | members, business_volume_entries | TEST-FR2 | Fully traced | Client re-confirmed own-BV over the architect's TBV recommendation (Q-I2). Chart therefore cannot on its own explain a member's slab. The same three-field constraint binds FR-10's full hierarchy window. |
| FR-10 | Full hierarchy view: whole structure from the top member, every branch expanded, in a separate read-only window that draws once and never updates | UN-31 | M4 | Full Hierarchy window (opened from Structure) | `get_direct_children_chart` (`full_tree: true`) | members, member_period_totals | TEST-FR10 | Fully traced | New 7 Aug 2026, CR-3. No new command — the pre-existing `full_tree` parameter. Accepted scale limit recorded as TR-7. |
| Rule-44 | Phone number is a search key: name substring, ID digits, or phone digits with a four-digit floor; results display the phone | UN-29 | M1/M2/M4 | every search box (Home, Structure, BV Entry, Correction, Add Member reference lookup) | `search_members` | members.phone (UNIQUE, indexed) | TEST-R44 | Fully traced | New 7 Aug 2026, CR-1. One shared search function — behaviour must not differ between screens. Digits-only comparison; stored value never rewritten. |
| Rule-45 | Full hierarchy view is a point-in-time, read-only draw, gated above 60 descendants by a confirmation naming the exact count | UN-31 | M4 | Full Hierarchy window | `get_direct_children_chart` (`full_tree: true`) | members, member_period_totals | TEST-R45 | Fully traced | New 7 Aug 2026, CR-3. Read-only, no live refresh — that is the rule, not an omission. |
| FR-3 | Member detail: name, phone, address, full Rewards breakdown, direct children (1 depth), TBV, leg count | UN-17 | M4 | Member Detail | `get_member_detail` | members, member_period_totals | TEST-FR3 | Fully traced | |
| Rule-2 | Unique 6-digit member ID, primary lookup key | UN-01 | M1 | all screens showing an ID | `add_member` | members.id | TEST-R2 | Fully traced | |
| Rule-1 | Level-width defaults (L2=9, L3=6, L4=3) are advisory, never block onboarding | UN-05 | M1 | Add Member, Structure (warning banner) | `add_member`, `update_settings` | settings | TEST-R1 | Fully traced | |
| Rule-32 | Depth overflow warns but allows | UN-05 | M1 | Add Member | `add_member` | settings.hierarchy_depth | TEST-R32 | Fully traced | Same advisory pattern as Rule-1. |
| V4.5 | ">60 descendants" confirm-before-open gate, naming the exact member count | UN-31 | M4 | Full Hierarchy window (the action that opens it) | `get_direct_children_chart` (`full_tree: true`, for the count) | members | TEST-V45 | Fully traced | **Was previously untraced prototype behaviour with no source rule.** CR-3 (7 Aug 2026) gave it a rule (Rule-45), a validation row and a home: it now gates the full hierarchy window, which is the only view that draws an unbounded number of nodes. The one-branch-at-a-time Structure chart is bounded by a single generation and needs no gate. LOW-1 is closed by this. |

## FR-4 — Add/Edit/Deactivate Member (Module M1)

| ID | Description | User Need | Design Component | UI Screen | API Command | DB Entity | Test Scenario | Status | Notes |
|---|---|---|---|---|---|---|---|---|---|
| FR-4 | Add member: name, phone, email (optional), Reference ID (mandatory), address; assigns 6-digit ID on save | UN-02, UN-03 | M1 | Add Member modal | `add_member` | members | TEST-FR4 | Fully traced | |
| Rule-30 | Reference ID must resolve to existing **active** member; root created once at setup, no Reference ID; loop-creating moves blocked | UN-03, UN-04 | M1 | Add Member modal | `add_member`, `create_root_member` | members | TEST-R30 | Fully traced | Loop-check is belt-and-braces only — Rule-37 makes cycles structurally impossible. |
| Rule-34 | Phone number unique across active **and** inactive members; match on inactive offers reactivation, preserving ID/position/history | UN-02 | M1 | Add Member modal (inline duplicate-check) | `add_member`, `reactivate_member` | members.phone (UNIQUE) | TEST-R34 | Fully traced | |
| Rule-35 | Member ID randomly allocated, never sequential, never released | UN-01 | M1 | (system-assigned, not user-facing) | `add_member` | members.id | TEST-R35 | **Conflicting (resolved)** | `requirement-spec.md` states range 100000–999999; `client-requirements-validation.md` narrows it to **100001–999999** (confirmed 4 Aug 2026). Tier-1 document wins — use 100001–999999. `architecture.md`'s DDL already reflects the corrected range. |
| Rule-37 | Introducer/sponsor fixed at creation, never changes — no transfer, no override | UN-04 | M1 | Edit Member modal (introducer field locked/disabled) | `edit_member` | members.introducer_member_id (immutable after insert) | TEST-R37 | Fully traced | Supersedes an earlier draft decision (moves-with-frozen-months); struck through in `requirement-spec.md`, retained for record. |
| Rule-28 | Edit permitted any time; deactivate (not hard-delete) stops appearance in new periods | UN-05 | M1 | Edit/Deactivate Member modal | `edit_member`, `deactivate_member`, `reactivate_member` | members.is_active | TEST-R28 | **Conflicting (resolved)** | `requirement-spec.md` line 422 says inactive members "stop appearing in new periods." `client-requirements-validation.md` V3.5 states inactive status has **zero effect on any calculation** — a pure display flag; members keep appearing, colour-coded. The validation document itself flags this exact wording gap. Tier-1 wins: `is_active` is display-only. `architecture.md`'s data model already models it this way. |
| — | Mandatory consent checkbox + captured date at Add Member | (data-protection compliance) | M1 | Add Member modal | `add_member` | members.consent_given, members.consent_date | TEST-CONSENT | Fully traced | New in `client-requirements-validation.md` (M1.7, RQ-22), not present in Rules 1–38's original numbering. DPDP Act 2023-driven. |
| Rule-42 | Members are **never** removed; all data persists permanently, including in every export | UN-05 | M1, M6 | (no delete control anywhere) | none — no delete command exists | members (no delete path) | TEST-R42 | Fully traced | Client requirement, confirmed 6 Aug 2026. The reason behind Rule-28's no-hard-delete and Rule-38's permanent snapshots. Do not propose an erasure path. |

## FR-5 — Business Volume Entry (Module M2)

| ID | Description | User Need | Design Component | UI Screen | API Command | DB Entity | Test Scenario | Status | Notes |
|---|---|---|---|---|---|---|---|---|---|
| FR-5 / Rule-15 | Search, select member, record Business Volume | UN-07 | M2 | BV Entry | `record_entry` | business_volume_entries | TEST-FR5 | Fully traced | Target <15s for a known member (SC-5/AC-15). |
| Rule-16 | Business Volume entered directly, up to 2 decimals, no currency mode | UN-07, UN-08 | M2 | BV Entry | `record_entry` | business_volume_entries.amount | TEST-R16 | Fully traced | Supersedes the original "two entry modes" decision (rupee mode dropped entirely). |
| Rule-22 | 2 decimal places throughout storage/calc; rounding only at display | UN-08 | M3 | (all figure displays) | — (internal) | fixed-point ×100 integer columns throughout | TEST-R22 | Fully traced | No per-child-term rounding before summing. |
| — | Zero **and** negative Business Volume both refused | UN-08 | M2 | BV Entry (inline validation) | `record_entry` | business_volume_entries (CHECK amount > 0) | TEST-VOL-ZERO | Fully traced | `client-requirements-validation.md` V2.4 — stricter than the architect's own original recommendation ("accept zero, refuse negative"), client explicitly overrode it. |
| Rule-36 | An ended-but-unclosed month keeps accepting entries; the **current** month is refused while any earlier month is outstanding, naming it | UN-14, UN-30 | M2, M5 | BV Entry (form with recording-month note, no locked state) | `record_entry`, `get_period_lock_status` | periods.status (`awaiting_close`), business_volume_entries.period_month, member_period_totals | TEST-R36 | Fully traced | **Amended 7 Aug 2026 by CR-2**, reversing RQ-11's answer of 3 Aug 2026. Previously: "entry locked entirely once a calendar month ends, until that month's reset completes." The lock is narrowed, not removed. `periods.status` value renamed `ended_locked` → `awaiting_close`. |
| — | Recalculation of an entry into a not-yet-closed month is confined to **that** period; no other live period's figures change | UN-09, UN-30 | M2/M3 | (implicit) | `record_entry` | member_period_totals (one row set per not-yet-closed period) | TEST-PERIOD-ISOLATION | Fully traced | New 7 Aug 2026, CR-2. `member_period_totals` may now hold rows for more than one not-yet-closed period; the composite PK already supports it, no schema change. |
| — | Month selector on the entry screen and figure screens, rendered **only** when more than one month is outstanding | — | M2/M4 | BV Entry, Home, Member Detail, Structure | `get_period_lock_status` | periods | TEST-MONTH-SWITCHER | Fully traced | New 7 Aug 2026, CR-2. Client's explicit preference: nothing new appears on screen in the ordinary single-month case. Figures default to the **oldest** outstanding month. |
| Rule-26 | Recalculation is immediate on every entry, chain-upward only (no full-tree rebuild) | UN-09, UN-14 | M3 (ADR-005) | (implicit — no "recalculate" button anywhere) | (triggered internally by `record_entry`/`edit_entry`) | member_period_totals | TEST-R26 | Fully traced | Internal design note; no visible difference to the admin. |
| — | Entries editable/reversible at any time, **including in closed months**; correction writes a new backup **version**, never overwrites the original | UN-21 | M2/M5 | BV Entry → "Correct a closed month" panel | `edit_entry` | business_volume_entries, monthly_snapshots (versioned), backups (versioned), audit_log | TEST-CORRECTION | **Partially traced (extended)** | `client-requirements-validation.md` RQ-7 — a deliberate reversal of the "permanent, uncorrectable once closed" framing, which the validation document states was the architect's own gloss on Rule-38, not the client's actual requirement. `reverse_entry` as a *distinct* IPC command is dropped (see [04](04-api-specification.md)) — `edit_entry` alone, always audited, is the mechanism. |

## FR-6 / FR-7 — Settings, Monthly Reset (Modules M5, M7)

| ID | Description | User Need | Design Component | UI Screen | API Command | DB Entity | Test Scenario | Status | Notes |
|---|---|---|---|---|---|---|---|---|---|
| FR-6 | All §7 settings editable | UN-25 | M7 | Settings | `get_settings`/`update_settings`, slab-row commands | settings, slab_table | TEST-FR6 | Fully traced | |
| Rule-3 / Rule-4 | Slab = highest threshold ≤ TBV; every threshold/percentage editable | UN-09 | M3/M7 | Settings (Slab table section) | `update_settings`, `update_slab_row` | slab_table | TEST-R3-4 | Fully traced | |
| Rule-27 | Slab rows addable/removable; top slab always the highest-percentage row | UN-09 | M7 | Settings (Slab table section) | `add_slab_row`, `remove_slab_row` | slab_table | TEST-R27 | Fully traced | |
| — | Slab-table monotonicity is **not** software-validated | (accepted risk) | M7 | Settings (explicit on-screen disclaimer already in prototype) | `update_slab_row` | slab_table | TEST-EDGE-MONOTONIC | Fully traced (as an accepted, documented risk) | ADR-009: client explicitly declined the safeguard (3 Aug 2026). Not a gap — a deliberate choice, must not be silently "fixed" by a future developer. |
| — | Settings change mid-period triggers recalculation of the **current open period only**, behind a pre-save warning | UN-25 | M7/M3 | Settings (warning modal) | `preview_settings_impact` (API-33), then `update_settings` / slab-row commands | member_period_totals | TEST-EDGE-SETTINGS | Fully traced | RQ-18/V7.6. Designed and built 6 Aug 2026 (variant C): names the open month, states closed months are unaffected, shows Rewards before → after, lists affected members. Fires on slab-table and royalty saves only. The preview needs its own command — the engine is Rust-side. See US-M7.3 in [09](09-implementation-backlog.md). |
| — | Data recovery when the database cannot be opened at launch | (derived — P-1 "every recovery path must be self-evident from the screen") | M5/M8 | Data-recovery screen (pre-sign-in) | `check_data_readable`, `list_restore_points`, `restore_from_backup` (API-34–36, unauthenticated) | backups | TEST-RECOVERY-DB | Fully traced | Decided and built 6 Aug 2026 (LOW-3, design D). Not in any source document — a gap this analysis identified and the architect approved filling. |
| Rule-43 | Whole-console backup on a configurable schedule (off/daily/weekly/monthly) or on demand; retained count client-adjustable (default 10); restorable on any machine, including a brand-new install | UN-28 | M7/M8 | Settings (Backup schedule, Restore cards), first-run setup screen's restore link, data-recovery screen (shared/reworded) | `get_console_backup_settings`/`update_console_backup_settings`, `run_console_backup_now`, `restore_from_backup_file` (API-37–40) | backups (`kind`/`schedule_kind`), settings | TEST-R43 | Fully traced | New client requirement, confirmed 7 Aug 2026 (RQ-23, M7.7/M8.6/M8.7). Generalizes the `backups` table (ADR-012) rather than adding a second one — orthogonal to, and does not alter, Rule-18's month-close gate or Rule-39's correction versioning. |
| FR-7 / Rule-17 | Reset is manual only; prompted on the 1st, admin may act later | UN-18 | M5 | Monthly Close status page | `get_outstanding_periods` | periods | TEST-FR7 | Fully traced | |
| Rule-18 | Reset flow: backup must be confirmed successful before anything is zeroed; failed/cancelled backup aborts, alert stays | UN-18, UN-19 | M5 | Monthly Close wizard | `begin_close`, `confirm_backup_and_close` | backups, monthly_snapshots, periods | TEST-R18 | Fully traced | |
| Rule-20 | Persistent, undismissable alert (banner + notification) while a month is outstanding; multiple outstanding months close oldest-first, each with its own backup/snapshot | UN-18 | M5/M8 | Global banner, Monthly Close status page | `get_outstanding_alert`, `get_outstanding_periods` | periods | TEST-R20 | Fully traced | |
| Rule-21 | Period = calendar month; reset closes whichever month it belongs to | UN-18 | M5 | Monthly Close wizard | `begin_close` | periods | TEST-R21 | **Partially traced** | 3rd bullet (late entries fall into the closing month) is struck through/superseded by Rule-36's hard entry lock — retained in the spec for record, no longer reachable in practice. |
| Rule-38 | Reset zeroes everything (BV, TBV, Rewards, royalty); immutable snapshot written first; all yearly reporting reads snapshots only | UN-19, UN-20 | M5 | Monthly Close wizard | `confirm_backup_and_close` | monthly_snapshots, member_period_totals | TEST-R38 | **Partially traced (extended)** | "Immutable" is qualified by the later correction above (entries editable post-close → new snapshot **version**, original never modified). Snapshot-per-version model already reflected in `architecture.md`'s DDL (`monthly_snapshots.version`, `UNIQUE(member_id, period_id, version)`). |
| Rule-31 | Backup downloaded locally **and** retained permanently in-system; nothing auto-deleted | UN-19 | M5/M6 | Monthly Close wizard, Reports (re-download) | `confirm_backup_and_close`, `list_backups`, `redownload_backup` | backups | TEST-R31 | **Partially traced (extended)** | `client-requirements-validation.md` RQ-19 adds: the downloaded copy must go to a **physically separate medium**, not merely another folder on the same disk. `architecture.md` §15.3 documents this as a stated, unenforced (process-discipline) risk, not a technical gap. |
| — | Empty elapsed month (no entries, recording locked) → **no snapshot**, excluded from yearly-averaging denominator | UN-23 | M5/M6 | Monthly Close (implicit — no record shown for an empty month) | `get_outstanding_periods` | periods, monthly_snapshots (absence) | TEST-EMPTY-MONTH | Fully traced | `requirement-spec.md` marks this ☐ open; `open-questions-checklist.md`'s own copy of the question is never updated. Resolved by `client-requirements-validation.md` **RQ-16** (3 Aug 2026): confirmed no record, excluded from the average. Cite RQ-16, not the stale checklist entry. |

## FR-8 — Exports (Module M6)

| ID | Description | User Need | Design Component | UI Screen | API Command | DB Entity | Test Scenario | Status | Notes |
|---|---|---|---|---|---|---|---|---|---|
| FR-8 / Rule-19 | Every export carries name, ID, phone, volume, Business Volume regardless of optional columns | UN-22 | M6 | Reports | `export_monthly` | members, member_period_totals | TEST-R19 | Fully traced | |
| Rule-33 | Configurable export columns, 4 defaults pre-ticked, full field list offered | UN-22 | M6 | Reports (Monthly data section) | `export_monthly` | settings.default_export_columns | TEST-R33 | Fully traced | Prototype's 12-column optional list is more granular than Appendix B's bare statement — additional detail, not a contradiction. |
| Rule-23 | Yearly average divided by count of periods that **have a snapshot**, not a fixed 12; count displayed alongside | UN-23 | M6 | Reports (Yearly average section) | `export_yearly_average` | monthly_snapshots | TEST-R23 | Fully traced | |
| Rule-24 | Low-threshold report filters on yearly average of **own** Business Volume, not TBV | UN-24 | M6 | Reports (Low-contribution section) | `export_low_contribution` | monthly_snapshots | TEST-R24 | Fully traced | Client's answer differs from the architect's original TBV recommendation (Q-B7) — deliberately re-confirmed. |
| — | "Closed month snapshot" export (always latest version) | UN-20 | M6 | Reports (Closed month snapshot section) | `redownload_backup` (recommended reconciliation — see HIGH-1) | backups, monthly_snapshots | TEST-EXPORT-SNAPSHOT | **Conflicting (resolved as recommendation)** | Prototype shows this as a 4th, distinct export card; `architecture.md` Appendix C names only 3 export commands plus `redownload_backup`/`list_backups`. Recommended default: same feature as `redownload_backup`, presented as an export card in the UI. See [04-api-specification.md](04-api-specification.md) and [11](11-open-questions-and-decisions.md) HIGH-1. |

## FR-9 — Authentication (Module M8)

| ID | Description | User Need | Design Component | UI Screen | API Command | DB Entity | Test Scenario | Status | Notes |
|---|---|---|---|---|---|---|---|---|---|
| FR-9 / Rule-29 | Single admin account; members never log in; PIN or complex password | UN-26 | M8 | Setup wizard, Login | `setup_first_run`, `login` | auth | TEST-R29 | **Conflicting (resolved)** | `requirement-spec.md` frames PIN vs. password as an either/or client decision still pending. `client-requirements-validation.md` M8.5 resolves: **both** may be set simultaneously; either authenticates. Tier-1 wins; `architecture.md`'s ADR-008 already reflects dual-credential support. |
| — | Failed-attempt lockout mandatory regardless of credential choice | UN-26 | M8 | Login (lockout countdown) | `login` | auth.failed_attempts, auth.locked_until | TEST-LOCKOUT | Fully traced | Stated as mandatory in `requirement-spec.md` itself, not merely derived. |
| — | One-time recovery codes, shown once at setup, hashed at rest; sole recovery path (no email/cloud) | UN-26 | M8 | Setup wizard (recovery reveal), Recovery flow | `setup_first_run`, `use_recovery_code` | auth.recovery_codes | TEST-RECOVERY | Fully traced | Explicit design consequence of the offline-only constraint (ADR-008) — loss of credential + recovery codes is "permanently unrecoverable by design," documented, not a bug. |

## Non-Functional Requirements (`client-requirements-validation.md` §11 / `architecture.md` §12)

| ID | Description | Design Component | Status | Notes |
|---|---|---|---|---|
| NFR-1 Performance | Screens <2s, recalculation <2s, extracts <30s | M3 (ADR-005 chain-upward), M6 | Fully traced | Complexity is O(depth × average width), independent of total member count. |
| NFR-2 Scalability | Design ceiling 25,000 members / 200,000 entries per year (client scale is 500–5,000) | M3, DB schema | Fully traced | |
| NFR-3 Availability | ~100% — offline desktop, no server dependency | ADR-001 | Fully traced | |
| NFR-4 Security | Encryption at rest (SQLCipher), session/inactivity lock, no member data in filenames | ADR-003, ADR-008, §11.1/11.3 | Fully traced | See [06-security-authorization-matrix.md](06-security-authorization-matrix.md). |
| NFR-5 Auditability | Minimal recording log of who-changed-what | M9, audit_log | Fully traced | |
| NFR-6 Maintainability | Settings-driven rule engine, no hardcoded business constants | ADR-010 | Fully traced | |
| NFR-7 Compliance | Consent capture at onboarding; permanent, complete retention; no hard-delete; all data persists in exports | M1 (consent fields), Rule-28, Rule-42 | Fully traced | Confirmed 6 Aug 2026 as an explicit client requirement — members are never removed and nothing is ever omitted. No erasure path exists by design; see Rule-42 in [03](03-business-rules.md). |
| NFR-8 Accessibility | "Standard good practice," shadcn/ui baseline, no formal conformance claimed | Presentation layer | Fully traced (as scoped) | Status is never colour-only (labelled pills), per `ui-theme.md`. |
| NFR-9 Localisation | English only, Indian date format, no currency anywhere | Presentation layer | Fully traced | |
| NFR-10 Reporting | Three report types, Excel format | M6 | Fully traced | |
| NFR-11 Logging | Audit log per NFR-5 | M9 | Fully traced | |
| NFR-12 Monitoring | **Explicitly declined by the client** — no mechanism to detect a close silently failing to write its record/backup | — | Fully traced (as a documented non-requirement) | `architecture.md` §11.12: "Deliberately absent; noted here so it is not mistaken for an oversight." |
| NFR-13 Backup & recovery | Internal retained copy is the actual close-gate; external medium is a reminded-but-not-enforced convenience layer. **Extended 7 Aug 2026:** whole-console backup on schedule or on demand, restorable on any machine | M5, ADR-007; M7/M8, ADR-012 (Rule-43) | Fully traced | Single point of failure if the client never takes the external copy — stated plainly (TR-4), not solved. The whole-console mechanism is additive, not a fix for that gap — it protects the console between closes and across machines, not the single-medium risk itself. |
| NFR-14 Hosting/deployment | Standalone offline desktop app, Windows + macOS, no auto-update | ADR-011, §17 | Fully traced | |
| NFR-15 Browser/device support | None — no browser, phone, or tablet use | (N/A by design) | Fully traced | |
| NFR-16 Data migration | None — system starts empty, no import tooling | (out of scope, OC-11) | Fully traced | Closed box by design, not deferred. |

## User Needs Coverage Checklist (UN-01 – UN-31)

All 31 user needs from `user-needs-document.md` trace to at least one Rule/FR/NFR row above (UN-28 added 7 August 2026; UN-29/30/31 added the same day by CR-1/CR-2/CR-3). Full detail is not repeated per-need here to avoid duplicating the table above; this section exists to make the coverage check explicit and auditable.

| UN | Topic | Traced via |
|---|---|---|
| UN-01 | Random, non-sequential member ID | Rule-35 |
| UN-02 | Fast, low-friction onboarding | FR-4, Rule-34 |
| UN-03 | Reference ID / hierarchy attachment | FR-4, Rule-30 |
| UN-04 | Introducer permanence | Rule-37 |
| UN-05 | Advisory, non-blocking structure limits | Rule-1, Rule-28, Rule-32 |
| UN-06 | (structure/identity, general) | FR-1, Rule-2 |
| UN-07 | Frictionless BV recording, <15s | FR-5, Rule-15 |
| UN-08 | Figures reconcile exactly, no rounding drift | Rule-16, Rule-22, zero/negative refusal row |
| UN-09 | Slab/differential correctness | Rule-3, Rule-4, Rule-26 |
| UN-10 – UN-13 | (calculation/recording, general) | Rule-6 – Rule-14 |
| UN-14 | No manual recalculate button | Rule-26, Rule-36 |
| UN-15 | Search by name/ID/phone | FR-1, Rule-44 |
| UN-16 | Chart shows name/ID/own-BV only | FR-2 |
| UN-17 | One screen fully explains a member's Rewards | FR-3 |
| UN-18 | Manual, safely-gated monthly close | FR-7, Rule-17, Rule-18, Rule-20 |
| UN-19 | Backup precedes zeroing; retained permanently | Rule-18, Rule-31, Rule-38 |
| UN-20 | Closed-month record stays available | Rule-38, closed-month export row |
| UN-21 | Corrections possible without breaking history | entries-editable-anytime row |
| UN-22 | Every export carries basic fields | Rule-19, Rule-33 |
| UN-23 | Yearly average, correct denominator | Rule-23, empty-month row |
| UN-24 | Low-contribution report, own-BV metric | Rule-24 |
| UN-25 | Every rate/threshold configurable | FR-6, Rule-4, Rule-27 |
| UN-26 | Single-admin access, credential + lockout | FR-9, Rule-29, lockout row, recovery-codes row |
| UN-27 | Vocabulary constraint enforced everywhere | (see §1.2 in `requirement-spec.md`; enforced across all UI/export rows above by convention, not a separate technical component) |
| UN-28 | Whole console, safe and movable — scheduled/on-demand backup, cross-device restore | Rule-43, NFR-13 (extended) |
| UN-29 | Finding a member by the number they are calling from | Rule-34, Rule-44, FR-1 |
| UN-30 | Recording a purchase reported after the month has turned | Rule-36 (amended) |
| UN-31 | Seeing the whole structure at once, without slowing the console down | Rule-45, FR-10 |

## Traceability Summary

| Metric | Count |
|---|---|
| Total requirement-level items (Rules + FR + new items) | 55 |
| Non-functional requirements | 16 |
| User needs | 31 |
| **Fully traced** | 49 Rules/FR items, all 16 NFRs, all 31 UNs |
| **Partially traced** (superseded wording retained for record) | 3 (Rule-21, Rule-31, Rule-38) |
| **Conflicting (resolved)** (tier-1 document overrides tier-2 wording) | 4 (Rule-28, Rule-29, Rule-35, closed-month-snapshot export) |
| **Missing** | 0 |
| **Needs clarification** | 0 — all cleared 6 Aug 2026 |
| **Untraced prototype behaviour** | 0 — the last one (the >60 gate) was given a rule by CR-3 on 7 Aug 2026 |

Changes on 6 August 2026: **NFR-7** moved to Fully traced (retention/erasure settled as an explicit client requirement, now Rule-42) and so did the **settings mid-period warning** (built, variant C). Two rows were added — **Rule-42** and the **data-recovery screen** — taking the total from 45 to 47. The closed-month-snapshot export conflict is resolved in favour of `redownload_backup`.

Changes on 7 August 2026: **Rule-43** (whole-console backup and cross-device restore) and **UN-28** added, taking the total from 47 to 48 requirement-level items and user needs from 27 to 28. **NFR-13** extended to cover the new mechanism.

Further changes on 7 August 2026 — client change requests **CR-1, CR-2, CR-3** (see [../final/06-decision-log-and-open-items.md](../final/06-decision-log-and-open-items.md) §5):
- **Added:** Rule-44 (phone as a search key), Rule-45 (full hierarchy point-in-time draw), FR-10 (full hierarchy view), UN-29/30/31, V4.5's own row, plus rows for period-isolated recalculation and the conditional month switcher — taking requirement-level items from 48 to 55 and user needs from 28 to 31.
- **Amended:** FR-1 (phone), FR-2's note (the constraint binds FR-10 too), **Rule-36** (the lock narrowed, its row rewritten and TEST-R36's scenario replaced), UN-15's need line.
- **Closed:** the ">60 descendants" row, which had stood as *"Prototype Behavior Not Explicitly Covered By Requirements"* since 6 Aug 2026. CR-3 gave it Rule-45, V4.5 and a home. **There are now no untraced prototype behaviours.**
- **No new API command.** API-06, API-07, API-08 and API-11 were amended; the surface stays at 40.

No orphan rows: every Rule 1–45, every FR-1–10, every UN-01–31, and every NFR has at least one entry above.
