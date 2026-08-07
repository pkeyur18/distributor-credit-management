# Delivery Plan — Epics, Stories, PI & Sprint Breakdown

31 user stories across Epic 0 and modules M1–M9, their dependency graph, and a proposed two-PI, eight-sprint sequence. **Sprint boundaries are a starting proposal — adjust them freely. The dependency constraints are not optional** — a story cannot start before what it depends on is Done (per [05](05-quality-and-acceptance.md) §6.1).

Solo-maintainer project, two-week sprints assumed. Adjust cadence to actual availability; the sequencing logic holds regardless of sprint length.

---

## 1. Epics

One epic per architecture module, plus Sprint 0 for scaffolding.

| Epic | Module | Stories | Depends on |
|---|---|---|---|
| **Epic 0** | Scaffolding | US-0.1, US-0.2 | None — first work in the project |
| **Epic M1** | Member Directory | US-M1.1 – US-M1.4 | Epic 0 |
| **Epic M2** | Business Volume Entry | US-M2.1, US-M2.2 | M1, M3 |
| **Epic M3** | Calculation Engine | US-M3.1, US-M3.2 | M1 (data model) |
| **Epic M4** | Member Detail & Hierarchy Chart | US-M4.1, US-M4.2 | M1, M3 |
| **Epic M5** | Monthly Close | US-M5.1 – US-M5.4 | M2, M3 |
| **Epic M6** | Reports & Exports | US-M6.1 – US-M6.4 | M5 |
| **Epic M7** | Settings | US-M7.1 – US-M7.4 | Low dependency; US-M7.3 needs M3, US-M7.4 needs the `backups` generalization |
| **Epic M8** | Authentication + whole-console backup/restore | US-M8.1 – US-M8.6 | Base auth (M8.1) has no data dependency on M1; M8.5/M8.6 need the `backups` generalization + M7.4 |
| **Epic M9** | Audit Log | US-M9.1 | Cross-cutting — wire into M1/M2/M5/M7 as each is built, not bolted on afterward |

---

## 2. Dependency graph

```
Epic 0 (scaffolding)
  │
  ├──> Epic M1 (members) ──┐         ┌──> Epic M8.1-4 (base auth)
  │                        │         │      (no data dependency on M1 — can run in parallel)
  │                        ▼         │
  │                    Epic M3 (calculation engine)
  │                        │
  │         ┌──────────────┼──────────────┐
  │         ▼              ▼              ▼
  │     Epic M2         Epic M4        Epic M7 (M7.1/M7.2 early;
  │     (entry)      (detail/chart)     M7.3 needs M3; M7.4 needs
  │         │                            `backups` table generalized)
  │         ▼
  │     Epic M5 (monthly close)
  │         │
  │         ▼
  │     Epic M6 (reports)

  `backups` table generalization (ADR-012) ──> M7.4, M8.5, M8.6
                                                (independent of M2–M6, parallel-safe
                                                 once M8.1-4 base auth exists)

Epic M9 (audit) — cross-cutting, wired into M1/M2/M5/M7 as each ships
```

---

## 3. Stories — all 25, with acceptance criteria and dependencies

### Epic 0 — Project Scaffolding

**US-0.1 — Initialize the Tauri v2 project.**
*Dependencies:* None.
*DoD:* Repo builds and runs an empty Tauri shell on Windows and macOS, with React + TypeScript frontend and Rust backend per ADR-002. shadcn/ui, Tailwind, Inter font bundled locally — no CDN, no web fonts.

**US-0.2 — Database & encryption foundation.**
*Dependencies:* US-0.1.
*Acceptance criteria:*
- Given a fresh install with no database file, when the app launches, then a new SQLCipher-encrypted SQLite file is created with all 10 tables and the seed data (7 default slab rows, **16** default settings — see [06](06-decision-log-and-open-items.md) C1) from [04](04-technical-architecture.md) §4.
- Given the database file, when opened with a plain SQLite client, then the contents are unreadable.

### Epic M1 — Member Directory

