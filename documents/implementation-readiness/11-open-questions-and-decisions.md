# Open Questions & Decisions

This is the consolidated record — every item below also appears at its point of relevance in deliverables 01–10, but is gathered here as the single place to check "what's still outstanding" without re-reading everything else.

---

## Blockers

**None.** Zero items meet the bar of "implementation should not begin until resolved."

---

## High Priority

### HIGH-1 — Export/backup command reconciliation
**Problem:** `architecture.md` Appendix C names exactly 3 export commands (`export_monthly`, `export_yearly_average`, `export_low_contribution`). The approved prototype's Reports screen shows a **4th** export card, "Closed month snapshot" (always the latest version), which functionally overlaps with the separately-documented `redownload_backup`/`list_backups` commands (M6 responsibilities).
**Source:** Cross-check finding, this analysis (Design ↔ Prototype).
**Impact:** API-spec ambiguity for module M6. Does not block starting the project or any other module.
**Recommended resolution:** Treat "Closed month snapshot" as the UI's presentation of `redownload_backup` — no new backend command needed. **Already applied** as the working assumption throughout [04-api-specification.md](04-api-specification.md).
**Requires confirmation from:** Architect (you) — this is a documentation/API-design reconciliation, not a client business decision. Recommend confirming before M6 is marked done (per [10-definition-of-done.md](10-definition-of-done.md)).
**Implementation impact:** None if the recommended default is accepted — `redownload_backup` already covers the behaviour.

### HIGH-2 — DPDP erasure/correction request route
**Problem:** Permanent retention (Rule-38) and no hard-delete, ever (Rule-28) are both explicit, deliberate requirements. No artifact anywhere describes how a data-subject erasure request under India's Digital Personal Data Protection Act, 2023 would be fulfilled given that constraint.
**Source:** This analysis (§6 compliance review, not previously flagged in any source document).
**Impact:** Genuine compliance risk if the client is subject to DPDP obligations and never resolves this — but it is not derivable from any artifact, so it cannot be assumed either way.
**Recommended resolution:** Do not build speculatively. Escalate to the client (and their legal counsel, if any) for a written answer. A likely resolution shape: an "erasure requested" flag that suppresses the member from all future exports/searches without violating the no-hard-delete/permanent-audit-trail requirements — but this is a guess, not a decision, and should not be implemented until confirmed.
**Requires confirmation from:** Client (Siddharth Patel) — this is a legal/business decision, not a technical one.
**Implementation impact:** Likely a small schema addition (a flag, not a delete path) once answered. Recommend resolving before M1 is marked done, per [10-definition-of-done.md](10-definition-of-done.md).

---

## Medium Priority

### MEDIUM-1 — Settings mid-period recalculation warning missing from the approved prototype
**Problem:** `client-requirements-validation.md` (RQ-18/V7.6) requires a warning when a settings change (royalty rate, royalty min-children, slab table) recalculates the current open period. The approved prototype's Settings screen saves silently with only a success toast — no such warning is shown anywhere in the actual UI.
**Source:** This analysis (Design ↔ Prototype cross-check).
**Impact:** A genuine gap between documented intent and the approved visual reference. Does not block other modules.
**Recommended resolution:** Add the warning dialog as a backlog item — tracked as **US-M7.3** in [09-implementation-backlog.md](09-implementation-backlog.md). This is additive UI work, not a redesign.
**Requires confirmation from:** No one — this is simply a gap to build, the requirement text is already clear.

---

## Low Priority

### LOW-1 — Hierarchy chart's ">60 descendants" confirmation gate has no requirement-document basis
**Problem:** The prototype shows a confirm-before-render gate when a full-hierarchy view would exceed 60 descendants. No Rule, FR, or UN describes this threshold or this UX pattern.
**Recommended resolution:** Keep it — reasonable, non-blocking UX polish, doesn't conflict with anything. Classified per the task's own required label: `Prototype Behavior Not Explicitly Covered By Requirements`. Not silently promoted to an approved requirement; recorded here so it's traceable as a UI-only decision if ever questioned.
**Requires confirmation from:** No one, unless the client should be told this exists (optional courtesy, not a gate).

### LOW-2 — "Remove the last remaining slab row" behaviour undefined
**Problem:** No source document states what happens if the admin tries to remove every slab row down to zero.
**Recommended resolution:** Reject — a slab table cannot be meaningfully empty. Tracked as **US-BACKLOG-3**.
**Requires confirmation from:** No one; this is a safe, obvious default, but flagged rather than silently assumed, per the task's instruction to distinguish assumptions from confirmed requirements.

