# Client Test-Data Tool Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the client a double-click Windows .exe (no node/cargo needed) that resets test app data and imports their own test CSV, reusing the app's real member/entry/close logic.

**Architecture:** Extract the CSV-parsing/import/reset logic already written for `bin/import_test_data.rs` into a shared, `Result`-returning library module (`test_data_shared`), so both the existing CLI dev tool and a new native GUI binary (`bin/test_tool.rs`, built with egui/eframe) call the same code. A GitHub Actions workflow, scoped to this branch only, builds the Windows `.exe` on a `windows-latest` runner.

**Tech Stack:** Rust (existing `bvconsole_lib` crate), egui/eframe (GUI), rfd (native file picker + confirm dialogs), GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-19-client-test-tool-design.md`

## Global Constraints

- All commits land on `feature/client-test-tool` only — never `develop` or `main`.
- No new dependency on node/cargo/any dev toolchain for the *client*; you (the developer) still use cargo to build.
- Reuse the app's real `m1_members`/`m2_entries`/`m3_calc`/`m5_close` functions — never hand-roll validation or calculation logic.
- GUI-facing errors must be plain-English message-box text, never a Rust panic/backtrace.
- CSV format is unchanged from `bin/import_test_data.rs`'s existing documented format (header: `member_name, phone, address, email, consent, introducer_phone, amount, entry_date`).

---

### Task 1: Extract pure CSV-parsing helpers into a shared module

**Files:**
- Create: `src-tauri/src/test_data_shared.rs`
- Modify: `src-tauri/src/lib.rs` (add `pub mod test_data_shared;` after the existing `pub mod session;` line)

**Interfaces:**
- Produces: `pub fn parse_csv(content: &str) -> Vec<Vec<String>>`, `pub struct Columns` with `pub fn from_header(header: &[String]) -> Self` and `pub fn get(&self, row: &[String], name: &str) -> Result<String, String>`, `pub fn parse_consent(raw: &str) -> Result<bool, String>`, `pub fn parse_amount(raw: &str) -> Result<i64, String>`, `pub fn last_day_of_month(period_month: &str) -> Result<String, String>`, `pub const IDENTIFIER: &str`, `pub fn default_app_data_dir() -> PathBuf` — all consumed by Task 2 and Task 5.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/test_data_shared.rs` with just the module doc comment and this test module (nothing else compiles yet, which is expected):

```rust
//! Shared logic for the testing-only tools (`bin/import_test_data.rs`,
//! `bin/test_tool.rs`) — CSV parsing, member/entry import, and app-data
//! reset. Returns `Result<_, String>` throughout (rather than panicking)
//! so the GUI tool can show errors in a message box instead of crashing.
//! See docs/superpowers/specs/2026-08-19-client-test-tool-design.md.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_handles_quoted_commas() {
        let rows = parse_csv("name,address\nAlice,\"123 Main St, Apt 4\"\n");
        assert_eq!(
            rows,
            vec![
                vec!["name".to_string(), "address".to_string()],
                vec!["Alice".to_string(), "123 Main St, Apt 4".to_string()],
            ]
        );
    }

    #[test]
    fn parse_amount_converts_rupees_to_cents() {
        assert_eq!(parse_amount("1234.56").unwrap(), 123456);
    }

    #[test]
    fn parse_amount_rejects_garbage() {
        assert!(parse_amount("not-a-number").is_err());
    }

    #[test]
    fn parse_consent_accepts_known_values() {
        assert!(parse_consent("YES").unwrap());
        assert!(!parse_consent("0").unwrap());
        assert!(parse_consent("maybe").is_err());
    }

    #[test]
    fn last_day_of_month_handles_february() {
        assert_eq!(last_day_of_month("2026-02").unwrap(), "2026-02-28");
    }

    #[test]
    fn columns_get_reports_missing_column() {
        let cols = Columns::from_header(&["name".to_string()]);
        let err = cols.get(&["Alice".to_string()], "phone").unwrap_err();
        assert!(err.contains("phone"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail to compile**

Run: `cd src-tauri && cargo test --lib test_data_shared`
Expected: compile error — `parse_csv`, `parse_amount`, etc. not found.

- [ ] **Step 3: Implement the helpers**

Add this above the `#[cfg(test)]` block in `src-tauri/src/test_data_shared.rs`:

