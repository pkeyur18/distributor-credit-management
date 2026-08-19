//! Shared logic for the testing-only tools (`bin/import_test_data.rs`,
//! `bin/test_tool.rs`) — CSV parsing, member/entry import, and app-data
//! reset. Returns `Result<_, String>` throughout (rather than panicking)
//! so the GUI tool can show errors in a message box instead of crashing.
//! See docs/superpowers/specs/2026-08-19-client-test-tool-design.md.

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
