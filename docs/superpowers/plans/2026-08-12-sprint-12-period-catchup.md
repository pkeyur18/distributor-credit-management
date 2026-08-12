# Sprint 12 — Period Catch-up & Entry Eligibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. (Not subagent-driven — this project's standing rule is inline execution only, no implementation subagents.)

**Goal:** Wire the period lifecycle state machine (US-M5.5) and the Rule-36 (CR-2) entry-eligibility contract (US-M5.2/M5.3/M2.3/M2.4) into real code — today only `m5_close`'s close flow exists; nothing creates a period row, transitions `open → awaiting_close`, or refuses an entry.

**Architecture:** A single catch-up routine (`m5_close::run_period_catchup`) runs once at successful login/setup, before the session flag is set. It backfills missing `periods` rows up to the current calendar month, then elapses every `open` row whose month has passed to `awaiting_close`. `m5_close::resolve_recording_period` reads that state to accept or refuse a `record_entry` call. Two new read commands (`get_period_lock_status`, `get_outstanding_alert`) expose the same state to the frontend, which already has typed IPC wrappers and reserved error-kind slots waiting since S7.

**Tech Stack:** Rust (rusqlite, chrono), React 19 + TypeScript, existing IPC command pattern (`#[tauri::command]` + `invokeCommand`).

## Global Constraints

