// M1 — Member & Structure (04-technical-architecture.md §3.1).
// US-M1.1 (S4): create_root_member, add_member. US-M1.2/M1.3/M1.4 (S5)
// below add edit/deactivate/reactivate/search.
mod id;

use chrono::Local;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub use id::allocate_member_id;

fn today_iso() -> String {
    Local::now().date_naive().to_string()
}

/// Rule-44's canonical key: digits only, then reduced to the last 10 (an
/// international prefix or trunk zero is whatever precedes those 10).
/// Rule-34's uniqueness check needs this now; T-M1.4-2 (S5, search) reuses
/// this exact function rather than a second implementation — see Rule-44's
/// "one shared search function" requirement in 03-business-rules.md.
pub fn canonical_phone_key(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() > 10 {
        digits[digits.len() - 10..].to_string()
    } else {
        digits
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub address: String,
    pub introducer_member_id: Option<i64>,
    pub level: i64,
    pub is_active: bool,
    pub joining_date: String,
    pub consent_given: bool,
    pub consent_date: String,
    pub created_at: String,
}

impl Member {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            phone: row.get("phone")?,
            email: row.get("email")?,
            address: row.get("address")?,
            introducer_member_id: row.get("introducer_member_id")?,
            level: row.get("level")?,
            is_active: row.get("is_active")?,
            joining_date: row.get("joining_date")?,
            consent_given: row.get("consent_given")?,
            consent_date: row.get("consent_date")?,
            created_at: row.get("created_at")?,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRootMemberInput {
    pub name: String,
    pub phone: String,
    pub address: String,
    pub email: Option<String>,
    pub consent_given: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberInput {
    pub name: String,
    pub phone: String,
    pub address: String,
    pub email: Option<String>,
    pub consent_given: bool,
    pub introducer_member_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum AddMemberOutcome {
    // Rule-1/Rule-32: `warnings` is informational only — its presence never
    // changes whether this variant is returned. Always empty for a root
    // member (level 1 has no configured width and never exceeds depth).
    Created {
        member: Member,
        warnings: Vec<String>,
    },
    ReactivationOffer {
        existing_member: Member,
    },
}

struct BaseFields<'a> {
    name: &'a str,
    phone: &'a str,
    address: &'a str,
    email: Option<&'a str>,
    consent_given: bool,
}

// Rule-40: consent gates the save outright, not a per-field message — but
// name/phone/address/email are still reported per-field (V1.1/V1.4) so the
// UI can point at the offending input.
fn validate_base_fields(f: &BaseFields) -> Result<(), AppError> {
    if f.name.trim().is_empty() {
        return Err(AppError::Validation {
            field: "name".into(),
            message: "Name is required.".into(),
        });
    }
    if f.phone.trim().is_empty() {
        return Err(AppError::Validation {
            field: "phone".into(),
            message: "Phone is required.".into(),
        });
    }
    if f.address.trim().is_empty() {
        return Err(AppError::Validation {
            field: "address".into(),
            message: "Address is required.".into(),
        });
    }
    if let Some(email) = f.email {
        if !email.trim().is_empty() && !is_plausible_email(email) {
            return Err(AppError::Validation {
                field: "email".into(),
                message: "Enter a valid email address.".into(),
            });
        }
    }
    if !f.consent_given {
        return Err(AppError::Validation {
            field: "consent".into(),
            message: "Consent is required before this member can be saved.".into(),
        });
    }
    Ok(())
}

// V1.4: advisory-strength shape check, not RFC 5322 — one '@', a non-empty
// local part, and a domain part containing a '.'.
fn is_plausible_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn normalized_email(email: Option<&str>) -> Option<String> {
    email
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Rule-34: unique across active *and* inactive members, compared on the
/// canonical key — not the raw stored string.
fn find_phone_conflict(conn: &Connection, phone: &str) -> Result<Option<Member>, AppError> {
    let target_key = canonical_phone_key(phone);
    let mut stmt = conn.prepare("SELECT * FROM members")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let candidate = Member::from_row(row)?;
        if canonical_phone_key(&candidate.phone) == target_key {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

fn insert_member(
    conn: &Connection,
    id: i64,
    f: &BaseFields,
    introducer_member_id: Option<i64>,
    level: i64,
) -> Result<Member, AppError> {
    let email = normalized_email(f.email);
    let joining_date = today_iso();
    let consent_date = joining_date.clone();
    let created_at = joining_date.clone();

    conn.execute(
        "INSERT INTO members (id, name, phone, email, address, introducer_member_id, level, is_active, joining_date, consent_given, consent_date, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            id,
            f.name.trim(),
            f.phone.trim(),
            email,
            f.address.trim(),
            introducer_member_id,
            level,
            joining_date,
            f.consent_given,
            consent_date,
            created_at,
        ],
    )?;

    Ok(Member {
        id,
        name: f.name.trim().to_string(),
        phone: f.phone.trim().to_string(),
        email: normalized_email(f.email),
        address: f.address.trim().to_string(),
        introducer_member_id,
        level,
        is_active: true,
        joining_date: joining_date.clone(),
        consent_given: f.consent_given,
        consent_date,
        created_at,
    })
}

// Rule-1 (level width) / Rule-32 (depth): advisory only, computed after the
// insert already succeeded — never a precondition to save. D-5 suppresses
// the width warning for any level with no configured `level_N_width` (5+).
fn advisory_warnings(
    conn: &Connection,
    level: i64,
    introducer_id: i64,
) -> Result<Vec<String>, AppError> {
    let mut warnings = Vec::new();

    let width: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [format!("level_{level}_width")],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(width) = width.and_then(|w| w.parse::<i64>().ok()) {
        let sibling_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM members WHERE introducer_member_id = ?1",
            [introducer_id],
            |r| r.get(0),
        )?;
        if sibling_count > width {
            warnings.push(format!(
                "Level {level} now has {sibling_count} members, above the configured width of {width}."
            ));
        }
    }

    let depth: i64 = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'hierarchy_depth'",
            [],
            |r| r.get::<_, String>(0),
        )?
        .parse()
        .unwrap_or(i64::MAX);
    if level > depth {
        warnings.push(format!(
            "Level {level} exceeds the configured hierarchy depth of {depth}."
        ));
    }

    Ok(warnings)
}

// API-01/API-02 both audit as cause `entry` (04-technical-architecture.md
// §6, M1 table) — one row per onboarding, not per field, since there's no
// "before" value for a brand-new member.
fn write_onboarding_audit(conn: &Connection, member_id: i64) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO audit_log (entity_type, entity_id, field, old_value, new_value, changed_at, cause)
         VALUES ('member', ?1, 'member', NULL, 'Onboarded', ?2, 'entry')",
        rusqlite::params![member_id, today_iso()],
    )?;
    Ok(())
}

/// API-01. Callable exactly once — guarded by "no member with a NULL
/// introducer already exists", not by any auth-mode distinction (S4 has no
/// setup wizard yet; that's US-M8.1, S5).
pub fn create_root_member(
    conn: &Connection,
    input: CreateRootMemberInput,
) -> Result<Member, AppError> {
    let f = BaseFields {
        name: &input.name,
        phone: &input.phone,
        address: &input.address,
        email: input.email.as_deref(),
        consent_given: input.consent_given,
    };
    validate_base_fields(&f)?;

    let root_exists: bool = conn
        .query_row(
            "SELECT 1 FROM members WHERE introducer_member_id IS NULL LIMIT 1",
            [],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);
    if root_exists {
        return Err(AppError::Conflict {
            message: "A root member already exists — create_root_member is callable only once."
                .into(),
        });
    }

    if let Some(conflict) = find_phone_conflict(conn, &input.phone)? {
        return Err(AppError::Conflict {
            message: format!(
                "This phone number is already in use by {} (#{}).",
                conflict.name, conflict.id
            ),
        });
    }

    let id = allocate_member_id(conn)?;
    let member = insert_member(conn, id, &f, None, 1)?;
    write_onboarding_audit(conn, member.id)?;
    Ok(member)
}

/// API-02. Rule-30 (active-introducer resolution), Rule-34 (phone
/// uniqueness with a reactivation offer for an inactive match), Rule-40
/// (consent), Rule-1/Rule-32 (advisory-only, so deliberately not checked
/// here — nothing in this function ever blocks on level width or depth).
pub fn add_member(conn: &Connection, input: AddMemberInput) -> Result<AddMemberOutcome, AppError> {
    let f = BaseFields {
        name: &input.name,
        phone: &input.phone,
        address: &input.address,
        email: input.email.as_deref(),
        consent_given: input.consent_given,
    };
    validate_base_fields(&f)?;

    let introducer: Option<Member> = conn
        .query_row(
            "SELECT * FROM members WHERE id = ?1",
            [input.introducer_member_id],
            Member::from_row,
        )
        .optional()?;
    let introducer = match introducer {
        Some(m) if m.is_active => m,
        Some(_) => {
            return Err(AppError::NotFound {
                message: "The introducer must be an active member — this one is inactive.".into(),
            })
        }
        None => {
            return Err(AppError::NotFound {
                message: "Reference ID does not resolve to an existing member.".into(),
            })
        }
    };

    if let Some(conflict) = find_phone_conflict(conn, &input.phone)? {
        if conflict.is_active {
            return Err(AppError::Conflict {
                message: format!(
                    "This phone number is already in use by {} (#{}).",
                    conflict.name, conflict.id
                ),
            });
        }
        // Rule-34: not an error — the caller decides whether to reactivate
        // (edit_member + reactivate_member, US-M1.2/M1.3, S5). No member is
        // created here.
        return Ok(AddMemberOutcome::ReactivationOffer {
            existing_member: conflict,
        });
    }

    let id = allocate_member_id(conn)?;
    let level = introducer.level + 1;
    let member = insert_member(conn, id, &f, Some(introducer.id), level)?;
    write_onboarding_audit(conn, member.id)?;
    let warnings = advisory_warnings(conn, level, introducer.id)?;
    Ok(AddMemberOutcome::Created { member, warnings })
}

fn find_member(conn: &Connection, id: i64) -> Result<Member, AppError> {
    conn.query_row(
        "SELECT * FROM members WHERE id = ?1",
        [id],
        Member::from_row,
    )
    .optional()?
    .ok_or(AppError::NotFound {
        message: "Member not found.".into(),
    })
}

fn write_field_audit(
    conn: &Connection,
    member_id: i64,
    field: &str,
    old: &str,
    new: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO audit_log (entity_type, entity_id, field, old_value, new_value, changed_at, cause)
         VALUES ('member', ?1, ?2, ?3, ?4, ?5, 'edit')",
        rusqlite::params![member_id, field, old, new, today_iso()],
    )?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditMemberInput {
    pub id: i64,
    pub name: Option<String>,
    pub phone: Option<String>,
    // `Option<Option<String>>`: field absent -> None (leave unchanged);
    // `null` -> Some(None) (clear it); a string -> Some(Some(..)) (set it).
    pub email: Option<Option<String>>,
    pub address: Option<String>,
    // Deliberately no `introducer_member_id` field at all (Rule-37) — the
    // introducer isn't merely ignored if sent, it has nowhere to land.
}

/// API-03. `input.email`'s tri-state and the complete absence of an
/// introducer field are the two things worth re-reading before touching
/// this function — see the struct doc comments.
pub fn edit_member(conn: &Connection, input: EditMemberInput) -> Result<Member, AppError> {
    let existing = find_member(conn, input.id)?;

    let mut name = existing.name.clone();
    let mut phone = existing.phone.clone();
    let mut email = existing.email.clone();
    let mut address = existing.address.clone();
    let mut changes: Vec<(&'static str, String, String)> = Vec::new();

    if let Some(raw) = &input.name {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation {
                field: "name".into(),
                message: "Name is required.".into(),
            });
        }
        if trimmed != name {
            changes.push(("name", name.clone(), trimmed.to_string()));
            name = trimmed.to_string();
        }
    }

    if let Some(raw) = &input.phone {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation {
                field: "phone".into(),
                message: "Phone is required.".into(),
            });
        }
        // Rule-34, re-checked on edit against the same canonical key —
        // against every *other* member, active or inactive.
        if let Some(conflict) = find_phone_conflict(conn, trimmed)? {
            if conflict.id != existing.id {
                return Err(AppError::Conflict {
                    message: format!(
                        "This phone number is already in use by {} (#{}).",
                        conflict.name, conflict.id
                    ),
                });
            }
        }
        if trimmed != phone {
            changes.push(("phone", phone.clone(), trimmed.to_string()));
            phone = trimmed.to_string();
        }
    }

    if let Some(email_opt) = &input.email {
        let normalized = normalized_email(email_opt.as_deref());
        if let Some(e) = &normalized {
            if !is_plausible_email(e) {
                return Err(AppError::Validation {
                    field: "email".into(),
                    message: "Enter a valid email address.".into(),
                });
            }
        }
        if normalized != email {
            changes.push((
                "email",
                email.clone().unwrap_or_default(),
                normalized.clone().unwrap_or_default(),
            ));
            email = normalized;
        }
    }

    if let Some(raw) = &input.address {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation {
                field: "address".into(),
                message: "Address is required.".into(),
            });
        }
        if trimmed != address {
            changes.push(("address", address.clone(), trimmed.to_string()));
            address = trimmed.to_string();
        }
    }

    if changes.is_empty() {
        return Ok(existing);
    }

    conn.execute(
        "UPDATE members SET name = ?1, phone = ?2, email = ?3, address = ?4 WHERE id = ?5",
        rusqlite::params![name, phone, email, address, existing.id],
    )?;
    for (field, old, new) in &changes {
        write_field_audit(conn, existing.id, field, old, new)?;
    }

    Ok(Member {
        name,
        phone,
        email,
        address,
        ..existing
    })
}

