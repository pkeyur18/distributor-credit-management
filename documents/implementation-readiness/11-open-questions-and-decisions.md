# Open Questions & Decisions

Consolidated register. Every item below also appears at its point of relevance in deliverables 01–10; this is the single place to check what remains outstanding.

**Status as of 6 August 2026: all seven items raised by the readiness analysis are closed.** Nothing outstanding blocks or gates any module. **7 August 2026:** a new client requirement (whole-console backup and cross-device restore) was raised, designed and closed the same day — see below. **Also 7 August 2026:** three further client change requests (**CR-1, CR-2, CR-3**) were raised after the `documents/final/` set was approved, designed and specified the same day — see the CR block at the end of this file. **8 August 2026:** two more client change requests (**CR-4, CR-5**) were raised the day before implementation begins — see the CR-4/CR-5 block at the end of this file. None is blocking.

> ⚠️ **This file was flagged stale on 7 August 2026** by [../final/00-master-index.md](../final/00-master-index.md) §3 — it carries figures that were correct on 6 August. Where it disagrees with `documents/final/`, the final set wins. The CR block below is current.

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

### LOW-1 — Hierarchy chart's ">60 descendants" gate ✅ CLOSED — re-opened and fully resolved 7 Aug 2026
Originally closed as "not applicable — already rectified in the prototype (commit `328c1a8`) as a deliberate design decision."

**Re-opened 7 Aug 2026 while specifying CR-3, because that closure was wrong in two ways:** the gate had no source rule and no traceability row (it stood as *"Prototype Behavior Not Explicitly Covered By Requirements"*), and it was **not actually present in the current prototype** — the documents described it, the code did not.

**Now genuinely closed.** CR-3 gives it **Rule-45**, **V4.5** and a home: it gates the **full hierarchy window**, which is the only view in the system that draws an unbounded number of nodes. The one-branch-at-a-time Structure chart is bounded by a single generation and needs no gate. The confirmation must name the **exact** member count, and Cancel must open nothing at all.

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
| LOW-1 | Re-opened and fully resolved 7 Aug 2026 by CR-3 — the gate now belongs to the full hierarchy window, backed by Rule-45/V4.5 | Build it; it is not in the prototype yet |
| LOW-2 | Built | None |
| LOW-3 | Built (design D) | None |
| LOW-4 | Deferred to the performance-testing phase | Sizing figure, when convenient |
| NEW-1 | Designed and closed — whole-console backup & cross-device restore (Rule-43) | Port prototyped behaviour to the real implementation |
| NEW-2 | Designed and closed 7 Aug 2026 — **CR-1**, phone number as a search key (Rule-44) | Build it |
| NEW-3 | Designed and closed 7 Aug 2026 — **CR-2**, entry permitted into an ended-but-unclosed month (Rule-36 amended) | Build it. **Reverses RQ-11** — read the amended rule before touching M2 or M5 |
| NEW-4 | Designed and closed 7 Aug 2026 — **CR-3**, the full hierarchy window (Rule-45, FR-10) | Build it |
| NEW-5 | Designed and closed 8 Aug 2026 — **CR-4**, own-Business-Volume reward (Rule-46) | Build it. Golden regression set recomputed — see [08-testing-strategy.md](08-testing-strategy.md) |
| NEW-6 | Designed and closed 8 Aug 2026 — **CR-5**, Home "Rewards by slab" chart | Build it. No new command |

**Nothing outstanding gates any module.** The conditions attached to the original "READY WITH CONDITIONS" verdict in [01-implementation-readiness-assessment.md](01-implementation-readiness-assessment.md) have all been met.

---

## Client change requests — 7 August 2026 (CR-1, CR-2, CR-3)

Raised by the client after the `documents/final/` set was approved. Recorded here for continuity; the authoritative account, with what each one reverses, is [../final/06-decision-log-and-open-items.md](../final/06-decision-log-and-open-items.md) §5.

### NEW-2 / CR-1 — Phone number as a search key ✅ DESIGNED AND CLOSED 7 Aug 2026
**Asked:** search by phone number as well as member ID and name, since a phone number is unique to a member.
**Decided:** every search box in the console matches on phone. Both sides are reduced to a canonical key (non-digits stripped, then a country prefix or trunk zero dropped) so formatting is irrelevant in either direction; the phone clause engages only at **four digits or more** so short queries do not sweep in unrelated members; results gain a **phone column**. One shared search function — behaviour must not differ between screens.
**Lands as:** Rule-44, FR-1 (amended), M2.1/M4.6, V4.4, AC-40/AC-41, UN-29, US-M1.4 (amended), API-06 (amended). No schema change, no new command.