- ADR-004: every volume/reward figure is a ×100 fixed-point integer — not touched by this sprint, no new figures introduced.
- Rule-36 (as amended by CR-2): outstanding month always accepts entries; current month accepts only when nothing earlier is outstanding; closed month never accepts via `record_entry` (correction panel only); future-dated always refused.
- The retired `PeriodLocked` error variant must never be reintroduced under a new meaning.
- `periods.status` values are exactly `open` / `awaiting_close` / `closed` — never `ended_locked` (already enforced by a DB `CHECK` constraint and a migration test).
- Rule-20: the outstanding-month banner/notification-list have **no dismissal route of any kind** — this sprint only adds data to what already renders correctly.
- No new Tauri commands — `get_period_lock_status` (API-07) and `get_outstanding_alert` (API-31) are already registered in `lib.rs`/`command_names.rs` as stubs; this sprint only replaces their bodies.
- `T-M5.5-4`: the catch-up routine runs **before** `session.mark_authenticated()` — the frontend must never observe a stale/pre-catch-up state on the first frame.
- All new user-facing copy must pass `npm run vocab-grep`.
- UI must match `documents/design/ui-prototype-v2.html` (`entryMonthNoteHtml`, `buildBanner`), reduced only where noted (no month-selector — that's US-M2.5, S13).

---

## File Map

| File | Change |
|---|---|
| `src-tauri/src/error.rs` | Modify — add `PeriodNotAcceptingEntries`/`PeriodClosed` variants |
| `src-tauri/src/m5_close/mod.rs` | Modify — add catch-up, lock-status, alert, eligibility-resolution logic |
| `src-tauri/src/m2_entries/mod.rs` | Modify — `period_month_of_date` becomes `pub(crate)`; `record_entry` gated; `get_or_create_open_period` removed |
| `src-tauri/src/commands.rs` | Modify — real `get_period_lock_status`/`get_outstanding_alert`; catch-up wired into `login`/`setup_first_run` |
| `src/lib/outstanding-alert-context.tsx` | Create — replaces `src/lib/use-outstanding-alert.ts` |
| `src/lib/utils.ts` | Modify — add shared `monthLabel` |
| `src/components/app-shell.tsx` | Modify — wraps content in the new provider |
| `src/screens/monthly-close.tsx` | Modify — use shared `monthLabel`; refresh the alert context after a close |
| `src/components/outstanding-month-banner.tsx` | Modify — "N more months" clause |
| `src/screens/business-volume-entry.tsx` | Modify — real recording-month bounds, note, refusal handling |

---

### Task 1: `AppError` gains the two Rule-36 error variants

**Files:**
- Modify: `src-tauri/src/error.rs`

**Interfaces:**
- Produces: `AppError::PeriodNotAcceptingEntries { month: String, blocking_month: String }`, `AppError::PeriodClosed { month: String }` — consumed by Task 5 (`m5_close::resolve_recording_period`).

- [ ] **Step 1: Add the two variants**

In `src-tauri/src/error.rs`, add to the `AppError` enum (after `AccountLocked`):

```rust
    /// Rule-36 (amended by CR-2): a current-month entry while an earlier
    /// month is still `awaiting_close`. `blocking_month` is always the
    /// oldest outstanding month — the one that must close first.
    #[error("{month} isn't open for entry until {blocking_month} is closed")]
    PeriodNotAcceptingEntries {
        month: String,
        blocking_month: String,
    },
    /// A fresh entry against an already-`closed` period — not offered via
    /// `record_entry`, only via `edit_entry`'s correction path (Rule-39).
    #[error("{month} is closed — use the correction panel instead")]
    PeriodClosed { month: String },
```

- [ ] **Step 2: Extend the `Serialize` impl**

In the same file, update the `kind` match to add:

```rust
            AppError::PeriodNotAcceptingEntries { .. } => "period_not_accepting_entries",
            AppError::PeriodClosed { .. } => "period_closed",
```

Change `serializer.serialize_struct("AppError", 5)?` to `serializer.serialize_struct("AppError", 7)?`, and after the existing `attemptsRemaining` block, add:

```rust
        if let AppError::PeriodNotAcceptingEntries { month, .. }
        | AppError::PeriodClosed { month } = self
        {
            state.serialize_field("month", month)?;
        } else {
            state.serialize_field("month", &None::<String>)?;
        }
        if let AppError::PeriodNotAcceptingEntries { blocking_month, .. } = self {
            state.serialize_field("blockingMonth", blocking_month)?;
        } else {
            state.serialize_field("blockingMonth", &None::<String>)?;
        }
```

- [ ] **Step 3: Compile check**

Run: `cd src-tauri && cargo build 2>&1 | tail -40`
Expected: builds clean (nothing references the new variants yet, so no other errors).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/error.rs
git commit -m "feat(S12): add PeriodNotAcceptingEntries/PeriodClosed error variants"
```

---

### Task 2: Period catch-up routine (US-M5.5)

**Files:**
- Modify: `src-tauri/src/m5_close/mod.rs`

**Interfaces:**
- Produces: `pub fn run_period_catchup(conn: &Connection) -> Result<(), AppError>` — consumed by Task 8 (`commands::login`/`setup_first_run`).
- Produces (private helpers reused by Task 3/4): `fn current_calendar_month() -> String`, `fn next_month(period_month: &str) -> String`, `fn last_day_of_month(period_month: &str) -> String`, `fn outstanding_period_months(conn: &Connection) -> Result<Vec<String>, AppError>`.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module at the bottom of `src-tauri/src/m5_close/mod.rs`:

```rust
    fn ym_offset(months: i64) -> String {
        let today = chrono::Local::now().date_naive();
        let shifted = if months >= 0 {
            today.checked_add_months(chrono::Months::new(months as u32))
        } else {
            today.checked_sub_months(chrono::Months::new((-months) as u32))
        };
        shifted.unwrap().format("%Y-%m").to_string()
    }

    #[test]
    fn run_period_catchup_creates_current_month_on_a_fresh_install() {
        let conn = seeded();
        run_period_catchup(&conn).unwrap();

        let current = ym_offset(0);
        let status: String = conn
            .query_row(
                "SELECT status FROM periods WHERE period_month = ?1",
                [&current],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "open");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn run_period_catchup_backfills_and_elapses_across_three_unopened_months() {
        // Last login was three calendar months ago: only that month's row
        // exists (as `open`, since it was current then). Today, three
        // month boundaries have passed.
        let conn = seeded();
        insert_period(&conn, &ym_offset(-3), "open");

        run_period_catchup(&conn).unwrap();

        let elapsed = [ym_offset(-3), ym_offset(-2), ym_offset(-1)];
        for month in &elapsed {
            let (status, ended_at): (String, Option<String>) = conn
                .query_row(
                    "SELECT status, ended_at FROM periods WHERE period_month = ?1",
                    [month],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(status, "awaiting_close", "{month} must have elapsed");
            let expected_ended_at = last_day_of_month(month);
            assert_eq!(
                ended_at,
                Some(expected_ended_at),
                "{month}'s ended_at must be that month's own last calendar day, not the run date"
            );
        }

        let current = ym_offset(0);
        let current_status: String = conn
            .query_row(
                "SELECT status FROM periods WHERE period_month = ?1",
                [&current],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(current_status, "open");

        let outstanding = get_outstanding_periods(&conn).unwrap();
        assert_eq!(
            outstanding
                .iter()
                .map(|p| p.period_month.clone())
                .collect::<Vec<_>>(),
            elapsed,
            "queued oldest-first"
        );

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 4, "the three elapsed months plus the current one");
    }

    #[test]
    fn run_period_catchup_is_idempotent_on_a_second_login() {
        let conn = seeded();
        insert_period(&conn, &ym_offset(-3), "open");
        run_period_catchup(&conn).unwrap();
        let after_first: i64 = conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();

        run_period_catchup(&conn).unwrap();
        let after_second: i64 = conn
            .query_row("SELECT COUNT(*) FROM periods", [], |r| r.get(0))
            .unwrap();

        assert_eq!(after_first, after_second, "a second login creates nothing");
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd src-tauri && cargo test m5_close::tests::run_period_catchup 2>&1 | tail -30`
Expected: FAIL — `run_period_catchup` not found (doesn't exist yet).

- [ ] **Step 3: Implement**

Add to `src-tauri/src/m5_close/mod.rs`, above the `#[cfg(test)]` module:

```rust
fn current_calendar_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

fn next_month(period_month: &str) -> String {
    let first = chrono::NaiveDate::parse_from_str(&format!("{period_month}-01"), "%Y-%m-%d")
        .expect("period_month is always a valid YYYY-MM value");
    first
        .checked_add_months(chrono::Months::new(1))
        .expect("chrono month arithmetic never overflows in practice")
        .format("%Y-%m")
        .to_string()
}

fn last_day_of_month(period_month: &str) -> String {
    let first = chrono::NaiveDate::parse_from_str(&format!("{period_month}-01"), "%Y-%m-%d")
        .expect("period_month is always a valid YYYY-MM value");
    first
        .checked_add_months(chrono::Months::new(1))
        .and_then(|next| next.pred_opt())
        .expect("every month has a last day")
        .to_string()
}

/// Rule-20's queue, as bare month strings — `get_outstanding_periods`
/// already owns the query; this just projects `period_month` out of it
/// rather than re-deriving the same `WHERE status = 'awaiting_close'`.
fn outstanding_period_months(conn: &Connection) -> Result<Vec<String>, AppError> {
    Ok(get_outstanding_periods(conn)?
        .into_iter()
        .map(|p| p.period_month)
        .collect())
}

/// US-M5.5 (D-9/D-10) — run once at successful login/setup, before the UI
/// takes over (T-M5.5-4). Two idempotent phases:
///   1. Backfill a row for every calendar month with none yet, up to and
///      including the current one, all inserted `open`.
///   2. Elapse every `open` row whose month has passed to `awaiting_close`,
///      `ended_at` set to that month's own last calendar day (not the run
///      date — the current month can't unlock while an earlier one stays
///      outstanding, so "when this period ended" must mean the calendar
///      boundary itself).
pub fn run_period_catchup(conn: &Connection) -> Result<(), AppError> {
    let current_month = current_calendar_month();

    let latest_existing: Option<String> =
        conn.query_row("SELECT MAX(period_month) FROM periods", [], |r| r.get(0))?;

    let mut month = match &latest_existing {
        Some(latest) if latest >= &current_month => None,
        Some(latest) => Some(next_month(latest)),
        None => Some(current_month.clone()),
    };
    while let Some(m) = month {
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
            [&m],
        )?;
        if m == current_month {
            break;
        }
        month = Some(next_month(&m));
    }

    let mut stmt = conn.prepare(
        "SELECT id, period_month FROM periods WHERE status = 'open' AND period_month < ?1",
    )?;
    let elapsed: Vec<(i64, String)> = stmt
        .query_map([&current_month], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    drop(stmt);
    for (id, period_month) in elapsed {
        let ended_at = last_day_of_month(&period_month);
        conn.execute(
            "UPDATE periods SET status = 'awaiting_close', ended_at = ?2 WHERE id = ?1",
            rusqlite::params![id, ended_at],
        )?;
    }
    Ok(())
}
```

Also add this test helper next to the existing `insert_period` helper in the `tests` module (it already exists per the S11 tests — reuse it, do not redefine).

- [ ] **Step 4: Run to verify it passes**

Run: `cd src-tauri && cargo test m5_close::tests:: 2>&1 | tail -50`
Expected: all `m5_close` tests, including the three new ones, PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/m5_close/mod.rs
git commit -m "feat(S12): period lifecycle catch-up routine (US-M5.5)"
```

---

### Task 3: `get_period_lock_status` and `get_outstanding_alert` (US-M5.2/M5.3)

**Files:**
- Modify: `src-tauri/src/m5_close/mod.rs`

**Interfaces:**
- Consumes: `outstanding_period_months`, `current_calendar_month` (Task 2).
- Produces: `pub struct PeriodLockStatus { recordable_period_months: Vec<String>, blocking_month: Option<String> }`, `pub fn get_period_lock_status(conn: &Connection) -> Result<PeriodLockStatus, AppError>`, `pub struct OutstandingAlert { outstanding_months: Vec<String>, current_month: String }`, `pub fn get_outstanding_alert(conn: &Connection) -> Result<OutstandingAlert, AppError>` — both consumed by Task 8 (`commands.rs`).

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module:

```rust
    #[test]
    fn get_period_lock_status_reports_only_the_current_month_when_nothing_outstanding() {
        let conn = seeded();
        let status = get_period_lock_status(&conn).unwrap();
        assert_eq!(status.recordable_period_months, vec![ym_offset(0)]);
        assert_eq!(status.blocking_month, None);
    }

    #[test]
    fn get_period_lock_status_lists_outstanding_months_and_names_the_blocker() {
        let conn = seeded();
        insert_period(&conn, &ym_offset(-2), "awaiting_close");
        insert_period(&conn, &ym_offset(-1), "awaiting_close");
        insert_period(&conn, &ym_offset(0), "open");

        let status = get_period_lock_status(&conn).unwrap();
        assert_eq!(
            status.recordable_period_months,
            vec![ym_offset(-2), ym_offset(-1)]
        );
        assert_eq!(status.blocking_month, Some(ym_offset(-2)));
    }

    #[test]
    fn get_outstanding_alert_reports_outstanding_and_current_months() {
        let conn = seeded();
        insert_period(&conn, &ym_offset(-1), "awaiting_close");
        insert_period(&conn, &ym_offset(0), "open");

        let alert = get_outstanding_alert(&conn).unwrap();
        assert_eq!(alert.outstanding_months, vec![ym_offset(-1)]);
        assert_eq!(alert.current_month, ym_offset(0));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd src-tauri && cargo test m5_close::tests::get_period_lock_status 2>&1 | tail -20`
Expected: FAIL — types/functions don't exist.

- [ ] **Step 3: Implement**

Add to `src-tauri/src/m5_close/mod.rs`, near the other public structs:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodLockStatus {
    /// Oldest-first. Never a plain boolean (CR-2) — the entry screen needs
    /// to know *which* months it may record into, not just whether it can.
    pub recordable_period_months: Vec<String>,
    /// The month that must close before the current month unlocks — set
    /// only when the current month itself is not recordable.
    pub blocking_month: Option<String>,
}

/// API-07 (US-M2.3/M5.3) — Rule-36 as amended by CR-2.
pub fn get_period_lock_status(conn: &Connection) -> Result<PeriodLockStatus, AppError> {
    let outstanding = outstanding_period_months(conn)?;
    if outstanding.is_empty() {
        Ok(PeriodLockStatus {
            recordable_period_months: vec![current_calendar_month()],
            blocking_month: None,
        })
    } else {
        let blocking_month = outstanding[0].clone();
        Ok(PeriodLockStatus {
            recordable_period_months: outstanding,
            blocking_month: Some(blocking_month),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutstandingAlert {
    pub outstanding_months: Vec<String>,
    pub current_month: String,
}

/// API-31 (US-M5.2) — Rule-20's banner/notification-list data source.
pub fn get_outstanding_alert(conn: &Connection) -> Result<OutstandingAlert, AppError> {
    Ok(OutstandingAlert {
        outstanding_months: outstanding_period_months(conn)?,
        current_month: current_calendar_month(),
    })
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd src-tauri && cargo test m5_close::tests:: 2>&1 | tail -50`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/m5_close/mod.rs
git commit -m "feat(S12): get_period_lock_status and get_outstanding_alert (API-07/API-31)"
```

---

### Task 4: `period_month_of_date` becomes `pub(crate)`

**Files:**
- Modify: `src-tauri/src/m2_entries/mod.rs`

**Interfaces:**
- Produces: `pub(crate) fn period_month_of_date(date: &str) -> Result<String, AppError>` — consumed by Task 5 (`m5_close::resolve_recording_period`).

- [ ] **Step 1: Change visibility**

In `src-tauri/src/m2_entries/mod.rs`, change:

```rust
fn period_month_of_date(date: &str) -> Result<String, AppError> {
```

to:

```rust
pub(crate) fn period_month_of_date(date: &str) -> Result<String, AppError> {
```

- [ ] **Step 2: Compile check**

Run: `cd src-tauri && cargo build 2>&1 | tail -20`
Expected: builds clean (visibility widening never breaks existing callers).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/m2_entries/mod.rs
git commit -m "refactor(S12): widen period_month_of_date to pub(crate) for m5_close reuse"
```

---

### Task 5: `resolve_recording_period` — the Rule-36 gate (US-M2.3/M2.4)

**Files:**
- Modify: `src-tauri/src/m5_close/mod.rs`

**Interfaces:**
- Consumes: `crate::m2_entries::period_month_of_date` (Task 4), `outstanding_period_months`/`current_calendar_month` (Task 2), `AppError::PeriodClosed`/`AppError::PeriodNotAcceptingEntries` (Task 1).
- Produces: `pub fn resolve_recording_period(conn: &Connection, entry_date: &str) -> Result<i64, AppError>` — consumed by Task 6 (`m2_entries::record_entry`).

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module:

```rust
    #[test]
    fn resolve_recording_period_accepts_an_outstanding_month() {
        let conn = seeded();
        let outstanding = ym_offset(-1);
        let period_id = insert_period(&conn, &outstanding, "awaiting_close");
        insert_period(&conn, &ym_offset(0), "open");

        let resolved =
            resolve_recording_period(&conn, &format!("{outstanding}-15")).unwrap();
        assert_eq!(resolved, period_id);
    }

    #[test]
    fn resolve_recording_period_refuses_the_current_month_while_earlier_is_outstanding() {
        let conn = seeded();
        let outstanding = ym_offset(-1);
        insert_period(&conn, &outstanding, "awaiting_close");
        let current = ym_offset(0);
        insert_period(&conn, &current, "open");

        let err = resolve_recording_period(&conn, &format!("{current}-05")).unwrap_err();
        match err {
            AppError::PeriodNotAcceptingEntries {
                month,
                blocking_month,
            } => {
                assert_eq!(month, current);
                assert_eq!(blocking_month, outstanding);
            }
            other => panic!("expected PeriodNotAcceptingEntries, got {other:?}"),
        }
    }

    #[test]
    fn resolve_recording_period_directs_a_closed_month_to_correction() {
        let conn = seeded();
        let closed = ym_offset(-2);
        insert_period(&conn, &closed, "closed");

        let err = resolve_recording_period(&conn, &format!("{closed}-10")).unwrap_err();
        match err {
            AppError::PeriodClosed { month } => assert_eq!(month, closed),
            other => panic!("expected PeriodClosed, got {other:?}"),
        }
    }

    #[test]
    fn resolve_recording_period_refuses_a_future_date() {
        let conn = seeded();
        let future = ym_offset(1);
        let err = resolve_recording_period(&conn, &format!("{future}-01")).unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn resolve_recording_period_accepts_the_current_month_when_nothing_outstanding() {
        let conn = seeded();
        let current = ym_offset(0);
        let period_id = insert_period(&conn, &current, "open");

        let resolved = resolve_recording_period(&conn, &format!("{current}-01")).unwrap();
        assert_eq!(resolved, period_id);
    }

    #[test]
    fn resolve_recording_period_auto_creates_the_current_month_row_if_missing() {
        // Defensive fallback for direct callers that skip login/catch-up —
        // in real operation the row always already exists.
        let conn = seeded();
        let current = ym_offset(0);
        let period_id = resolve_recording_period(&conn, &format!("{current}-01")).unwrap();

        let status: String = conn
            .query_row("SELECT status FROM periods WHERE id = ?1", [period_id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(status, "open");
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd src-tauri && cargo test m5_close::tests::resolve_recording_period 2>&1 | tail -30`
Expected: FAIL — `resolve_recording_period` not found.

- [ ] **Step 3: Implement**

Add to `src-tauri/src/m5_close/mod.rs`:

```rust
/// US-M2.3/US-M2.4 (Rule-36 as amended by CR-2) — resolves `entry_date` to
/// the period it must be recorded against, or refuses it. Replaces
/// `m2_entries::get_or_create_open_period`, which was a narrow stand-in for
/// exactly this routine (see that module's own header comment).
pub fn resolve_recording_period(conn: &Connection, entry_date: &str) -> Result<i64, AppError> {
    let target_month = crate::m2_entries::period_month_of_date(entry_date)?;
    let current_month = current_calendar_month();

    if target_month > current_month {
        return Err(AppError::Validation {
            field: "entryDate".into(),
            message: "Entries cannot be dated in the future.".into(),
        });
    }

    let existing: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, status FROM periods WHERE period_month = ?1",
            [&target_month],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    let (period_id, status) = match existing {
        Some(row) => row,
        None if target_month == current_month => {
            conn.execute(
                "INSERT INTO periods (period_month, status) VALUES (?1, 'open')",
                [&target_month],
            )?;
            (conn.last_insert_rowid(), "open".to_string())
        }
        None => {
            return Err(AppError::NotFound {
                message: "That month has no recorded period.".into(),
            });
        }
    };

    match status.as_str() {
        "closed" => Err(AppError::PeriodClosed { month: target_month }),
        "awaiting_close" => Ok(period_id),
        _ => match outstanding_period_months(conn)?.into_iter().next() {
            Some(blocking_month) => Err(AppError::PeriodNotAcceptingEntries {
                month: target_month,
                blocking_month,
            }),
            None => Ok(period_id),
        },
    }
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd src-tauri && cargo test m5_close::tests:: 2>&1 | tail -60`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/m5_close/mod.rs
git commit -m "feat(S12): resolve_recording_period — the Rule-36 entry-eligibility gate"
```

---

### Task 6: Wire the gate into `record_entry`; remove `get_or_create_open_period`

**Files:**
- Modify: `src-tauri/src/m2_entries/mod.rs`

**Interfaces:**
- Consumes: `crate::m5_close::resolve_recording_period` (Task 5).

- [ ] **Step 1: Write the failing tests (TEST-R36 matrix + period isolation)**

Add to the `tests` module in `src-tauri/src/m2_entries/mod.rs` (needs `chrono::Months` — already imported via `chrono::NaiveDate` at the top; add `use chrono::Months;` inside the test module if not already in scope):

```rust
    fn ym_offset(months: i64) -> String {
        let today = chrono::Local::now().date_naive();
        let shifted = if months >= 0 {
            today.checked_add_months(chrono::Months::new(months as u32))
        } else {
            today.checked_sub_months(chrono::Months::new((-months) as u32))
        };
        shifted.unwrap().format("%Y-%m").to_string()
    }

    fn insert_period(conn: &Connection, month: &str, status: &str) -> i64 {
        conn.execute(
            "INSERT INTO periods (period_month, status) VALUES (?1, ?2)",
            [month, status],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    // TEST-R36 — the exit-gate matrix (T-M2.4-5): with an earlier month
    // `awaiting_close` and today as "current", every branch of Rule-36
    // (as amended by CR-2) behaves.
    #[test]
    fn test_r36_outstanding_month_entry_is_accepted() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let outstanding = ym_offset(-1);
        insert_period(&db.conn, &outstanding, "awaiting_close");
        insert_period(&db.conn, &ym_offset(0), "open");

        let entry = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 1_000,
                entry_date: format!("{outstanding}-20"),
            },
        )
        .unwrap();
        assert_eq!(entry.period_month, outstanding);
    }

    #[test]
    fn test_r36_current_month_entry_is_refused_naming_the_blocker() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let outstanding = ym_offset(-1);
        insert_period(&db.conn, &outstanding, "awaiting_close");
        let current = ym_offset(0);
        insert_period(&db.conn, &current, "open");

        let err = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 1_000,
                entry_date: format!("{current}-05"),
            },
        )
        .unwrap_err();
        match err {
            AppError::PeriodNotAcceptingEntries {
                month,
                blocking_month,
            } => {
                assert_eq!(month, current);
                assert_eq!(blocking_month, outstanding);
            }
            other => panic!("expected PeriodNotAcceptingEntries, got {other:?}"),
        }
    }

    #[test]
    fn test_r36_closed_month_entry_is_refused() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let closed = ym_offset(-2);
        insert_period(&db.conn, &closed, "closed");

        let err = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 1_000,
                entry_date: format!("{closed}-10"),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::PeriodClosed { .. }));
    }

    #[test]
    fn test_r36_future_dated_entry_is_refused() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let future = ym_offset(1);

        let err = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 1_000,
                entry_date: format!("{future}-01"),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn test_r36_current_month_entry_saves_once_the_blocker_closes() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let now_closed = ym_offset(-1);
        insert_period(&db.conn, &now_closed, "closed");
        let current = ym_offset(0);
        insert_period(&db.conn, &current, "open");

        let entry = record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 1_000,
                entry_date: format!("{current}-05"),
            },
        )
        .unwrap();
        assert_eq!(entry.period_month, current);
    }

    #[test]
    fn recording_into_the_outstanding_month_leaves_the_current_periods_totals_untouched() {
        let db = TempDb::new();
        let root = insert_member(&db.conn, None);
        let outstanding = ym_offset(-1);
        let outstanding_id = insert_period(&db.conn, &outstanding, "awaiting_close");
        let current = ym_offset(0);
        let current_id = insert_period(&db.conn, &current, "open");
        record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 500_00,
                entry_date: format!("{current}-01"),
            },
        )
        .unwrap();

        record_entry(
            &db.conn,
            RecordEntryInput {
                member_id: root,
                amount: 1_000_00,
                entry_date: format!("{outstanding}-15"),
            },
        )
        .unwrap();

        assert_eq!(
            member_period_total(&db.conn, root, outstanding_id),
            Some(1_000_00)
        );
        assert_eq!(
            member_period_total(&db.conn, root, current_id),
            Some(500_00),
            "the current period's own row must be byte-identical to before"
        );
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd src-tauri && cargo test m2_entries::tests::test_r36 2>&1 | tail -40`
Expected: FAIL — `record_entry` currently accepts everything unconditionally (no gate), so the refusal-expecting tests fail (`unwrap_err()` panics on an `Ok`).

- [ ] **Step 3: Implement**

In `src-tauri/src/m2_entries/mod.rs`:

1. Delete the `get_or_create_open_period` function entirely (lines ~57–78, the one with the doc comment starting "Rule-21: one period row per calendar month").
2. Replace `record_entry`'s body:

```rust
pub fn record_entry(
    conn: &Connection,
    input: RecordEntryInput,
) -> Result<BusinessVolumeEntry, AppError> {
    validate_amount(input.amount)?;
    if !member_exists(conn, input.member_id)? {
        return Err(AppError::NotFound {
            message: "Member not found.".into(),
        });
    }
    let period_month = period_month_of_date(&input.entry_date)?;
    let created_at = today_iso();

    let tx = conn.unchecked_transaction()?;
    let period_id = crate::m5_close::resolve_recording_period(&tx, &input.entry_date)?;
    tx.execute(
        "INSERT INTO business_volume_entries (member_id, amount, entry_date, period_month, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![input.member_id, input.amount, input.entry_date, period_month, created_at],
    )?;
    let entry_id = tx.last_insert_rowid();
    m3_calc::recalculate_chain(&tx, input.member_id, period_id)?;
    write_audit(&tx, entry_id, None, input.amount, "entry")?;
    tx.commit()?;

    load_entry(conn, entry_id)
}
```

3. Update the module's own header comment (lines 1–11) — it currently says "US-M2.3/M2.4 ... are **not** built yet" and calls `get_or_create_open_period` "a narrow stand-in scoped to what `record_entry`/`edit_entry` need". Replace with:

```rust
// M2 — Business Volume Entry (04-technical-architecture.md §3.1, §6 API-08/
// API-09; 02-business-rules.md Rule-15/16/16a/36/39). US-M2.1/M2.2, S7;
// US-M2.3/M2.4, S12.
//
// `record_entry` refuses a current-month entry while an earlier month is
// still `awaiting_close`, and refuses a closed-month write outright (that
// stays `edit_entry`'s job via Rule-39's correction path) — via
// `m5_close::resolve_recording_period`, which owns period-state resolution
// end to end. Period *transitions* (`open` → `awaiting_close` → `closed`)
// are US-M5.5's `run_period_catchup`, run at login — nothing in this module
// ever changes a period's status.
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd src-tauri && cargo test m2_entries:: 2>&1 | tail -80`
Expected: **all** `m2_entries` tests pass — the six new TEST-R36/isolation tests, and every pre-existing S7 test (`record_entry_creates_the_period_and_recalculates_the_chain`, `a_second_entry_the_same_month_reuses_the_same_period_row`, `edit_entry_*`, etc.) unchanged. The pre-existing tests use hardcoded `2026-08-*` dates that pass today because the current calendar month is `2026-08` — the same wall-clock assumption `m3_calc`/`m5_close`'s own S6/S7/S11 tests already make; not altered by this task.

- [ ] **Step 5: Full workspace test + clippy**

Run: `cd src-tauri && cargo test 2>&1 | tail -80 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -60`
Expected: everything passes, zero clippy warnings.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/m2_entries/mod.rs
git commit -m "feat(S12): gate record_entry on Rule-36 entry eligibility (TEST-R36)"
```

