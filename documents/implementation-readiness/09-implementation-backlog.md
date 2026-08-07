# Implementation Backlog

Structured as Epics (one per architecture module, plus a Sprint-0 epic) → Features → User Stories. Story IDs are `US-<module>.<n>`. Every story references the Rule-##/UN-## IDs it implements, so this backlog stays traceable back to [02-requirements-traceability-matrix.md](02-requirements-traceability-matrix.md) without restating the full rule text (see [03-business-rules.md](03-business-rules.md) for that).

---

## Epic 0 — Project Scaffolding (Sprint 0, no module owner)

No `package.json`, `Cargo.toml`, `tsconfig.json`, or `src-tauri/` tree exists in the repository yet — this is genuinely greenfield.

**Feature 0.1 — Stack scaffolding**
- **US-0.1** Initialize Tauri v2 project with React + TypeScript frontend, Rust backend, per ADR-002.
  - *Dependencies:* None — first story in the project.
  - *Technical considerations:* Match the exact stack named in `architecture.md`/`ui-theme.md` (shadcn/ui, Tailwind, Inter font bundled locally — no CDN/web fonts, per the offline constraint).
  - *DoD:* Repo builds and runs an empty Tauri shell on both target platforms (Windows, macOS).

**Feature 0.2 — Database & encryption foundation**
- **US-0.2** Wire up `rusqlite` with SQLCipher, implement the DDL from [05-data-model-specification.md](05-data-model-specification.md) (all 10 entities).
  - *Dependencies:* US-0.1.
  - *Acceptance criteria:*
    - Given a fresh install with no database file, When the app launches, Then a new encrypted SQLite file is created with all 10 tables and the seed data (7 default slab rows, 13 default settings) from Appendix B.
    - Given the database file, When opened with a plain (non-SQLCipher-aware) SQLite client, Then the contents are unreadable.
  - *Related requirements:* ADR-003, all Rule-## via the entities they touch.

---

## Epic M1 — Member Directory

