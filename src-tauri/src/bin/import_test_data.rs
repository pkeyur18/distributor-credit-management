// Testing-only tool. Bulk-loads a hand-provided CSV (real member/business
// volume data, not synthetic) into an already-set-up console.db by calling
// the same real functions the running app calls, via `test_data_shared` —
// same reasoning as `qa_dataset.rs`/`generate_dataset`: generated writes can
// never drift from the app's own validation/calculation rules this way.
//
// Not registered anywhere (no `lib.rs`/`commands.rs` change beyond
// `test_data_shared`). Delete this file to remove the tool.
//
// Prerequisite: run the app once and complete first-run PIN/password setup
// first. This tool unlocks the existing database with that same credential
// (exactly like a real login) rather than minting its own key.
//
// CSV columns (header row required, any column order):
//   member_name, phone, address, email, consent, introducer_phone, amount, entry_date
// - One row = one business volume entry. A member's identity columns
//   (name/address/email/consent/introducer_phone) are only read the first
//   time that phone number appears; later rows for the same phone only
//   contribute another entry.
// - introducer_phone empty = root member (exactly one such row allowed).
//   Every introducer's own row must appear before the member(s) it
//   introduces — this is a straight top-down pass, not a resolver.
// - consent: yes/no/true/false/1/0 (case-insensitive).
// - amount: plain decimal rupees, e.g. 1234.56 (same as the app's own
//   entry form — see src/lib/utils.ts's parseAmountToCents).
// - entry_date: YYYY-MM-DD.
//
// Usage:
//   cargo run --bin import_test_data -- --csv data.csv --credential 123456 \
//     [--closed-months 2026-01,2026-02] [--app-data-dir /path/to/app-data]
use std::collections::HashSet;
use std::path::PathBuf;

use bvconsole_lib::test_data_shared;

struct Args {
    csv: PathBuf,
    credential: String,
    closed_months: HashSet<String>,
    app_data_dir: PathBuf,
}

fn parse_args() -> Args {
    let mut csv = None;
    let mut credential = None;
    let mut closed_months = HashSet::new();
    let mut app_data_dir = test_data_shared::default_app_data_dir();

    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--csv" => csv = Some(PathBuf::from(it.next().expect("--csv needs a value"))),
            "--credential" => credential = Some(it.next().expect("--credential needs a value")),
            "--closed-months" => {
                closed_months = it
                    .next()
                    .expect("--closed-months needs a value")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            "--app-data-dir" => {
                app_data_dir = PathBuf::from(it.next().expect("--app-data-dir needs a value"))
            }
            other => panic!("unknown flag: {other}"),
        }
    }

    Args {
        csv: csv.expect("--csv <path> is required"),
        credential: credential.expect("--credential <pin-or-password> is required"),
        closed_months,
        app_data_dir,
    }
}

fn main() {
    let args = parse_args();

    let content = std::fs::read_to_string(&args.csv)
        .unwrap_or_else(|e| panic!("reading {}: {e}", args.csv.display()));
    let conn = test_data_shared::unlock_db(&args.app_data_dir, &args.credential)
        .unwrap_or_else(|e| panic!("{e}"));
    let summary = test_data_shared::import_csv(&conn, &content, &args.closed_months)
        .unwrap_or_else(|e| panic!("{e}"));

    println!(
        "imported {} member(s), {} closed-month entries across {} month(s), {} open-period entries",
        summary.members, summary.closed_entries, summary.closed_months, summary.open_entries
    );
}