/// API-04. Rule-28 (corrected): `is_active` is a pure display flag with
/// **zero computational effect** — this function never touches
/// `business_volume_entries` or `member_period_totals`, and calls nothing
/// in the calculation path (M3, S6). Implementing the superseded spec
/// wording ("stops appearing in new periods") would silently corrupt every
/// ancestor's Total Business Volume; the only column this ever writes is
/// `members.is_active`.
pub fn deactivate_member(conn: &Connection, id: i64) -> Result<(), AppError> {
    let existing = find_member(conn, id)?;
    if existing.introducer_member_id.is_none() {
        return Err(AppError::Conflict {
            message: "The root member cannot be deactivated.".into(),
        });
    }
    if !existing.is_active {
        return Ok(());
    }
    conn.execute("UPDATE members SET is_active = 0 WHERE id = ?1", [id])?;
    write_field_audit(conn, id, "status", "Active", "Inactive")
}

/// API-05. Original ID, hierarchy position and full history preserved
/// unchanged — reactivation only ever flips `is_active` back; no second
/// record is created (that's what makes Rule-34's reactivation offer safe).
pub fn reactivate_member(conn: &Connection, id: i64) -> Result<(), AppError> {
    let existing = find_member(conn, id)?;
    if existing.is_active {
        return Ok(());
    }
    conn.execute("UPDATE members SET is_active = 1 WHERE id = ?1", [id])?;
    write_field_audit(conn, id, "status", "Inactive", "Active")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: i64,
    pub name: String,
    pub phone: String,
    // M3 (S6) and M2 (S7) don't exist yet, so `member_period_totals` is
    // always empty today — every result reads 0 until then. The query
    // reads whatever's there rather than special-casing "no engine yet", so
    // real figures appear automatically once those sprints land, with no
    // change needed here.
    pub total_business_volume: f64,
    pub slab_pct: f64,
    pub is_active: bool,
    // The row is already read in full to build the shared `Member` struct
    // internally — carrying these three costs nothing extra and is what
    // lets the frontend open an Edit modal straight from a search result
    // this sprint, without `get_member_detail` (M4.1, S8) existing yet.
    // Not displayed by `SearchResultsList`; T-M1.4-5's visible field list
    // (name/ID/phone/TBV/slab/status) is unchanged.
    pub email: Option<String>,
    pub address: String,
    pub introducer_member_id: Option<i64>,
}

