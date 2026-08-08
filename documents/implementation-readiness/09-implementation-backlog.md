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
    - Given a fresh install with no database file, When the app launches, Then a new encrypted SQLite file is created with all 10 tables and the seed data (7 default slab rows, 16 default settings) from Appendix B.
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
- **US-M1.4** Search by name, ID or phone. *(amended 7 Aug 2026, CR-1)*
  - *Requirement refs:* FR-1, UN-15, UN-29, Rule-44.
  - *Acceptance criteria:*
    - Given a query matching a name substring or a 6-digit ID, When submitted, Then matching members are listed with name, ID, **phone**, TBV, slab, status; given no query, Then no results are shown (not an error, not "all members").
    - Given a query of 4 or more digits matching a member's phone number — with or without spaces, dashes or a country prefix — When submitted, Then that member is listed.
    - Given a query of fewer than 4 digits, When submitted, Then no member is matched **on phone**; name and ID matching are unaffected.
  - *Technical considerations:* one shared search function backs every search box in the console (Home, Structure, BV Entry, Correction panel, Add-Member reference lookup, which keeps its active-only filter per Rule-30). Behaviour must not differ between screens. Both sides are reduced to a canonical key before the phone comparison — digits, then an international prefix or trunk zero dropped, so the match works in both directions. The stored value is never rewritten. No schema change — `members.phone` is already unique and indexed.

---

## Epic M2 — Business Volume Entry

**Feature M2.1 — Record and correct entries**
- **US-M2.1** Record a Business Volume entry.
  - *Requirement refs:* FR-5, Rule-15, Rule-16, Rule-16a, UN-07, UN-08.
  - *Acceptance criteria:*
    - Given an open period and a selected member, When the admin enters an amount >0 with up to 2 decimals and saves, Then the entry is recorded and the member's ancestor chain recalculates immediately, visible on screen with no separate recalculate step.
    - Given an amount of 0 or a negative number, When the admin attempts to save, Then the save is rejected.
    - Given a month is outstanding, When the admin opens this screen, Then the entry form **is** rendered, headed by the name of the month it is recording into. *(Amended 7 Aug 2026, CR-2 — this criterion previously required a locked state with no form at all.)*
  - *Dependencies:* US-M1.1 (member must exist), US-0.2.
- **US-M2.2** Correct an entry, including in a closed month.
  - *Requirement refs:* Rule-39 (extends Rule-38), UN-21.
  - *Acceptance criteria:*
    - Given an entry in the current open period, When edited, Then the ancestor chain recalculates and the change is audited.
    - Given an entry in a **closed** month, When edited, Then a warning is shown before saving ("recalculates the affected chain and writes a new snapshot version — the original record is never overwritten"), and on save a new `monthly_snapshots`/`backups` version is created, leaving the original version untouched.
  - *Technical considerations:* This is the sole correction mechanism — no separate "reverse/void" action exists (see [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md), reverse_entry technical decision).

**Feature M2.2 — Entry eligibility by period** *(new 7 Aug 2026, CR-2)*
- **US-M2.3** Record into a month that has ended but is not closed.
  - *Requirement refs:* Rule-36 (amended), M2.3, M2.6, UN-30, AC-42, V2.6.
  - *Acceptance criteria:*
    - Given June has ended and is not closed, and today is in August, When a figure dated in June is saved, Then it is recorded into June and June's ancestor chain recalculates immediately.
    - Given the same state, When the entry screen is opened, Then it names June as the month being recorded into, and the date field is bounded to June's first and last day.
    - Given the same state, When the figure is saved, Then **no other period's figures change**.
    - Given no month is outstanding, When the entry screen is opened, Then it names the current month and the date field is bounded to the month start and today.
  - *Dependencies:* US-M2.1, US-M5.3 (the outstanding-period state must exist).
  - *Technical considerations:* the target period is derived from `entry_date`, never from "the period being closed". `member_period_totals` may hold rows for more than one not-yet-closed period; the composite PK already carries this, so there is no schema change. Recalculation must be confined to the entry's own period.
