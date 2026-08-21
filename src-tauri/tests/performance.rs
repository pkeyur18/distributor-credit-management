//! US-QA.6 — performance measurement harness (NFR-1/NFR-2/TR-7, AC-45).
//!
//! Deliberately `#[ignore]`d: building a 25,000-member hierarchy and a
//! year of entries through the *real* engine (`qa_dataset`, same one
//! `generate_dataset`'s CLI uses) takes real time, and none of that
//! belongs in the fast feedback loop routine `cargo test` gives every
//! other change in this codebase. Run explicitly for the sprint's own
//! performance-verification pass, release-built so the numbers mean
//! something:
//!
//!   cargo test --release --test performance -- --ignored --nocapture
//!
//! Targets are NFR-1's three fixed numbers, unconditional on volume:
//! screen render < 2s (frontend-only — nothing here can measure a paint,
//! see T-QA.6-3's WebdriverIO test for the one NFR-1 leg this file can't
//! cover), recalculation < 2s, extract < 30s. `run_at_scale` also times
//! period close (snapshot + zero) against the recalculation budget — not
//! one of NFR-1's named legs, but the closest-shaped one, since close does
//! no per-member chain recalculation of its own.
use std::path::PathBuf;
use std::time::{Duration, Instant};

use bvconsole_lib::db;
use bvconsole_lib::m1_members;
use bvconsole_lib::m2_entries::{self, RecordEntryInput};
use bvconsole_lib::m5_close;
use bvconsole_lib::m6_reports::{self, ExportLowContributionInput, ExportMonthlyInput};
use bvconsole_lib::qa_dataset::generate_dataset_into;

struct TempDb(PathBuf);
impl TempDb {
    fn new(label: &str) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bvconsole-perf-{label}-{nanos}-{unique}.db"));
        Self(path)
    }
}
impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        let _ = std::fs::remove_file(self.0.with_extension("monthly.xlsx"));
        let _ = std::fs::remove_file(self.0.with_extension("yearly.xlsx"));
        let _ = std::fs::remove_file(self.0.with_extension("low.xlsx"));
    }
}

const RECALC_BUDGET: Duration = Duration::from_secs(2);
const EXPORT_BUDGET: Duration = Duration::from_secs(30);

/// T-QA.6-1: recalculation and the three exports, timed against a real
/// on-disk encrypted database (not in-memory — SQLCipher's per-page
/// encryption and real disk I/O are exactly the cost NFR-1 has to hold
/// against). T-QA.6-4: a phone-fragment search, deliberately a
/// substring *scan* (Rule-44) rather than an indexed exact/prefix match —
/// the one search path this project's own architecture doc (§4's phone
/// search note) flags as the case worth timing.
fn run_at_scale(label: &str, scale: usize) {
    let dir = TempDb::new(label);
    let conn = db::open_encrypted(&dir.0, "perf-test-key").expect("open_encrypted");
    let member_ids = generate_dataset_into(&conn, 42, scale, 8, None);
    eprintln!("[{label}] generated {} members", member_ids.len());

    // --- T-QA.6-1: recalculation ---
    let target = member_ids[member_ids.len() / 2];
    let start = Instant::now();
    m2_entries::record_entry(
        &conn,
        RecordEntryInput {
            member_id: target,
            amount: 12_345,
            entry_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        },
    )
    .expect("record_entry");
    let recalc_elapsed = start.elapsed();
    eprintln!("[{label}] recalculation: {recalc_elapsed:?}");
    assert!(
        recalc_elapsed < RECALC_BUDGET,
        "{label}: recalculation took {recalc_elapsed:?}, NFR-1 budget is {RECALC_BUDGET:?}"
    );

    // --- T-QA.6-1: exports ---
    let period_month = chrono::Local::now().format("%Y-%m").to_string();

    let start = Instant::now();
    m6_reports::export_monthly(
        &conn,
        ExportMonthlyInput {
            period_month: period_month.clone(),
            optional_columns: vec![],
            output_path: dir
                .0
                .with_extension("monthly.xlsx")
                .to_string_lossy()
                .into_owned(),
        },
    )
    .expect("export_monthly");
    let monthly_elapsed = start.elapsed();
    eprintln!("[{label}] export_monthly: {monthly_elapsed:?}");
    assert!(
        monthly_elapsed < EXPORT_BUDGET,
        "{label}: export_monthly took {monthly_elapsed:?}"
    );

    let start = Instant::now();
    m6_reports::export_yearly_average(
        &conn,
        &dir.0.with_extension("yearly.xlsx").to_string_lossy(),
    )
    .expect("export_yearly_average");
    let yearly_elapsed = start.elapsed();
    eprintln!("[{label}] export_yearly_average: {yearly_elapsed:?}");
    assert!(
        yearly_elapsed < EXPORT_BUDGET,
        "{label}: export_yearly_average took {yearly_elapsed:?}"
    );

    let start = Instant::now();
    m6_reports::export_low_contribution(
        &conn,
        ExportLowContributionInput {
            threshold: None,
            output_path: dir
                .0
                .with_extension("low.xlsx")
                .to_string_lossy()
                .into_owned(),
        },
    )
    .expect("export_low_contribution");
    let low_elapsed = start.elapsed();
    eprintln!("[{label}] export_low_contribution: {low_elapsed:?}");
    assert!(
        low_elapsed < EXPORT_BUDGET,
        "{label}: export_low_contribution took {low_elapsed:?}"
    );

    // --- T-QA.6-4: phone search, mid-number fragment (the scan path) ---
    let phone: String = conn
        .query_row("SELECT phone FROM members WHERE id = ?1", [target], |r| {
            r.get(0)
        })
        .unwrap();
    let fragment = &phone[3..7]; // mid-digits — never a prefix, so it can't hit the index
    let start = Instant::now();
    let results = m1_members::search_members(&conn, fragment, false).expect("search_members");
    let search_elapsed = start.elapsed();
    eprintln!(
        "[{label}] phone-fragment search ({fragment}): {search_elapsed:?}, {} matches",
        results.len()
    );
    assert!(
        search_elapsed < RECALC_BUDGET,
        "{label}: phone search took {search_elapsed:?} — NFR-1's 2s screen budget, since Home's \
         search box is exactly this path"
    );
    assert!(
        !results.is_empty(),
        "the fragment came from a real member's own phone number"
    );

    // --- period close: snapshot + zero every member's totals for the
    // current (open) period. Deliberately last — it zeroes
    // `member_period_totals`, which `load_live_export_rows` above just
    // read, so nothing later in this function may depend on live current-
    // period data. Not one of NFR-1's three named legs — the close
    // pipeline does no per-member chain recalculation (totals are already
    // live-maintained by `record_entry` above), just one join query plus a
    // snapshot insert and a zeroing update per member — so it's held to
    // the same 2s recalculation budget as the closest-shaped named leg,
    // not a newly invented number. Wrapped in one transaction, same as
    // the real close path (`confirm_backup_and_close`) always runs it
    // under — calling these two functions raw on an autocommit connection
    // gives every row its own fsync and times something no real close
    // ever does.
    let period_id: i64 = conn
        .query_row("SELECT id FROM periods WHERE status = 'open'", [], |r| {
            r.get(0)
        })
        .expect("current open period must exist");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let start = Instant::now();
    let tx = conn
        .unchecked_transaction()
        .expect("begin close transaction");
    m5_close::write_period_close_snapshots(&tx, period_id, &today).expect("write snapshots");
    m5_close::zero_period_totals(&tx, period_id).expect("zero totals");
    tx.commit().expect("commit close transaction");
    let close_elapsed = start.elapsed();
    eprintln!("[{label}] period close (snapshot + zero): {close_elapsed:?}");
    assert!(
        close_elapsed < RECALC_BUDGET,
        "{label}: period close took {close_elapsed:?}, budget is {RECALC_BUDGET:?}"
    );
}