---

### Task 7: Real `get_period_lock_status`/`get_outstanding_alert` commands; wire catch-up into login

**Files:**
- Modify: `src-tauri/src/commands.rs`

**Interfaces:**
- Consumes: `m5_close::run_period_catchup`, `m5_close::get_period_lock_status`, `m5_close::get_outstanding_alert` (Tasks 2/3).

- [ ] **Step 1: Replace the two stubs**

In `src-tauri/src/commands.rs`, delete the line:

```rust
auth_stub!(get_period_lock_status);
```

and replace it with:

```rust
/// API-07 (US-M2.3/M5.3, S12).
#[tauri::command]
pub fn get_period_lock_status(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<m5_close::PeriodLockStatus, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m5_close::get_period_lock_status(conn)
}
```

Delete the line:

```rust
auth_stub!(get_outstanding_alert);
```

and replace it with:

```rust
/// API-31 (US-M5.2, S12).
#[tauri::command]
pub fn get_outstanding_alert(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
) -> Result<m5_close::OutstandingAlert, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m5_close::get_outstanding_alert(conn)
}
```

- [ ] **Step 2: Wire catch-up into `setup_first_run` and `login`**

In `setup_first_run`, change:

```rust
    let (result, master_key) = m8_auth::setup_first_run(&paths.auth_path, input)?;
    let conn = crate::db::open_encrypted(
        &paths.db_path,
        &m8_auth::crypto::sqlcipher_raw_key_pragma(&master_key),
    )?;
    *db.0.lock().expect("db mutex poisoned") = Some(conn);
    session.mark_authenticated();
    Ok(result)
```

