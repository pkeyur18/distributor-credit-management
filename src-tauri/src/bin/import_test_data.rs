// Testing-only tool. Bulk-loads a hand-provided CSV (real member/business
// volume data, not synthetic) into an already-set-up console.db by calling
// the same real functions the running app calls (`m1_members::add_member`,
// `m2_entries::record_entry`, `m5_close`'s snapshot/zero pair) — same
// reasoning as `qa_dataset.rs`/`generate_dataset`: generated writes can
// never drift from the app's own validation/calculation rules this way.
//
// Not registered anywhere (no `lib.rs`/`commands.rs` change) — Cargo
// auto-discovers `src/bin/*.rs`. Delete this file to remove the tool.
//
// Prerequisite: run the app once and complete first-run PIN/password setup
// first. This tool unlocks the existing database with that same credential
// (exactly like a real login) rather than minting its own key — a CSV-
// imported database has to open under the app's real auth exactly as any
// other session would.
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
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use bvconsole_lib::m8_auth::crypto::{sqlcipher_raw_key_pragma, unwrap_master_key};
use bvconsole_lib::m8_auth::store::AuthStore;
use bvconsole_lib::{db, m1_members, m2_entries, m3_calc, m5_close};
use chrono::NaiveDate;

const IDENTIFIER: &str = "com.siddharthpatel.bvconsole"; // src-tauri/tauri.conf.json "identifier"

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
    let mut app_data_dir = default_app_data_dir();

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

fn default_app_data_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("HOME/USERPROFILE not set");
    if cfg!(target_os = "macos") {
        PathBuf::from(home)
            .join("Library/Application Support")
            .join(IDENTIFIER)
    } else if cfg!(target_os = "windows") {
        let appdata =
            std::env::var("APPDATA").unwrap_or_else(|_| format!("{home}/AppData/Roaming"));
        PathBuf::from(appdata).join(IDENTIFIER)
    } else {
        let xdg = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{home}/.local/share"));
        PathBuf::from(xdg).join(IDENTIFIER)
    }
}

/// Minimal RFC4180 reader — quoted fields (with `""` escaping) so addresses
/// containing commas survive, which is the one thing a naive `split(',')`
/// gets wrong here.
fn parse_csv(content: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' => in_quotes = true,
                ',' => row.push(std::mem::take(&mut field)),
                '\r' => {}
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                }
                _ => field.push(c),
            }
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows.retain(|r| !(r.len() == 1 && r[0].trim().is_empty()));
    rows
}

struct Columns(HashMap<String, usize>);

impl Columns {
    fn from_header(header: &[String]) -> Self {
        Self(
            header
                .iter()
                .enumerate()
                .map(|(i, name)| (name.trim().to_string(), i))
                .collect(),
        )
    }

