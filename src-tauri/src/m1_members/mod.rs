// M1 — Member & Structure (04-technical-architecture.md §3.1).
// US-M1.1 (S4): create_root_member, add_member. edit/deactivate/reactivate
// (US-M1.2/M1.3) and search_members (US-M1.4, Rule-44) are S5 — their
// command slots exist as stubs in `crate::commands` but the logic lives
// here only once its own story lands.
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
    Created { member: Member },
    ReactivationOffer { existing_member: Member },
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
    insert_member(conn, id, &f, None, 1)
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
    let member = insert_member(conn, id, &f, Some(introducer.id), introducer.level + 1)?;
    Ok(AddMemberOutcome::Created { member })
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
            AddMemberOutcome::Created { member } => {
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
            AddMemberOutcome::Created { member } => member,
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
            AddMemberOutcome::Created { member } => member,
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
                AddMemberOutcome::Created { member } => member.id,
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
}
