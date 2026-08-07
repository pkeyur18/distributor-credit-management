# Business Rules Specification

Source: `requirement-spec.md` Rules 1–38, corrected/extended where `client-requirements-validation.md` (tier 1) overrides or adds to the original text. Every correction is marked **[CORRECTED]** or **[NEW]** and cites its source. Rules not corrected are marked **[AS SPECIFIED]**.

---

### Rule-1 — Level widths are advisory
**Rule:** Level 2/3/4 width defaults (9/6/3) are informational only. Onboarding never rejects a member for exceeding them; UI may show a soft warning.
**Source:** requirement-spec.md Rule 1 (draft L21–27 contradicted by Scenarios 3 & 5, confirmed advisory).
**Applies to:** Member onboarding (M1).
**Validation:** None blocking — a count-vs-setting comparison only, purely presentational.
**Error behaviour:** None. Not an error case.
**Implementation impact:** `add_member` must never reject on level-width count.
**Test requirement:** Onboard a 7th child under a Level-2 member with width setting 6 → succeeds, shows warning.

### Rule-2 — Unique 6-digit member ID
**Rule:** Every member gets a unique 6-digit ID at onboarding, used as the primary lookup key.
**Source:** requirement-spec.md Rule 2. **[AS SPECIFIED]**
**Applies to:** M1 (creation), search (M4), entry (M2).
**Validation:** Uniqueness enforced at the DB layer (PK).
**Error behaviour:** N/A — allocation is system-controlled, not user-entered (see Rule-35).
**Implementation impact:** `members.id` is the PK, not a surrogate row ID.
**Test requirement:** Two members can never share an ID; search by ID returns exactly one row.

### Rule-3 — Slab lookup
**Rule:** A member's slab is the highest slab whose threshold is ≤ their Total Business Volume. Below the lowest threshold → 0%.
**Source:** requirement-spec.md Rule 3, proved by Scenarios 2 & 4 (`>=` boundary). **[AS SPECIFIED]**
**Applies to:** M3 calculation engine.
**Validation:** N/A (internal lookup, not user input).
**Error behaviour:** N/A.
**Implementation impact:** Slab table must be queried in threshold-descending order; first match wins.
**Test requirement:** TBV exactly at a threshold boundary lands in the higher slab (Scenario 2: C at 3,000 → 8%; Scenario 4: A at 10,000 → 14%).