- **US-M2.4** Refuse a current-month entry while an earlier month is outstanding.
  - *Requirement refs:* Rule-36 (amended), M2.7, V2.3, V2.7, AC-43.
  - *Acceptance criteria:*
    - Given June is outstanding and today is in August, When a figure dated in August is saved, Then it is refused and the refusal **names June**.
    - Given the refusal, When the screen is inspected, Then the form is still available and only that date is rejected.
    - Given June is then closed, When the same August-dated figure is saved, Then it is accepted.
    - Given a figure dated in an already-closed month, When entry is attempted here, Then it is not offered — the correction panel is pointed to (Rule-39).
  - *Dependencies:* US-M2.3.
  - *Technical considerations:* typed errors `PeriodNotAcceptingEntries { month, blocking_month }` and `PeriodClosed { month }`. The retired `PeriodLocked` variant must not be reintroduced under a new meaning (see [04-api-specification.md](04-api-specification.md)).
- **US-M2.5** Month selector for multiple outstanding months.
  - *Requirement refs:* Rule-36 (amended), Rule-20.
  - *Acceptance criteria:*
    - Given exactly one month is available to record into, When the entry screen is opened, Then **no month selector is rendered at all**.
    - Given two or more months are outstanding, When the entry screen is opened, Then a selector listing them is rendered, defaulting to the **oldest**, and changing it re-bounds the date field.
    - Given two or more months hold live figures, When Home, Member Detail or Structure is opened, Then the figures shown are the **oldest** outstanding month's, with a switcher available; with a single month in play, no switcher appears anywhere.
  - *Dependencies:* US-M2.3.
  - *Note:* the client has stated this situation will not arise in practice. It is built so the system is correct if it ever does, not because it is expected.

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

**Feature M4.2 — Full hierarchy window** *(new 7 Aug 2026, CR-3)*
- **US-M4.3** Full hierarchy view in a separate window.
  - *Requirement refs:* FR-10, UN-31, Rule-45, M4.7, V4.5, AC-44, AC-45.
  - *Acceptance criteria:*
    - Given any position in the Structure screen, When "View full hierarchy" is activated, Then the view opened is rooted at the **top member** — never at the currently-viewed member.
    - Given a structure of more than 60 descendants, When the action is activated, Then a confirmation naming the **exact** member count is shown first; **Cancel opens nothing at all**; Open draws the window.
    - Given a structure of 60 or fewer descendants, When the action is activated, Then it opens immediately with no confirmation.
    - Given the window is open, When inspected, Then it is a **separate window**, every branch is expanded, each node shows exactly name, ID and own Business Volume, and the header carries the top member's name, the member count and an "as at" date and time.
    - Given the window is open, When a figure is recorded in the main console, Then the window does **not** change and its timestamp still names when it was drawn.
    - Given the window is open or drawing, When the main console is used, Then it stays responsive.
    - Given the window is open, When the toolbar is used, Then zoom (10% out to 150% in), fit-width, in-window search with highlight-and-scroll, and Print all work.
    - Given the top member has nobody beneath them, When the window opens, Then it shows the single root node and states plainly there is nothing beneath it — not an error.
  - *Dependencies:* US-M4.2 (the node component), US-M3.1 (figures to show).
  - *Technical considerations:* `get_direct_children_chart` with `full_tree: true` — **no new command**. Node positions come from a single post-order layout pass emitting connectors as one pre-computed path, never measured back out of the rendered DOM as the main Structure screen does. The window subscribes to nothing and holds no handle on live console state. Read-only means no node links, no hover-lift affordance, no writes.
  - *Known accepted limit:* TR-7 — the top-down layout's width grows with leaf count; at 25,000 members the canvas is extremely wide and a print spans many pages. Chosen deliberately by the client over a width-stable indented outline. **Do not switch layouts unilaterally** — raise it as a change request.

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
- **US-M5.3** Entry eligibility by period. *(amended 7 Aug 2026, CR-2 — was "Entry lock enforcement")*
  - *Requirement refs:* Rule-36 (amended), M5.2.
  - *Acceptance criteria:*
    - Given an outstanding close, When the admin opens Business Volume Entry, Then entries dated in the outstanding month **are** accepted, and entries dated in the current month are refused naming that outstanding month.
    - Given the close completes, When a current-month entry is attempted again, Then it is accepted.
  - *Note:* the enforcement itself is US-M2.3/US-M2.4 in Epic M2. This story is M5's side of the contract — publishing which periods are recordable via `get_period_lock_status`, and releasing the current month on close.
  - *Superseded criterion:* "no entry of any kind is accepted until the close completes."
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
  - *Remaining for the real build:* port this behaviour to the Rust/React implementation of M7. **This depends on a new IPC command** — `preview_settings_impact` (API-33) — because the calculation engine is Rust-side and the frontend cannot dry-run it. The prototype hid this: everything there runs in one JavaScript scope. Build the command before the UI.
  - *Dependencies:* API-33; M3 (engine) must exist first.