**US-M1.1 — Add a new member.**
*Requirement refs:* FR-4, Rule-30, Rule-34, Rule-35, Rule-40.
*Acceptance criteria:*
- Given a valid Reference ID resolving to an active member, valid unique phone, and consent ticked, when saved, then a new member is created with a randomly-allocated 6-digit ID in **100001–999999**.
- Given a Reference ID that does not resolve to an active member, when save is attempted, then it is rejected with a clear message.
- Given a phone already used by an **active** member, when save is attempted, then it is rejected.
- Given a phone used by an **inactive** member, when save is attempted, then a reactivation offer is shown instead of an error.
- Given consent is unticked, when save is attempted, then Save is disabled.
*Technical:* ID allocation draws from currently-unallocated numbers only.

**US-M1.2 — Edit an existing member.**
*Requirement refs:* Rule-28, Rule-34, Rule-37.
*Acceptance criteria:*
- Given any member, when the admin edits name/phone/email/address, then the change is saved and audited.
- Given any member, when the Edit modal is viewed, then the introducer/Reference ID field is displayed but **not editable**.

**US-M1.3 — Deactivate and reactivate a member.** ⚠️ *Highest-risk regression in the project.*
*Requirement refs:* Rule-28 (corrected — [06](06-decision-log-and-open-items.md) C5).
*Acceptance criteria:*
- Given an active non-root member with active descendants, when deactivated, then their calculation contribution (own BV feeding ancestor TBV) is **unchanged** — deactivation is display-only.
- Given the root member, when deactivation is attempted, then the action is unavailable.
- Given an inactive member, when reactivated, then their original ID, hierarchy position, and full history are preserved unchanged.
*Testing:* see [05](05-quality-and-acceptance.md) §3.1's dedicated unit test for Rule-28.

**US-M1.4 — Search by name or ID.**
*Requirement refs:* FR-1, UN-15.
*Acceptance criteria:* Given a query matching a name substring or exact 6-digit ID, when submitted, then matching members are listed with name, ID, TBV, slab, status; given no query, then no results are shown — not an error, not "all members."

### Epic M2 — Business Volume Entry

**US-M2.1 — Record a Business Volume entry.**
*Requirement refs:* FR-5, Rule-15, Rule-16, Rule-16a.
*Dependencies:* US-M1.1, US-0.2.
*Acceptance criteria:*
- Given an open period and a selected member, when an amount >0 with up to 2 decimals is entered and saved, then the entry is recorded and the ancestor chain recalculates immediately, visible on screen with no separate recalculate step.
- Given an amount of 0 or negative, when save is attempted, then it is rejected.
- Given the current period is locked, when the entry screen is opened, then no entry form is rendered — a locked state naming the outstanding month is shown instead.

**US-M2.2 — Correct an entry, including in a closed month.**
*Requirement refs:* Rule-39.
*Acceptance criteria:*
- Given an entry in the current open period, when edited, then the ancestor chain recalculates and the change is audited.
- Given an entry in a **closed** month, when edited, then a warning is shown before saving, and on save a new `monthly_snapshots`/`backups` version is created — the original is never overwritten.
*Technical:* `edit_entry` is the sole correction mechanism — no separate reverse/void action.

### Epic M3 — Calculation Engine

**US-M3.1 — Implement TBV/slab/differential/royalty/Rewards computation.**
*Requirement refs:* Rule-3, Rule-6–13, Rule-25.
*Dependencies:* US-0.2.
*Acceptance criteria:* Given the five client worked scenarios as input trees, when the engine runs, then it reproduces totals **35 / 22 / 450 / 1,000 / 980** exactly.

**US-M3.2 — Chain-upward incremental recalculation.**
*Requirement refs:* Rule-26, ADR-005.
*Acceptance criteria:* Given a Business Volume write at any depth, when recalculation runs, then only the ancestor chain from that member to root is recomputed, **and every direct child of every ancestor on that chain is re-scanned** for its differential term — not just the changed leaf — all inside one transaction.
*Testing:* the *O*(depth × width) performance test in [05](05-quality-and-acceptance.md) §3.1.

### Epic M4 — Member Detail & Hierarchy Chart

**US-M4.1 — Member detail view.**
*Requirement refs:* FR-3, UN-17.
*Acceptance criteria:* Given a member, when their detail view opens, then it shows contact info, full Rewards breakdown per direct child (with a note that differential and royalty never both pay on the same leg), direct children (1 depth), TBV, and leg count.