```rust
use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveDate;

pub const IDENTIFIER: &str = "com.siddharthpatel.bvconsole"; // src-tauri/tauri.conf.json "identifier"

pub fn default_app_data_dir() -> PathBuf {
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
pub fn parse_csv(content: &str) -> Vec<Vec<String>> {
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

pub struct Columns(HashMap<String, usize>);

impl Columns {
    pub fn from_header(header: &[String]) -> Self {
        Self(
            header
                .iter()
                .enumerate()
                .map(|(i, name)| (name.trim().to_string(), i))
                .collect(),
        )
    }

    pub fn get(&self, row: &[String], name: &str) -> Result<String, String> {
        let idx = *self
            .0
            .get(name)
            .ok_or_else(|| format!("CSV is missing required column '{name}'"))?;
        Ok(row.get(idx).map(|s| s.trim().to_string()).unwrap_or_default())
    }
}

pub fn parse_consent(raw: &str) -> Result<bool, String> {
    match raw.to_lowercase().as_str() {
        "yes" | "true" | "1" => Ok(true),
        "no" | "false" | "0" => Ok(false),
        other => Err(format!("consent value '{other}' isn't one of yes/no/true/false/1/0")),
    }
}

/// Same rupees→×100-fixed-point conversion the app's own entry form does
/// (`src/lib/utils.ts`'s `parseAmountToCents`).
pub fn parse_amount(raw: &str) -> Result<i64, String> {
    let rupees: f64 = raw.parse().map_err(|_| format!("amount '{raw}' isn't a valid number"))?;
    Ok((rupees * 100.0).round() as i64)
}

pub fn last_day_of_month(period_month: &str) -> Result<String, String> {
    let start = NaiveDate::parse_from_str(&format!("{period_month}-01"), "%Y-%m-%d")
        .map_err(|_| format!("period_month '{period_month}' isn't YYYY-MM"))?;
    start
        .checked_add_months(chrono::Months::new(1))
        .and_then(|next| next.pred_opt())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .ok_or_else(|| format!("computing last day of {period_month}"))
}
```

Add to `src-tauri/src/lib.rs`, right after `pub mod session;`:

```rust
pub mod test_data_shared;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib test_data_shared`
Expected: 6 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/test_data_shared.rs src-tauri/src/lib.rs
git commit -m "feat: extract CSV-parsing helpers into test_data_shared module"
```

---

### Task 2: Add import/reset orchestration to the shared module, rewire `import_test_data.rs`

**Files:**
- Modify: `src-tauri/src/test_data_shared.rs` (append orchestration functions)
- Modify: `src-tauri/src/bin/import_test_data.rs` (delete the code now in `test_data_shared`, call the shared functions instead)

**Interfaces:**
- Consumes: everything from Task 1 (`parse_csv`, `Columns`, `parse_consent`, `parse_amount`, `last_day_of_month`, `default_app_data_dir`).
- Produces: `pub fn unlock_db(app_data_dir: &Path, credential: &str) -> Result<rusqlite::Connection, String>`, `pub struct ImportSummary { pub members: usize, pub closed_entries: usize, pub closed_months: usize, pub open_entries: usize }`, `pub fn import_csv(conn: &rusqlite::Connection, csv_content: &str, closed_months: &HashSet<String>) -> Result<ImportSummary, String>`, `pub fn reset_data(app_data_dir: &Path) -> Result<Vec<PathBuf>, String>` — all consumed by Task 5 (GUI wiring) and this task's own `import_test_data.rs` rewrite.

- [ ] **Step 1: Add the orchestration functions to `test_data_shared.rs`**

Add these imports to the top of `src-tauri/src/test_data_shared.rs` (alongside the existing `use` lines from Task 1):

```rust
use std::collections::HashSet;
use std::path::Path;

use crate::m8_auth::crypto::{sqlcipher_raw_key_pragma, unwrap_master_key};
use crate::m8_auth::store::AuthStore;
use crate::{db, m1_members, m2_entries, m3_calc, m5_close};
```

Append this below `last_day_of_month` (still above the `#[cfg(test)]` block):