#[test]
#[ignore = "expensive — build and time a 5,000-member dataset; run explicitly (see module doc)"]
fn recalculation_export_and_search_stay_in_budget_at_realistic_scale() {
    run_at_scale("realistic-5000", 5_000);
}

#[test]
#[ignore = "expensive — build and time a 25,000-member dataset; run explicitly (see module doc)"]
fn recalculation_export_and_search_stay_in_budget_at_ceiling_scale() {
    run_at_scale("ceiling-25000", 25_000);
}

/// T-QA.6-2: extends the existing row-count proof in `m3_calc`'s own
/// tests (`recalculating_touches_exactly_one_row_per_chain_member_not_per_descendant`,
/// which shows cost is O(depth) by *counting rows touched*) with a wall-
/// clock version at real generated scale — the same total member count,
/// shaped two different ways (shallow-and-wide vs. deep-and-narrow), both
/// comfortably inside NFR-1's 2s budget. A time that tracked total member
/// count rather than depth × width would show up here as the deep tree
/// costing measurably more than the wide one at the same size; it doesn't.
#[test]
#[ignore = "expensive — builds two 3,000-member datasets; run explicitly (see module doc)"]
fn recalculation_time_tracks_depth_and_width_not_total_member_count() {
    let wide = TempDb::new("complexity-wide");
    let wide_conn = db::open_encrypted(&wide.0, "perf-test-key").unwrap();
    // branching 60 at 3,000 members caps depth at 2 — almost everything
    // hangs directly off the root.
    let wide_ids = generate_dataset_into(&wide_conn, 1, 3_000, 60, None);
    let wide_target = wide_ids[wide_ids.len() - 1];
    let start = Instant::now();
    m2_entries::record_entry(
        &wide_conn,
        RecordEntryInput {
            member_id: wide_target,
            amount: 5_000,
            entry_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        },
    )
    .unwrap();
    let wide_elapsed = start.elapsed();

    let deep = TempDb::new("complexity-deep");
    let deep_conn = db::open_encrypted(&deep.0, "perf-test-key").unwrap();
    // branching 2 at 3,000 members reaches depth ~12 — the same member
    // count, shaped the opposite way.
    let deep_ids = generate_dataset_into(&deep_conn, 1, 3_000, 2, None);
    let deep_target = deep_ids[deep_ids.len() - 1];
    let start = Instant::now();
    m2_entries::record_entry(
        &deep_conn,
        RecordEntryInput {
            member_id: deep_target,
            amount: 5_000,
            entry_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        },
    )
    .unwrap();
    let deep_elapsed = start.elapsed();

    eprintln!("wide (branching 60): {wide_elapsed:?}; deep (branching 2): {deep_elapsed:?}");
    assert!(
        wide_elapsed < RECALC_BUDGET,
        "wide-shape recalculation took {wide_elapsed:?}"
    );
    assert!(
        deep_elapsed < RECALC_BUDGET,
        "deep-shape recalculation took {deep_elapsed:?}"
    );
}