- **US-M7.4** Whole-console backup schedule and retention setting. **✅ Designed and prototyped 7 Aug 2026.**
  - *Requirement refs:* Rule-43, RQ-23.
  - *Acceptance criteria:* Given the Backup schedule card, When the admin picks off/daily/weekly/monthly, Then the setting is saved immediately (no separate Save step, matching the segmented-control pattern elsewhere). Given a new retention count, When saved, Then it takes effect on the next prune (existing excess backups beyond the new count are pruned then, not immediately). Given "Back up now", When clicked, Then a `manual` backup is taken and appears at the top of the Restore card's list.
  - *Dependencies:* API-37/38; the `backups` table generalization (§8 architecture.md, ADR-012) must land first.

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

**Feature M8.2 — Whole-console backup & cross-device restore** *(new 7 Aug 2026, Rule-43/RQ-23, ADR-012)*
- **US-M8.5** Take a whole-console backup, scheduled or on demand. **✅ Designed and prototyped 7 Aug 2026.**
  - *Requirement refs:* Rule-43, M8.6.
  - *Acceptance criteria:* Given a due schedule (per `settings.console_backup_schedule`) at successful login, When login completes, Then a `scheduled` backup is taken silently before the UI takes over. Given "Back up now" in Settings, When clicked, Then a `manual` backup is taken immediately. Either way, retention (`settings.console_backup_retention_count`) is enforced afterward, oldest `scheduled`/`manual` row first — `period_close` and `pre_restore_safety` rows are never pruned by this.
  - *Dependencies:* API-39; the `backups` table generalization must land first; no background service exists while the app is closed, so the schedule can only be checked at login — this is a design constraint, not a gap to "fix" with a background timer.
- **US-M8.6** Restore the console from a backup file — running console or brand-new install alike. **✅ Designed and prototyped 7 Aug 2026.**
  - *Requirement refs:* Rule-43, M8.7.
  - *Acceptance criteria:* Given the ordinary first-run setup screen (shown unconditionally, no separate welcome/choice screen), When "Restore from a backup file instead" (a plain link, not a competing button) is chosen, Then the operator lands on the same recovery screen the db-error path uses — reworded for this reason ("Restore from a backup file" heading, no internal restore-points list since a brand-new machine has none of its own yet) — and choosing a file there opens a picker; a successful restore lands on the sign-in screen using that file's own credential. Given an already-running, authenticated console, When "Restore from a file…" or a listed backup is chosen in Settings, Then a checklist-confirm modal (checkbox + Restore button, same pattern as the month-close wizard) must be completed before anything happens. Given any restore completes, When it finishes, Then a `pre_restore_safety` backup of the prior live state was written first, and any authenticated session is dropped, requiring fresh sign-in.
  - *Dependencies:* API-40; API-35/36 widened to read every `backups.kind`, not only `period_close`.

---

## Epic M9 — Audit Log

**Feature M9.1 — Change history**
- **US-M9.1** Record and display audit entries.
  - *Requirement refs:* NFR-5.
  - *Acceptance criteria:* Given any mutating command (member edit, entry edit, settings change, period close), When it succeeds, Then an audit_log row is written with entity, field, before/after, timestamp, and cause; given the audit screen, When filtered by member name, Then only that member's entries are shown.