```rust
pub fn unlock_db(app_data_dir: &Path, credential: &str) -> Result<rusqlite::Connection, String> {
    let db_path = app_data_dir.join("console.db");
    let auth_path = app_data_dir.join("auth.json");
    if !auth_path.exists() || !db_path.exists() {
        return Err(format!(
            "no app data found at {} — run the app once and complete first-run setup first",
            app_data_dir.display()
        ));
    }

    let auth = AuthStore::load(&auth_path).map_err(|e| format!("reading auth.json: {e}"))?;
    let master_key = [&auth.pin_envelope, &auth.password_envelope]
        .into_iter()
        .flatten()
        .find_map(|envelope| unwrap_master_key(credential, envelope))
        .ok_or_else(|| "PIN/password didn't match the one set up in the app".to_string())?;
    db::open_encrypted(&db_path, &sqlcipher_raw_key_pragma(&master_key))
        .map_err(|e| format!("opening console.db: {e}"))
}

struct PendingEntry {
    member_id: i64,
    amount: i64,
    entry_date: String,
}

fn import_closed_month(
    conn: &rusqlite::Connection,
    period_month: &str,
    entries: &[PendingEntry],
) -> Result<(), String> {
    let last_day = last_day_of_month(period_month)?;
    conn.execute(
        "INSERT INTO periods (period_month, status, ended_at, closed_at) VALUES (?1, 'closed', ?2, ?2)",
        rusqlite::params![period_month, last_day],
    )
    .map_err(|e| format!("inserting closed period {period_month}: {e}"))?;
    let period_id = conn.last_insert_rowid();

    for entry in entries {
        conn.execute(
            "INSERT INTO business_volume_entries (member_id, amount, entry_date, period_month, created_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            rusqlite::params![entry.member_id, entry.amount, entry.entry_date, period_month],
        )
        .map_err(|e| format!("inserting closed-month entry: {e}"))?;
        m3_calc::recalculate_chain(conn, entry.member_id, period_id)
            .map_err(|e| format!("recalculating chain for closed month {period_month}: {e}"))?;
    }

    m5_close::write_period_close_snapshots(conn, period_id, &last_day)
        .map_err(|e| format!("writing snapshots for {period_month}: {e}"))?;
    m5_close::zero_period_totals(conn, period_id)
        .map_err(|e| format!("zeroing totals for {period_month}: {e}"))?;
    Ok(())
}

pub struct ImportSummary {
    pub members: usize,
    pub closed_entries: usize,
    pub closed_months: usize,
    pub open_entries: usize,
}

pub fn import_csv(
    conn: &rusqlite::Connection,
    csv_content: &str,
    closed_months: &HashSet<String>,
) -> Result<ImportSummary, String> {
    let mut rows = parse_csv(csv_content);
    if rows.is_empty() {
        return Err("CSV is empty".to_string());
    }
    let header = rows.remove(0);
    let cols = Columns::from_header(&header);

    let mut member_ids: HashMap<String, i64> = HashMap::new();
    let mut open_entries: Vec<PendingEntry> = Vec::new();
    let mut closed_entries: HashMap<String, Vec<PendingEntry>> = HashMap::new();

    for (i, row) in rows.iter().enumerate() {
        let line = i + 2; // header is line 1
        let phone = cols.get(row, "phone")?;
        if phone.is_empty() {
            return Err(format!("line {line}: phone is required"));
        }

        let member_id = if let Some(&id) = member_ids.get(&phone) {
            id
        } else {
            let name = cols.get(row, "member_name")?;
            let address = cols.get(row, "address")?;
            let email = cols.get(row, "email")?;
            let email = if email.is_empty() { None } else { Some(email) };
            let consent_given = parse_consent(&cols.get(row, "consent")?)?;
            let introducer_phone = cols.get(row, "introducer_phone")?;

            let id = if introducer_phone.is_empty() {
                let outcome = m1_members::create_root_member(
                    conn,
                    m1_members::CreateRootMemberInput {
                        name,
                        phone: phone.clone(),
                        address,
                        email,
                        consent_given,
                    },
                )
                .map_err(|e| format!("line {line}: creating root member: {e}"))?;
                outcome.id
            } else {
                let introducer_id = *member_ids.get(&introducer_phone).ok_or_else(|| {
                    format!(
                        "line {line}: introducer phone {introducer_phone} hasn't appeared yet — \
                         list every introducer's own row before the members it introduced"
                    )
                })?;
                match m1_members::add_member(
                    conn,
                    m1_members::AddMemberInput {
                        name,
                        phone: phone.clone(),
                        address,
                        email,
                        consent_given,
                        introducer_member_id: introducer_id,
                    },
                )
                .map_err(|e| format!("line {line}: adding member: {e}"))?
                {
                    m1_members::AddMemberOutcome::Created { member, .. } => member.id,
                    m1_members::AddMemberOutcome::ReactivationOffer { existing_member } => {
                        return Err(format!(
                            "line {line}: phone {phone} already belongs to inactive member #{} — \
                             unexpected in a fresh import",
                            existing_member.id
                        ));
                    }
                }
            };
            member_ids.insert(phone.clone(), id);
            id
        };

        let amount = parse_amount(&cols.get(row, "amount")?)?;
        let entry_date = cols.get(row, "entry_date")?;
        NaiveDate::parse_from_str(&entry_date, "%Y-%m-%d")
            .map_err(|_| format!("line {line}: entry_date '{entry_date}' isn't YYYY-MM-DD"))?;
        let period_month = entry_date[..7].to_string();

        let entry = PendingEntry { member_id, amount, entry_date };
        if closed_months.contains(&period_month) {
            closed_entries.entry(period_month).or_default().push(entry);
        } else {
            open_entries.push(entry);
        }
    }

    let mut sorted_closed_months: Vec<&String> = closed_entries.keys().collect();
    sorted_closed_months.sort();
    for period_month in &sorted_closed_months {
        import_closed_month(conn, period_month, &closed_entries[*period_month])?;
    }

    let current_month = chrono::Local::now().format("%Y-%m").to_string();
    let mut needed_months: std::collections::BTreeSet<String> =
        open_entries.iter().map(|e| e.entry_date[..7].to_string()).collect();
    needed_months.insert(current_month.clone());
    for month in &needed_months {
        conn.execute(
            "INSERT OR IGNORE INTO periods (period_month, status) VALUES (?1, 'open')",
            [month],
        )
        .map_err(|e| format!("ensuring period row for {month}: {e}"))?;
    }
    {
        let mut stmt = conn
            .prepare("SELECT id, period_month FROM periods WHERE status = 'open' AND period_month < ?1")
            .map_err(|e| format!("querying open periods: {e}"))?;
        let elapsed: Vec<(i64, String)> = stmt
            .query_map([&current_month], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| format!("querying open periods: {e}"))?
            .collect::<Result<_, _>>()
            .map_err(|e| format!("reading open periods: {e}"))?;
        drop(stmt);
        for (id, period_month) in elapsed {
            let ended_at = last_day_of_month(&period_month)?;
            conn.execute(
                "UPDATE periods SET status = 'awaiting_close', ended_at = ?2 WHERE id = ?1",
                rusqlite::params![id, ended_at],
            )
            .map_err(|e| format!("elapsing period {period_month}: {e}"))?;
        }
    }

    let open_count = open_entries.len();
    for entry in open_entries {
        let entry_date = entry.entry_date.clone();
        m2_entries::record_entry(
            conn,
            m2_entries::RecordEntryInput {
                member_id: entry.member_id,
                amount: entry.amount,
                entry_date: entry.entry_date,
            },
        )
        .map_err(|e| format!("recording open-period entry dated {entry_date}: {e}"))?;
    }

    Ok(ImportSummary {
        members: member_ids.len(),
        closed_entries: closed_entries.values().map(|v| v.len()).sum(),
        closed_months: closed_entries.len(),
        open_entries: open_count,
    })
}

pub fn reset_data(app_data_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let targets = ["console.db", "console.db-wal", "console.db-shm", "backups-manifest.json", "backups"]
        .map(|name| app_data_dir.join(name));
    let mut deleted = Vec::new();
    for path in targets {
        if path.exists() {
            if path.is_dir() {
                std::fs::remove_dir_all(&path).map_err(|e| format!("deleting {}: {e}", path.display()))?;
            } else {
                std::fs::remove_file(&path).map_err(|e| format!("deleting {}: {e}", path.display()))?;
            }
            deleted.push(path);
        }
    }
    Ok(deleted)
}
```