### Rule-4 — Slab thresholds/percentages configurable
**Rule:** Every threshold and percentage in the slab table is editable in settings.
**Source:** requirement-spec.md Rule 4. **[AS SPECIFIED]**
**Applies to:** M7 Settings.
**Validation:** None on cross-row consistency (see the monotonicity note below).
**Error behaviour:** Duplicate-threshold guard on save (matches prototype's error toast).
**Implementation impact:** `update_slab_row` must not assume the current 7-row table shape.
**Test requirement:** Move the 2% slab to 200 and the 6% slab to 1,000; confirm existing member slabs re-lookup correctly on next recalculation.

### Rule-5 — Bottom-up calculation order
**Rule:** Calculation is a post-order tree traversal — a member's TBV cannot be computed until every direct child's TBV is final. Results propagate to the root.
**Source:** requirement-spec.md Rule 5. **[AS SPECIFIED]**
**Applies to:** M3.
**Validation:** N/A — structural/algorithmic, not a user-facing validation.
**Error behaviour:** N/A.
**Implementation impact:** Chain-upward recalculation (ADR-005) is the practical implementation of this rule — only the changed member's ancestor chain needs recomputation, not a full tree walk, because siblings' TBVs are already final.
**Test requirement:** Multi-depth Scenario 3 reproduces exactly (A=8,000, six children at 1,250 each).

### Rule-6 — Total Business Volume formula
**Rule:** `TotalBusinessVolume(x) = BusinessVolume(x) + Σ TotalBusinessVolume(c)` for every **direct** child c. One level deep only; full-depth coverage is transitive.
**Source:** requirement-spec.md Rule 6, confirmed by client 2026-08-03 against Scenario 3. **[AS SPECIFIED]**
**Applies to:** M3.
**Validation:** N/A.
**Error behaviour:** N/A.
**Implementation impact:** A member's own Business Volume term is never omitted, even when it must be derived (Scenario 3: A's own BV = 500, never stated directly in the source examples).
**Test requirement:** Scenario 3 (total 450) and Scenario 4/5 (P's own BV = 0, confirmed as a write-up simplification, not a different rule).

### Rule-7 — Slab driven by TBV
**Rule:** `slab%(x) = lookup(TotalBusinessVolume(x))`, never by the member's own Business Volume.
**Source:** requirement-spec.md Rule 7. **[AS SPECIFIED]**
**Applies to:** M3.
**Implementation impact:** A member can show a small own-BV figure while sitting on a high slab (explicit, documented consequence — FR-2's chart-value note).
**Test requirement:** All five scenarios.

### Rule-8 — Differential earnings
**Rule:** `Differential(x) = Σ [(slab%(x) − slab%(c)) × TotalBusinessVolume(c)]` for every direct child c. Base is the child's TBV, not their own BV; only direct children contribute; a member earns nothing on their own Business Volume.
**Source:** requirement-spec.md Rule 8, confirmed 2026-08-03. **[AS SPECIFIED]**
**Applies to:** M3.
**Implementation impact:** Grandchildren must never contribute a separate term — they are already inside the child's TBV.
**Test requirement:** Scenario 3 (only scenario where child TBV ≠ child own-BV, disambiguates the base).

### Rule-9 — Differential never negative
**Rule:** Because `TBV(parent) ≥ TBV(child)` by construction, `slab%(parent) ≥ slab%(child)` always holds. No clamping, no negative-earnings case, no error state — a structural guarantee.
**Source:** requirement-spec.md Rule 9. **[AS SPECIFIED]**
**Implementation impact:** No defensive negative-differential check is needed in normal operation — **except** that Rule-4's monotonicity gap (see below) can theoretically break this guarantee if the admin misconfigures the slab table. Do not silently clamp; see [07-error-edge-case-matrix.md](07-error-edge-case-matrix.md).
**Test requirement:** Confirm no scenario ever produces a negative differential term.

### Rule-10 — Royalty qualification
**Rule:** Let Q = direct children of x on the top slab. If `|Q| ≥ royalty_min_children` (default 3), `Royalty(x) = Σ royalty_rate × TBV(c)` for c in Q (default rate 1%). Otherwise 0. Direct children only, both for counting and for paying.
**Source:** requirement-spec.md Rule 10. **[AS SPECIFIED]**
**Applies to:** M3.
**Implementation impact:** "Top slab" is always the highest-percentage row currently in the table (Rule-27), never hardcoded to 14%/10,000.
**Test requirement:** Scenario 4 (pure royalty, 1,000) and Scenario 5 (differential + royalty together, 980).

### Rule-11 — Royalty and differential never double-pay
**Rule:** If a child is on the top slab, the parent is automatically on the top slab too (TBV(parent) ≥ TBV(child)), so that child's differential term is exactly 0. Disjoint by construction — no explicit exclusion logic needed.
**Source:** requirement-spec.md Rule 11. **[AS SPECIFIED]**
**Test requirement:** AC-6 in `client-requirements-validation.md` — explicit demonstration that royalty and differential recipients don't double-count.

### Rule-12 — Rewards
**Rule:** `Rewards(x) = Differential(x) + Royalty(x)`.
**Source:** requirement-spec.md Rule 12. **[AS SPECIFIED]**

### Rule-13 — Rewards are a separate ledger
**Rule:** Rewards are never added to any member's Business Volume. They do not raise the earner's own slab, do not enter any ancestor's TBV, and do not compound into the next period.
**Source:** requirement-spec.md Rule 13. **[AS SPECIFIED]**
**Implementation impact:** `member_period_totals.rewards`/`royalty` must never feed back into `business_volume` or `total_business_volume` columns for any member, including the earner.
**Test requirement:** After Rewards are computed for a member, re-run the recalculation and confirm their own TBV is unchanged.

### Rule-14 — Unit value (reference only)
**Rule:** 1 unit = 500 Rs, configurable, kept on the settings screen. Reference only — never displayed on any screen, report, or export; plays no part in any calculation. The client converts final Rewards to rupees by hand, outside the application.
**Source:** requirement-spec.md Rule 14. **[AS SPECIFIED]**
**Error behaviour:** N/A — this is a stored, unused-in-calculation reference value.
**Implementation impact:** No screen, export, or calculation may read this setting except the settings screen itself.

### Rule-15 — Business Volume entry flow
**Rule:** Admin searches by name or ID, selects a member, records Business Volume against them.
**Source:** requirement-spec.md Rule 15. **[AS SPECIFIED]**

### Rule-16 — Points-only entry
**Rule:** Admin enters Business Volume directly, nothing else. Up to two decimal places. No rupee entry mode, no currency conversion, no rupee field anywhere on this screen.
**Source:** requirement-spec.md Rule 16, supersedes an earlier "two entry modes" decision. **[AS SPECIFIED]**
**Validation:** Numeric, ≥0.01 (see zero/negative rule below), max 2 decimals.
**Error behaviour:** Reject non-numeric, reject >2 decimal places, Save button disabled until valid.

### Rule-16a — Zero and negative Business Volume both refused **[CORRECTED]**
**Rule:** Neither a negative figure nor a figure of zero is a permitted Business Volume entry. Both are refused at entry.
**Source:** client-requirements-validation.md V2.4, RQ-17. Overrides the architect's own original recommendation ("accept zero, refuse negative") — client decision, 3 August 2026, stricter than proposed.
**Applies to:** M2 BV Entry screen (including the closed-month correction panel).
**Validation:** `amount > 0` (DB-level CHECK constraint, matches architecture's DDL).
**Error behaviour:** Inline validation error; Save disabled.
**Implementation impact:** Do not implement the originally-recommended "zero is a valid no-op entry" path.
**Test requirement:** Attempt to save 0 and −5 → both rejected with a clear message.

### Rule-17 — Manual reset only
**Rule:** Reset is manual only, never automatic. Admin is prompted on the 1st of each month but may act later.
**Source:** requirement-spec.md Rule 17. **[AS SPECIFIED]**

### Rule-18 — Reset flow gated by backup
**Rule:** Backup must be generated and its success confirmed before any figure is zeroed. A failed or cancelled backup aborts the reset entirely; the alert stays up; nothing is touched.
**Source:** requirement-spec.md Rule 18. **[AS SPECIFIED]**
**Applies to:** M5.
**Error behaviour:** Backup write failure → abort, no partial state, alert remains, admin can retry.
**Implementation impact:** Backup-write-then-verify (existence + checksum + readability) must happen inside the same transactional boundary as the abort decision — see `architecture.md` §15.

### Rule-19 — Every export carries basic fields
**Rule:** Every exported report includes name, ID, phone number, volume, and Business Volume, regardless of which optional columns are selected.
**Source:** requirement-spec.md Rule 19. **[AS SPECIFIED]**

### Rule-20 — Persistent reset alert
**Rule:** Raised as soon as the month being closed has ended. Appears as both an undismissable banner on every screen and a notification-list entry. Clears only on successful completion of the reset — no snooze, no dismiss. Multiple outstanding months are all listed; only the oldest can be closed; each closes separately with its own backup and snapshot.
**Source:** requirement-spec.md Rule 20, a client-added requirement not present in the original draft. **[AS SPECIFIED]**
**Implementation impact:** The banner component must have literally no dismiss affordance, not even a disguised one (`ui-theme.md` reinforces this explicitly).

### Rule-21 — Period boundaries
**Rule:** A period is a calendar month, 1st to last day. The reset closes whichever month it belongs to, whenever actually pressed.
**Source:** requirement-spec.md Rule 21. **[PARTIALLY SUPERSEDED]** — the original third bullet ("points entered between the 1st and reset count into the month being closed") is struck through and superseded by Rule-36's hard entry lock, which makes that scenario unreachable. Retained in the source spec for historical record; do not implement the struck-through bullet.

### Rule-22 — Precision
**Rule:** Business Volume and Rewards carry two decimal places throughout storage and calculation. Rounding happens only at the point of display — never at an intermediate step.
**Source:** requirement-spec.md Rule 22. **[AS SPECIFIED]**
**Implementation impact:** Fixed-point integer storage (×100, matching architecture's ADR-004 decision to avoid float drift) is the correct implementation of "no intermediate rounding," not literal decimal rounding at each step.
**Test requirement:** Sum a column of stored ×100 integers and confirm it matches a hand calculator to the cent, with no per-row rounding applied first.

### Rule-23 — Yearly average method
**Rule:** Sum the member's figures across periods that actually have a snapshot, divide by the count of those periods — not a fixed 12. The report displays that count next to each average.
**Source:** requirement-spec.md Rule 23. **[AS SPECIFIED]**
**Implementation impact:** Directly depends on the empty-month rule below (no snapshot → excluded from both the sum and the denominator).

### Rule-24 — Low-threshold report metric
**Rule:** Filters on the yearly average of the member's **own** Business Volume, not TBV. Default threshold 100, configurable.
**Source:** requirement-spec.md Rule 24. **[AS SPECIFIED]** — client's answer differs from the architect's original TBV recommendation; deliberately re-confirmed. The yearly-average export still carries both figures; only the filter metric is own-BV.

### Rule-25 — Royalty stacks at every level
**Rule:** Each member is assessed independently against their own direct children. The same underlying volume can attract royalty at several levels of the same chain.
**Source:** requirement-spec.md Rule 25, re-confirmed 2026-08-03 with the payout consequence explicitly understood. **[AS SPECIFIED]**
**Test requirement:** The worked illustration in requirement-spec.md §4.2 (A/B/C at 10,000 under P, P/Q/R identical siblings under T — same 10,000 attracts royalty twice in one chain).

### Rule-26 — Recalculation trigger
**Rule:** Recalculates immediately on every Business Volume entry. Every affected member's TBV, slab, and Rewards are correct on screen the instant an entry is saved. No manual "recalculate" button, no batch-only mode. Implementation updates only the affected chain upward, not the whole tree.
**Source:** requirement-spec.md Rule 26. **[AS SPECIFIED]**
**Implementation impact:** ADR-005 — chain-upward incremental recalculation, inside one DB transaction per write, re-scanning **all** direct children of each ancestor node (not just the changed leaf), since a node's own slab shift changes every sibling's differential term too.

### Rule-27 — Slab rows addable/removable
**Rule:** Admin may add and remove slab rows, not merely re-threshold the existing seven. Top slab is always recomputed as the highest-percentage row.
**Source:** requirement-spec.md Rule 27, confirmed 2026-08-03. **[AS SPECIFIED]**

### Rule-28 — Member lifecycle **[CORRECTED]**
**Rule:** Edit is permitted at any time. Removal marks a member inactive — **inactive status has zero effect on any calculation; it is a pure display flag.** Members are never hard-deleted; history stays intact.
**Source:** requirement-spec.md Rule 28 originally states inactive members "stop appearing in new periods" (line 422). client-requirements-validation.md V3.5/RQ-2 (4 August 2026) overrides this: inactive members continue to appear, roll up, and calculate exactly as if active — only their display treatment changes (distinct colour/pill). The validation document itself flags this exact wording conflict against the spec.
**Applies to:** M1, M3 (must NOT special-case inactive members in the calculation path), M4 (chart/list display).
**Validation:** N/A for calculation. Display layer must apply a distinct visual treatment (Rule-34's neighbouring requirement).
**Error behaviour:** N/A.
**Implementation impact:** Do **not** filter or zero out an inactive member's Business Volume/TBV contribution anywhere in M3. This is the single most consequential correction in this document — implementing the original spec wording would silently corrupt every ancestor's TBV/Rewards the moment any member is deactivated.
**Test requirement:** Deactivate a mid-tree member with active children beneath them; confirm the deactivated member's Business Volume still rolls up to the root unchanged, and their own TBV/slab/Rewards are still computed normally.

### Rule-29 — Access control **[CORRECTED/EXTENDED]**
**Rule:** One administrator account, used solely by the client. No other accounts, no roles. Members never log in. Protected by a 6-digit PIN **and/or** a complex password — both may be configured simultaneously; either credential authenticates. Failed-attempt lockout is mandatory regardless of which credential type is used.
**Source:** requirement-spec.md Rule 29 frames PIN-vs-password as a pending either/or client decision. client-requirements-validation.md M8.5 (4 August 2026) resolves it as **both simultaneously supported**, not exclusive.
**Applies to:** M8.
**Validation:** PIN: exactly 6 numeric digits. Password: ≥8 chars, letter + number (per prototype's validation rule — not contradicted by any source document, treated as a reasonable derived minimum).
**Error behaviour:** Wrong credential → generic "incorrect" message (no hint which part was wrong); after 5 failed attempts, timed lockout.
**Implementation impact:** `auth` table stores both `pin_hash` and `password_hash` as independently-nullable columns; login accepts either match.
**Test requirement:** Set both a PIN and a password; confirm login succeeds with either.

### Rule-30 — Reference and hierarchy integrity
**Rule:** Reference ID must resolve to an existing, active member — rejected otherwise. The single root member is created once, at initial setup, with no Reference ID, and never again. Any move placing a member beneath their own descendant is blocked.
**Source:** requirement-spec.md Rule 30. **[AS SPECIFIED]**
**Implementation impact:** The loop-check is a belt-and-braces guard only — Rule-37 (no transfers, ever) makes the hierarchy a tree by construction, so this check can never actually fire in normal use. Keep it anyway; do not remove defensive code just because it's provably unreachable under current rules.

### Rule-31 — Backup storage and retention **[CORRECTED/EXTENDED]**
**Rule:** Each backup is downloaded to the administrator's computer **and** retained permanently inside the system, where any past month can be re-downloaded at any time. Nothing is auto-deleted. The downloaded copy must additionally be written to a **physically separate medium** (not merely a different folder on the same disk).
**Source:** requirement-spec.md Rule 31, extended by client-requirements-validation.md RQ-19 (4 August 2026).
**Implementation impact:** The internal retained copy remains the actual close-gate (write-verified before proceeding); the external/separate-medium copy is prompted at the same time but its failure does not block the close — it re-prompts/reminds instead. `architecture.md` §15.3 documents this as an accepted, unenforced process-discipline risk (single point of failure if the client never actually takes the external copy) — not a technical gap to be silently "solved" by, e.g., forcing the close to block on it, which would contradict the documented design decision.

### Rule-32 — Depth overflow
**Rule:** If onboarding would exceed the configured maximum depth, the system warns but allows.
**Source:** requirement-spec.md Rule 32. **[AS SPECIFIED]** — consistent with Rule-1's advisory-only pattern.

### Rule-33 — Configurable export columns
**Rule:** Every field is offered as an export column, with the client's four defaults (name, ID, phone, Business Volume) pre-ticked. Full optional list: email, address, reference number, introducer name, hierarchy level, direct legs count, TBV, slab %, Rewards, royalty earned, joining date, active/inactive status.
**Source:** requirement-spec.md Rule 33. **[AS SPECIFIED]**

### Rule-34 — Phone number uniqueness
**Rule:** A phone number identifies exactly one member, unique across active and inactive members alike. A match on an inactive member offers reactivation instead of erroring — preserving the original 6-digit ID, hierarchy position, and full history. A duplicate record is never created.
**Source:** requirement-spec.md Rule 34. **[AS SPECIFIED]**

### Rule-35 — Member ID allocation **[CORRECTED]**
**Rule:** Each member receives a randomly-chosen, currently-available 6-digit number in the range **100001–999999**. Allocation is random, never sequential. IDs are never released once assigned (deactivation is not deletion, so a deactivated member's number stays permanently taken).
**Source:** requirement-spec.md Rule 35 originally states the range as 100000–999999. client-requirements-validation.md (4 August 2026) confirms the usable range starts at **100001** — 100000 itself is never assigned.
**Implementation impact:** ID-allocation logic must exclude 100000 from the candidate pool. `architecture.md`'s DDL already reflects the corrected range.

### Rule-36 — Reset enforcement
**Rule:** Once a calendar month ends, all entry of Business Volume is locked until that month's reset completes. No entry of any kind is accepted while a reset is outstanding.
**Source:** requirement-spec.md Rule 36, reverses an earlier "non-blocking alert only" decision. **[AS SPECIFIED]**
**Implementation impact:** This is what makes Rule-21's struck-through third bullet unreachable, and is why Rule-38's "immutable" framing was later loosened by the entries-editable-anytime correction (below) rather than by this rule.

### Rule-37 — Transfers prohibited
**Rule:** A member's sponsor/introducer is fixed at creation and can never change. No override exists.
**Source:** requirement-spec.md Rule 37, reverses Rule-28's originally-planned "move with frozen months" provision. **[AS SPECIFIED]**
**Implementation impact:** Combined with Rule-30, this makes the hierarchy a tree by construction — cycles are structurally impossible under any sequence of permitted operations.

### Rule-38 — Reset scope **[EXTENDED — "immutable" is qualified]**
**Rule:** The reset zeroes everything — Business Volume, TBV, Rewards, royalty all go to 0. Before anything is cleared, an immutable snapshot of the closing period is written per member (Business Volume, TBV, slab %, Rewards, royalty, active/inactive status). All yearly reporting is built exclusively from snapshots, never from live values.
**Source:** requirement-spec.md Rule 38. **[AS SPECIFIED, with a downstream qualification]** — "immutable" here means the snapshot is never silently altered; it does **not** mean a closed month can never be corrected. See the entries-editable-anytime rule below, which is the actual, later mechanism for correcting a closed month (a new snapshot **version** is written; version 1 is never touched).

### Rule-39 — Entries editable/reversible at any time, including closed months **[NEW / EXTENDS Rule-38]**
**Rule:** An entry can be edited (or, in the client's own wording, "reversed") at any time, in any month, open or closed. Editing a closed-month entry rewrites that month's permanent record by writing a new, versioned snapshot — the original backup/snapshot version is never overwritten. Reporting always reads the latest version. A UI warning is shown before a closed-month edit ("Editing a record recalculates the affected chain and writes a new snapshot version — the original record is never overwritten").
**Source:** client-requirements-validation.md M2.4/M2.5, M5.9/M5.10, RQ-7 (4 August 2026). A deliberate reversal of the "permanent, uncorrectable once closed" framing that an earlier draft of this specification carried — the validation document states plainly that framing was the architect's own gloss on Rule-38, not the client's actual requirement.
**Applies to:** M2, M5.
**Validation:** Same as Rule-16/16a (amount >0, ≤2 decimals) for the corrected value; date must fall within the target month's bounds.
**Error behaviour:** N/A beyond standard entry validation — this is a permitted, audited operation, not an error path.
**Implementation impact:** `edit_entry` is the single mechanism for both live and closed-month corrections (see [04-api-specification.md](04-api-specification.md) — `reverse_entry` is dropped as a separate command, confirmed dead by the architect this session). Every correction writes an `audit_log` row and a new `monthly_snapshots`/`backups` version.
**Test requirement:** Edit an entry in a closed month; confirm a new snapshot version is created, the original version's data is byte-identical to before, and the yearly-average export reads the new version.

### Rule-40 — Consent capture **[NEW]**
**Rule:** Add Member requires a mandatory consent checkbox; the date is auto-captured. Save is refused until it is ticked.
**Source:** client-requirements-validation.md M1.7, RQ-22 (4 August 2026) — DPDP Act 2023-driven, not present in the original draft or in requirement-spec.md's Rules 1–38.
**Applies to:** M1.
**Validation:** Checkbox must be checked before Save is enabled.
**Error behaviour:** Save button disabled/greyed until ticked; no separate error message needed since the action is simply unavailable.
**Implementation impact:** `members.consent_given` (boolean) and `members.consent_date` (auto-set on save) are new required columns, already present in `architecture.md`'s DDL.

### Rule-41 — Slab-table monotonicity is not validated **[ACCEPTED RISK, NOT A RULE TO BUILD]**
**Statement:** The system does **not** check that slab percentages rise monotonically with thresholds. The client explicitly declined this safeguard (3 August 2026) after the architect recommended it, accepting the residual risk that a misconfigured table could produce a negative differential (violating Rule-9's "structural guarantee") or unexpected Rewards.
**Source:** client-requirements-validation.md V3.4/V7.5/RQ-1; architecture.md ADR-009.
**Implementation impact:** Do **not** add this validation speculatively — it was considered and explicitly rejected. If a future session encounters what looks like a negative-differential bug, check the slab table configuration first; it may be this accepted risk manifesting, not a code defect. The Settings UI carries an explicit on-screen disclaimer instead of a code-level guard.

---

## BUSINESS RULE REQUIRED — none identified

Every business rule needed to build the calculation engine, hierarchy, entry, reset, export, and auth modules is present above, sourced from an approved artifact. No gap was found that would require inventing a rule.

One rule was added to this set on 6 August 2026 as a result of the decisions recorded in [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md); a second was added 7 August 2026 (Rule-43, below) for the whole-console backup and cross-device restore requirement:

### Rule-42 — Members are never removed; all data persists **[NEW — client requirement]**
**Rule:** No member is ever removed from the application, under any circumstance. All member data persists permanently — on screen, in calculations, and **in every export**. Deactivation (Rule-28) is the only lifecycle change available, and it is display-only.
**Source:** Client requirement, confirmed via the architect 6 August 2026. Consistent with, and the reason behind, Rule-28's no-hard-delete and Rule-38's permanent snapshots.
**Implementation impact:** No delete path, no "erasure requested" flag, no export filter that would omit a member. Do not propose one.
**Test requirement:** Every export includes deactivated members; no code path removes a `members` row.

### Rule-43 — Whole-console backup schedule and cross-device restore **[NEW — client requirement]**
**Rule:** The entire console — every member, entry, monthly record and setting, not one month — can be backed up on a configurable schedule (off/daily/weekly/monthly) or on demand, with the most recent backups retained (default 10, client-adjustable) and older ones pruned automatically. The backup is a verified copy of the whole encrypted database file, credentials included. It can be restored on any machine, including a brand-new install with nothing set up yet, bringing the console to exactly the state the backup holds. Restoring always names what it will replace, requires deliberate confirmation, and the console backs up its own current state immediately before overwriting it.
**Source:** client-requirements-validation.md RQ-23, M7.7/M8.6/M8.7 (7 August 2026). architecture.md ADR-012, §15.5.
**Applies to:** M7 (schedule/retention setting), M8 (taking and restoring a backup).
**Validation:** Schedule value must be one of `off`/`daily`/`weekly`/`monthly`; retention count ≥ 1. A restore is refused if the target file's checksum does not verify.
**Error behaviour:** An invalid schedule/retention value is refused with a field-level error, same pattern as other settings fields. A failed checksum on restore refuses the restore outright — nothing is overwritten.
**Implementation impact:** Generalizes the existing `backups` table rather than adding a second one — `period_id` becomes nullable, plus new `kind` (`period_close`/`scheduled`/`manual`/`pre_restore_safety`) and `schedule_kind` columns (see [05-data-model-specification.md](05-data-model-specification.md)). The schedule is checked once per `login` (no background service exists while the app is closed) via `run_console_backup_now`; `restore_from_backup_file` is a new **unauthenticated** command (a brand-new install has nothing to authenticate against) reused across a plain link on the ordinary first-run setup screen, the existing db-error recovery screen (same screen, reworded rather than duplicated for the voluntary case), and the authenticated Settings "Restore" card. This is additive to Rule-31/Rule-39 — the month-close backup gate and correction-versioning mechanism are unchanged.
**Test requirement:** A scheduled backup fires on the next login once its interval has elapsed and not before; retention prunes the oldest `scheduled`/`manual` backup once the count is exceeded while leaving `period_close` and `pre_restore_safety` rows untouched; restoring from a backup with a tampered checksum is refused; restoring while authenticated forces a fresh login afterward.