to:

```rust
    let (result, master_key) = m8_auth::setup_first_run(&paths.auth_path, input)?;
    let conn = crate::db::open_encrypted(
        &paths.db_path,
        &m8_auth::crypto::sqlcipher_raw_key_pragma(&master_key),
    )?;
    m5_close::run_period_catchup(&conn)?;
    *db.0.lock().expect("db mutex poisoned") = Some(conn);
    session.mark_authenticated();
    Ok(result)
```

In `login`, change:

```rust
    let master_key = m8_auth::login(&paths.auth_path, input)?;
    let conn = crate::db::open_encrypted(
        &paths.db_path,
        &m8_auth::crypto::sqlcipher_raw_key_pragma(&master_key),
    )?;
    *db.0.lock().expect("db mutex poisoned") = Some(conn);
    session.mark_authenticated();
    Ok(())
```

to:

```rust
    let master_key = m8_auth::login(&paths.auth_path, input)?;
    let conn = crate::db::open_encrypted(
        &paths.db_path,
        &m8_auth::crypto::sqlcipher_raw_key_pragma(&master_key),
    )?;
    m5_close::run_period_catchup(&conn)?;
    *db.0.lock().expect("db mutex poisoned") = Some(conn);
    session.mark_authenticated();
    Ok(())
```