    fn get(&self, row: &[String], name: &str) -> String {
        let idx = *self
            .0
            .get(name)
            .unwrap_or_else(|| panic!("CSV is missing required column '{name}'"));
        row.get(idx)
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
}

fn parse_consent(raw: &str) -> bool {
    match raw.to_lowercase().as_str() {
        "yes" | "true" | "1" => true,
        "no" | "false" | "0" => false,
        other => panic!("consent value '{other}' isn't one of yes/no/true/false/1/0"),
    }
}

/// Same rupees→×100-fixed-point conversion the app's own entry form does
/// (`src/lib/utils.ts`'s `parseAmountToCents`) — the CSV's `amount` column
/// is a plain decimal, not the already-scaled integer the DB stores.
fn parse_amount(raw: &str) -> i64 {
    let rupees: f64 = raw
        .parse()
        .unwrap_or_else(|_| panic!("amount '{raw}' isn't a valid number"));
    (rupees * 100.0).round() as i64
}

fn last_day_of_month(period_month: &str) -> String {
    let start = NaiveDate::parse_from_str(&format!("{period_month}-01"), "%Y-%m-%d")
        .expect("period_month must be YYYY-MM");
    start
        .checked_add_months(chrono::Months::new(1))
        .and_then(|next| next.pred_opt())
        .expect("last day of month")
        .format("%Y-%m-%d")
        .to_string()
}

struct PendingEntry {
    member_id: i64,
    amount: i64,
    entry_date: String,
}

fn import_closed_month(conn: &rusqlite::Connection, period_month: &str, entries: &[PendingEntry]) {
    let last_day = last_day_of_month(period_month);
    conn.execute(
        "INSERT INTO periods (period_month, status, ended_at, closed_at) VALUES (?1, 'closed', ?2, ?2)",
        rusqlite::params![period_month, last_day],
    )
    .unwrap_or_else(|e| panic!("inserting closed period {period_month}: {e}"));
    let period_id = conn.last_insert_rowid();

    for entry in entries {
        conn.execute(
            "INSERT INTO business_volume_entries (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            rusqlite::params![entry.member_id, entry.amount, entry.entry_date, period_month],
        )
        .unwrap_or_else(|e| panic!("inserting closed-month entry: {e}"));
        m3_calc::recalculate_chain(conn, entry.member_id, period_id)
            .unwrap_or_else(|e| panic!("recalculating chain for closed month {period_month}: {e}"));
    }

    m5_close::write_period_close_snapshots(conn, period_id, &last_day)
        .unwrap_or_else(|e| panic!("writing snapshots for {period_month}: {e}"));
    m5_close::zero_period_totals(conn, period_id)
        .unwrap_or_else(|e| panic!("zeroing totals for {period_month}: {e}"));
}

fn main() {
    let args = parse_args();

    let db_path = args.app_data_dir.join("console.db");
    let auth_path = args.app_data_dir.join("auth.json");
    assert!(
        auth_path.exists() && db_path.exists(),
        "no app data found at {} — run the app once and complete first-run setup before importing",
        args.app_data_dir.display()
    );

    let auth = AuthStore::load(&auth_path).expect("reading auth.json");
    let master_key = [&auth.pin_envelope, &auth.password_envelope]
        .into_iter()
        .flatten()
        .find_map(|envelope| unwrap_master_key(&args.credential, envelope))
        .expect("--credential didn't match the PIN or password set up in the app");
    let conn = db::open_encrypted(&db_path, &sqlcipher_raw_key_pragma(&master_key))
        .expect("opening console.db with the recovered key");

    let content = std::fs::read_to_string(&args.csv)
        .unwrap_or_else(|e| panic!("reading {}: {e}", args.csv.display()));
    let mut rows = parse_csv(&content);
    assert!(!rows.is_empty(), "CSV is empty");
    let header = rows.remove(0);
    let cols = Columns::from_header(&header);

    let mut member_ids: HashMap<String, i64> = HashMap::new();
    let mut open_entries: Vec<PendingEntry> = Vec::new();
    let mut closed_entries: HashMap<String, Vec<PendingEntry>> = HashMap::new();

    for (i, row) in rows.iter().enumerate() {
        let line = i + 2; // header is line 1
        let phone = cols.get(row, "phone");
        assert!(!phone.is_empty(), "line {line}: phone is required");

        let member_id = if let Some(&id) = member_ids.get(&phone) {
            id
        } else {
            let name = cols.get(row, "member_name");
            let address = cols.get(row, "address");
            let email = cols.get(row, "email");
            let email = if email.is_empty() { None } else { Some(email) };
            let consent_given = parse_consent(&cols.get(row, "consent"));
            let introducer_phone = cols.get(row, "introducer_phone");

            let id = if introducer_phone.is_empty() {
                let outcome = m1_members::create_root_member(
                    &conn,
                    m1_members::CreateRootMemberInput {
                        name,
                        phone: phone.clone(),
                        address,
                        email,
                        consent_given,
                    },
                )
                .unwrap_or_else(|e| panic!("line {line}: creating root member: {e}"));
                outcome.id
            } else {
                let introducer_id = *member_ids.get(&introducer_phone).unwrap_or_else(|| {
                    panic!(
                        "line {line}: introducer phone {introducer_phone} hasn't appeared yet — \
                         list every introducer's own row before the members it introduced"
                    )
                });
                match m1_members::add_member(
                    &conn,
                    m1_members::AddMemberInput {
                        name,
                        phone: phone.clone(),
                        address,
                        email,
                        consent_given,
                        introducer_member_id: introducer_id,
                    },
                )
                .unwrap_or_else(|e| panic!("line {line}: adding member: {e}"))
                {
                    m1_members::AddMemberOutcome::Created { member, .. } => member.id,
                    m1_members::AddMemberOutcome::ReactivationOffer { existing_member } => panic!(
                        "line {line}: phone {phone} already belongs to inactive member #{} — \
                         unexpected in a fresh import",
                        existing_member.id
                    ),
                }
            };
            member_ids.insert(phone.clone(), id);
            id
        };

        let amount = parse_amount(&cols.get(row, "amount"));
        let entry_date = cols.get(row, "entry_date");
        NaiveDate::parse_from_str(&entry_date, "%Y-%m-%d")
            .unwrap_or_else(|_| panic!("line {line}: entry_date '{entry_date}' isn't YYYY-MM-DD"));
        let period_month = entry_date[..7].to_string();

        let entry = PendingEntry {
            member_id,
            amount,
            entry_date,
        };
        if args.closed_months.contains(&period_month) {
            closed_entries.entry(period_month).or_default().push(entry);
        } else {
            open_entries.push(entry);
        }
    }

    let mut closed_months: Vec<&String> = closed_entries.keys().collect();
    closed_months.sort();
    for period_month in closed_months {
        import_closed_month(&conn, period_month, &closed_entries[period_month]);
    }

    // `setup_first_run` already inserted a `periods` row for the real current
    // calendar month (open) before this import ever ran — so
    // `m5_close::run_period_catchup`'s own "have I caught up?" check
    // (MAX(period_month) >= current_month) sees that row and concludes
    // there's nothing to backfill, even when an earlier month (e.g. the one
    // right before whatever --closed-months covered) still has no row at
    // all. That gap only exists because this tool inserts historical months
    // out of real-time order — production code never hits it, since periods
    // are only ever created forward as time actually passes. So: fill every
    // month any open-path entry needs (plus today's, in case it's somehow
    // still missing) directly, then elapse anything now-past to
    // awaiting_close — the same two phases run_period_catchup does, just
    // driven by "which months do we need" instead of a MAX comparison.
    let current_month = chrono::Local::now().format("%Y-%m").to_string();
    let mut needed_months: std::collections::BTreeSet<String> = open_entries
        .iter()
        .map(|e| e.entry_date[..7].to_string())
        .collect();
    needed_months.insert(current_month.clone());
    for month in &needed_months {
        conn.execute(
            "INSERT OR IGNORE INTO periods (period_month, status) VALUES (?1, 'open')",
            [month],
        )
        .unwrap_or_else(|e| panic!("ensuring period row for {month}: {e}"));
    }
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, period_month FROM periods WHERE status = 'open' AND period_month < ?1",
            )
            .unwrap();
        let elapsed: Vec<(i64, String)> = stmt
            .query_map([&current_month], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        drop(stmt);
        for (id, period_month) in elapsed {
            let ended_at = last_day_of_month(&period_month);
            conn.execute(
                "UPDATE periods SET status = 'awaiting_close', ended_at = ?2 WHERE id = ?1",
                rusqlite::params![id, ended_at],
            )
            .unwrap_or_else(|e| panic!("elapsing period {period_month}: {e}"));
        }
    }

    let open_count = open_entries.len();
    for entry in open_entries {
        let entry_date = entry.entry_date.clone();
        m2_entries::record_entry(
            &conn,
            m2_entries::RecordEntryInput {
                member_id: entry.member_id,
                amount: entry.amount,
                entry_date: entry.entry_date,
            },
        )
        .unwrap_or_else(|e| panic!("recording open-period entry dated {entry_date}: {e}"));
    }

    println!(
        "imported {} member(s), {} closed-month entries across {} month(s), {} open-period entries",
        member_ids.len(),
        closed_entries.values().map(|v| v.len()).sum::<usize>(),
        closed_entries.len(),
        open_count
    );
}
