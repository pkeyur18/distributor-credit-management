# Error & Edge-Case Matrix

Organized by major workflow. "Behaviour" is stated as defined wherever a source document settles it; where undefined, it is marked and a recommendation given rather than inventing a silent default.

## Workflow: Add / Edit / Deactivate / Reactivate Member (M1)

| Scenario | Behaviour | Source |
|---|---|---|
| Reference ID does not resolve to an existing member | Rejected at entry with a clear message | Rule-30 |
| Reference ID resolves to an **inactive** member | Rejected — must be existing **and active** | Rule-30 |
| Phone number matches an active member | Rejected — "already in use" | Rule-34 |
| Phone number matches an inactive member | **Not an error** — named reactivation offer, preserving ID/position/history | Rule-34 |
| Attempt to create a second root member | Rejected — root creation is a one-time, first-run-only action | Rule-30 |
| Attempt to move a member beneath its own descendant | Blocked, reason shown | Rule-30 (structurally unreachable given Rule-37, but the guard remains) |
| Attempt to change a member's introducer after creation | **Blocked outright, no override** | Rule-37 |
| Attempt to hard-delete a member | **Not offered anywhere in the UI or API** | Rule-28 |
| Attempt to deactivate the root member | Rejected/disabled — root cannot be deactivated | Confirmed in prototype (`m.id === ROOT_ID` disables the button); no explicit Rule states this, but it is a reasonable derived consequence of "exactly one root, fixed permanently" (Rule-30's structural constraint) |
| Onboarding exceeds configured level-width or depth guidance | **Not an error** — warns, allows | Rule-1, Rule-32 |
| Consent checkbox not ticked | Save disabled — not a submit-then-reject error, the action is simply unavailable | Rule-40 |

## Workflow: Business Volume Entry (M2)

| Scenario | Behaviour | Source |
|---|---|---|
| Amount is zero | Refused | Rule-16a |
| Amount is negative | Refused | Rule-16a |
| Amount has more than 2 decimal places | Refused (field-level validation) | Rule-16 |
| Entry attempted while a reset is outstanding | **Entire screen locked** — no entry form rendered, locked empty state shown naming the outstanding month | Rule-36 |
| Entry date outside the current period's bounds | Refused (date field clamped to period bounds) | Rule-21 |
| Correcting an entry in the **current open** month | Permitted, standard edit | Rule-39 |
| Correcting an entry in a **closed** month | Permitted — writes a new snapshot/backup version, original never overwritten; warning shown before the edit is applied | Rule-39 |
| Concurrent entry conflict (two writes to the same member at once) | **Not applicable** — single-user, single-session system (OC-1); no concurrency-control mechanism is needed or built | `architecture.md` §9.4 |

## Workflow: Monthly Close (M5)

| Scenario | Behaviour | Source |
|---|---|---|
| Backup generation fails | **Abort entirely** — nothing is zeroed, nothing is touched, the outstanding-month alert stays up, admin may retry | Rule-18 |
| Backup generation is cancelled by the admin | Same as failure — abort, no partial state | Rule-18 |
| Admin attempts to close a month that is not the oldest outstanding | Rejected — oldest-first enforced | Rule-20 |
| Multiple months are outstanding | All listed in the alert; each closes separately with its own backup and snapshot; months are never merged | Rule-20 |
| A calendar month elapses with **zero entries** (fully locked out the whole month) | **No snapshot produced**, excluded from the yearly-averaging denominator | RQ-16 (see [03-business-rules.md](03-business-rules.md)) — resolves an item `requirement-spec.md` marks ☐ open |
| External (physically-separate-medium) backup copy fails to write | **Does not block the close** — the internal retained copy is the actual gate; external failure re-prompts/reminds instead | Rule-31, `architecture.md` §15.1 |
| Internal retained backup copy write fails | **Blocks the close** — this is the real gate | Rule-18, Rule-31 |
| Admin loses/never takes the external backup copy at all | **Not defended** — documented single point of failure (internal copy + live DB share the same disk) if this happens; accepted risk, not solved by the system | `architecture.md` §15.3, TR-4 |

## Workflow: Settings / Slab Table (M7)

| Scenario | Behaviour | Source |
|---|---|---|
| Duplicate slab threshold on add/edit | Rejected — duplicate-threshold guard on save | Rule-4 (validated in prototype) |
| Slab percentages configured non-monotonically vs. thresholds | **Not validated, not blocked** — explicitly declined by the client as a residual risk | Rule-41 / ADR-009 |
| Attempt to remove the last remaining slab row | **Undefined in any source document.** Recommendation: reject (a slab table cannot be empty — the implicit 0% base has no meaning without at least one real row to compare against). Needs a UI-layer decision, not a blocker. | Not derivable — flagged, see [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md) LOW item |
| Settings change alters the current open period's figures (e.g. royalty rate) | Recalculates the current open period only; closed months are never affected | Described in prose (RQ-18/V7.6) |
| Settings change is saved without the required warning being shown first | **UI gap** — architecture text requires a warning dialog; the approved prototype saves silently with only a success toast | MEDIUM item, see [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md) |

## Workflow: Authentication (M8)

| Scenario | Behaviour | Source |
|---|---|---|
| Wrong PIN or password entered | Generic "incorrect" message (does not reveal which credential type or which part was wrong); failed-attempt counter increments | Rule-29, mandatory-lockout note |
| 5 consecutive failed attempts | Timed lockout with countdown (exponential backoff); no login possible until it elapses | Rule-29 mandatory-lockout note |
| Recovery code entered is invalid or already used | Refused — recovery codes are single-use | ADR-008 |
| Both credential **and** all recovery codes are lost | **Permanently unrecoverable by design** — no vendor backdoor, no email flow (no network exists) | `architecture.md` §11.5, documented deliberately |
| Session inactivity timeout elapses | Auto-locks; encryption key dropped from memory; re-authentication required to resume, genuinely re-deriving the key (not just a UI overlay) | NFR-4 |
| Device is stolen or accessed while the session is already unlocked | **Not defended** by the application — explicitly the client's own physical-security responsibility | `architecture.md` §11.5 threat table |

## Workflow: Exports / Reports (M6)

| Scenario | Behaviour | Source |
|---|---|---|
| Export requested for a period with no snapshot (empty month) | Excluded from yearly-average calculation and its denominator; not shown as a closed-month option in the snapshot export selector | RQ-16, Rule-23 |
| Export requested for a corrected (multi-version) closed month | Always returns the **latest** version; the original stays in the audit trail, not in the export | Rule-39 |
| Low-contribution report with no members below the threshold | Empty state shown, not an error | Confirmed in prototype |
| Export column selection is empty (no optional columns ticked) | Not an error — the 4 mandatory columns (name, ID, phone, Business Volume) are always included regardless (Rule-19) | Rule-19 |

## Downstream / System-Level Failures

| Scenario | Behaviour | Source |
|---|---|---|
| Database file corruption or unreadable at launch | **Not explicitly addressed in any source document.** Recommendation: detect on launch, present a clear recovery-from-backup path rather than a silent crash — this is standard practice for an offline single-file-DB app, but needs explicit design before M5/M8 hardening is considered complete. | Not derivable — flagged, see [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md) LOW item |
| Application crash mid-transaction (e.g. during chain-upward recalculation or during close) | Protected by SQLite's own transactional guarantees — a crash mid-transaction rolls back cleanly, no partial recalculation or partial close state can persist | `architecture.md` §9.1, "either the whole chain updates consistently or none of it does" |
| Disk full during backup write | Covered by the same write-verify mechanism as any backup failure — verification fails, close aborts (Rule-18) | Rule-18 |
| Timeout / retry | **Not applicable** — no network calls exist anywhere in the system; every operation is local disk I/O with no remote timeout surface | Structural (ADR-001) |

## Summary

Most error and edge-case behaviour across the eight major workflows is explicitly defined in the approved artifacts — a strong signal for readiness. Three items are genuinely undefined and are carried into [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md) as LOW-priority items (empty slab table, corrupted DB file at launch, settings-warning UI gap already tracked as MEDIUM). None of the three block starting implementation; all three should be resolved before the specific module they touch (M7, M8, M7 respectively) is marked done.