Also update the two module comments that will now be stale:
- The `// M2 — ...` section comment near `record_entry`/`edit_entry` (around line 132) still correctly says "US-M2.3/M2.4, S12" — no change needed there.
- The `// M5 — US-M5.1, S11 (US-M5.2..M5.5 are S12-S13 and stay stubs).` comment (around line 217) — change to:

```rust
// M5 — US-M5.1, S11; US-M5.2/M5.3/M5.5, S12 (US-M5.4 is S13 and stays a stub).
```

- [ ] **Step 3: Build check**

Run: `cd src-tauri && cargo build 2>&1 | tail -40`
Expected: builds clean.

- [ ] **Step 4: Full test + clippy + fmt**

Run: `cd src-tauri && cargo test 2>&1 | tail -80 && cargo clippy --all-targets -- -D warnings 2>&1 | tail -60 && cargo fmt --check 2>&1 | tail -40`
Expected: all pass, zero clippy warnings, `fmt --check` clean (run `cargo fmt` first if not).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat(S12): wire real get_period_lock_status/get_outstanding_alert; run catch-up at login"
```

---

### Task 8: Shared `monthLabel` helper

**Files:**
- Modify: `src/lib/utils.ts`
- Modify: `src/screens/monthly-close.tsx`

**Interfaces:**
- Produces: `export function monthLabel(periodMonth: string): string` — consumed by Task 9 and Task 13.

- [ ] **Step 1: Add the helper to `utils.ts`**

In `src/lib/utils.ts`, add:

```ts
// "2026-06" -> "June 2026" — every screen that names a period month uses
// this, not its own formatting (monthly-close.tsx, business-volume-entry.tsx).
export function monthLabel(periodMonth: string): string {
  const [year, month] = periodMonth.split("-").map(Number);
  return new Date(year, month - 1, 1).toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  });
}
```

- [ ] **Step 2: Remove the local copy in `monthly-close.tsx` and import the shared one**

In `src/screens/monthly-close.tsx`, delete the local `function monthLabel(...)` definition (lines ~26–31), and change the `@/lib/utils` import — if there isn't one already, add:

```ts
import { monthLabel } from "@/lib/utils";
```

(Place it alongside the other `@/lib/...` imports near the top of the file.)

- [ ] **Step 3: Type-check**

Run: `npm run build 2>&1 | tail -40`
Expected: builds clean (vocab-grep + tsc + vite build all pass — `monthLabel`'s output strings are just month/year names, nothing excluded-vocabulary).

- [ ] **Step 4: Commit**

```bash
git add src/lib/utils.ts src/screens/monthly-close.tsx
git commit -m "refactor(S12): share monthLabel between monthly-close and business-volume-entry"
```

---

### Task 9: `OutstandingAlertProvider` — real data, shared refresh

**Files:**
- Create: `src/lib/outstanding-alert-context.tsx`
- Delete: `src/lib/use-outstanding-alert.ts`

**Interfaces:**
- Consumes: `getOutstandingAlert` and `OutstandingAlert` from `@/lib/ipc/m8-auth` (already exist).
- Produces: `export function OutstandingAlertProvider({ children }: { children: ReactNode })`, `export function useOutstandingAlert(): { alert: OutstandingAlert | null; refresh: () => void }` — consumed by Task 10 (`app-shell.tsx`) and Task 11 (`monthly-close.tsx`).

- [ ] **Step 1: Delete the stub, create the context**

Delete `src/lib/use-outstanding-alert.ts`.

Create `src/lib/outstanding-alert-context.tsx`:

```tsx
import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { getOutstandingAlert, type OutstandingAlert } from "@/lib/ipc/m8-auth";