- [ ] **Step 2: Rewrite `import_test_data.rs` to use the shared module**

Replace the whole file `src-tauri/src/bin/import_test_data.rs` with:

```rust
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
```

- [ ] **Step 3: Run tests and confirm both binaries build**

Run: `cd src-tauri && cargo test --lib test_data_shared && cargo build --bin import_test_data`
Expected: tests still pass; `import_test_data` compiles cleanly.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/test_data_shared.rs src-tauri/src/bin/import_test_data.rs
git commit -m "feat: move import/reset orchestration into test_data_shared, rewire import_test_data.rs"
```

---

### Task 3: Scaffold the `test_tool` GUI shell

**Files:**
- Modify: `src-tauri/Cargo.toml` (add `eframe`, `egui`, `rfd` dependencies)
- Create: `src-tauri/src/bin/test_tool.rs`

**Interfaces:**
- Consumes: `test_data_shared::default_app_data_dir` (Task 1).
- Produces: `TestToolApp` struct with fields `status: String`, `credential: String`, `closed_months_input: String`, `pending_csv: Option<PathBuf>` — Task 4 and Task 5 add behavior to its existing buttons.

- [ ] **Step 1: Add GUI dependencies**

Run: `cd src-tauri && cargo add eframe egui rfd`
Expected: `Cargo.toml`'s `[dependencies]` gains `eframe`, `egui`, `rfd` at whatever current stable versions `cargo add` resolves.

- [ ] **Step 2: Create the GUI shell**

Create `src-tauri/src/bin/test_tool.rs`:

```rust
// Testing-only tool. GUI wrapper around test_data_shared's reset/import
// logic for a client with no dev toolchain — a double-click .exe, no
// terminal, no cargo/node required on their machine.
// See docs/superpowers/specs/2026-08-19-client-test-tool-design.md.
//
// Not registered anywhere (no `lib.rs`/`commands.rs` change beyond
// `test_data_shared`, shared with `import_test_data.rs`). Delete this
// file (and test_data_shared.rs, if import_test_data.rs is also gone)
// to remove the tool.
use std::path::PathBuf;

