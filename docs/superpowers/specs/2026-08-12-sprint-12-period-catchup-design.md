# Sprint 12 — Period Lifecycle Catch-up & Entry Eligibility

**Stories:** US-M5.5, US-M5.2, US-M5.3, US-M2.3, US-M2.4
**Refs:** Rule-17/20/21/36 (as amended by CR-2), RQ-16, AC-18/19/20/21/42/43
**Branch:** `feature/sprint-12-period-catchup` off `develop`

This spec covers only the implementation decisions not already pinned down by
`PI/01-backlog.md` and `documents/refinement/`. Rule text and acceptance
criteria are not restated here — see those documents for the "what" and
"why"; this covers the "how" and the seams between the five stories, which
is the part the source specification left as a genuine gap (D-9).

## 1. Period catch-up (US-M5.5)

New function `m5_close::run_period_catchup(conn: &Connection) -> Result<(), AppError>`,
run once inside `commands::login` and `commands::setup_first_run`, before
`session.mark_authenticated()`.

Two phases, both idempotent:

1. **Backfill.** Find `MAX(period_month)` across existing `periods` rows (or
   `None` on a fresh install). Insert a row for every calendar month from the
   one after that (or the current month, if none exist yet) up to and
   including the current month — all inserted as `open`.
2. **Elapse.** `UPDATE periods SET status = 'awaiting_close', ended_at = ?
   WHERE status = 'open' AND period_month < current_month`.

   `ended_at` is the **last calendar day of that period's own month**
   (e.g. `2026-06-30` for `2026-06`), not the date the catch-up routine
   happened to run. Rationale (user correction, 12 Aug 2026): while an
   earlier month stays outstanding, the current month cannot accept
   entries either (Rule-36), so "when this period ended" must mean the
   calendar boundary itself, not whenever the admin next happened to open
   the console — those two dates can be arbitrarily far apart and only one
   of them is true.

Second phase naturally no-ops on a re-run (nothing left in `open` with
`period_month < current_month`), which is what T-M5.5-6's idempotency case
tests directly.

## 2. Entry eligibility gate (US-M2.3/US-M2.4/US-M5.3)

New `m5_close::resolve_recording_period(conn, entry_date) -> Result<i64, AppError>`
(returns the resolved `period_id`), replacing `m2_entries::get_or_create_open_period`
inside `record_entry`. `edit_entry` is untouched — Rule-39's correction path
is a separate contract and already handles closed months correctly.

Resolution:
- `target_month = period_month_of_date(entry_date)`
- `target_month > current_month` → `AppError::Validation { field: "entryDate", .. }` (future-dated, same family as the existing date-bound checks).
- Look up the period row for `target_month`:
  - missing, and `target_month == current_month` → create it as `open` (defensive fallback for direct unit-test calls that skip login/catch-up; in real operation this row always already exists).
  - missing, `target_month < current_month` → `AppError::NotFound` (unreachable in real operation; catch-up guarantees the row).
  - `closed` → `AppError::PeriodClosed { month: target_month }`.
  - `awaiting_close` → accepted unconditionally (this is CR-2's whole point).
  - `open` (only ever the current month, by construction) → accepted only if no `awaiting_close` row exists; otherwise `AppError::PeriodNotAcceptingEntries { month: target_month, blocking_month: oldest_outstanding }`.

`get_period_lock_status` (API-07) is a read-only projection of the same
state: outstanding months oldest-first as `recordablePeriodMonths` with
`blockingMonth = Some(oldest)` when any exist; otherwise `[current_month]`
with `blockingMonth = None`.

`get_outstanding_alert` (API-31) is a simpler read: `{ outstandingMonths,
currentMonth }`, already matching the existing frontend `OutstandingAlert`
type exactly.

## 3. Error types

`error.rs` gains:
```rust
PeriodNotAcceptingEntries { month: String, blocking_month: String },
PeriodClosed { month: String },
```
Serialized with `kind: "period_not_accepting_entries" | "period_closed"` and
`month`/`blockingMonth` fields — the frontend (`src/lib/ipc/errors.ts`) has
carried these reserved slots since S7 and needs no changes.

## 4. Frontend wiring

- `use-outstanding-alert.ts` becomes a context (`OutstandingAlertProvider`/`useOutstandingAlert`), mirroring `auth-context.tsx`'s existing shape: `{ alert, refresh }`. `AppShell` mounts the provider around its `Outlet`.
- `MonthlyClose`'s existing local `refresh()` (already invoked from `onClosed`) additionally calls the context's `refresh()` — the one moment AC-20 requires the banner/notification-list to clear.
- `outstanding-month-banner.tsx` — add the "N more month(s) outstanding after that" clause from the prototype (`buildBanner()`), using the already-passed `outstandingMonths` array.
- `business-volume-entry.tsx` — `currentMonthBounds()` replaced by a fetch of `getPeriodLockStatus()`; the active recording month is the oldest recordable one; date bounds derive from that month (capped at today only when it equals the current month); add the recording-month note (mirrors the prototype's `entryMonthNoteHtml`). On `period_not_accepting_entries`/`period_closed`, only the date/save action is blocked — the rest of the form (member search, selection) stays live, per T-M2.4-2. No month-selector control — that's US-M2.5, S13, explicitly out of scope.

## 5. Testing (exit gate)

- Three-months-unopened catch-up creates exactly three `awaiting_close` rows, oldest-first, only the oldest closable, each accepting entries dated within itself; a second consecutive login creates nothing (T-M5.5-6).
- Full TEST-R36 matrix on `record_entry`: outstanding-month entry accepted; current-month entry refused naming the blocker; closed-month entry directed to correction; future-dated refused; after close, the current-month entry saves.
- Period isolation: writing into one live (outstanding or current) period leaves every other period's `member_period_totals` rows byte-identical.
- Banner has no dismissal route of any kind (extends the existing S11-era assertion to cover the multi-month text).