**Feature M1.1 — Onboard and maintain members**
- **US-M1.1** Add a new member.
  - *Requirement refs:* FR-4, Rule-30, Rule-34, Rule-35, Rule-40, UN-02, UN-03.
  - *Acceptance criteria:*
    - Given a valid Reference ID resolving to an active member, valid unique phone, and consent ticked, When the admin saves, Then a new member is created with a randomly-allocated 6-digit ID in 100001–999999.
    - Given a Reference ID that does not resolve to an active member, When the admin attempts to save, Then the save is rejected with a clear message.
    - Given a phone number already used by an **active** member, When the admin attempts to save, Then the save is rejected.
    - Given a phone number already used by an **inactive** member, When the admin attempts to save, Then a reactivation offer is shown instead of an error.
    - Given the consent checkbox is unticked, When the admin attempts to save, Then Save is disabled.
  - *Technical considerations:* ID allocation must draw from currently-unallocated numbers only (deactivated members' IDs stay taken forever).
  - *Testing considerations:* see [08-testing-strategy.md](08-testing-strategy.md) unit tests for Rule-34/35/40.
- **US-M1.2** Edit an existing member.
  - *Requirement refs:* Rule-28, Rule-34, Rule-37.
  - *Acceptance criteria:*
    - Given any member, When the admin edits name/phone/email/address, Then the change is saved and audited.
    - Given any member, When the admin views the Edit modal, Then the introducer/Reference ID field is displayed but **not editable**.
- **US-M1.3** Deactivate and reactivate a member.
  - *Requirement refs:* Rule-28 (corrected).
  - *Acceptance criteria:*
    - Given an active non-root member with active descendants, When the admin deactivates them, Then their calculation contribution (own BV feeding ancestor TBV) is **unchanged** — deactivation is display-only.
    - Given the root member, When the admin attempts to deactivate, Then the action is unavailable.
    - Given an inactive member, When reactivated, Then their original ID, hierarchy position, and full history are preserved unchanged.
  - *This story carries the project's single highest-risk regression* — see the dedicated unit test called out in [08-testing-strategy.md](08-testing-strategy.md).

**Feature M1.2 — Search**
- **US-M1.4** Search by name or ID.
  - *Requirement refs:* FR-1, UN-15.
  - *Acceptance criteria:* Given a query matching a name substring or an exact 6-digit ID, When submitted, Then matching members are listed with name, ID, TBV, slab, status; given no query, Then no results are shown (not an error, not "all members").

---

## Epic M2 — Business Volume Entry

**Feature M2.1 — Record and correct entries**
- **US-M2.1** Record a Business Volume entry.
  - *Requirement refs:* FR-5, Rule-15, Rule-16, Rule-16a, UN-07, UN-08.
  - *Acceptance criteria:*
    - Given an open period and a selected member, When the admin enters an amount >0 with up to 2 decimals and saves, Then the entry is recorded and the member's ancestor chain recalculates immediately, visible on screen with no separate recalculate step.
    - Given an amount of 0 or a negative number, When the admin attempts to save, Then the save is rejected.
    - Given the current period is locked (outstanding reset), When the admin opens this screen, Then no entry form is rendered — a locked state naming the outstanding month is shown instead.
  - *Dependencies:* US-M1.1 (member must exist), US-0.2.
- **US-M2.2** Correct an entry, including in a closed month.
  - *Requirement refs:* Rule-39 (extends Rule-38), UN-21.
  - *Acceptance criteria:*
    - Given an entry in the current open period, When edited, Then the ancestor chain recalculates and the change is audited.
    - Given an entry in a **closed** month, When edited, Then a warning is shown before saving ("recalculates the affected chain and writes a new snapshot version — the original record is never overwritten"), and on save a new `monthly_snapshots`/`backups` version is created, leaving the original version untouched.
  - *Technical considerations:* This is the sole correction mechanism — no separate "reverse/void" action exists (see [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md), reverse_entry technical decision).

---

## Epic M3 — Calculation Engine

No dedicated UI or IPC surface (see [04-api-specification.md](04-api-specification.md)) — this epic is pure backend logic, triggered as a side-effect of M2/M7 writes.

**Feature M3.1 — Core calculation**
- **US-M3.1** Implement TBV/slab/differential/royalty/Rewards computation per Rules 3, 6–13, 25.
  - *Acceptance criteria:* Given the five client worked scenarios as input trees, When the calculation engine runs, Then it reproduces totals 35 / 22 / 450 / 1,000 / 980 exactly.
  - *Dependencies:* US-0.2 (data model in place).
- **US-M3.2** Chain-upward incremental recalculation (ADR-005).
  - *Requirement refs:* Rule-26.
  - *Acceptance criteria:* Given a Business Volume write at any depth, When recalculation runs, Then only the ancestor chain from that member to the root is recomputed (not the full tree), and every direct child of every ancestor on that chain is re-scanned for its differential term (not just the changed leaf), all inside one transaction.
  - *Testing considerations:* the O(depth × width) performance test in [08](08-testing-strategy.md).

---

## Epic M4 — Member Detail & Hierarchy Chart

**Feature M4.1 — Views**
- **US-M4.1** Member detail view.
  - *Requirement refs:* FR-3, UN-17.
  - *Acceptance criteria:* Given a member, When their detail view opens, Then it shows contact info, full Rewards breakdown per direct child (with a "differential and royalty never pay on the same leg" note), direct children (1 depth), TBV, and leg count.
- **US-M4.2** Hierarchy chart.
  - *Requirement refs:* FR-2, UN-16.
  - *Acceptance criteria:* Given a member, When the chart is opened, Then each node shows exactly name, ID, and **own** Business Volume — never TBV.

---

## Epic M5 — Monthly Close

**Feature M5.1 — Gated close flow**
- **US-M5.1** Close the oldest outstanding month.
  - *Requirement refs:* Rule-17, Rule-18, Rule-20, Rule-38, UN-18, UN-19.
  - *Acceptance criteria:*
    - Given an outstanding month, When the admin begins the close wizard, Then backup generation must complete and verify successfully before the commit step becomes available.
    - Given backup generation fails or is cancelled, When the admin is on the backup step, Then the close aborts entirely, nothing is zeroed, and the outstanding alert remains.
    - Given backup succeeds and the admin commits, When the close completes, Then a permanent snapshot is written for every member, all live figures (BV, TBV, Rewards, royalty) are zeroed, and the alert clears for that month.
    - Given multiple months are outstanding, When one closes, Then only the next-oldest becomes closable — never a combined/merged period.
  - *Dependencies:* US-M3.2 (recalculation must be stable before close logic depends on final figures).
- **US-M5.2** Persistent outstanding-month alert.
  - *Requirement refs:* Rule-20.
  - *Acceptance criteria:* Given a month has ended without being closed, When any screen loads, Then an undismissable banner and a notification-list entry both appear, and neither clears except by completing the close.
- **US-M5.3** Entry lock enforcement.
  - *Requirement refs:* Rule-36.
  - *Acceptance criteria:* Given an outstanding close, When the admin opens Business Volume Entry, Then no entry of any kind is accepted until the close completes.
- **US-M5.4** Empty-month handling.
  - *Requirement refs:* RQ-16.
  - *Acceptance criteria:* Given a calendar month elapses with zero entries recorded, When it becomes eligible for close, Then no snapshot is produced for it and it is excluded from the yearly-averaging denominator.

---

## Epic M6 — Reports & Exports

**Feature M6.1 — Exports**
- **US-M6.1** Monthly data export.
  - *Requirement refs:* Rule-19, Rule-33, UN-22.
  - *Acceptance criteria:* Given any column selection, When exported, Then name/ID/phone/Business Volume are always present regardless of selection.
- **US-M6.2** Yearly average export.
  - *Requirement refs:* Rule-23, UN-23.
  - *Acceptance criteria:* Given N closed periods with snapshots, When exported, Then each member's average is divided by N (the actual snapshot count, not a fixed 12), and N is displayed alongside the figure.
- **US-M6.3** Low-contribution report.
  - *Requirement refs:* Rule-24, UN-24.
  - *Acceptance criteria:* Given a configurable threshold (default 100), When exported, Then members are filtered by yearly average of **own** Business Volume, not TBV.
- **US-M6.4** Closed-month snapshot re-download.
  - *Requirement refs:* Rule-31, Rule-39 (maps to `redownload_backup` — see HIGH-1 in [11](11-open-questions-and-decisions.md)).
  - *Acceptance criteria:* Given a closed month with multiple corrected versions, When re-downloaded, Then the latest version is always returned.

---

## Epic M7 — Settings

**Feature M7.1 — Configuration**
- **US-M7.1** Edit slab table (add/remove/edit rows).
  - *Requirement refs:* Rule-4, Rule-27, Rule-41.
  - *Acceptance criteria:* Given a duplicate threshold, When saved, Then rejected. Given a non-monotonic percentage-vs-threshold configuration, When saved, Then it is **accepted without warning beyond the static on-screen disclaimer** (deliberate, Rule-41).
- **US-M7.2** Edit royalty settings, structure guidance, reporting settings, session timeout.
  - *Requirement refs:* remaining §7 items.
- **US-M7.3** Mid-period recalculation warning. **✅ Designed and built in the prototype, 6 Aug 2026 (variant C).**
  - *Requirement refs:* RQ-18/V7.6.
  - *Acceptance criteria:*
    - Given a change to the slab table or royalty settings, When the admin saves, Then a warning names the open month, states that closed months are unaffected, shows Rewards before → after, and lists the affected members — before anything is committed.
    - Given the admin cancels, When the modal closes, Then nothing is saved and their typed values remain exactly as they were.
    - Given a duplicate slab threshold, When the admin saves, Then the change is refused outright and no warning is offered.
    - Given a royalty change (which cannot move any slab), When the modal opens, Then it lists members who start or stop earning royalty, with a "Members earning royalty: before → after" row.
  - *Technical note:* the preview reuses the live Total Business Volume — slab and royalty settings never feed `rollupTBV` — and re-runs `computeRewards` alone against a temporarily-swapped `SETTINGS`, restored in a `finally` block.
  - *Remaining for the real build:* port this behaviour to the Rust/React implementation of M7; the prototype now carries the approved reference behaviour.

---

## Epic M8 — Authentication

**Feature M8.1 — Setup, login, lockout, recovery**
- **US-M8.1** First-run setup.
  - *Requirement refs:* Rule-29, UN-26.
  - *Acceptance criteria:* Given no `auth` row exists, When first launched, Then the admin is guided to set a PIN and/or password and is shown one-time recovery codes with a mandatory "I have saved this" gate before proceeding.
- **US-M8.2** Login with lockout.
  - *Requirement refs:* Rule-29, mandatory-lockout note.
  - *Acceptance criteria:* Given 5 consecutive failed attempts, When the 6th is tried, Then login is refused with a timed countdown, regardless of which credential type was attempted.
- **US-M8.3** Session lock / inactivity timeout.
  - *Requirement refs:* NFR-4.
- **US-M8.4** Credential recovery.
  - *Requirement refs:* Rule-29, ADR-008.
  - *Acceptance criteria:* Given a valid, unused recovery code, When used to set a new credential, Then all prior recovery codes are invalidated and a fresh set is issued.

---

## Epic M9 — Audit Log

**Feature M9.1 — Change history**
- **US-M9.1** Record and display audit entries.
  - *Requirement refs:* NFR-5.
  - *Acceptance criteria:* Given any mutating command (member edit, entry edit, settings change, period close), When it succeeds, Then an audit_log row is written with entity, field, before/after, timestamp, and cause; given the audit screen, When filtered by member name, Then only that member's entries are shown.

---

## Backlog items arising directly from this analysis — all resolved 6 Aug 2026

Every item raised by the readiness analysis has since been decided; the three UI ones are built in the approved prototype and now need porting to the real implementation like any other approved behaviour.

- **US-BACKLOG-1** — ✅ Resolved. "Closed month snapshot" downloads a closed month as `.xlsx` and maps to `redownload_backup`; no new command. No longer gates M6.
- **US-BACKLOG-2** — ✅ Closed, not applicable. The client requires that members are never removed and that all data persists throughout, including in exports. No erasure mechanism is to be built. No longer gates M1.
- **US-M7.3** — ✅ Built in the prototype (variant C). Port to M7.
- **US-BACKLOG-3** — ✅ Built. The last slab row cannot be removed: control disabled with an explanatory `aria-label` and hint, handler refuses with a named message. Port to M7.
- **US-BACKLOG-4** — ✅ Built. Data-recovery screen at launch (design D), listing retained backups by the month they hold. Port to M5/M8.

## Suggested sequencing

1. Epic 0 (scaffolding) — blocks everything.
2. Epic M1 (members) and Epic M8 (auth) in parallel — M8 has no data dependency on M1.
3. Epic M3 (calculation engine) — depends on M1's data model, blocks M2/M4/M5/M6.
4. Epic M2 (entry) — depends on M1 + M3.
5. Epic M4 (detail/chart) — depends on M1 + M3.
6. Epic M7 (settings) — can start early (low dependency), but US-M7.3 depends on M3 existing to know what triggers recalculation.
7. Epic M5 (monthly close) — depends on M2 + M3.
8. Epic M6 (exports) — depends on M5 (needs snapshots to exist).
9. Epic M9 (audit) — cross-cutting, should be wired into M1/M2/M5/M7 as they're built, not bolted on afterward.