use bvconsole_lib::test_data_shared;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        "BV Console — Test Tool",
        options,
        Box::new(|_cc| Ok(Box::new(TestToolApp::default()))),
    )
}

#[derive(Default)]
struct TestToolApp {
    status: String,
    credential: String,
    closed_months_input: String,
    pending_csv: Option<PathBuf>,
}

impl eframe::App for TestToolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BV Console — Test Tool");
            ui.label("Close the main app before using this tool.");
            ui.separator();

            if ui.button("Reset Test Data").clicked() {
                self.status = "reset not wired yet".to_string();
            }

            ui.separator();

            if ui.button("Import CSV...").clicked() {
                self.status = "import not wired yet".to_string();
            }

            ui.separator();
            ui.label(&self.status);
        });
    }
}
```

- [ ] **Step 3: Build and manually verify the shell**

Run: `cd src-tauri && cargo build --bin test_tool`
Expected: builds cleanly.

Run: `cd src-tauri && cargo run --bin test_tool`
Expected: a window titled "BV Console — Test Tool" opens with two buttons; clicking either sets the status label to its placeholder text. Close the window when done.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/bin/test_tool.rs
git commit -m "feat: scaffold test_tool GUI shell with egui/eframe"
```

---

### Task 4: Wire the Reset button

**Files:**
- Modify: `src-tauri/src/bin/test_tool.rs`

**Interfaces:**
- Consumes: `test_data_shared::default_app_data_dir()`, `test_data_shared::reset_data(&Path) -> Result<Vec<PathBuf>, String>` (Task 2).

- [ ] **Step 1: Replace the Reset button's placeholder handler**

In `src-tauri/src/bin/test_tool.rs`, replace:

```rust
            if ui.button("Reset Test Data").clicked() {
                self.status = "reset not wired yet".to_string();
            }
```

with:

