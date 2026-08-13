# 02 — Business Rules & Calculation Model

All 47 business rules (Rule-1 … Rule-46, plus Rule-16a) in corrected form, the calculation model, all six worked scenarios re-derived, and the 16-row settings inventory. This is the heart of the system — modules M1, M2, M3, M5, M6, M7 are built directly from this file.

Every rule below is stated in its **current, corrected** form. Where a rule was corrected or extended after `requirement-spec.md` was written, the correction is marked and the authority cited — see [06](06-decision-log-and-open-items.md) for the full reasoning behind each correction.

---

## 1. Hierarchy model

- A single tree. Exactly **one root member** at Level 1, fixed permanently — can never grow beyond one person.
- Every non-root member has exactly **one parent** (introducer), set at onboarding via a mandatory Reference ID, **fixed forever** (Rule-37).
- Depth is configurable in settings (advisory only — see Rule-32, O2).
- Because transfers are prohibited outright and a Reference ID must already exist, **the hierarchy is a tree by construction** — there is no sequence of permitted operations that can create a loop (Rule-30's note).

---

## 2. The three quantities — read [01](01-product-and-scope.md) §2 first

Business Volume (own, manual entry) → Total Business Volume (rolled up, one level deep, derived) → Slab % (looked up from Total Business Volume) → Rewards (Differential + Royalty + own-Business-Volume reward, a separate ledger).

---

## 3. The 47 rules

### Rule-1 — Level widths are advisory
**Rule:** Level 2/3/4 width defaults (9/6/3) are informational only. Onboarding never rejects a member for exceeding them; the UI may show a soft warning.
**Applies to:** M1.
**Validation:** None blocking — a count-vs-setting comparison only, purely presentational.
**Implementation:** `add_member` must never reject on level-width count.
**Test:** Onboard a 7th child under a Level-2 member with width setting 6 → succeeds, shows warning.

### Rule-2 — Unique 6-digit member ID
**Rule:** Every member gets a unique 6-digit ID at onboarding, the primary lookup key for search, entry and reference linking.
**Applies to:** M1 (creation), M4 (search), M2 (entry).
**Implementation:** `members.id` is the PK, not a surrogate row ID.
**Note:** The ID remains the *primary* lookup key. Since 7 Aug 2026 it is not the only one — phone number is a second, equally unique lookup key (Rule-44), and name remains a non-unique one.
**Test:** Two members can never share an ID; search by ID returns exactly one row.

### Rule-3 — Slab lookup
**Rule:** A member's slab is the **highest** slab whose threshold is **≤** their Total Business Volume. Below the lowest threshold → 0%.
**Applies to:** M3.
**Implementation:** Slab table queried in threshold-descending order; first match wins.
**Test:** TBV exactly at a threshold boundary lands in the higher slab (Scenario 2: C at 3,000 → 8%; Scenario 4: A at 10,000 → 14%).

### Rule-4 — Slab thresholds/percentages configurable
**Rule:** Every threshold and percentage in the slab table is editable in settings. Both client examples — moving 2% to 200, moving 6% to 1,000 — must work.
**Applies to:** M7.
**Validation:** None on cross-row consistency — see Rule-41.
**Error behaviour:** Duplicate-threshold guard on save.
**Implementation:** `update_slab_row` must not assume the current 7-row shape.

### Rule-5 — Bottom-up calculation order
**Rule:** Calculation is a post-order tree traversal — a member's TBV cannot be computed until every direct child's TBV is final. Results propagate to the root.
**Applies to:** M3.
**Implementation:** Chain-upward recalculation (ADR-005, [04](04-technical-architecture.md) §5) is the practical implementation — only the changed member's ancestor chain needs recomputing, since siblings' TBVs are already final.
**Test:** Scenario 3 (multi-depth) reproduces exactly.

### Rule-6 — Total Business Volume formula
**Rule:**
```
TotalBusinessVolume(x) = BusinessVolume(x) + Σ TotalBusinessVolume(c)   for every direct child c of x
```
One level deep only. Full-depth coverage is transitive — each child's figure is itself complete.
**Applies to:** M3.
**Implementation:** A member's own Business Volume term is **never** omitted, even when it must be derived (Scenario 3: A's own BV = 500, never stated directly).
**Test:** Scenario 3 (total 450); Scenario 4/5 (P's own BV = 0, confirmed a write-up simplification, not a different rule).

### Rule-7 — Slab driven by TBV
**Rule:** `slab%(x) = lookup(TotalBusinessVolume(x))`, never by the member's own Business Volume.
**Applies to:** M3.
**Implementation:** A member can show a small own-BV figure while sitting on a high slab — explicit, documented consequence (FR-2's chart-value note).
**Test:** All six scenarios.

### Rule-8 — Differential earnings
**Rule:**
```
Differential(x) = Σ [(slab%(x) − slab%(c)) × TotalBusinessVolume(c)]   for every DIRECT child c of x
```
Base is the child's **Total Business Volume**, not their own Business Volume. Only direct children contribute a term — grandchildren are already inside the child's TBV. A member earns **nothing on their own Business Volume through this term** — see Rule-46 for the separate term that does pay on it.
**Applies to:** M3.
**Implementation:** Grandchildren must never contribute a separate term.
**Test:** Scenario 3 — the only scenario where child TBV ≠ child own-BV, so it disambiguates the base.

### Rule-9 — Differential never negative
**Rule:** Because `TBV(parent) ≥ TBV(child)` by construction (Rule-6), `slab%(parent) ≥ slab%(child)` always holds. No clamping, no negative-earnings case, no error state — a **structural** guarantee, not a check.
**Implementation:** No defensive negative-differential check is needed in normal operation — **except** Rule-41's monotonicity gap can theoretically break this guarantee if the slab table is misconfigured. Do not silently clamp — see [05](05-quality-and-acceptance.md) §2.
**Test:** Confirm no scenario ever produces a negative differential term.

### Rule-10 — Royalty qualification
**Rule:** Let `Q` = direct children of x on the **top slab** (highest-percentage row, whatever its threshold).
```
if |Q| >= royalty_min_children   (default 3, configurable)
    Royalty(x) = Σ royalty_rate × TotalBusinessVolume(c)   for c in Q   (rate default 1%)
else
    Royalty(x) = 0
```
Direct children only, both for counting and for paying.
**Applies to:** M3.
**Test:** Scenario 4 (pure royalty, 1,000); Scenario 5 (differential + royalty together, 980).

### Rule-11 — Royalty and differential never double-pay
**Rule:** If a child is on the top slab, the parent's TBV is at least the child's TBV, so the parent is on the top slab too, so that child's differential term is exactly 0 (Rule-9). Automatically disjoint — **no explicit exclusion logic is needed.**
**Test:** AC-6 — explicit demonstration that royalty and differential recipients don't double-count.

### Rule-12 — Rewards **[AMENDED 8 Aug 2026, CR-4]**
**Rule:** `Rewards(x) = Differential(x) + Royalty(x) + OwnReward(x)` (Rule-46, added 8 Aug 2026 — see §4.1/§5.6). Differential and Royalty are unchanged from the original formulas.

### Rule-13 — Rewards are a separate ledger
**Rule:** Rewards are **never** added to any member's Business Volume. They do not raise the earner's own slab, do not enter any ancestor's TBV, and do not compound into the next period.
**Implementation:** `member_period_totals.rewards`/`royalty`/`own_reward` must never feed back into `business_volume`/`total_business_volume` for any member, including the earner.
**Test:** After computing Rewards for a member, re-run recalculation and confirm their own TBV is unchanged.

### Rule-14 — Unit value (reference only)
**Rule:** 1 unit = 500 Rs, configurable, kept on the settings screen. **Reference only** — never displayed on any screen, report, or export; plays **no part in any calculation.** The client converts final Rewards to rupees by hand, outside the application.
**Implementation:** No screen, export, or calculation may read this setting except the settings screen itself.

### Rule-15 — Business Volume entry flow
**Rule:** Admin searches by name, ID or phone (Rule-44), selects a member, records Business Volume against them.

### Rule-16 — Points-only entry
**Rule:** Admin enters Business Volume directly, nothing else. Up to **two decimal places**. No rupee entry mode, no currency conversion, no rupee field anywhere on this screen.
**Validation:** Numeric, ≥0.01 (see Rule-16a), max 2 decimals.
**Error behaviour:** Reject non-numeric, reject >2 decimal places, Save disabled until valid.

### Rule-16a — Zero and negative Business Volume both refused **[CORRECTED — stricter than the original recommendation]**
**Rule:** Neither a negative figure nor a figure of zero is a permitted Business Volume entry. Both are refused at entry.
**Source:** RQ-17, 3 Aug 2026. Overrides the architect's own original recommendation ("accept zero, refuse negative").
**Applies to:** M2 entry screen, including the closed-month correction panel.
**Validation:** `amount > 0` — DB-level CHECK constraint.
**Implementation:** Do not implement a "zero is a valid no-op entry" path. A member with no activity in a month simply has no entry that month.
**Test:** Attempt to save 0 and −5 → both rejected with a clear message.

### Rule-17 — Manual reset only
**Rule:** Reset is manual only, never automatic. Admin is prompted on the 1st of each month but may act later.

### Rule-18 — Reset flow gated by backup
**Rule:** Backup must be generated and its success confirmed before any figure is zeroed. A failed or cancelled backup **aborts the reset entirely**; the alert stays up; nothing is touched.
**Applies to:** M5.
**Implementation:** Backup write-then-verify (existence + checksum + readability) happens inside the same transactional boundary as the abort decision.

### Rule-19 — Every export carries basic fields
**Rule:** Every exported report includes name, ID, phone number, Business Volume, and Total Business Volume, regardless of which optional columns are selected.
**Resolved 8 Aug 2026** ([06](06-decision-log-and-open-items.md) C9, D-1): five mandatory columns, matching this rule's own wording. Rule-33 is amended below to match.

### Rule-20 — Persistent reset alert
**Rule:** Raised as soon as the month being closed has ended. Appears as **both** an undismissable banner on every screen and a notification-list entry. Clears **only** on successful completion of the reset — no snooze, no dismiss. Multiple outstanding months are all listed; only the oldest can be closed; each closes separately with its own backup and snapshot.
**Source:** A client-added requirement, not present in the original draft.
**Implementation:** The banner component must have literally no dismiss affordance, not even a disguised one.

### Rule-21 — Period boundaries **[PARTIALLY SUPERSEDED]**
**Rule:** A period is a calendar month, 1st to last day. The reset closes whichever month it belongs to, whenever actually pressed.
**Superseded detail:** The original third bullet ("points entered between the 1st and reset count into the month being closed") is struck through and stays struck through. It was originally superseded by Rule-36's hard entry lock, which made the scenario unreachable. Since Rule-36's amendment of 7 Aug 2026 (CR-2) that scenario is reachable again — an entry *can* now be made between the 1st and the reset — but the bullet's rule is still wrong and must still not be implemented: an entry counts into **the month its own date falls in**, never into "the month being closed". A figure dated 2 August is an August figure even if July is still awaiting close; it is simply refused until July closes, rather than being silently absorbed into July. Retained in the source spec for historical record.

### Rule-22 — Precision
**Rule:** Business Volume and Rewards carry **two decimal places** throughout storage and calculation. Rounding happens **only at the point of display** — never at an intermediate step.
**Implementation:** Fixed-point integer storage (×100, ADR-004) is the correct implementation of "no intermediate rounding," not literal decimal rounding at each step.
**Test:** Sum a column of stored ×100 integers and confirm it matches a hand calculator to the cent, with no per-row rounding applied first.

### Rule-23 — Yearly average method
**Rule:** Sum the member's figures across periods that **actually have a snapshot**, divide by the **count of those periods** — not a fixed 12. The report displays that count next to each average.
**Implementation:** Directly depends on the empty-month rule (RQ-16) — no snapshot → excluded from both the sum and the denominator.

### Rule-24 — Low-threshold report metric
**Rule:** Filters on the yearly average of the member's **own** Business Volume, not Total Business Volume. Default threshold 100, configurable.
**Source:** Client's answer differs from the original recommendation of Total Business Volume; deliberately re-confirmed. The yearly-average export still carries **both** figures — only the filter metric is own-BV.

### Rule-25 — Royalty stacks at every level
**Rule:** Each member is assessed **independently** against their own direct children. The same underlying volume can therefore attract royalty at several levels of the same chain.
**Test:** The worked illustration below (§4.5) — A/B/C at 10,000 under P, P/Q/R identical siblings under T; the same 10,000 attracts royalty twice in one chain.

### Rule-26 — Recalculation trigger
**Rule:** The system recalculates **immediately on every Business Volume entry**. Every affected member's TBV, slab, and Rewards are correct on screen the instant an entry is saved. **There is no manual "recalculate" button and no batch-only mode.** Implementation updates only the affected chain upward, not the whole tree.
**Scale:** 500–5,000 members expected; ~1,000 entries/month, explicitly variable ([06](06-decision-log-and-open-items.md) C7).
**Implementation:** ADR-005 — chain-upward incremental recalculation inside one DB transaction per write, re-scanning **all** direct children of each ancestor (not just the changed leaf) — see [04](04-technical-architecture.md) §5.2.

### Rule-27 — Slab rows addable/removable
**Rule:** Admin may add and remove slab rows, not merely re-threshold the existing seven. The slab table can grow to eight rows or shrink to five (never zero — see Rule-41's neighbour, the last-row refusal in [05](05-quality-and-acceptance.md)). The top slab is always recomputed as the highest-percentage row, so the royalty trigger stays correct automatically.

### Rule-28 — Member lifecycle **[CORRECTED — see [06](06-decision-log-and-open-items.md) C5, the highest-risk conflict in this project]**
**Rule:** Edit is permitted at any time. Removal marks a member **inactive** — **inactive status has zero effect on any calculation; it is a pure display flag.** Members are never hard-deleted; history stays intact.
**Source:** `requirement-spec.md` originally stated inactive members "stop appearing in new periods" — overridden by V3.5/RQ-2, 4 Aug 2026.
**Applies to:** M1, M3 (must **not** special-case inactive members in the calculation path), M4 (chart/list display).
**Implementation:** Do **not** filter or zero out an inactive member's Business Volume/TBV contribution anywhere in M3. This is the single most consequential correction in this document set — implementing the original spec wording would silently corrupt every ancestor's TBV/Rewards the moment any member is deactivated.
**Test:** Deactivate a mid-tree member with active children beneath; confirm the deactivated member's Business Volume still rolls up to the root unchanged, and their own TBV/slab/Rewards are still computed normally.

### Rule-29 — Access control **[CORRECTED/EXTENDED]**
**Rule:** One administrator account, used solely by the client. No other accounts, no roles. Members never log in. Protected by a 6-digit PIN **and/or** a complex password — **both may be configured simultaneously; either credential authenticates.** Failed-attempt lockout is mandatory regardless of which credential type is used.
**Source:** M8.5, 4 Aug 2026, resolves what `requirement-spec.md` framed as a pending either/or choice.
**Applies to:** M8.
**Validation:** PIN: exactly 6 numeric digits. Password: ≥8 chars, letter + number.
**Error behaviour:** Wrong credential → generic "incorrect" message (no hint which part was wrong); after 5 failed attempts, timed lockout. **The exact lockout ladder beyond the first threshold is an open item — see [06](06-decision-log-and-open-items.md) O4.**
**Implementation:** `auth` table stores `pin_hash` and `password_hash` as independently-nullable columns; login accepts either match.
**Test:** Set both a PIN and a password; confirm login succeeds with either.

### Rule-30 — Reference and hierarchy integrity
**Rule:** Reference ID must resolve to an existing, **active** member — rejected otherwise. The single root member is created **once**, at initial setup, with no Reference ID, and never again. Any move placing a member beneath their own descendant is blocked.
**Implementation:** The loop-check is a belt-and-braces guard only — Rule-37 (no transfers, ever) makes the hierarchy a tree by construction, so this check can never actually fire in normal use. Keep it anyway — do not remove provably-unreachable defensive code.

### Rule-31 — Backup storage and retention **[CORRECTED/EXTENDED]**
**Rule:** Each backup is downloaded to the administrator's computer **and** retained permanently inside the system, where any past month can be re-downloaded at any time. Nothing is auto-deleted. The downloaded copy must additionally be written to a **physically separate medium** (not merely a different folder on the same disk).
**Source:** Extended by RQ-19, 4 Aug 2026.
**Implementation:** The internal retained copy remains the actual close-gate (write-verified before proceeding); the external/separate-medium copy is prompted at the same time but its failure does **not** block the close — it re-prompts/reminds instead. This is an accepted, unenforced process-discipline risk (TR-4) — do not silently "solve" it by forcing the close to block on it, which would contradict the documented decision.

### Rule-32 — Depth overflow
**Rule:** If onboarding would exceed the configured maximum depth, the system **warns but allows**. Consistent with Rule-1's advisory-only pattern.

### Rule-33 — Configurable export columns
**Rule:** Every field is offered as an export column. The five mandatory columns (Rule-19: name, ID, phone, Business Volume, Total Business Volume) are pre-ticked and untickable. Full optional list: email, address, reference number, introducer name, hierarchy level, direct legs count, slab %, Rewards, royalty earned, joining date, active/inactive status.
**Amended 8 Aug 2026** ([06](06-decision-log-and-open-items.md) C9, D-1): Total Business Volume moves from this optional list to Rule-19's mandatory set.

### Rule-34 — Phone number uniqueness
**Rule:** A phone number identifies exactly one member, **unique across active and inactive members alike**. A match on an inactive member offers **reactivation** instead of erroring — preserving the original 6-digit ID, hierarchy position, and full history. A duplicate record is never created.
**Note:** This uniqueness is what makes Rule-44 possible — because a phone number resolves to exactly one member, it is a safe lookup key, not merely a duplicate-entry guard.

### Rule-35 — Member ID allocation **[CORRECTED — see [06](06-decision-log-and-open-items.md) C4]**
**Rule:** Each member receives a randomly-chosen, currently-available 6-digit number in the range **100001–999999**. Allocation is random, never sequential. IDs are **never released** once assigned — a deactivated member's number stays permanently taken, which is what makes reactivation possible.
**Source:** `requirement-spec.md` originally stated 100000–999999; confirmed 4 Aug 2026 that the usable range starts at 100001.
**Implementation:** ID-allocation logic must exclude 100000 from the candidate pool.

### Rule-36 — Reset enforcement **[AMENDED 7 Aug 2026 — CR-2; the lock is narrowed, not removed]**
**Rule:** Business Volume may be recorded into **any month that has ended but has not yet been closed**. The current, still-running month accepts entries **only when no earlier month is outstanding**. Recording into an already-closed month remains available solely through the correction path (Rule-39).

The entry screen always names the month it is recording into. An attempt to record into the current month while an earlier month is outstanding is **refused, naming the month that must be closed first**.

| Target month's state | Entry accepted? | Path |
|---|---|---|
| Ended, awaiting close | ✅ Yes | Business Volume Entry screen |
| Current month, **no** earlier month outstanding | ✅ Yes | Business Volume Entry screen |
| Current month, an earlier month **is** outstanding | ❌ Refused, naming the blocking month | — |
| Already closed | ❌ Not on the entry screen | Correction panel (Rule-39) |
| Future-dated | ❌ Refused | — |

**Superseded wording (3 Aug 2026 – 7 Aug 2026), kept for the record:** *"Once a calendar month ends, all entry of Business Volume is locked until that month's reset completes. No entry of any kind is accepted while a reset is outstanding."*
**Source:** Client change request **CR-2**, 7 Aug 2026. A member who purchases on the last day of a month commonly reports it two to three days later; under the previous wording that figure could not be recorded at all. The client's stated condition is reproduced exactly above — the previous month stays open for entry until it is closed, and the current month unlocks only once it is.
**Rationale — why this is safe:** live figures belong to a period. An entry dated inside the ended-but-unclosed month is already an entry against **that** period's live figures, so it contaminates nothing. What must stay blocked is a **current-month** entry, whose figures would mix into a period that has not yet been snapshotted and zeroed (Rule-38). The lock therefore narrows from *"no entry at all"* to *"no entry into a month that cannot yet hold figures."*
**Applies to:** M2 (entry eligibility), M5 (the close that releases the current month).
**Validation:** V2.3, V2.6, V2.7.
**Implementation:** `record_entry` derives the target period from `entry_date` and refuses when that period is `closed`, or when it is the current month and any earlier period is still `awaiting_close`. The period status `awaiting_close` (formerly `ended_locked`) is deliberately named for what it now is — ended, still accepting entries, waiting to be closed.
**Unchanged by this amendment:** Rule-20's alert and banner remain undismissable; Rule-21 (one period row per calendar month) still holds; Rule-39 remains the only route into a closed month; Rule-18/38's close sequence is untouched.
**Consequence for Rule-21:** Rule-21's struck-through third bullet is no longer made unreachable by a total lock — but it stays struck through, because a month still cannot be *closed* out of order, and a period row is still created per calendar month.
**Test:** With June outstanding and today in August — a June-dated entry saves and recalculates June's chain; an August-dated entry is refused naming June. After June closes, an August-dated entry saves.

### Rule-37 — Transfers prohibited
**Rule:** A member's sponsor/introducer is fixed at creation and can **never** change. No override exists.
**Source:** Reverses Rule-28's originally-planned "move with frozen months" provision.
**Implementation:** Combined with Rule-30, makes the hierarchy a tree by construction — cycles are structurally impossible under any sequence of permitted operations.

### Rule-38 — Reset scope **["immutable" is qualified — see Rule-39]**
**Rule:** The reset zeroes **everything** — Business Volume, TBV, Rewards, royalty all go to 0. Before anything is cleared, an immutable snapshot of the closing period is written per member: Business Volume, TBV, slab %, Rewards, royalty earned, active/inactive status. **All yearly reporting is built exclusively from snapshots, never from live values.**
**Qualification:** "Immutable" means the snapshot is never *silently* altered — it does **not** mean a closed month can never be corrected. Rule-39 is the actual, later mechanism for correcting a closed month: a new snapshot **version** is written; version 1 is never touched.

### Rule-39 — Entries editable/reversible at any time, including closed months **[NEW/EXTENDS Rule-38]**
**Rule:** An entry can be edited at any time, in any month, open or closed. Editing a closed-month entry rewrites that month's permanent record by writing a **new, versioned snapshot** — the original backup/snapshot version is **never overwritten**. Reporting always reads the latest version. A UI warning is shown before a closed-month edit ("Editing a record recalculates the affected chain and writes a new snapshot version — the original record is never overwritten").
**Source:** RQ-7, 4 Aug 2026 — a deliberate reversal of the "permanent, uncorrectable once closed" framing, which the client's own validation document states was the architect's gloss on Rule-38, not the client's actual requirement.
**Applies to:** M2, M5.
**Validation:** Same as Rule-16/16a for the corrected value; date must fall within the target month's bounds.
**Implementation:** `edit_entry` is the **single** mechanism for both live and closed-month corrections — `reverse_entry` is dropped as a separate command, confirmed dead. Every correction writes an `audit_log` row and a new `monthly_snapshots`/`backups` version.
**Test:** Edit an entry in a closed month; confirm a new snapshot version is created, the original version's data is byte-identical to before, and the yearly-average export reads the new version.

### Rule-40 — Consent capture **[NEW]**
**Rule:** Add Member requires a mandatory consent checkbox; the date is auto-captured. Save is refused until it is ticked.
**Source:** RQ-22, 4 Aug 2026 — DPDP Act 2023-driven, not present in the original draft.
**Applies to:** M1.
**Implementation:** `members.consent_given` (boolean) and `members.consent_date` (auto-set on save) are required columns.

### Rule-41 — Slab-table monotonicity is not validated **[ACCEPTED RISK, NOT A RULE TO BUILD]**
**Statement:** The system does **not** check that slab percentages rise monotonically with thresholds. The client explicitly declined this safeguard, accepting the residual risk that a misconfigured table could produce a negative differential (violating Rule-9) or unexpected Rewards.
**Implementation:** **Do not add this validation speculatively** — it was considered and explicitly rejected. If a future negative-differential bug appears, check the slab table configuration first; it may be this accepted risk, not a code defect. The Settings UI carries an on-screen disclaimer instead of a code-level guard. **See [06](06-decision-log-and-open-items.md) §6 — do not re-raise.**

### Rule-42 — Members are never removed; all data persists **[NEW — client requirement]**
**Rule:** No member is ever removed from the application, under any circumstance. All member data persists permanently — on screen, in calculations, and **in every export**. Deactivation (Rule-28) is the only lifecycle change available, and it is display-only.
**Implementation:** No delete path, no "erasure requested" flag, no export filter that would omit a member. Do not propose one. **See [06](06-decision-log-and-open-items.md) C8, §6 — do not re-raise.**
**Test:** Every export includes deactivated members; no code path removes a `members` row.

### Rule-43 — Whole-console backup schedule and cross-device restore **[NEW — client requirement, 7 Aug 2026]**
**Rule:** The entire console — every member, entry, monthly record and setting, not one month — can be backed up on a configurable schedule (off/daily/weekly/monthly) or on demand, with the most recent backups retained (default 10, client-adjustable) and older ones pruned automatically. The backup is a verified copy of the whole encrypted database file, credentials included. Restorable on any machine, including a brand-new install, bringing the console to exactly the state the backup holds. Restoring always names what it will replace, requires deliberate confirmation, and the console backs up its own current state immediately before overwriting it.
**Applies to:** M7 (schedule/retention setting), M8 (taking and restoring a backup).
**Validation:** Schedule value must be one of `off`/`daily`/`weekly`/`monthly`; retention count ≥ 1. A restore is refused if the target file's checksum does not verify.
**Implementation:** Generalizes the existing `backups` table (ADR-012) rather than adding a second one. Full detail: [04](04-technical-architecture.md) §9.

### Rule-44 — Phone number is a search key **[NEW — client requirement, 7 Aug 2026, CR-1]**
**Rule:** Every member search accepts a **phone number** as well as a name and a 6-digit member ID. Because a phone number is unique across active and inactive members alike (Rule-34), it resolves to exactly one member — the fastest unambiguous handle the administrator already holds when a member telephones or walks in.

Matching rules, applied together — a member matches when **any** clause holds:

1. the query, case-insensitive, is a substring of the member's **name**; or
2. the query's digits are a substring of the member's **6-digit ID**; or
3. the query's digits are a substring of the member's **phone number's digits**, and the query contains **at least 4 digits**.

**Number normalisation:** before comparison, both the query and the stored phone number are reduced to a single canonical key — non-digits are stripped, and then an **international prefix or a trunk zero is dropped** by keeping the last 10 digits of anything longer than 10 (and stripping leading zeros from anything shorter). So `+91 98765 43210`, `098765 43210` and `9876543210` all produce the same key and find the same member, **in either direction** — a number stored plainly is still found when the administrator types it with a country code, and vice versa. Stripping non-digits alone is **not sufficient** and will fail that case.

**The stored value is never rewritten.** Normalisation happens only at the moment of comparison; the member's phone is displayed exactly as it was entered.

**The same key governs Rule-34's uniqueness.** A number is the same number however it was written down, so a duplicate check must compare canonical keys too — otherwise the same person could be added twice under two spellings of one number.
**The four-digit floor** prevents a two- or three-digit query from sweeping in every member whose phone happens to contain those digits. Below four digits, only name and ID are matched — exactly the behaviour that existed before this rule.
**Applies to:** M4 (home and structure search), M2 (entry and correction search), M1 (Add Member reference lookup, which additionally filters to active members per Rule-30).
**Scope:** One shared search behaviour serves every search box in the console. Search behaviour must never differ between screens.
**Display:** Search results show the phone number as a column alongside name and member ID, so the administrator can confirm they have the right person before selecting. Phone is personal data under the DPDP Act 2023; it is visible only to the single administrator role, which already sees it on Member Detail and in exports (Rule-33).
**Validation:** V4.4.
**Test:** A member whose stored phone is `+91 98765 43210` is found by `9876543210`, by `98765 43210`, by `+919876543210`, by `09876543210`, and by the fragment `4321`. A member stored as plain `9876543210` is equally found by `+91 98765 43210` — the normalisation must work in both directions. A query of `987` matches them by phone **not at all**, and matches by name or ID only if those contain `987`.

### Rule-45 — Full hierarchy view is a point-in-time draw **[NEW — client requirement, 7 Aug 2026, CR-3]**
**Rule:** The console offers a **full hierarchy view** — the entire structure, every branch expanded at once, rooted always at the top member — drawn in a **separate window**. It renders once, at the moment it is opened, and never updates. It carries the date and time it was drawn, so a printed copy is self-dating. It is **read-only**: nothing can be recorded, edited or navigated from it. Closing it discards it entirely.
**Source:** Client change request **CR-3**, 7 Aug 2026. The main Structure screen deliberately opens one branch at a time (FR-2, UN-16); that is the right default for daily use but cannot show the shape of the whole network. The client's binding constraint on this request was that **the main console must not slow down** — hence a separate window that draws and then forgets.
**Applies to:** M4.
**Binding constraint inherited from FR-2:** each node shows exactly **name, ID and own Business Volume** — never Total Business Volume. The full view relaxes *how much of the tree* is shown; it relaxes nothing about *what a node shows*.
**Size gate:** opening is gated behind an explicit confirmation naming the exact member count whenever the structure exceeds **60 descendants**. This is the >60 confirm-before-render gate already carried by the design system; the full hierarchy window is where it now lives.
**Validation:** V4.5.
**Implementation:** A separate top-level window, so the main console's DOM, layout and paint budget are untouched. Node positions come from a single post-order layout pass and the connectors are emitted as one pre-computed path during that pass — never measured back out of the rendered DOM. Read-only means read-only: the window subscribes to nothing and holds no handle on live state.
**Test:** Open the full view on a network of more than 60 members; the count is named before it opens, every branch is expanded, each node shows three fields, and the main console remains responsive throughout. Record an entry in the main console afterwards; the already-open window does not change, and its timestamp still names when it was drawn.

### Rule-46 — Reward on own Business Volume **[NEW — client requirement, 8 Aug 2026, CR-4]**
**Rule:**
```
OwnReward(x) = slab%(x) × BusinessVolume(x)
```
A member's own Business Volume now also earns, at the member's **own** slab (Rule-7 — still looked up from Total Business Volume, unchanged). This is a pure addition: Differential (Rule-8) and Royalty (Rule-10) are untouched, still computed exactly as before, still excluding the member's own Business Volume from their own base.
**Source:** Client change request **CR-4**, 8 Aug 2026. Confirmed by the client, reversing the earlier confirmed position that a member earns nothing on their own Business Volume (3 Aug 2026, [06](06-decision-log-and-open-items.md) §5) — differential and royalty stay exactly as designed, this is a third, additive term.
**Applies to:** M3.
**Structural guarantee:** `OwnReward(x) ≥ 0` always — both factors are non-negative by construction, same as Rule-9's guarantee for the differential term.
**Test:** Scenario 6 (§5.6) — the client's own worked example, own-BV reward = 4, total = 10.

---

## 4. The calculation model

### 4.1 The formulas

```
TotalBusinessVolume(x) = BusinessVolume(x) + Σ TotalBusinessVolume(c)      for every DIRECT child c
slab%(x) = lookup(TotalBusinessVolume(x))                                    [highest threshold ≤ TBV]
Differential(x) = Σ [(slab%(x) − slab%(c)) × TotalBusinessVolume(c)]         for every DIRECT child c
Royalty(x) = Σ royalty_rate × TotalBusinessVolume(c)   if ≥ royalty_min_children direct children on top slab, else 0
OwnReward(x) = slab%(x) × BusinessVolume(x)                                  [Rule-46, added 8 Aug 2026, CR-4]
Rewards(x) = Differential(x) + Royalty(x) + OwnReward(x)
```

### 4.2 Structural guarantees

- **Differential is never negative** — parent's TBV always ≥ child's, by construction (Rule-9), *unless* the slab table is misconfigured non-monotonically (Rule-41, an accepted risk).
- **Royalty and differential never double-pay the same leg** (Rule-11) — automatically disjoint, no exclusion logic needed.
- **A member earns nothing on their own Business Volume through the differential term** — only on the gap to their children (Rule-8) — but does earn separately through `OwnReward` (Rule-46), at their own slab, on their own Business Volume.
- **Royalty stacks** — the same underlying volume can earn royalty at multiple levels of the same chain (Rule-25).
- Recalculation is **immediate**, chain-upward only, inside one DB transaction (Rule-26, ADR-005).

### 4.3 Default slab table

| Slab | Threshold | Applies when TBV is | Notes |
|---|---|---|---|
| 0% | — | 0 – 99 | Implicit base slab |
| 2% | 100 | 100 – 399 | |
| 4% | 400 | 400 – 1,199 | |
| 6% | 1,200 | 1,200 – 2,999 | |
| 8% | 3,000 | 3,000 – 4,999 | |
| 10% | 5,000 | 5,000 – 6,999 | |
| 12% | 7,000 | 7,000 – 9,999 | |
| 14% | 10,000 | ≥ 10,000 | **Top slab** — triggers royalty eligibility |

The `>=` boundary rule is proved by the scenarios below: Scenario 2 places C on 8% at exactly 3,000; Scenario 4 places A on 14% at exactly 10,000.

---

## 5. Worked scenarios — the golden regression set

All six re-derived from Rules 6–12/46 alone (not from the client's stated answers) and all six reconcile. **These are the primary acceptance test.** See [00](00-master-index.md) §5. Scenarios 1–5 are the client's original five; Scenario 6 (§5.6) is the client's own worked example for Rule-46, added 8 Aug 2026 (CR-4).

### 5.1 Scenario 1 — basic differential

D has three direct children: A (BV 300), B (BV 50), C (BV 1,000). D's own BV = 500.

`TotalBusinessVolume(D) = 500 + 300 + 50 + 1,000 = 1,850` → **6% slab**.

| Child | Child TBV | Child slab | D slab | Differential % | Rewards |
|---|---|---|---|---|---|
| A | 300 | 2% | 6% | 4% | **12** |
| B | 50 | 0% | 6% | 6% | **3** |
| C | 1,000 | 4% | 6% | 2% | **20** |
| | | | | **Total** | **35** |

Royalty: 0 direct children on the top slab → not eligible.

**Own-Business-Volume reward (Rule-46):** D's own BV = 500, at D's own slab (6%) → `OwnReward(D) = 6% × 500 = 30`.

**Rewards(D) = 35 + 0 + 30 = 65** ✅

### 5.2 Scenario 2 — differential collapses to zero on an equal slab

Identical to Scenario 1 except C's Business Volume is 3,000.

`TotalBusinessVolume(D) = 500 + 300 + 50 + 3,000 = 3,850` → **8% slab**.

| Child | Child TBV | Child slab | D slab | Differential % | Rewards |
|---|---|---|---|---|---|
| A | 300 | 2% | 8% | 6% | **18** |
| B | 50 | 0% | 8% | 8% | **4** |
| C | 3,000 | 8% | 8% | 0% | **0** |
| | | | | **Total** | **22** |

Royalty: C is on 8% — not the top slab. 0 qualifying children → not eligible.

**Own-Business-Volume reward (Rule-46):** D's own BV = 500, at D's own slab (8%) → `OwnReward(D) = 8% × 500 = 40`.

**Rewards(D) = 22 + 0 + 40 = 62** ✅

### 5.3 Scenario 3 — multi-depth rollup

A has six direct children B–G, each with TBV 1,250 (6% slab). D (one of the six) has three further children p1–p3 already folded into D's 1,250.

`TotalBusinessVolume(A) = BusinessVolume(A) + 6 × 1,250 = BusinessVolume(A) + 7,500 = 8,000` → so `BusinessVolume(A) = 500` (derived, never stated directly in the client's examples — confirmed by the client to be intentional). 8,000 → **12% slab**.

| Child | Child TBV | Child slab | A slab | Differential % | Rewards |
|---|---|---|---|---|---|
| B–G (six) | 1,250 each | 6% | 12% | 6% | **75 each** |
| | | | | **Total** | **450** |

**Key point:** p1, p2, p3 contribute **nothing** directly to A's earnings — already absorbed into D's TBV of 1,250, and A earns on D's TBV. This is what makes the differential model self-limiting.

Royalty: no direct child on the top slab → not eligible.

**Own-Business-Volume reward (Rule-46):** A's own BV = 500 (derived, same figure as above), at A's own slab (12%) → `OwnReward(A) = 12% × 500 = 60`.

**Rewards(A) = 450 + 0 + 60 = 510** ✅

### 5.4 Scenario 4 — pure royalty

P has four direct children A, B, C, D — TBV 10,000 / 20,000 / 30,000 / 40,000, all 14% (top slab).

`TotalBusinessVolume(P) = 10,000+20,000+30,000+40,000 = 100,000` → **14%** (top slab). BusinessVolume(P) = 0 — a write-up simplification the client confirmed, not a rule exception; own BV is always counted, it was simply zero here.

**Differential:** every child is on 14%, P is on 14% → all four terms are 0. Total **0**.

**Royalty:** 4 direct children on the top slab ≥ 3 → eligible.

| Child | Child TBV | Royalty @ 1% |
|---|---|---|
| A | 10,000 | **100** |
| B | 20,000 | **200** |
| C | 30,000 | **300** |
| D | 40,000 | **400** |
| | **Total** | **1,000** |

**Own-Business-Volume reward (Rule-46):** P's own BV = 0 here (the same write-up simplification noted above) → `OwnReward(P) = 14% × 0 = 0`. This scenario's total is unaffected by Rule-46 for that reason, not because the rule doesn't apply.

**Rewards(P) = 0 + 1,000 + 0 = 1,000** ✅

### 5.5 Scenario 5 — differential and royalty together

P has seven direct children: A/B/C/D at 10,000 each (14%), E at 2,000 (6%), F at 3,000 (8%), G at 4,000 (8%).

`TotalBusinessVolume(P) = 4×10,000 + 2,000+3,000+4,000 = 49,000` → **14%** (top slab). BusinessVolume(P) = 0 for the same reason as Scenario 4.

**Differential:**

| Child | Child TBV | Child slab | P slab | Differential % | Rewards |
|---|---|---|---|---|---|
| A–D | 10,000 each | 14% | 14% | 0% | 0 |
| E | 2,000 | 6% | 14% | 8% | **160** |
| F | 3,000 | 8% | 14% | 6% | **180** |
| G | 4,000 | 8% | 14% | 6% | **240** |
| | | | | **Subtotal** | **580** |

**Royalty:** A, B, C, D on top slab = 4 ≥ 3 → eligible. 1% × 10,000 = 100 each → **400**.

**Own-Business-Volume reward (Rule-46):** P's own BV = 0 here (the same write-up simplification as Scenario 4) → `OwnReward(P) = 14% × 0 = 0`.

**Rewards(P) = 580 + 400 + 0 = 980** ✅

This scenario proves royalty is a *consequence* of qualification, not a "zero differential" precondition — P earns a non-zero differential **and** royalty in the same period.

### 5.6 Scenario 6 — own Business Volume reward (Rule-46) **[NEW — client's own worked example, 8 Aug 2026, CR-4]**

A has three direct children B, C, D, each with BV 100 (own BV, no children of their own — so TBV = own BV = 100, **2% slab**). A's own BV = 100.

`TotalBusinessVolume(A) = 100 + 100 + 100 + 100 = 400` → **4% slab**.

| Child | Child TBV | Child slab | A slab | Differential % | Rewards |
|---|---|---|---|---|---|
| B | 100 | 2% | 4% | 2% | **2** |
| C | 100 | 2% | 4% | 2% | **2** |
| D | 100 | 2% | 4% | 2% | **2** |
| | | | | **Total** | **6** |

Royalty: B/C/D are on 2% — not the top slab. 0 qualifying children → not eligible.

**Own-Business-Volume reward (Rule-46):** A's own BV = 100, at A's own slab (4%) → `OwnReward(A) = 4% × 100 = 4`.

**Rewards(A) = 6 + 0 + 4 = 10** ✅ — matches the client's own hand-worked figure exactly.

### 5.7 Royalty stacking illustration (Rule-25)

A, B, C each hold TBV 10,000 under P, all top slab. `TBV(P) = 30,000`, 3 top-slab children → **P collects 1% × 30,000 = 300**. P has two counterparts, Q and R, identical, under T. `TBV(T) = 90,000`, 3 top-slab children → **T collects 1% × 90,000 = 900**. Total paid across the chain: **1,800**. A's original 10,000 attracted royalty **twice** — once via P, once via T. Confirmed and accepted by the client (Rule-25).

### 5.8 Verification

All six totals reconcile: **65 / 62 / 510 / 1,000 / 980 / 10**, matching the client's own hand-worked figures exactly (scenarios 4 and 5 are unchanged from before Rule-46 — the top member's own BV is 0 in both, a pre-existing write-up simplification, not a sign the rule is unimplemented).

| Scenario | Differential | Royalty | OwnReward | Total | Match |
|---|---|---|---|---|---|
| 1 | 35 | 0 | 30 | 65 | ✅ |
| 2 | 22 | 0 | 40 | 62 | ✅ |
| 3 | 450 | 0 | 60 | 510 | ✅ |
| 4 | 0 | 1,000 | 0 | 1,000 | ✅ |
| 5 | 580 | 400 | 0 | 980 | ✅ |
| 6 | 6 | 0 | 4 | 10 | ✅ |

---

## 6. Settings inventory — 16 rows

**Authoritative count is 16, not 13** — see [06](06-decision-log-and-open-items.md) C1.

| # | Setting | Default | Rule |
|---|---|---|---|
| 1 | Slab thresholds (rows, addable/removable) | 100 / 400 / 1,200 / 3,000 / 5,000 / 7,000 / 10,000 | Rule-4, Rule-27 |
| 2 | Slab percentages | 2 / 4 / 6 / 8 / 10 / 12 / 14 | Rule-4 |
| 3 | Reference unit value (display-only) | 500 | Rule-14 |
| 4 | Hierarchy depth (advisory) | **4** *(no source default — see O2)* | Rule-1, Rule-32 |
| 5 | Level 2 width (advisory) | 9 | Rule-1 |
| 6 | Level 3 width (advisory) | 6 | Rule-1 |
| 7 | Level 4 width (advisory) | 3 | Rule-1 |
| 8 | Royalty qualifying count | 3 | Rule-10 |
| 9 | Royalty rate | 1% | Rule-10 |
| 10 | Yearly cycle start/end | 1 Jan – 31 Dec | Rule-23 |
| 11 | Low-contribution threshold | 100 | Rule-24 |
| 12 | Default export columns | name, ID, phone, Business Volume, Total Business Volume | Rule-33 |
| 13 | Session inactivity timeout | **15 minutes** *(no source default — see O3)* | §11.3 |
| 14 | Whole-console backup schedule | Off | Rule-43 |
| 15 | Whole-console backup retention count | 10 | Rule-43 |
| 16 | Whole-console backup folder | App-data `backups/` subfolder | Rule-43 |

Rows in **bold** carry a default not stated in any source document — see [06](06-decision-log-and-open-items.md) O2/O3 before seeding.

---

## 7. Business-rule → module map

| Rule | Module(s) |
|---|---|
| 1, 2, 30, 32, 34, 35, 37, 40, 42, 44 | M1 |
| 15, 16, 16a, 22, 36, 39, 44 | M2 |
| 3, 5–13, 25, 26, 41 | M3 |
| 2, 6, 7, 12, 44, 45 | M4 |
| 17, 18, 20, 21, 31, 36, 38, 39 | M5 |
| 19, 23, 24, 33, 42 | M6 |
| 4, 14, 27, 43 | M7 |
| 29, 43 | M8 |