**US-M4.2 — Hierarchy chart.**
*Requirement refs:* FR-2, UN-16.
*Acceptance criteria:* Given a member, when the chart opens, then each node shows exactly name, ID, and **own** Business Volume — never TBV.

### Epic M5 — Monthly Close

**US-M5.1 — Close the oldest outstanding month.**
*Requirement refs:* Rule-17, Rule-18, Rule-20, Rule-38.
*Dependencies:* US-M3.2 (recalculation must be stable before close logic depends on final figures).
*Acceptance criteria:*
- Given an outstanding month, when the close wizard begins, then backup generation must complete and verify successfully before the commit step becomes available.
- Given backup generation fails or is cancelled, when on the backup step, then the close aborts entirely — nothing is zeroed, the alert remains.
- Given backup succeeds and the admin commits, when the close completes, then a permanent snapshot is written for every member, all live figures are zeroed, and the alert clears for that month.
- Given multiple months outstanding, when one closes, then only the next-oldest becomes closable — never a combined period.

**US-M5.2 — Persistent outstanding-month alert.**
*Requirement refs:* Rule-20.
*Acceptance criteria:* Given a month has ended without being closed, when any screen loads, then an undismissable banner and a notification-list entry both appear, and neither clears except by completing the close.

**US-M5.3 — Entry lock enforcement.**
*Requirement refs:* Rule-36.
*Acceptance criteria:* Given an outstanding close, when the entry screen is opened, then no entry of any kind is accepted until the close completes.

**US-M5.4 — Empty-month handling.**
*Requirement refs:* RQ-16.
*Acceptance criteria:* Given a calendar month elapses with zero entries, when it becomes eligible for close, then no snapshot is produced and it is excluded from the yearly-averaging denominator.

### Epic M6 — Reports & Exports

**US-M6.1 — Monthly data export.**
*Requirement refs:* Rule-19, Rule-33.
*Acceptance criteria:* Given any column selection, when exported, then the mandatory columns are always present regardless of selection. *(See [06](06-decision-log-and-open-items.md) O1 for whether that mandatory set is four or five columns.)*

**US-M6.2 — Yearly average export.**
*Requirement refs:* Rule-23.
*Acceptance criteria:* Given N closed periods with snapshots, when exported, then each member's average divides by N — the actual snapshot count, not a fixed 12 — with N displayed alongside.

**US-M6.3 — Low-contribution report.**
*Requirement refs:* Rule-24.
*Acceptance criteria:* Given a configurable threshold (default 100), when exported, then members are filtered by yearly average of **own** Business Volume, not TBV.

**US-M6.4 — Closed-month snapshot re-download.**
*Requirement refs:* Rule-31, Rule-39. Maps to `redownload_backup`, no new command.
*Acceptance criteria:* Given a closed month with multiple corrected versions, when re-downloaded, then the latest version is always returned.

### Epic M7 — Settings

**US-M7.1 — Edit slab table (add/remove/edit rows).**
*Requirement refs:* Rule-4, Rule-27, Rule-41.
*Acceptance criteria:* Given a duplicate threshold, when saved, then rejected. Given a non-monotonic configuration, when saved, then it is **accepted without warning beyond the static disclaimer** — deliberate, Rule-41.

**US-M7.2 — Royalty settings, structure guidance, reporting settings, session timeout.**
*Requirement refs:* remaining §6 settings items ([02](02-business-rules.md) §6).

**US-M7.3 — Mid-period recalculation warning.** *(Approved reference behaviour, built in the prototype — variant C. Port it.)*
*Requirement refs:* RQ-18.
*Acceptance criteria:*
- Given a change to the slab table or royalty settings, when saved, then a warning names the open month, states closed months are unaffected, shows Rewards before → after, and lists affected members — before anything is committed.
- Given cancellation, when the modal closes, then nothing is saved and typed values remain exactly as they were.
- Given a duplicate slab threshold, when saved, then the change is refused outright, no warning offered.
- Given a royalty change (which cannot move any slab), when the modal opens, then it lists members who start/stop earning royalty.
*Dependencies:* API-33 (`preview_settings_impact`); M3 must exist first. **This needs the Rust-side dry-run command — the frontend cannot compute it.**

