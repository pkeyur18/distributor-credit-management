# Open Questions & Decisions

Consolidated register. Every item below also appears at its point of relevance in deliverables 01–10; this is the single place to check what remains outstanding.

**Status as of 6 August 2026: all seven items raised by the readiness analysis are closed.** Nothing outstanding blocks or gates any module. **7 August 2026:** a new client requirement (whole-console backup and cross-device restore) was raised, designed and closed the same day — see below.

---

## Blockers

**None**, at any point in this analysis. No item ever met the bar of "implementation should not begin until resolved."

---

## Closed — decided by the architect / client

### HIGH-1 — Export/backup command reconciliation ✅ RESOLVED 6 Aug 2026
**Was:** `architecture.md` Appendix C names three export commands, but the approved prototype's Reports screen shows a fourth card, "Closed month snapshot," overlapping the separately-documented `redownload_backup`/`list_backups`.
**Decision (architect):** Confirmed. "Closed month snapshot" downloads a closed month's data as `.xlsx` — used when entries in that month have since been corrected, or when the client simply wants another copy. It maps to `redownload_backup`; no new backend command exists.
**Applied:** [04-api-specification.md](04-api-specification.md) carries this as the definitive mapping (API-20). No further work.

### HIGH-2 — Data-subject erasure route ✅ CLOSED 6 Aug 2026 — not an issue
**Was raised as:** a possible gap, on the reasoning that permanent retention (Rule-38) plus no hard-delete (Rule-28) leaves no path to fulfil an erasure request.
**Decision (architect, on the client's stated requirement):** There is no gap and no compliance issue. The client has specifically required that **members are never removed from the application at all**, and that **all data persists throughout — including in exports**. Permanent, complete retention is the deliberate requirement, not an oversight to be worked around.
**Applied:** recorded as a confirmed requirement in [06-security-authorization-matrix.md](06-security-authorization-matrix.md) §6. Removed from the outstanding register. **Not to be re-raised in future analysis.**

### MEDIUM-1 — Settings mid-period recalculation warning ✅ BUILT 6 Aug 2026
**Was:** RQ-18/V7.6 requires a pre-save warning when a settings change re-works the open month; the prototype saved silently with only a success toast.
**Decision:** Build it. Design approved from mockups — **variant C**: the warning names the open month, states that closed months are unaffected, shows Rewards before → after, and lists the members actually affected.
**Built in** `documents/design/ui-prototype-v2.html`:
- `previewRecalcImpact()` — dry run against candidate settings, reusing the live Total Business Volume (unaffected by slab/royalty settings) and re-running `computeRewards` only.
- `confirmSettingsRecalc()` — the variant-C modal.
- Fires on **every** save of the Slab table and Royalty sections, and only those two — the other three settings sections change nothing already calculated and still save silently.
- On a **Royalty** save no member's slab can move, so the list shows members who **start or stop earning royalty** instead, with a "Members earning royalty: before → after" row. Decided 6 Aug 2026.
- Cancel is a true no-op; the duplicate-threshold guard still refuses bad input *before* the warning is offered.

### LOW-1 — Hierarchy chart's ">60 descendants" gate ✅ CLOSED — not applicable
Already rectified in the prototype (commit `328c1a8`) as a deliberate design decision. No action, no flag.

### LOW-2 — Removing the last slab row ✅ BUILT 6 Aug 2026
**Decision:** Reject, as recommended. Built: the row's remove control is `disabled` with an explanatory `aria-label` when one row remains, an explanatory hint appears beneath the table, and `removeSlabRow()` refuses with a named message if reached another way.

### LOW-3 — Corrupted/unreadable data at launch ✅ BUILT 6 Aug 2026
**Decision:** Build it — "very much required." Design D approved.
**Built:** a full-screen data-recovery state (`APP.authPhase === 'db-error'`) in the same frame as the lock screen, listing the most recent retained backups by the month they hold, marking corrected months, and stating plainly that anything recorded after the chosen backup will need entering again. Reachable in the prototype via `#db-recovery` so the failure state stays demonstrable without appearing anywhere in the product UI.

### LOW-4 — Business Volume entries per month (sizing figure) ⏸️ DEFERRED
Still not supplied by the client. **Deferred by decision, 6 Aug 2026** — performance-testing strategy is a later phase. The member-count ceiling (500–5,000 actual, 25,000 architectural) is settled and is what actually drives the architecture; this figure affects only realistic data-volume rehearsal. Not a gate on any module.

---

## New client requirement — 7 August 2026

### NEW-1 — Whole-console backup and cross-device restore ✅ DESIGNED AND CLOSED 7 Aug 2026
**Was raised as:** a new client requirement, outside the original readiness analysis and every source document up to 6 August 2026 — the client wants the entire console (not one month) backed up on a configurable schedule, and able to be restored on a different desktop or laptop entirely, ending up in exactly the state the original machine held.
**Decisions (brainstormed and confirmed with the client the same day):**
- The backup is a verified copy of the whole encrypted database file, nothing excluded — credentials included, so a restored machine needs no re-setup.
- The schedule (off/daily/weekly/monthly) is checked once, at successful login — the only point the application is reliably running, since it has no background service while closed.
- Retention: keep the most recent backups, default 10, client-adjustable count, oldest pruned automatically.
- Restore is reachable from two places: a plain link on the ordinary first-run setup screen — no separate welcome/choice screen — leading to the same recovery screen the db-error case uses, reworded rather than duplicated (a brand-new install, nothing to log into yet, and no local backups of its own to list, so it goes straight to a file picker); and a new "Restore" card in Settings (a deliberate rollback on an already-running console).
- Restore confirmation is a checklist modal (checkbox + Restore button) — the same weight already given to closing a month, not a heavier or lighter treatment — and the console automatically takes one more backup of its own current state immediately before any restore overwrites it.
**Applied:** `client-requirements-validation.md` RQ-23 (M7.7/M8.6/M8.7), `user-needs-document.md` UN-28, `architecture.md` ADR-012/§15.5, `03-business-rules.md` Rule-43, `04-api-specification.md` API-37–40, `05-data-model-specification.md` (`backups` table generalized), `02-requirements-traceability-matrix.md`, `09-implementation-backlog.md` (US-M7.4/US-M8.5/US-M8.6), `06-security-authorization-matrix.md` §3, `PRODUCT.md`, `DESIGN.md`, and prototyped in `documents/design/ui-prototype-v2.html`.
**Remaining work:** port the prototyped behaviour to the real Rust/React implementation, same as every other approved prototype feature — no further design decision is outstanding.

---

## Technical decisions taken during analysis

### `reverse_entry` is dropped
`architecture.md` Appendix C listed `reverse_entry` as distinct from `edit_entry`. No requirement document describes a functionally separate reversal — `client-requirements-validation.md` RQ-7 treats "edited or reversed" as synonymous — and the approved prototype implements only editing.
**Decision (architect):** speculative leftover. `edit_entry`, append-only and fully audited, is the complete mechanism, including for closed-month corrections (Rule-39). Applied in [04-api-specification.md](04-api-specification.md).

### Improvements made while building the above
Small, in-scope corrections to `ui-prototype-v2.html` made alongside the three features:
- **Escape now closes a modal** — the impeccable critique flagged the total absence of an Escape handler and it was still unfixed. Modals opened with `dismissable: false` (add/edit member) deliberately still ignore it.
- **`role="dialog"` / `aria-modal` / `aria-labelledby`** added to the modal primitive.
- **Toast icons had no size rule** (`.toast svg`), so every toast icon rendered at the SVG default size. Pre-existing defect, one-line fix.
- **`hashchange` listener** for the recovery-screen trigger — a hash appended to an already-open page is a same-document navigation, so `init()` never re-runs and the trigger would otherwise silently do nothing.

---

## Resolved, but the original source document is stale — cite the correction

### Empty elapsed month → yearly-average treatment
`requirement-spec.md` marks this ☐ open, and `open-questions-checklist.md`'s copy of the same question is never updated to ✅ despite all 22 of its other questions being closed. The resolution is in `client-requirements-validation.md` **RQ-16** (3 August 2026): an empty month produces no snapshot and is excluded from the averaging denominator. **Cite RQ-16, not the stale ☐ markers.**

### PIN vs. complex password
`requirement-spec.md` Rule-29 frames this as a pending either/or choice. `client-requirements-validation.md` M8.5 (4 August 2026) resolves it: **both** may be configured at once, either authenticates. See Rule-29 (corrected) in [03-business-rules.md](03-business-rules.md).

### Slab-table monotonicity validation
Reads like an obvious gap on first encounter, but the client was offered this safeguard and **explicitly declined it** (V3.4/V7.5/RQ-1, ADR-009). Do not add it unprompted — it is a documented accepted risk (Rule-41).

---

## Summary

| ID | Outcome | Remaining work |
|---|---|---|
| HIGH-1 | Confirmed — maps to `redownload_backup` | None |
| HIGH-2 | Closed — client requires full permanent retention, no removal | None; do not re-raise |
| MEDIUM-1 | Built (variant C) | None |
| LOW-1 | Not applicable — already rectified | None |
| LOW-2 | Built | None |
| LOW-3 | Built (design D) | None |
| LOW-4 | Deferred to the performance-testing phase | Sizing figure, when convenient |
| NEW-1 | Designed and closed — whole-console backup & cross-device restore (Rule-43) | Port prototyped behaviour to the real implementation |

**Nothing outstanding gates any module.** The conditions attached to the original "READY WITH CONDITIONS" verdict in [01-implementation-readiness-assessment.md](01-implementation-readiness-assessment.md) have all been met.