```rust
            if ui.button("Reset Test Data").clicked() {
                let confirmed = rfd::MessageDialog::new()
                    .set_title("Reset Test Data")
                    .set_description(
                        "This deletes console.db and backups (keeps your PIN/password). \
                         Close the main app first. Continue?",
                    )
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show();
                if confirmed == rfd::MessageDialogResult::Yes {
                    let app_data_dir = test_data_shared::default_app_data_dir();
                    self.status = match test_data_shared::reset_data(&app_data_dir) {
                        Ok(deleted) if deleted.is_empty() => {
                            "nothing to reset — no app data found".to_string()
                        }
                        Ok(deleted) => format!(
                            "reset done — {} item(s) removed. Log in with your PIN/password to start fresh.",
                            deleted.len()
                        ),
                        Err(e) => format!("reset failed: {e}"),
                    };
                }
            }
```

`rfd::MessageDialogResult`'s exact variant names can differ slightly between rfd versions — if this doesn't compile as written, check the installed version's docs (`cargo doc --open -p rfd` or docs.rs for the version in `Cargo.lock`) and adjust the comparison to that version's actual enum.

- [ ] **Step 2: Build and manually verify**

Run: `cd src-tauri && cargo build --bin test_tool`
Expected: builds cleanly (adjust the `MessageDialogResult` match if the installed rfd version's API differs, per the note above).

Run: `cd src-tauri && cargo run --bin test_tool`, click "Reset Test Data", click "No" in the confirm dialog.
Expected: status label stays unchanged (no deletion happened).

Run it again against a real app-data dir (see Task 7 for how to set one up), click "Reset Test Data", click "Yes".
Expected: status shows "reset done — N item(s) removed...", and `console.db`/`console.db-wal`/`console.db-shm`/`backups-manifest.json`/`backups/` are gone from the app-data dir while `auth.json` remains.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/bin/test_tool.rs
git commit -m "feat: wire Reset Test Data button to test_data_shared::reset_data"
```

---

### Task 5: Wire the Import button

**Files:**
- Modify: `src-tauri/src/bin/test_tool.rs`

**Interfaces:**
- Consumes: `test_data_shared::default_app_data_dir()`, `test_data_shared::unlock_db`, `test_data_shared::import_csv`, `test_data_shared::ImportSummary` (Task 2).

- [ ] **Step 1: Replace the Import button's placeholder handler**

In `src-tauri/src/bin/test_tool.rs`, replace:

```rust
            if ui.button("Import CSV...").clicked() {
                self.status = "import not wired yet".to_string();
            }
```

with:

```rust
            if ui.button("Choose CSV...").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).pick_file() {
                    self.pending_csv = Some(path);
                }
            }
            ui.label(match &self.pending_csv {
                Some(p) => format!("Selected: {}", p.display()),
                None => "No CSV selected".to_string(),
            });

            ui.label("PIN/password:");
            ui.add(egui::TextEdit::singleline(&mut self.credential).password(true));

            ui.label("Closed months (comma-separated YYYY-MM, optional):");
            ui.text_edit_singleline(&mut self.closed_months_input);

            if ui.button("Run Import").clicked() {
                self.status = run_import(&self.credential, &self.pending_csv, &self.closed_months_input);
            }
```

Add this free function below the `TestToolApp` impl block, at the bottom of the file:

```rust
fn run_import(credential: &str, csv_path: &Option<PathBuf>, closed_months_input: &str) -> String {
    let Some(csv_path) = csv_path else {
        return "pick a CSV file first".to_string();
    };
    let content = match std::fs::read_to_string(csv_path) {
        Ok(c) => c,
        Err(e) => return format!("reading {}: {e}", csv_path.display()),
    };
    let closed_months: std::collections::HashSet<String> = closed_months_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let app_data_dir = test_data_shared::default_app_data_dir();
    let conn = match test_data_shared::unlock_db(&app_data_dir, credential) {
        Ok(c) => c,
        Err(e) => return format!("unlock failed: {e}"),
    };
    match test_data_shared::import_csv(&conn, &content, &closed_months) {
        Ok(summary) => format!(
            "imported {} member(s), {} closed-month entries across {} month(s), {} open-period entries",
            summary.members, summary.closed_entries, summary.closed_months, summary.open_entries
        ),
        Err(e) => format!("import failed: {e}"),
    }
}
```

- [ ] **Step 2: Build and manually verify**

Run: `cd src-tauri && cargo build --bin test_tool`
Expected: builds cleanly.

Run: `cd src-tauri && cargo run --bin test_tool` against a reset (or freshly first-run) app-data dir. Click "Run Import" with no CSV selected.
Expected: status shows "pick a CSV file first".

Choose a small valid CSV (see Task 7 for a sample), enter the app's PIN/password, click "Run Import".
Expected: status shows "imported N member(s), ...", matching the counts in the CSV.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/bin/test_tool.rs
git commit -m "feat: wire Import CSV button to test_data_shared::unlock_db/import_csv"
```