### LOW-3 — Corrupted/unreadable database file at launch has no defined recovery path
**Problem:** No source document addresses what the application should do if the SQLite/SQLCipher file is corrupted or unreadable at startup.
**Recommended resolution:** Detect on launch, present a clear path to restore from the most recent internal or external backup rather than crashing silently. Tracked as **US-BACKLOG-4**.
**Requires confirmation from:** Architect — this is a technical design decision, not a business one, but should be made explicitly rather than left to whatever the Tauri/SQLite error path happens to produce by default.

### LOW-4 — Business Volume entries per month (sizing figure) still not supplied
**Problem:** `open-questions-checklist.md` and `requirement-spec.md` §10 both flag this as outstanding. The member-count design ceiling (500–5,000 actual, 25,000 architectural) is settled and drives the actual performance architecture (ADR-005's chain-upward approach is member-count-independent), so this figure affects test planning and realistic data-volume rehearsal, not the architecture itself.
**Recommended resolution:** Request from the client for performance-test realism; do not block anything on it.
**Requires confirmation from:** Client, low urgency.

---

## Resolved This Session (Technical Decisions, not open questions)

### DECIDED — `reverse_entry` is dropped
**Context:** `architecture.md` Appendix C lists `reverse_entry` as an IPC command distinct from `edit_entry`. No requirement document (including the later `client-requirements-validation.md` RQ-7, which treats "edited or reversed" as synonymous) describes a functionally distinct reversal/void action, and the approved prototype implements only editing — no separate reverse/void UI element exists anywhere.
**Decision:** Confirmed directly by the architect during this session's clarification step: `reverse_entry` was speculative/leftover in the architecture document; `edit_entry` (append-only, fully audited) is the real and complete mechanism, including for closed-month corrections (Rule-39).
**Applied:** [04-api-specification.md](04-api-specification.md) drops the command from the surface with this rationale noted inline. [05-data-model-specification.md](05-data-model-specification.md)'s `audit_log.cause` enum retains the option to note this if the client wants the word "reversal" preserved as a cause label, but recommends `edit`/`correction` only going forward.

---

## Resolved, But the Original Source Document Is Stale — Cite the Correction, Not the Stale Text

### RESOLVED — Empty elapsed month → yearly-average treatment
`requirement-spec.md` marks this ☐ open ("Flagged for confirmation — empty elapsed months"). `open-questions-checklist.md`'s own copy of the same question (lines 112–116, 850–853) is **never updated** to a ✅ anywhere in that file, despite every one of its other 22 questions being closed. The actual resolution exists in `client-requirements-validation.md`'s **RQ-16** (3 August 2026): confirmed — an empty month produces no snapshot and is excluded from the yearly-averaging denominator, matching what both stale documents had already recommended. **When building or citing this rule, reference RQ-16, not the stale ☐ markers in the other two files** — they were simply never swept up when the later document closed the question.

### RESOLVED — PIN vs. complex password
`requirement-spec.md` Rule-29 frames this as still pending client choice, and `open-questions-checklist.md` line 745 shows it deferred. `client-requirements-validation.md` M8.5 (4 August 2026) resolves it: **both** may be configured simultaneously, not an either/or choice. See Rule-29 (corrected) in [03-business-rules.md](03-business-rules.md).

### RESOLVED — Slab-table monotonicity validation
Not a contradiction, but worth restating here since it reads like an obvious gap on first encounter: the client was offered this safeguard and **explicitly declined it** (`client-requirements-validation.md` V3.4/V7.5/RQ-1, `architecture.md` ADR-009). Do not "fix" this by adding validation unprompted — it is a documented, deliberate accepted risk (Rule-41 in [03-business-rules.md](03-business-rules.md)).

---

## Summary Table

| ID | Priority | Owner needed | Blocks |
|---|---|---|---|
| HIGH-1 | High | Architect | M6 done-marking only |
| HIGH-2 | High | Client (legal) | M1 done-marking only |
| MEDIUM-1 | Medium | None (build it) | M7 done-marking only |
| LOW-1 | Low | None (keep as-is) | Nothing |
| LOW-2 | Low | None (safe default) | M7 done-marking only |
| LOW-3 | Low | Architect | M5/M8 done-marking only |
| LOW-4 | Low | Client, low urgency | Performance-test realism only |

**No item in this table blocks starting implementation.** All are scoped to specific modules' "done" bar, per [10-definition-of-done.md](10-definition-of-done.md).