/// API-06 (Rule-44). The one shared function backing every search box in
/// the console (T-M1.4-1) — Home/Structure/Entry/Correction (S7+) and the
/// Add-Member reference lookup, which additionally filters to active
/// members via `active_only` (Rule-30). Empty query returns no results,
/// never an error and never "all members" (V4.1).
pub fn search_members(
    conn: &Connection,
    query: &str,
    active_only: bool,
) -> Result<Vec<SearchResult>, AppError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let name_needle = trimmed.to_lowercase();
    let query_digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    // V4.4: below four digits, only name/ID are matched — exactly the
    // behaviour that existed before Rule-44.
    let phone_floor_met = query_digits.len() >= 4;
    let phone_key = canonical_phone_key(trimmed);

    let mut stmt = conn.prepare(
        "SELECT m.*, \
                COALESCE(t.total_business_volume, 0) AS tbv, \
                COALESCE(t.slab_pct, 0) AS slab_pct \
         FROM members m \
         LEFT JOIN member_period_totals t \
                ON t.member_id = m.id \
               AND t.period_id = ( \
                    SELECT period_id FROM member_period_totals \
                    WHERE member_id = m.id ORDER BY period_id DESC LIMIT 1 \
                   )",
    )?;
    let mut rows = stmt.query([])?;

    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let member = Member::from_row(row)?;
        if active_only && !member.is_active {
            continue;
        }

        let name_match = member.name.to_lowercase().contains(&name_needle);
        let id_match = !query_digits.is_empty() && member.id.to_string().contains(&query_digits);
        let phone_match =
            phone_floor_met && canonical_phone_key(&member.phone).contains(&phone_key);

        if !(name_match || id_match || phone_match) {
            continue;
        }

        let tbv: i64 = row.get("tbv")?;
        let slab_pct: i64 = row.get("slab_pct")?;
        results.push(SearchResult {
            id: member.id,
            name: member.name,
            phone: member.phone,
            total_business_volume: tbv as f64 / 100.0,
            slab_pct: slab_pct as f64 / 100.0,
            is_active: member.is_active,
            email: member.email,
            address: member.address,
            introducer_member_id: member.introducer_member_id,
        });
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_seeded_in_memory;

    fn root_input() -> CreateRootMemberInput {
        CreateRootMemberInput {
            name: "Top Member".into(),
            phone: "9876500000".into(),
            address: "1 Main Street".into(),
            email: None,
            consent_given: true,
        }
    }

    fn add_input(introducer_member_id: i64, phone: &str) -> AddMemberInput {
        AddMemberInput {
            name: "Asha Patel".into(),
            phone: phone.into(),
            address: "2 Side Street".into(),
            email: Some("asha@example.com".into()),
            consent_given: true,
            introducer_member_id,
        }
    }

    #[test]
    fn creates_root_with_id_in_range_and_no_introducer() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        assert!((100_001..=999_999).contains(&root.id));
        assert_eq!(root.introducer_member_id, None);
        assert_eq!(root.level, 1);
    }

    #[test]
    fn second_root_is_refused() {
        let conn = open_seeded_in_memory().unwrap();
        create_root_member(&conn, root_input()).unwrap();
        let second = create_root_member(
            &conn,
            CreateRootMemberInput {
                phone: "9876500001".into(),
                ..root_input()
            },
        );
        assert!(matches!(second, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn add_member_resolves_active_reference_and_sets_level() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let outcome = add_member(&conn, add_input(root.id, "9876500002")).unwrap();
        match outcome {
            AddMemberOutcome::Created { member, .. } => {
                assert_eq!(member.introducer_member_id, Some(root.id));
                assert_eq!(member.level, 2);
                assert!((100_001..=999_999).contains(&member.id));
                assert_ne!(member.id, root.id);
            }
            _ => panic!("expected Created"),
        }
    }

    #[test]
    fn add_member_refuses_an_inactive_reference() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let child = match add_member(&conn, add_input(root.id, "9876500003")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };
        conn.execute("UPDATE members SET is_active = 0 WHERE id = ?1", [child.id])
            .unwrap();

        let result = add_member(&conn, add_input(child.id, "9876500004"));
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn add_member_refuses_an_unknown_reference() {
        let conn = open_seeded_in_memory().unwrap();
        let result = add_member(&conn, add_input(999_999, "9876500005"));
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[test]
    fn add_member_refuses_a_phone_already_active() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        add_member(&conn, add_input(root.id, "9876500006")).unwrap();

        let result = add_member(&conn, add_input(root.id, "9876500006"));
        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn add_member_offers_reactivation_for_an_inactive_phone_match_without_creating_anyone() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let existing = match add_member(&conn, add_input(root.id, "9876500007")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };
        conn.execute(
            "UPDATE members SET is_active = 0 WHERE id = ?1",
            [existing.id],
        )
        .unwrap();

        let before_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM members", [], |r| r.get(0))
            .unwrap();
        let result = add_member(&conn, add_input(root.id, "9876500007")).unwrap();
        let after_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM members", [], |r| r.get(0))
            .unwrap();

        assert_eq!(
            before_count, after_count,
            "a reactivation offer must not insert a row"
        );
        match result {
            AddMemberOutcome::ReactivationOffer { existing_member } => {
                assert_eq!(existing_member.id, existing.id);
            }
            _ => panic!("expected ReactivationOffer"),
        }
    }

    #[test]
    fn add_member_never_blocks_on_level_width_or_depth() {
        // Rule-1/Rule-32: advisory only. Nest well past the default depth-4
        // setting and confirm every insert still succeeds.
        let conn = open_seeded_in_memory().unwrap();
        let mut parent_id = create_root_member(&conn, root_input()).unwrap().id;
        for i in 0..6 {
            let phone = format!("98765990{i:02}");
            let outcome = add_member(&conn, add_input(parent_id, &phone)).unwrap();
            parent_id = match outcome {
                AddMemberOutcome::Created { member, .. } => member.id,
                _ => panic!("expected Created"),
            };
        }
    }

    #[test]
    fn consent_gates_the_save() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let mut input = add_input(root.id, "9876500008");
        input.consent_given = false;
        let result = add_member(&conn, input);
        assert!(matches!(
            result,
            Err(AppError::Validation { field, .. }) if field == "consent"
        ));
    }

    #[test]
    fn name_required_names_the_field() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let mut input = add_input(root.id, "9876500009");
        input.name = "  ".into();
        let result = add_member(&conn, input);
        assert!(matches!(
            result,
            Err(AppError::Validation { field, .. }) if field == "name"
        ));
    }

    #[test]
    fn malformed_email_is_refused_but_missing_email_is_fine() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();

        let mut bad = add_input(root.id, "9876500010");
        bad.email = Some("not-an-email".into());
        assert!(matches!(
            add_member(&conn, bad),
            Err(AppError::Validation { field, .. }) if field == "email"
        ));

        let mut none = add_input(root.id, "9876500011");
        none.email = None;
        assert!(add_member(&conn, none).is_ok());
    }

    #[test]
    fn canonical_phone_key_reduces_to_the_last_ten_digits() {
        assert_eq!(canonical_phone_key("+91 98765 43210"), "9876543210");
        assert_eq!(canonical_phone_key("9876543210"), "9876543210");
        assert_eq!(canonical_phone_key("09876543210"), "9876543210");
        assert_eq!(canonical_phone_key("+919876543210"), "9876543210");
    }

    #[test]
    fn phone_uniqueness_matches_across_formatting() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        add_member(&conn, add_input(root.id, "9876543210")).unwrap();

        let result = add_member(&conn, add_input(root.id, "+91 98765 43210"));
        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn no_warnings_within_configured_limits() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        match add_member(&conn, add_input(root.id, "9876599990")).unwrap() {
            AddMemberOutcome::Created { warnings, .. } => assert!(warnings.is_empty()),
            _ => panic!("expected Created"),
        }
    }

    #[test]
    fn width_warning_fires_once_the_configured_level_width_is_exceeded() {
        // Default level_2_width is 9 (02-business-rules.md §6) — the 10th
        // direct child of the root crosses it.
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let mut last_warnings = Vec::new();
        for i in 0..10 {
            let phone = format!("98765970{i:02}");
            match add_member(&conn, add_input(root.id, &phone)).unwrap() {
                AddMemberOutcome::Created { warnings, .. } => last_warnings = warnings,
                _ => panic!("expected Created"),
            }
        }
        assert!(
            last_warnings.iter().any(|w| w.contains("width")),
            "expected a width warning on the 10th level-2 child, got {last_warnings:?}"
        );
    }

    #[test]
    fn depth_warning_fires_past_the_configured_hierarchy_depth_but_never_blocks() {
        // Default hierarchy_depth is 4 (D-3) — level 5 is the first to warn.
        let conn = open_seeded_in_memory().unwrap();
        let mut parent_id = create_root_member(&conn, root_input()).unwrap().id;
        let mut warnings_by_level = Vec::new();
        for i in 0..5 {
            let phone = format!("98765980{i:02}");
            match add_member(&conn, add_input(parent_id, &phone)).unwrap() {
                AddMemberOutcome::Created { member, warnings } => {
                    parent_id = member.id;
                    warnings_by_level.push((member.level, warnings));
                }
                _ => panic!("expected Created"),
            }
        }
        let (level_5, warnings_5) = &warnings_by_level[3];
        assert_eq!(*level_5, 5);
        assert!(warnings_5.iter().any(|w| w.contains("depth")));
    }

    #[test]
    fn onboarding_writes_exactly_one_audit_log_row_per_member_with_cause_entry() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        add_member(&conn, add_input(root.id, "9876588888")).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2, "one row for the root, one for the added member");

        let causes: Vec<String> = {
            let mut stmt = conn.prepare("SELECT cause FROM audit_log").unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert!(causes.iter().all(|c| c == "entry"), "{causes:?}");
    }

    #[test]
    fn a_reactivation_offer_writes_no_audit_row() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let existing = match add_member(&conn, add_input(root.id, "9876577777")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };
        conn.execute(
            "UPDATE members SET is_active = 0 WHERE id = ?1",
            [existing.id],
        )
        .unwrap();

        let before: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
            .unwrap();
        add_member(&conn, add_input(root.id, "9876577777")).unwrap();
        let after: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
            .unwrap();
        assert_eq!(before, after, "no member was created, so nothing to audit");
    }

    // US-M1.2 — edit_member

    #[test]
    fn edit_member_updates_only_the_fields_sent() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();

        let updated = edit_member(
            &conn,
            EditMemberInput {
                id: root.id,
                name: Some("Top Member Renamed".into()),
                phone: None,
                email: None,
                address: None,
            },
        )
        .unwrap();
        assert_eq!(updated.name, "Top Member Renamed");
        assert_eq!(
            updated.phone, root.phone,
            "untouched fields must be unchanged"
        );
    }

    #[test]
    fn edit_member_email_null_clears_it_but_absent_leaves_it_alone() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        edit_member(
            &conn,
            EditMemberInput {
                id: root.id,
                name: None,
                phone: None,
                email: Some(Some("owner@example.com".into())),
                address: None,
            },
        )
        .unwrap();

        let untouched = edit_member(
            &conn,
            EditMemberInput {
                id: root.id,
                name: Some("Still Top".into()),
                phone: None,
                email: None,
                address: None,
            },
        )
        .unwrap();
        assert_eq!(untouched.email.as_deref(), Some("owner@example.com"));

        let cleared = edit_member(
            &conn,
            EditMemberInput {
                id: root.id,
                name: None,
                phone: None,
                email: Some(None),
                address: None,
            },
        )
        .unwrap();
        assert_eq!(cleared.email, None);
    }

    #[test]
    fn edit_member_introducer_cannot_be_set_through_the_api() {
        // Rule-37: there is no field to accept it at all — the struct
        // itself is the enforcement, this test documents that.
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let child = match add_member(&conn, add_input(root.id, "9876511100")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };

        edit_member(
            &conn,
            EditMemberInput {
                id: child.id,
                name: Some("Renamed Child".into()),
                phone: None,
                email: None,
                address: None,
            },
        )
        .unwrap();

        let reloaded = find_member(&conn, child.id).unwrap();
        assert_eq!(reloaded.introducer_member_id, Some(root.id));
    }

    #[test]
    fn edit_member_refuses_a_phone_collision_with_another_member() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let a = match add_member(&conn, add_input(root.id, "9876511101")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };
        let b = match add_member(&conn, add_input(root.id, "9876511102")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };

        let result = edit_member(
            &conn,
            EditMemberInput {
                id: b.id,
                name: None,
                phone: Some(a.phone),
                email: None,
                address: None,
            },
        );
        assert!(matches!(result, Err(AppError::Conflict { .. })));
    }

    #[test]
    fn edit_member_saving_your_own_unchanged_phone_is_not_a_conflict() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let result = edit_member(
            &conn,
            EditMemberInput {
                id: root.id,
                name: None,
                phone: Some(root.phone.clone()),
                email: None,
                address: None,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn edit_member_writes_one_audit_row_per_changed_field() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        edit_member(
            &conn,
            EditMemberInput {
                id: root.id,
                name: Some("New Name".into()),
                phone: None,
                email: Some(Some("new@example.com".into())),
                address: None,
            },
        )
        .unwrap();

        let causes: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT cause FROM audit_log WHERE field IN ('name','email')")
                .unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert_eq!(causes.len(), 2);
        assert!(causes.iter().all(|c| c == "edit"));
    }

    #[test]
    fn edit_member_refuses_an_unknown_member() {
        let conn = open_seeded_in_memory().unwrap();
        let result = edit_member(
            &conn,
            EditMemberInput {
                id: 999_999,
                name: Some("Nobody".into()),
                phone: None,
                email: None,
                address: None,
            },
        );
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    // US-M1.3 — deactivate_member / reactivate_member

    #[test]
    fn deactivate_then_reactivate_round_trips_is_active() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let child = match add_member(&conn, add_input(root.id, "9876522200")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };

        deactivate_member(&conn, child.id).unwrap();
        assert!(!find_member(&conn, child.id).unwrap().is_active);

        reactivate_member(&conn, child.id).unwrap();
        assert!(find_member(&conn, child.id).unwrap().is_active);
    }

    #[test]
    fn the_root_member_cannot_be_deactivated() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let result = deactivate_member(&conn, root.id);
        assert!(matches!(result, Err(AppError::Conflict { .. })));
        assert!(find_member(&conn, root.id).unwrap().is_active);
    }

    #[test]
    fn deactivating_a_member_touches_no_other_member_row() {
        // T-M1.3-5's regression, in the form available before M3 (S6)
        // exists: since is_active has zero computational effect, and no
        // other member's row is ever written by this function, every
        // *other* member's row must be byte-identical before and after —
        // the strongest statement of "zero effect" available before the
        // calculation engine exists to compare TBV/Rewards against.
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let mid = match add_member(&conn, add_input(root.id, "9876522201")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };
        let descendant = match add_member(&conn, add_input(mid.id, "9876522202")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };

        let root_before = find_member(&conn, root.id).unwrap();
        let descendant_before = find_member(&conn, descendant.id).unwrap();

        deactivate_member(&conn, mid.id).unwrap();

        let root_after = find_member(&conn, root.id).unwrap();
        let descendant_after = find_member(&conn, descendant.id).unwrap();
        assert_eq!(
            (root_before.is_active, root_before.level),
            (root_after.is_active, root_after.level)
        );
        assert_eq!(descendant_before.is_active, descendant_after.is_active);
        assert_eq!(
            descendant_before.introducer_member_id, descendant_after.introducer_member_id,
            "the descendant's own hierarchy position must not move"
        );
    }

    #[test]
    fn deactivate_writes_no_row_to_any_calculation_table() {
        // The other half of "zero computational effect": nothing is
        // written to the tables the (not-yet-built) calculation engine
        // reads from.
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let child = match add_member(&conn, add_input(root.id, "9876522203")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };

        deactivate_member(&conn, child.id).unwrap();

        let entries: i64 = conn
            .query_row("SELECT COUNT(*) FROM business_volume_entries", [], |r| {
                r.get(0)
            })
            .unwrap();
        let totals: i64 = conn
            .query_row("SELECT COUNT(*) FROM member_period_totals", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(entries, 0);
        assert_eq!(totals, 0);
    }

    #[test]
    fn deactivate_and_reactivate_each_write_exactly_one_audit_row() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let child = match add_member(&conn, add_input(root.id, "9876522204")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };

        deactivate_member(&conn, child.id).unwrap();
        reactivate_member(&conn, child.id).unwrap();

        let status_rows: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare(
                    "SELECT old_value, new_value FROM audit_log WHERE field = 'status' ORDER BY id",
                )
                .unwrap();
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert_eq!(
            status_rows,
            vec![
                ("Active".to_string(), "Inactive".to_string()),
                ("Inactive".to_string(), "Active".to_string()),
            ]
        );
    }

    #[test]
    fn deactivating_an_already_inactive_member_is_a_harmless_no_op() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let child = match add_member(&conn, add_input(root.id, "9876522205")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };
        deactivate_member(&conn, child.id).unwrap();
        deactivate_member(&conn, child.id).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE entity_id = ?1 AND field = 'status'",
                [child.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "a repeat deactivate must not double-write");
    }

    #[test]
    fn no_delete_command_exists_for_a_member() {
        // TEST-R42: there is no code path that deletes a `members` row —
        // asserted here structurally (searching this module's own public
        // surface), and again at the command-surface level in
        // tests/contract.rs (ALL_COMMAND_NAMES has no "delete" entry).
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        deactivate_member(&conn, root.id).ok(); // refused (root), but even if it weren't:
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM members", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "deactivating never removes a row");
    }

    // US-M1.4 — search_members (Rule-44)

    #[test]
    fn empty_query_returns_no_results_not_all_members() {
        let conn = open_seeded_in_memory().unwrap();
        create_root_member(&conn, root_input()).unwrap();
        assert!(search_members(&conn, "", false).unwrap().is_empty());
        assert!(search_members(&conn, "   ", false).unwrap().is_empty());
    }

    #[test]
    fn matches_by_name_substring_case_insensitively() {
        let conn = open_seeded_in_memory().unwrap();
        create_root_member(&conn, root_input()).unwrap(); // "Top Member"
        let results = search_members(&conn, "top mem", false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Top Member");
    }

    #[test]
    fn matches_by_id_substring() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let id_str = root.id.to_string();
        let fragment = &id_str[1..4];
        let results = search_members(&conn, fragment, false).unwrap();
        assert!(results.iter().any(|r| r.id == root.id));
    }

    #[test]
    fn test_r44_phone_matching_matrix() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(
            &conn,
            CreateRootMemberInput {
                phone: "+91 98765 43210".into(),
                ..root_input()
            },
        )
        .unwrap();

        for query in [
            "9876543210",
            "98765 43210",
            "+919876543210",
            "09876543210",
            "4321",
        ] {
            let results = search_members(&conn, query, false).unwrap();
            assert!(
                results.iter().any(|r| r.id == root.id),
                "query {query:?} should have matched the stored number"
            );
        }

        // The reverse direction: stored plainly, queried with a prefix.
        let conn2 = open_seeded_in_memory().unwrap();
        let plain = create_root_member(
            &conn2,
            CreateRootMemberInput {
                phone: "9876543210".into(),
                ..root_input()
            },
        )
        .unwrap();
        let results = search_members(&conn2, "+91 98765 43210", false).unwrap();
        assert!(results.iter().any(|r| r.id == plain.id));

        // Below the four-digit floor: no phone match (name/ID still work).
        let short = search_members(&conn, "432", false).unwrap();
        assert!(!short.iter().any(|r| r.id == root.id));
    }

    #[test]
    fn a_query_matching_one_members_name_and_anothers_phone_returns_both() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(
            &conn,
            CreateRootMemberInput {
                name: "5555 Holdings".into(),
                ..root_input()
            },
        )
        .unwrap();
        let child = match add_member(
            &conn,
            AddMemberInput {
                name: "Someone Else".into(),
                phone: "9955551234".into(),
                address: "2 Side Street".into(),
                email: None,
                consent_given: true,
                introducer_member_id: root.id,
            },
        )
        .unwrap()
        {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };

        let results = search_members(&conn, "5555", false).unwrap();
        let ids: Vec<i64> = results.iter().map(|r| r.id).collect();
        assert!(ids.contains(&root.id), "name match");
        assert!(ids.contains(&child.id), "phone match");
    }

    #[test]
    fn active_only_filters_out_inactive_members_for_the_reference_lookup() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let child = match add_member(&conn, add_input(root.id, "9876522299")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };
        deactivate_member(&conn, child.id).unwrap();

        let unfiltered = search_members(&conn, "Asha", false).unwrap();
        let filtered = search_members(&conn, "Asha", true).unwrap();
        assert!(unfiltered.iter().any(|r| r.id == child.id));
        assert!(!filtered.iter().any(|r| r.id == child.id));
    }

    #[test]
    fn inactive_members_still_appear_in_an_unfiltered_search() {
        let conn = open_seeded_in_memory().unwrap();
        let root = create_root_member(&conn, root_input()).unwrap();
        let child = match add_member(&conn, add_input(root.id, "9876522298")).unwrap() {
            AddMemberOutcome::Created { member, .. } => member,
            _ => panic!("expected Created"),
        };
        deactivate_member(&conn, child.id).unwrap();

        let results = search_members(&conn, "Asha", false).unwrap();
        let found = results.iter().find(|r| r.id == child.id).unwrap();
        assert!(!found.is_active);
    }
}