**US-M7.4 — Whole-console backup schedule and retention setting.** *(Approved reference behaviour, prototyped 7 Aug 2026. Port it.)*
*Requirement refs:* Rule-43.
*Acceptance criteria:* Given the backup schedule card, when off/daily/weekly/monthly is picked, then it saves immediately, no separate Save step. Given a new retention count, when saved, then it takes effect on the next prune, not immediately. Given "Back up now," when clicked, then a manual backup is taken and appears at the top of the Restore list.
*Dependencies:* API-37/38; the `backups` table generalization (ADR-012) must land first.

### Epic M8 — Authentication

**US-M8.1 — First-run setup.**
*Requirement refs:* Rule-29.
*Acceptance criteria:* Given no `auth` row exists, when first launched, then the admin is guided to set a PIN and/or password and is shown one-time recovery codes with a mandatory "I have saved this" gate before proceeding.

**US-M8.2 — Login with lockout.**
*Requirement refs:* Rule-29. Lockout ladder beyond the first threshold: see [06](06-decision-log-and-open-items.md) O4 — define before building.
*Acceptance criteria:* Given 5 consecutive failed attempts, when the 6th is tried, then login is refused with a timed countdown, regardless of which credential type was attempted.

**US-M8.3 — Session lock / inactivity timeout.**
*Requirement refs:* NFR-4.

**US-M8.4 — Credential recovery.**
*Requirement refs:* Rule-29, ADR-008.
*Acceptance criteria:* Given a valid, unused recovery code, when used to set a new credential, then all prior codes are invalidated and a fresh set issued.

**US-M8.5 — Take a whole-console backup, scheduled or on demand.** *(Approved reference behaviour, prototyped 7 Aug 2026. Port it.)*
*Requirement refs:* Rule-43.
*Acceptance criteria:* Given a due schedule at successful login, when login completes, then a scheduled backup is taken silently before the UI takes over. Given "Back up now" in Settings, when clicked, then a manual backup is taken immediately. Either way, retention is enforced afterward — oldest scheduled/manual row first; `period_close`/`pre_restore_safety` rows are never pruned by this.
*Dependencies:* API-39; `backups` table generalization; **no background service exists while the app is closed, so the schedule can only be checked at login — this is a design constraint, not a gap to fix with a background timer.**

**US-M8.6 — Restore the console from a backup file.** *(Approved reference behaviour, prototyped 7 Aug 2026. Port it.)*
*Requirement refs:* Rule-43.
*Acceptance criteria:* Given the ordinary first-run setup screen, when "Restore from a backup file instead" (a plain link) is chosen, then the operator lands on the same recovery screen the db-error path uses, reworded, and choosing a file opens a picker; a successful restore lands on sign-in using that file's own credential. Given an already-running console, when a restore is chosen in Settings, then a checklist-confirm modal must be completed first. Given any restore completes, then a `pre_restore_safety` backup of the prior state was written first, and any authenticated session is dropped, requiring fresh sign-in.
*Dependencies:* API-40; API-35/36 widened to read every `backups.kind`.

### Epic M9 — Audit Log

**US-M9.1 — Record and display audit entries.**
*Requirement refs:* NFR-5.
*Acceptance criteria:* Given any mutating command, when it succeeds, then an `audit_log` row is written with entity, field, before/after, timestamp, cause; given the audit screen filtered by member name, then only that member's entries are shown.

---

## 4. Proposed PI & sprint breakdown

**Two PIs, eight two-week sprints.** PI-1 delivers a working core (structure, calculation, entry, viewing) that can already reproduce the five golden scenarios end to end. PI-2 delivers everything that depends on the core being stable — close, reporting, full settings, console backup/restore, audit — plus hardening and UAT.

### PI-1 — Foundation & Core Calculation (Sprints 1–4)

**Goal:** by the end of PI-1, an admin can set up the console, build a hierarchy, record activity, and see every figure calculate correctly on screen — reproducing all five golden scenarios through the real UI.

