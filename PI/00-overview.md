# PI Plan — Overview

## Distributor Business Volume & Beneficiary Management System

| | |
|---|---|
| **Client** | Siddharth Patel |
| **Solution Architect / Developer** | Keyur Patel |
| **Status** | Planning reference for the build |
| **Compiled** | 8 August 2026 |
| **Capacity basis** | Solo, full-time. Two-week sprints assumed |
| **Scheduling basis** | **Relative sprint numbering only — no calendar dates anywhere in this set** |
| **Repository state at compile time** | Documentation-only. Zero application code |

---

## 1. What this folder is

`documents/refinement/` **defines** the system — 47 business rules, 40 IPC contracts, 10 entities, 10 functional requirements, 47 acceptance criteria, six golden regression totals.

`PI/` **plans the build of it** — every requirement decomposed into Epic → Feature → User Story → **Task**, sequenced across sprints, covering development, testing, release and handover end to end.

The division is strict and load-bearing:

> **No business rule, validation rule, acceptance criterion or API contract is restated in this folder.** Where a task needs one, it is cited by ID. A rule copied into two places is a rule that will drift, and this project has already paid that cost once — nineteen source documents in which later ones silently corrected earlier ones (`00-master-index.md` §1).

If a statement in `PI/` disagrees with `documents/refinement/`, **the refinement set wins**, with one exception: the **fourteen** decisions recorded in [05-decisions-and-gaps.md](05-decisions-and-gaps.md), which were taken on 8 August 2026 and are newer than every document in that set. Those fourteen carry a propagation task list; until it is executed, `PI/05` is the authority for those fourteen items alone.

⚠️ **Three of them (D-9, D-11, D-12/D-13) correct defects in the specification itself** — two documents contradicting each other on how every slab resolves, a state machine with no trigger, and an audit enum that cannot represent half the events requiring an audit entry. Read `PI/05` §3 before building M3 or M5.

---

## 2. The document set

| File | Contains | Read when |
|---|---|---|
| **[00-overview.md](00-overview.md)** *(this file)* | Purpose, PI objectives, exit criteria, conventions, the constraints that bind every task | First, once |
| **[01-backlog.md](01-backlog.md)** | 13 epics, 24 features, 57 user stories, 309 tasks, 161.0 ideal days. The core deliverable | Picking up any piece of work |
| **[02-roadmap.md](02-roadmap.md)** | 2 PIs, 16 sprints, dependency graph, milestones, exit gates, risk register | Sequencing, or deciding what comes next |
| **[03-test-plan.md](03-test-plan.md)** | Test work as scheduled items — tooling, data, environments, per-sprint test load, UAT script, defect triage | Before writing a test, and before any sprint's test work starts |
| **[04-release-and-handover.md](04-release-and-handover.md)** | Versioning, pre-release gate, build and signing runbooks, installer verification, handover pack, training, hypercare | From Sprint 15 onward, and once at Sprint 1 to set the version scheme |
| **[05-decisions-and-gaps.md](05-decisions-and-gaps.md)** | D-1…D-14 with rationale and consequence, the propagation task list, remaining gaps with owners | When something in this set looks unsourced |
| **[06-traceability.md](06-traceability.md)** | Requirement → work item → verification across **eleven** namespaces, plus the orphan check | Verifying nothing was missed; checking a module against its Definition of Done |

---

## 3. Reading order

**Starting the build:** `documents/refinement/00-master-index.md` → its §6 ("the thirteen things most likely to be got wrong") → [02-roadmap.md](02-roadmap.md) → [01-backlog.md](01-backlog.md).

**Picking up a task:** [01-backlog.md](01-backlog.md) for the task and its rule IDs → `documents/refinement/02-business-rules.md` for those rules → `04-technical-architecture.md` §6 for the API contract → `03-functional-specification.md` §5 for the screen → `07-design-system.md` for the component spec → `05-quality-and-acceptance.md` §6 for the Definition of Done.

**Something looks unsourced:** [05-decisions-and-gaps.md](05-decisions-and-gaps.md) first, then `documents/refinement/06-decision-log-and-open-items.md`.

---

## 4. Programme increments

### PI-1 — Foundation and Core Calculation (Sprints 1–9)

**Objective:** the admin can set up the console, build a hierarchy, record activity, and see every figure calculate correctly on screen — with all six golden scenarios reproducing through the real UI, not just in a unit test.

**Exit criteria:**

1. All six golden totals — **65 / 62 / 510 / 1,000 / 980 / 10** — reproduce through the built UI.
2. Epics E0, E-UI, E-QA, M1, M3, M4 and M8 Feature M8.1 meet the module-level Definition of Done (`05-quality-and-acceptance.md` §6.2).
3. Business Volume Entry (M2 Feature M2.1) records and corrects entries with immediate chain-upward recalculation and no recalculate control anywhere.
4. The full hierarchy window opens on a >60-descendant network with the size gate naming the real count, while the main console stays measurably responsive.
5. The vocabulary grep passes clean against every literal string in the build.

### PI-2 — Configuration, Close, Reporting, Release (Sprints 10–16)

**Objective:** a full month can be recorded, closed safely, corrected, exported, and the whole console backed up and restored — then packaged, verified on clean machines, accepted by the client and handed over.

**Exit criteria:**