interface OutstandingAlertContextValue {
  alert: OutstandingAlert | null;
  refresh: () => void;
}

const OutstandingAlertContext = createContext<OutstandingAlertContextValue | null>(null);

/**
 * US-M5.2 (API-31) — mounted once inside `AppShell`, so the banner and the
 * notification-list read the same fetch instead of each rolling their own.
 * `refresh()` is what `MonthlyClose` calls after a completed close — the
 * only moment Rule-20 allows the alert to clear (AC-20: never on
 * navigation, logout or a timer).
 */
export function OutstandingAlertProvider({ children }: { children: ReactNode }) {
  const [alert, setAlert] = useState<OutstandingAlert | null>(null);

  function refresh() {
    getOutstandingAlert()
      .then(setAlert)
      .catch(() => setAlert(null));
  }

  useEffect(() => {
    refresh();
  }, []);

  return (
    <OutstandingAlertContext.Provider value={{ alert, refresh }}>
      {children}
    </OutstandingAlertContext.Provider>
  );
}

export function useOutstandingAlert() {
  const ctx = useContext(OutstandingAlertContext);
  if (!ctx) {
    throw new Error("useOutstandingAlert must be used inside OutstandingAlertProvider");
  }
  return ctx;
}
```

- [ ] **Step 2: Type-check (expect import errors elsewhere — fixed in the next tasks)**

Run: `npx tsc --noEmit 2>&1 | tail -40`
Expected: errors in `app-shell.tsx` (`Cannot find module '@/lib/use-outstanding-alert'`) — expected, fixed in Task 10.

- [ ] **Step 3: Commit**

```bash
git add src/lib/outstanding-alert-context.tsx src/lib/use-outstanding-alert.ts
git commit -m "feat(S12): OutstandingAlertProvider replaces the null-stub hook"
```

---

### Task 10: `AppShell` consumes the provider

**Files:**
- Modify: `src/components/app-shell.tsx`

**Interfaces:**
- Consumes: `OutstandingAlertProvider`, `useOutstandingAlert` (Task 9).

- [ ] **Step 1: Rewrite the component**

Replace the full contents of `src/components/app-shell.tsx`:

```tsx
import { Outlet } from "react-router";
import { Sidebar } from "./sidebar";
import { OutstandingMonthBanner } from "./outstanding-month-banner";
import { NotificationList } from "./notification-list";
import { OutstandingAlertProvider, useOutstandingAlert } from "@/lib/outstanding-alert-context";
import { useInactivityLock } from "@/lib/use-inactivity-lock";