| Sprint | Stories | Focus |
|---|---|---|
| **Sprint 1** | US-0.1, US-0.2 | Tauri/React/Rust scaffolding; encrypted DB with full schema and seed data |
| **Sprint 2** | US-M1.1, US-M1.2, US-M1.3, US-M1.4 · US-M8.1, US-M8.2, US-M8.3 | Member directory and base authentication **in parallel** — M8's base auth has no data dependency on M1 |
| **Sprint 3** | US-M3.1, US-M3.2 | The calculation engine, pure and unit-tested against the five scenarios *before* anything else is built on top — this is the highest-value, highest-risk work in the project |
| **Sprint 4** | US-M2.1, US-M2.2 · US-M4.1, US-M4.2 · US-M8.4 | Entry, correction, member detail, hierarchy chart, credential recovery. **Sprint 4 exit gate:** all five golden scenarios reproduce through the real UI |

### PI-2 — Configuration, Close, Reporting & Hardening (Sprints 5–8)

**Goal:** by the end of PI-2, a full month can be recorded, closed safely, corrected, exported, and the whole console backed up and restored — ready for client UAT and handover.

| Sprint | Stories | Focus |
|---|---|---|
| **Sprint 5** | US-M7.1, US-M7.2, US-M7.3 | Slab table, royalty, structure/reporting settings, and the mid-period recalculation warning — needs M3 (Sprint 3) done |
| **Sprint 6** | US-M5.1, US-M5.2, US-M5.3, US-M5.4 | Monthly close — the gated flow, alert, entry lock, empty-month handling. **This module carries the project's highest safety requirement** — the backup gate |
| **Sprint 7** | US-M6.1, US-M6.2, US-M6.3, US-M6.4 · US-M7.4 · US-M9.1 (wired retroactively into M1/M2/M5/M7) | Reports/exports, console backup scheduling, and audit-log completion across everything built so far |
| **Sprint 8** | US-M8.5, US-M8.6 · performance testing at 25,000-member ceiling · UAT prep and execution · vocabulary sweep, security tests | Cross-device restore, hardening, and the client's own reconciliation of the five scenarios — the actual acceptance gate |

### Sizing note

31 stories across 8 sprints averages ~4 stories/sprint, but effort is not uniform — US-M3.1/M3.2 (the calculation engine) and US-M5.1 (the gated close transaction) are disproportionately more careful work than, say, US-M7.2 (a settings form). Sprint 3 and Sprint 6 should be treated as the two sprints where schedule risk actually lives; the others have slack by comparison.

### What must not move

- **Sprint 3 cannot start before Sprint 1/2 are Done.** The calculation engine needs the schema and a member hierarchy to test against.
- **Sprint 4's exit gate (all five scenarios reproducing through the real UI) is a genuine go/no-go**, not a nice-to-have — it is the client's own stated trust bar (R-9, SC-2).
- **US-M7.4 and US-M8.5/US-M8.6 cannot start before the `backups` table generalization (ADR-012) lands.** That generalization can happen any time from Sprint 1 onward (it's a schema decision, not a feature), but the three stories that build on it cannot ship before it exists.
- **US-M9.1 is cross-cutting.** Wire the audit-log call into every mutating command *as it is built*, not as a bolt-on in Sprint 7 — Sprint 7's US-M9.1 line is a completeness check, not the first time audit logging is written.

---

## 5. Sequencing rationale, for reference

Restated from `09-implementation-backlog.md`'s own analysis, unchanged:

1. Epic 0 blocks everything.
2. Epic M1 and Epic M8 (base auth) in parallel — M8 has no data dependency on M1.
3. Epic M3 depends on M1's data model; blocks M2/M4/M5/M6.
4. Epic M2 depends on M1 + M3.
5. Epic M4 depends on M1 + M3.
6. Epic M7 can start early (low dependency), but M7.3 depends on M3 existing (to know what triggers recalculation), and M7.4 depends only on the `backups` table generalization, not on M3.
7. Epic M5 depends on M2 + M3.
8. Epic M6 depends on M5 (needs snapshots to exist).
9. US-M8.5/US-M8.6 depend on the `backups` table generalization and US-M7.4's schedule/retention setting; otherwise independent of M2–M6, so can proceed in parallel once M8's base auth exists.
10. Epic M9 is cross-cutting — wire into M1/M2/M5/M7 as they're built, not bolted on afterward.
