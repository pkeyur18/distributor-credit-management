//! Shared logic for the testing-only tools (`bin/import_test_data.rs`,
//! `bin/test_tool.rs`) — CSV parsing, member/entry import, and app-data
//! reset. Returns `Result<_, String>` throughout (rather than panicking)
//! so the GUI tool can show errors in a message box instead of crashing.
//! See docs/superpowers/specs/2026-08-19-client-test-tool-design.md.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::m8_auth::crypto::{sqlcipher_raw_key_pragma, unwrap_master_key};
use crate::m8_auth::store::AuthStore;
use crate::{db, m1_members, m2_entries, m3_calc, m5_close};

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