/**
 * T-UI.2-1/T-UI.2-2 — fixed 236px sidebar + fluid content column, sticky at
 * full viewport height. Wraps every route except the auth phases, which
 * render standalone (there is nothing to navigate to before signing in).
 */
export function AppShell() {
  useInactivityLock();

  return (
    <OutstandingAlertProvider>
      <AppShellLayout />
    </OutstandingAlertProvider>
  );
}

function AppShellLayout() {
  const { alert } = useOutstandingAlert();

  return (
    <div className="grid h-screen grid-cols-[236px_1fr]">
      <Sidebar />
      <div className="flex h-screen flex-col overflow-hidden">
        <OutstandingMonthBanner alert={alert} />
        <header className="sticky top-0 z-10 flex h-14 shrink-0 items-center justify-end border-b border-border bg-surface px-8">
          <NotificationList alert={alert} />
        </header>
        <main className="flex-1 overflow-y-auto px-8 pb-10 pt-5">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Type-check**

Run: `npx tsc --noEmit 2>&1 | tail -40`
Expected: no errors referencing `app-shell.tsx` or `use-outstanding-alert` anymore.

- [ ] **Step 3: Commit**

```bash
git add src/components/app-shell.tsx
git commit -m "feat(S12): AppShell mounts OutstandingAlertProvider"
```

---

### Task 11: Banner shows the "N more months" clause; `MonthlyClose` refreshes the alert

**Files:**
- Modify: `src/components/outstanding-month-banner.tsx`
- Modify: `src/screens/monthly-close.tsx`

**Interfaces:**
- Consumes: `useOutstandingAlert` (Task 9), `monthLabel` (Task 8).

- [ ] **Step 1: Update the banner**

Replace the body of `OutstandingMonthBanner` in `src/components/outstanding-month-banner.tsx`:

```tsx
export function OutstandingMonthBanner({ alert }: OutstandingMonthBannerProps) {
  if (!alert || alert.outstandingMonths.length === 0) return null;

  const oldest = alert.outstandingMonths[0];
  const moreCount = alert.outstandingMonths.length - 1;
  const moreClause =
    moreCount > 0
      ? ` ${moreCount} more month${moreCount > 1 ? "s are" : " is"} outstanding after that.`
      : "";

  return (
    <div
      role="status"
      className="flex items-center gap-2.5 border-b px-8 py-2.5 text-[12.5px]"
      style={{
        backgroundColor: "var(--warning-weak)",
        borderColor: "color-mix(in srgb, var(--warning) 35%, var(--border))",
      }}
    >
      <AlertTriangle
        className="h-[15px] w-[15px] shrink-0"
        style={{ color: "var(--warning-text)" }}
      />
      <p style={{ color: "var(--warning-text)" }}>
        <span className="font-semibold">{oldest} has ended and is awaiting close.</span>
        {moreClause} You can still record entries dated in {oldest}. {alert.currentMonth} entries
        unlock once {oldest} is closed.
      </p>
      <Link
        to="/close"
        className="ml-auto shrink-0 rounded-sm border border-border bg-surface px-3 py-1 text-[13px] font-medium text-ink hover:bg-bg"
      >
        Close {oldest}
      </Link>
    </div>
  );
}
```

(Only the `oldest`/`moreCount`/`moreClause` derivation and the `<p>` body changed — imports and props stay the same.)

- [ ] **Step 2: Wire `MonthlyClose`'s existing refresh to the alert context**

In `src/screens/monthly-close.tsx`, add the import:

```ts
import { useOutstandingAlert } from "@/lib/outstanding-alert-context";
```

Inside `export function MonthlyClose() {`, add (right after `const toast = useToast();`):

```tsx
  const { refresh: refreshAlert } = useOutstandingAlert();
```

Change the existing `refresh` function from:

```tsx
  async function refresh() {
    try {
      setOutstanding(await getOutstandingPeriods());
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    }
  }
```

to:

```tsx
  async function refresh() {
    try {
      setOutstanding(await getOutstandingPeriods());
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    }
    refreshAlert();
  }
```

- [ ] **Step 3: Type-check and vocab-grep**

Run: `npm run build 2>&1 | tail -40`
Expected: builds clean — "outstanding", "month", "close" are all ordinary product vocabulary, not excluded terms.

- [ ] **Step 4: Commit**

```bash
git add src/components/outstanding-month-banner.tsx src/screens/monthly-close.tsx
git commit -m "feat(S12): banner names every outstanding month; close refreshes the alert (AC-20)"
```

---

### Task 12: `BusinessVolumeEntry` — real recording-month bounds, note, refusal handling (T-M2.3-3/4, T-M2.4-2/3)

**Files:**
- Modify: `src/screens/business-volume-entry.tsx`

**Interfaces:**
- Consumes: `getPeriodLockStatus`, `PeriodLockStatus` from `@/lib/ipc/m2-entries` (already exist), `monthLabel` from `@/lib/utils` (Task 8), `AlertNote` from `@/components/ui/alert-note` (already exists).

- [ ] **Step 1: Replace the month-bounds logic and add the lock-status fetch**

In `src/screens/business-volume-entry.tsx`, replace the `currentMonthBounds` function and its doc comment:

```ts
// T-M2.1-4: bounded to the recording month, defaulting to today. US-M2.3's
// outstanding-month recording (S12) doesn't exist yet, so "the recording
// month" is always the current calendar month for now.
function currentMonthBounds() {
  const now = new Date();
  const first = new Date(now.getFullYear(), now.getMonth(), 1);
  return { min: isoDate(first), max: isoDate(now) };
}
```

with:

```ts
function currentYm(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

// T-M2.1-4/T-M2.3-4: bounded to the recording month. The current month
// caps at today (can't record ahead of itself); an outstanding earlier
// month is bounded by its own last calendar day.
function monthBounds(ym: string) {
  const [year, month] = ym.split("-").map(Number);
  const first = new Date(year, month - 1, 1);
  const last = new Date(year, month, 0);
  const isCurrent = ym === currentYm();
  return { min: isoDate(first), max: isoDate(isCurrent ? new Date() : last) };
}
```

Update the imports at the top of the file to add:

```ts
import { getPeriodLockStatus } from "@/lib/ipc/m2-entries";
import type { PeriodLockStatus } from "@/lib/ipc/m2-entries";
import { AlertNote } from "@/components/ui/alert-note";
import { monthLabel } from "@/lib/utils";
```

(`centsToDisplay, displayToCents` stay imported from `@/lib/utils` as before — add `monthLabel` to that same import line rather than a new one.)

- [ ] **Step 2: Fetch lock status on mount, derive the recording month, rebase the date field**

Inside `export function BusinessVolumeEntry() {`, replace:

```tsx
  const [date, setDate] = useState(() => currentMonthBounds().max);
```

with:

```tsx
  const [lockStatus, setLockStatus] = useState<PeriodLockStatus | null>(null);
  const recordingMonth = lockStatus?.recordablePeriodMonths[0] ?? currentYm();
  const [date, setDate] = useState(() => monthBounds(recordingMonth).max);
```

Add, alongside the other `useEffect` (the `?member=` prefill one):

```tsx
  useEffect(() => {
    getPeriodLockStatus().then((status) => {
      setLockStatus(status);
      setDate(monthBounds(status.recordablePeriodMonths[0]).max);
    });
  }, []);
```

Replace the later line:

```tsx
  const bounds = currentMonthBounds();
```

with:

```tsx
  const bounds = monthBounds(recordingMonth);
```

- [ ] **Step 3: Render the recording-month note; refuse gracefully on `period_not_accepting_entries`/`period_closed`**

Replace the amount-error catch in `handleSave`:

```tsx
    } catch (raw) {
      const presented = toErrorPresentation(raw);
      setAmountError(presented.message);
    } finally {
```

with:

```tsx
    } catch (raw) {
      const presented = toErrorPresentation(raw);
      if (
        presented.kind === "period_not_accepting_entries" ||
        presented.kind === "period_closed" ||
        presented.field === "entryDate"
      ) {
        setDateError(presented.message);
      } else {
        setAmountError(presented.message);
      }
    } finally {
```

Add the `dateError` state next to `amountError`:

```tsx
  const [dateError, setDateError] = useState<string | null>(null);
```

Clear it alongside `amountError` at the top of `handleSave` (where `setAmountError(null)` already runs):

```tsx
    setAmountError(null);
    setDateError(null);
```

Add the note above the form card — right after the `<h1>`:

```tsx
      {lockStatus && (
        <AlertNote variant="warn" className="mt-3.5 max-w-md">
          Recording into <strong>{monthLabel(recordingMonth)}</strong>.{" "}
          {lockStatus.blockingMonth
            ? `${monthLabel(currentYm())} entries can be recorded once ${monthLabel(
                lockStatus.blockingMonth,
              )} is closed.`
            : "Dates are limited to this month."}
        </AlertNote>
      )}
```

Add an error hint under the date `Input` (mirroring the amount field's `InputHint`):

```tsx
          <Input
            id="entry-date"
            type="date"
            min={bounds.min}
            max={bounds.max}
            value={date}
            disabled={!selected}
            aria-invalid={!!dateError}
            onChange={(e) => {
              setDate(e.target.value);
              setDateError(null);
            }}
          />
          {dateError && <InputHint error>{dateError}</InputHint>}
```

(replaces the existing plain `<Input id="entry-date" ...>` block — same props plus `aria-invalid`/`onChange`'s error-clear, plus the new hint line after it).

- [ ] **Step 4: Type-check and build**

Run: `npm run build 2>&1 | tail -60`
Expected: builds clean.

- [ ] **Step 5: Manual verification (dev server)**

Run: `npm run tauri dev` (or `npm run dev` if a browser-only smoke test suffices for this screen), then:
1. Complete first-run setup, add a root member.
2. Navigate to Business Volume Entry — confirm the note reads "Recording into `<current month>`. Dates are limited to this month." and the date field is bounded to the current month with today as the default.
3. Record an entry — confirm it saves as before (no eligibility regression on the ordinary path).

(Exercising the outstanding-month/refusal branches end-to-end requires a period already `awaiting_close`, which — with no month-selector UI yet — only arises after a real calendar-month boundary or a manually seeded database; the Rust-side TEST-R36 suite from Task 6 is what actually gates this behavior. This manual pass only needs to confirm the happy path renders correctly and nothing regressed.)

Stop the dev server once confirmed.

- [ ] **Step 6: Commit**

```bash
git add src/screens/business-volume-entry.tsx
git commit -m "feat(S12): business-volume-entry reads real period-lock status (T-M2.3/T-M2.4)"
```

---

### Task 13: Final verification pass and self-audit

**Files:** none (verification only)

- [ ] **Step 1: Full Rust verification**

Run:
```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo audit
```
Expected: all clean, all pass, no vulnerabilities.

- [ ] **Step 2: Full frontend verification**

Run:
```bash
npm run test
npm run build
npm audit
```
Expected: `vitest run` + `node --test scripts/*.test.mjs` pass, `build` (vocab-grep + tsc + vite build) passes, no vulnerabilities.

- [ ] **Step 3: Self-audit against the S12 backlog task list**

Re-read `PI/01-backlog.md`'s US-M5.5, US-M5.2, US-M5.3, US-M2.3, US-M2.4 task lists (`T-M5.5-1` through `T-M5.5-6`, `T-M5.2-1` through `T-M5.2-6`, `T-M5.3-1` through `T-M5.3-3`, `T-M2.3-1` through `T-M2.3-5`, `T-M2.4-1` through `T-M2.4-5`) and confirm each is either done or explicitly out of scope (US-M2.5's month-selector is S13). Specifically check the roadmap's own S12 exit gate: "the application unopened across three month boundaries produces all three periods, queued oldest-first, each accepting entries dated within itself" and "the full TEST-R36 matrix passes" — both are covered by Task 2 and Task 6's tests; re-run them once more standalone to be certain:

```bash
cd src-tauri
cargo test m5_close::tests::run_period_catchup_backfills_and_elapses_across_three_unopened_months -- --nocapture
cargo test m2_entries::tests::test_r36 -- --nocapture
```

- [ ] **Step 4: Push the branch (no merge, no PR — per this project's standing workflow)**

```bash
git push -u origin feature/sprint-12-period-catchup
```

---

## Plan Self-Review Notes

- **Spec coverage:** US-M5.5 (Task 2), US-M5.2 (Task 3 + Task 9/11), US-M5.3 (Task 3), US-M2.3 (Task 5/6/12), US-M2.4 (Task 1/5/6/12) all have tasks. US-M2.5 (month-selector) is confirmed S13 and intentionally absent.
- **Type consistency checked:** `PeriodLockStatus.recordablePeriodMonths`/`blockingMonth` (Rust `PeriodLockStatus` → serde camelCase) matches the pre-existing `src/lib/ipc/m2-entries.ts` interface exactly, verified against the source file before writing Task 3. `OutstandingAlert.outstandingMonths`/`currentMonth` matches the pre-existing `src/lib/ipc/m8-auth.ts` interface exactly. `AppError::PeriodNotAcceptingEntries`/`PeriodClosed` field names (`month`, `blockingMonth`) match `src/lib/ipc/errors.ts`'s `RawAppError`/`PRESENTATIONS` map exactly, verified against the source file before writing Task 1.
- **No placeholders:** every step above has literal code, not a description of code.