---

## Backlog items arising directly from this analysis — all resolved 6–7 Aug 2026

Every item raised by the readiness analysis has since been decided; the UI ones are built in the approved prototype and now need porting to the real implementation like any other approved behaviour.

- **US-BACKLOG-1** — ✅ Resolved. "Closed month snapshot" downloads a closed month as `.xlsx` and maps to `redownload_backup`; no new command. No longer gates M6.
- **US-BACKLOG-2** — ✅ Closed, not applicable. The client requires that members are never removed and that all data persists throughout, including in exports. No erasure mechanism is to be built. No longer gates M1.
- **US-M7.3** — ✅ Built in the prototype (variant C). Port to M7.
- **US-BACKLOG-3** — ✅ Built. The last slab row cannot be removed: control disabled with an explanatory `aria-label` and hint, handler refuses with a named message. Port to M7.
- **US-BACKLOG-4** — ✅ Built. Data-recovery screen at launch (design D), listing retained backups by the month they hold. Port to M5/M8. **Needs three new pre-flight commands** (API-34–36) which are, unavoidably, the only unauthenticated commands besides the auth trio — the database cannot be opened, so nothing can be authenticated against. `restore_from_backup` must verify the stored checksum before overwriting.
- **US-BACKLOG-5** — ✅ Built 7 Aug 2026. New client requirement (RQ-23), not raised by the original readiness analysis: whole-console backup on a schedule or on demand, restorable on any machine including a brand-new install. See US-M7.4/US-M8.5/US-M8.6 above. **Needs four new commands** (API-37–40) and generalizes the `backups` table (`kind`/`schedule_kind`, nullable `period_id`) rather than adding a second table.
- **US-BACKLOG-6** — ✅ Specified 7 Aug 2026, **not yet built in the prototype at the time of writing**. Three client change requests raised after this document set was approved: **CR-1** phone as a search key, **CR-2** entry into an ended-but-unclosed month, **CR-3** the full hierarchy window. They land as US-M1.4 (amended), US-M2.3/M2.4/M2.5 (new), US-M4.3 (new) and US-M5.3 (amended). **No new IPC command** — API-06/07/08/11 are amended and the surface stays at 40. Full reasoning: [../final/06-decision-log-and-open-items.md](../final/06-decision-log-and-open-items.md) §5.
- **US-BACKLOG-7** — ✅ Closed by CR-3. The hierarchy chart's ">60 descendants" confirm-before-render gate had stood since 6 Aug 2026 as prototype behaviour with no source rule and no traceability row (LOW-1). It now belongs to the full hierarchy window, backed by Rule-45 and V4.5. **There are no remaining untraced prototype behaviours.**

## Suggested sequencing

1. Epic 0 (scaffolding) — blocks everything.
2. Epic M1 (members) and Epic M8 (auth) in parallel — M8 has no data dependency on M1.
3. Epic M3 (calculation engine) — depends on M1's data model, blocks M2/M4/M5/M6.
4. Epic M2 (entry) — depends on M1 + M3. **Feature M2.2 (US-M2.3/M2.4/M2.5, entry eligibility by period) additionally depends on M5's outstanding-period state**, so it lands with Epic M5 rather than with the rest of M2.
5. Epic M4 (detail/chart) — depends on M1 + M3. US-M4.3 (full hierarchy window) depends on US-M4.2's node component.
6. Epic M7 (settings) — can start early (low dependency), but US-M7.3 depends on M3 existing to know what triggers recalculation; US-M7.4 depends only on the `backups` table generalization (ADR-012), not on M3.
7. Epic M5 (monthly close) — depends on M2 + M3.
8. Epic M6 (exports) — depends on M5 (needs snapshots to exist).
9. US-M8.5/US-M8.6 (whole-console backup/restore) — depend on the `backups` table generalization and US-M7.4's schedule/retention setting; otherwise independent of M2–M6, so can proceed in parallel with them once M8's base auth (Feature M8.1) exists.
9. Epic M9 (audit) — cross-cutting, should be wired into M1/M2/M5/M7 as they're built, not bolted on afterward.