1. All thirteen epics meet the module-level Definition of Done.
2. A complete monthly-close cycle — backup → verify → snapshot → zero → alert clears — exercised end to end against realistic data volume.
3. Performance targets met at the 25,000-member design ceiling: screens < 2s, recalculation < 2s, extracts < 30s (the full hierarchy window is explicitly outside the screen budget per NFR-1's agreed exception; what binds it is main-console responsiveness).
4. Full UAT pass — the client reconciles all six scenarios against their own hand-worked numbers and confirms the on-screen figures match (SC-2).
5. Installers built and verified on clean Windows and macOS machines; handover pack complete against `01-product-and-scope.md` §12.
6. Project-level Definition of Done met (`05-quality-and-acceptance.md` §6.3), with the three recorded deviations (D-6, D-7, D-8) accepted rather than silently skipped.

---

## 5. Conventions used throughout this set

| Convention | Rule |
|---|---|
| **Story IDs** | The 36 existing IDs (`US-0.1` … `US-M9.1`) are carried forward **unchanged**. `documents/refinement/delivery-plan.md` §3 owns their acceptance criteria; this set does not restate or re-derive them. New stories are **appended** with new prefixes (`US-UI.n`, `US-QA.n`, `US-REL.n`, `US-M6.5`), never renumbered — per `00-master-index.md` §8 |
| **Task IDs** | `T-<story-id>-<n>`, e.g. `T-M1.1-3`. Stable once written |
| **Estimates** | Ideal solo days at quarter-day granularity. `0.25` is the floor — nothing is estimated in hours |
| **Sprint references** | `S1` … `S16`. Never a date, never a month name |
| **Rule/AC/API references** | Always by ID, never by restated content |
| **Prototype references** | `ui-prototype-v2.html` is the client-signed UI of record (tier 4). Where a written document leaves a visual or interaction detail unstated, the prototype decides it |

---

## 6. Constraints binding every task in this set

Reproduced here **once**, because each is a place where the obvious implementation is the wrong one. The full list of thirteen is `00-master-index.md` §6; these are the ones that touch more than one epic.

1. **The six golden totals are 65 / 62 / 510 / 1,000 / 980 / 10.** If one moves during development, a rule has been implemented wrongly. Stop and find it before continuing — do not adjust the expected value.
2. **`is_active` has zero computational effect.** Deactivation is a display flag. The project's highest-risk regression; implementing the superseded spec wording silently corrupts every ancestor's Total Business Volume.
3. **Rule-36 as amended (CR-2): an ended-but-unclosed month *accepts* entries; the *current* month is refused** until the older one closes. Any comment, variable name or document describing a total entry lock is stale. The schema value is `awaiting_close`, never `ended_locked`.
4. **An entry belongs to the month its own date falls in** — never to "the month being closed". `period_month` derives from `entry_date` and is fixed there.
4a. **Slab lookup scans thresholds descending, first match wins** (Rule-3, D-11). `slab_table.sort_order` is display-only. Separately, **Rule-10's "top slab" is the highest-*percentage* row, whatever its threshold** — not the highest threshold. Under Rule-41's accepted risk both distinctions become real, and both are where a silent defect would live.
5. **There is no recalculate control and there must never be one.** No command triggers a calculation. `preview_settings_impact` (API-33) asks what the engine *would* produce and writes nothing.
6. **There is no delete path for anything** — member, entry, snapshot or backup. Do not add one; do not propose an erasure route.
7. **Backup verification gates the close transactionally.** Not a prompt, not a warning. Nothing is zeroed until the internal retained copy is written *and* verified.
8. **Differential re-scans *all* of an ancestor's direct children**, not only the one on the changed chain — when an ancestor's slab moves, every sibling's term moves with it.
9. **Restricted vocabulary in every visible string** — screen labels, buttons, column headings, toasts, tooltips, placeholder and empty-state copy, error messages, extract filenames, **and test fixtures and mock data**. Permitted: *member, Business Volume, Rewards, royalty, volume, slab, level, leg*. Excluded, absolutely: the commercial terms listed in `01-product-and-scope.md` §3. Enforced by an automated grep (`US-QA.4`) that fails the build.
10. **Never abbreviate the three core quantities.** "Business Volume", "Total Business Volume" and "Rewards" are spelled out in code, comments and UI alike. `BV`/`ICP` are retired terms.

### Five accepted risks that must not be "fixed"

Each looks like an oversight and is a recorded client decision. Re-raising them costs the client's time; two would deliver scope they explicitly turned down. Full list: `06-decision-log-and-open-items.md` §6.

- Slab-table monotonicity is **not** validated (Rule-41, ADR-009) — the safeguard was offered and declined.
- No monitoring of a silently-failed close (NFR-12) — declined. Do not build it, do not test for it.
- No data-subject erasure route (Rule-42) — permanent complete retention is the requirement.
- The external-medium backup copy is **not** enforced (TR-4) — the internal retained copy is the real gate.
- The full hierarchy chart gets very wide at scale (TR-7) — the client chose this layout knowingly.

---

## 7. What this plan deliberately does not include

| Item | Why |
|---|---|
| Calendar dates, milestones with deadlines | Relative sprint numbering chosen deliberately — see §  Capacity basis above |
| A CI pipeline | **D-7.** Replaced by a scripted local pre-release gate. Recorded as an explicit deviation from `05-quality-and-acceptance.md` §6.3, not silently dropped |
| Automated E2E on macOS | **D-8.** `tauri-driver` cannot drive WKWebView. Covered by a scripted manual checklist instead |
| A paid code-signing certificate | **D-6.** Windows uses a self-signed certificate with a one-time trust install; macOS ships unsigned. The paid-CA option is deferred, not refused |
| Data migration work | NFR-16 — the system starts empty. No import tooling is in scope |
| Concurrency control | Single-user, single-machine, single-session (OC-1, ADR-001) |
| Any member-facing surface | Permanently out of scope (OS-1) |