### NEW-3 / CR-2 — Entry into an ended-but-unclosed month ✅ DESIGNED AND CLOSED 7 Aug 2026
**Asked:** remove the hard entry lock when the previous month is not closed, because a purchase made on the last day of a month is often reported two or three days later. The client's condition: entries **for the previous month** stay possible while it is unclosed; **current-month** entries require it to be closed first.
**Decided:** Rule-36 is **narrowed, not removed**, exactly as stated. No time limit, no configurable grace window, no countdown — the grace lasts as long as the month stays unclosed. A configurable "grace days" setting was offered and declined.
**⚠️ Reverses RQ-11's answer of 3 Aug 2026** ("hard stop kept, no grace period"), and with it OC-2, OC-6's severity, R-4's position, AC-19, V2.3 and V2.5.
**Lands as:** Rule-36 (amended), M2.3/M2.6/M2.7, M5.2, V2.3/V2.5/V2.6/V2.7, AC-19/AC-42/AC-43, UN-30, US-M2.3/M2.4/M2.5 (new), US-M2.1/US-M5.3 (amended), API-07/API-08 (amended).
**Documentation-only schema consequences:** `periods.status` `ended_locked` → `awaiting_close`; `member_period_totals` widened to any not-yet-closed period (composite PK already supports it). The `PeriodLocked` error variant is **retired**, replaced by `PeriodNotAcceptingEntries { month, blocking_month }` and `PeriodClosed { month }`.
**Multiple outstanding months:** any outstanding month accepts entries, not merely the oldest. The client noted this will not arise in practice and chose the permissive behaviour for the hypothetical case. Figure screens show the **oldest**; a month switcher renders **only** when more than one month is outstanding.

### NEW-4 / CR-3 — Full hierarchy window ✅ DESIGNED AND CLOSED 7 Aug 2026
**Asked:** a "View Full Hierarchy" button on the Structure screen opening a new window with the full hierarchy expanded — with the binding constraint that the main console must not be slowed down: *"it just opens new window with expanded full hierarchy with all data and forgets."*
**Decided:** a separate read-only window, rooted always at the **top member**, drawing once and never updating, carrying an "as at" timestamp. Zoom to 10%, fit-width, in-window search-and-highlight, print. Gated above **60 descendants** by a confirmation naming the exact count.
**Layout:** top-down chart, fully expanded — chosen by the client over a width-stable indented outline after being shown the width behaviour. Recorded as **TR-7**, accepted, with the zoom floor, fit-width, in-window search and the size gate as the agreed mitigations. The outline is the named fallback.
**Lands as:** Rule-45, FR-10, M4.7, V4.5, AC-44/AC-45, UN-31, US-M4.3 (new), API-11 (amended — the pre-existing `full_tree` parameter, no new command).
**Closes LOW-1**, which had stood since 6 Aug 2026 as untraced prototype behaviour that was not in fact present in the prototype.

## Client change requests — 8 August 2026 (CR-4, CR-5)

Raised by the client the day before implementation begins. The authoritative account is [../final/06-decision-log-and-open-items.md](../final/06-decision-log-and-open-items.md) §5.

### NEW-5 / CR-4 — Reward on own Business Volume ✅ DESIGNED AND CLOSED 8 Aug 2026
**Asked:** a member's own Business Volume should also earn a reward, at the member's own slab — worked example supplied (A with children B/C/D at 100 BV/2% each, A's own BV 100, A's TBV 400 at 4%, total Rewards = 6 + 4 = 10).
**Decided:** a third, additive term — `OwnReward(x) = slab%(x) × BusinessVolume(x)`. Differential (Rule-8) and Royalty (Rule-10) are **not redefined**.
**Reverses:** the 3 August 2026 decision that a member earns nothing on their own Business Volume — superseded specifically by this addition, not by a redefinition of the differential term.
**Lands as:** Rule-46 (new), Rule-12 (amended), M3.5 (new), V4.3 (amended), AC-46, US-M3.1 (amended), US-M4.1 (amended). Golden regression set recomputed: scenarios 1–3 move (65/62/510), 4–5 unchanged (1,000/980 — own BV is 0 in both), scenario 6 added (the client's own worked example, total 10).

### NEW-6 / CR-5 — "Rewards by slab" chart on Home ✅ DESIGNED AND CLOSED 8 Aug 2026
**Asked:** alongside the existing "Members by slab" chart, a second chart showing total accumulated Rewards per slab, in as simple/user-friendly a form as possible.
**Decided:** reuse the members-by-slab card's exact bar-list pattern, placed directly below it — same shape the client already reads, each bar summing Rewards instead of counting members, current live period only.
**Lands as:** FR-1 (extended), V4.6 (new), AC-47, US-M4.4 (new). **No new API command** — client-side aggregation, matching the sibling chart's existing pattern.