---

### Task 6: CI workflow to build the Windows exe

**Files:**
- Create: `.github/workflows/build-test-tool-windows.yml`

**Interfaces:** none (standalone CI config).

- [ ] **Step 1: Add the workflow file**

Create `.github/workflows/build-test-tool-windows.yml`:

```yaml
# Builds test_tool.exe for the client. Scoped to feature/client-test-tool
# only — this branch (and this file) never merges to develop or main.
name: Build test tool (Windows)

on:
  push:
    branches: [feature/client-test-tool]
  workflow_dispatch: {}

jobs:
  build:
    runs-on: windows-latest
    defaults:
      run:
        working-directory: src-tauri
    steps:
      - uses: actions/checkout@v5

      - uses: dtolnay/rust-toolchain@1.97.1

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Build test_tool.exe
        run: cargo build --release --bin test_tool

      - uses: actions/upload-artifact@v4
        with:
          name: test_tool-windows
          path: src-tauri/target/release/test_tool.exe
```

- [ ] **Step 2: Push and verify the run**

Run: `git push -u origin feature/client-test-tool`
Expected: pushing triggers the workflow (visible under the repo's Actions tab). Watch it to completion.

If it fails compiling `rusqlite`'s bundled-sqlcipher-vendored-openssl feature on Windows (needs a C toolchain + Perl to build OpenSSL from source), add the missing prerequisite as a step here — GitHub's `windows-latest` image normally ships MSVC and Strawberry Perl already, so this is only expected to bite if that assumption is wrong. Verify by reading the actual failure in the Actions log rather than pre-guessing which tool is missing.

If it succeeds: download the `test_tool-windows` artifact from the run and confirm it contains a working `test_tool.exe`.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/build-test-tool-windows.yml
git commit -m "ci: build test_tool.exe on windows-latest for feature/client-test-tool"
```

---

### Task 7: Manual end-to-end verification on macOS

**Files:** none — verification only.

- [ ] **Step 1: Set up a scratch app-data dir via a real first run**

Run the main app once (`npm run tauri dev` or the built app) and complete first-run PIN/password setup. Note the PIN/password you chose — you'll need it below. Confirm `~/Library/Application Support/com.siddharthpatel.bvconsole/` now has `auth.json` and `console.db`.

- [ ] **Step 2: Prepare a sample CSV**

Create `/tmp/test-import.csv`:

```csv
member_name,phone,address,email,consent,introducer_phone,amount,entry_date
Root Person,5550000001,1 Main St,root@example.com,yes,,1000.00,2026-08-01
Child Person,5550000002,2 Main St,child@example.com,yes,5550000001,500.50,2026-08-02
```

- [ ] **Step 3: Run the GUI tool end-to-end**

Quit the main app (db file lock). Run: `cd src-tauri && cargo run --bin test_tool`

Click "Reset Test Data" → "Yes". Verify `console.db`/`console.db-wal`/`console.db-shm`/`backups-manifest.json`/`backups/` are gone, `auth.json` remains.

Relaunch the main app, log in with the PIN/password from Step 1. Confirm it opens straight to a blank/empty state (no members), not first-run setup.

Quit the main app again. Back in the GUI tool: "Choose CSV..." → pick `/tmp/test-import.csv`, enter the PIN/password, click "Run Import". Confirm the status shows "imported 2 member(s), 0 closed-month entries across 0 month(s), 2 open-period entries".

- [ ] **Step 4: Verify the imported data in the real app**

Relaunch the main app, log in. Confirm "Root Person" and "Child Person" both appear with the right phone/amount/relationship. No commit for this task — it's verification only, confirming Tasks 1–5 work together against the real app.
