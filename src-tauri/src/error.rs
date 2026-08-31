use serde::ser::SerializeStruct;
use serde::Serialize;

// Variants are added as the story that first needs them lands (e.g.
// Validation/NotFound/Conflict/AuthRequired/NotImplemented with M1 and the
// S4 command-surface scaffold) — not pre-built speculatively.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// A field-level input rejection (e.g. missing name, malformed email).
    #[error("validation error: {field}: {message}")]
    Validation { field: String, message: String },
    /// A referenced entity doesn't exist, or doesn't satisfy a required
    /// state (e.g. Reference ID not found, or found but inactive).
    #[error("not found: {message}")]
    NotFound { message: String },
    /// A save is refused because it collides with existing state (e.g.
    /// phone number already in use by an active member) — distinct from
    /// the reactivation-offer case, which is a normal response, not this.
    #[error("conflict: {message}")]
    Conflict { message: String },
    /// An authenticated command was called without a session. S4 only has
    /// the gate primitive (see `session.rs`) — real login is US-M8.1, S5.
    #[error("authentication required")]
    AuthRequired,
    /// The command is registered (so the surface/allowlist is complete
    /// from S4 onward per the roadmap's S4 exit gate) but its module
    /// hasn't shipped yet.
    #[error("not implemented: {command}")]
    NotImplemented { command: &'static str },
    /// Rule-29: a wrong PIN, wrong password, or no auth configured yet —
    /// always this same generic message, never which credential type or
    /// which part was wrong. Also the failure of `unwrap_master_key`
    /// (a malformed/mismatched envelope looks identical to a wrong
    /// credential from the caller's side, deliberately). `attempts_remaining`
    /// (T-M8.2-5) counts down to the *next* lockout threshold — showing it
    /// doesn't reveal which part of the credential was wrong, only how many
    /// tries are left, which the prototype's own login screen already does.
    #[error("incorrect PIN or password")]
    InvalidCredential { attempts_remaining: i64 },
    /// D-2's lockout ladder. `retry_after_seconds` drives the login
    /// screen's live countdown.
    #[error("locked — try again in {retry_after_seconds}s")]
    AccountLocked { retry_after_seconds: i64 },
    /// Rule-36 (amended by CR-2): a current-month entry while an earlier
    /// month is still `awaiting_close`. `blocking_month` is always the
    /// oldest outstanding month — the one that must close first.
    #[error("{month} isn't open for entry until {blocking_month} is closed")]
    PeriodNotAcceptingEntries {
        month: String,
        blocking_month: String,
    },
    /// A fresh entry against an already-`closed` period — not offered via
    /// `record_entry`, only via the correction panel's `edit_entry`/
    /// `add_closed_month_entry` (Rule-39).
    #[error("{month} is closed — use the correction panel instead")]
    PeriodClosed { month: String },
    /// M6 (S13): `rust_xlsxwriter`'s `XlsxError` doesn't implement
    /// `std::error::Error` in a way `#[from]` can lean on cleanly, and
    /// wrapping it as `Io` would misreport a formatting/limit failure as a
    /// filesystem one.
    #[error("export error: {0}")]
    Export(String),
    /// US-M8.6 (S14): raised by `login` specifically when Argon2id
    /// verification *succeeds* but the subsequent SQLCipher open still
    /// fails. Argon2 verification never touches `db_path` — only
    /// `auth_path` — so that ordering is an unambiguous signal that the
    /// database file itself is unreadable, not that the credential was
    /// wrong. The frontend routes this to the data-recovery screen instead
    /// of the generic "incorrect PIN or password" message.
    #[error("this console's data could not be opened")]
    DataUnreadable,
}

// Tauri commands return errors to the WebView as JSON, never as an opaque
// Display string — the frontend matches on `kind`.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let kind = match self {
            AppError::Database(_) => "database",
            AppError::Io(_) => "io",
            AppError::Validation { .. } => "validation",
            AppError::NotFound { .. } => "not_found",
            AppError::Conflict { .. } => "conflict",
            AppError::AuthRequired => "auth_required",
            AppError::NotImplemented { .. } => "not_implemented",
            AppError::InvalidCredential { .. } => "invalid_credential",
            AppError::AccountLocked { .. } => "account_locked",
            AppError::PeriodNotAcceptingEntries { .. } => "period_not_accepting_entries",
            AppError::PeriodClosed { .. } => "period_closed",
            AppError::Export(_) => "export",
            AppError::DataUnreadable => "data_unreadable",
        };
        let mut state = serializer.serialize_struct("AppError", 7)?;
        state.serialize_field("kind", kind)?;
        state.serialize_field("message", &self.to_string())?;
        if let AppError::Validation { field, .. } = self {
            state.serialize_field("field", field)?;
        } else {
            state.serialize_field("field", &None::<String>)?;
        }
        if let AppError::AccountLocked {
            retry_after_seconds,
        } = self
        {
            state.serialize_field("retryAfterSeconds", retry_after_seconds)?;
        } else {
            state.serialize_field("retryAfterSeconds", &None::<i64>)?;
        }
        if let AppError::InvalidCredential { attempts_remaining } = self {
            state.serialize_field("attemptsRemaining", attempts_remaining)?;
        } else {
            state.serialize_field("attemptsRemaining", &None::<i64>)?;
        }
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
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn to_json(err: &AppError) -> serde_json::Value {
        serde_json::to_value(err).unwrap()
    }

    // T-UI.5-2 (src/lib/ipc/errors.ts) reads these `kind` strings — a
    // rename here silently breaks the frontend's presentation mapping
    // without either side's own tests catching it, since they never run
    // against each other. This is the seam test. `"export"` is the one
    // kind with no dedicated frontend case (`AppErrorKind` in errors.ts
    // has no `export` entry) — it falls through to the generic "unknown"
    // presentation there, which still surfaces this struct's own message,
    // so that's accepted behaviour, asserted here as-is rather than a gap
    // this test should paper over.
    #[test]
    fn every_kind_string_is_the_frontends_expected_wire_value() {
        assert_eq!(
            to_json(&AppError::Database(rusqlite::Error::InvalidQuery))["kind"],
            "database"
        );
        assert_eq!(
            to_json(&AppError::Io(std::io::Error::other("boom")))["kind"],
            "io"
        );
        assert_eq!(
            to_json(&AppError::Validation {
                field: "name".into(),
                message: "Name is required.".into()
            })["kind"],
            "validation"
        );
        assert_eq!(
            to_json(&AppError::NotFound {
                message: "not found".into()
            })["kind"],
            "not_found"
        );
        assert_eq!(
            to_json(&AppError::Conflict {
                message: "conflict".into()
            })["kind"],
            "conflict"
        );
        assert_eq!(to_json(&AppError::AuthRequired)["kind"], "auth_required");
        assert_eq!(
            to_json(&AppError::NotImplemented { command: "foo" })["kind"],
            "not_implemented"
        );
        assert_eq!(
            to_json(&AppError::InvalidCredential {
                attempts_remaining: 3
            })["kind"],
            "invalid_credential"
        );
        assert_eq!(
            to_json(&AppError::AccountLocked {
                retry_after_seconds: 30
            })["kind"],
            "account_locked"
        );
        assert_eq!(
            to_json(&AppError::PeriodNotAcceptingEntries {
                month: "August 2026".into(),
                blocking_month: "June 2026".into()
            })["kind"],
            "period_not_accepting_entries"
        );
        assert_eq!(
            to_json(&AppError::PeriodClosed {
                month: "May 2026".into()
            })["kind"],
            "period_closed"
        );
        assert_eq!(
            to_json(&AppError::Export("bad sheet".into()))["kind"],
            "export"
        );
        assert_eq!(
            to_json(&AppError::DataUnreadable)["kind"],
            "data_unreadable"
        );
    }

    #[test]
    fn validation_carries_its_field_and_message_others_null_out_field() {
        let err = AppError::Validation {
            field: "phone".into(),
            message: "Enter a valid phone number.".into(),
        };
        assert_eq!(
            to_json(&err),
            json!({
                "kind": "validation",
                "message": "validation error: phone: Enter a valid phone number.",
                "field": "phone",
                "retryAfterSeconds": null,
                "attemptsRemaining": null,
                "month": null,
                "blockingMonth": null,
            })
        );

        // Every other variant's `field` is explicitly null, never absent —
        // the frontend's RawAppError reads `err.field` unconditionally.
        assert_eq!(
            to_json(&AppError::AuthRequired)["field"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn account_locked_carries_its_countdown_others_null_out_retry_after_seconds() {
        let err = AppError::AccountLocked {
            retry_after_seconds: 45,
        };
        assert_eq!(to_json(&err)["retryAfterSeconds"], 45);
        assert_eq!(
            to_json(&AppError::AuthRequired)["retryAfterSeconds"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn invalid_credential_carries_attempts_remaining_and_never_the_credential_itself() {
        let err = AppError::InvalidCredential {
            attempts_remaining: 2,
        };
        let value = to_json(&err);
        assert_eq!(value["attemptsRemaining"], 2);
        // Rule-29: the message is always the same generic sentence,
        // regardless of which credential type or part was wrong — nothing
        // about the actual PIN/password ever reaches this struct to leak.
        assert_eq!(value["message"], "incorrect PIN or password");
    }

    #[test]
    fn period_not_accepting_entries_carries_both_month_and_blocking_month() {
        let err = AppError::PeriodNotAcceptingEntries {
            month: "August 2026".into(),
            blocking_month: "June 2026".into(),
        };
        let value = to_json(&err);
        assert_eq!(value["month"], "August 2026");
        assert_eq!(value["blockingMonth"], "June 2026");
        assert_eq!(
            value["message"],
            "August 2026 isn't open for entry until June 2026 is closed"
        );
    }

    #[test]
    fn period_closed_carries_month_but_never_blocking_month() {
        let err = AppError::PeriodClosed {
            month: "May 2026".into(),
        };
        let value = to_json(&err);
        assert_eq!(value["month"], "May 2026");
        assert_eq!(value["blockingMonth"], serde_json::Value::Null);
    }

    #[test]
    fn database_and_io_errors_wrap_the_underlying_error_via_from() {
        let db_err: AppError = rusqlite::Error::InvalidQuery.into();
        assert!(matches!(db_err, AppError::Database(_)));

        let io_err: AppError = std::io::Error::other("disk full").into();
        assert!(matches!(io_err, AppError::Io(_)));
        assert_eq!(to_json(&io_err)["message"], "io error: disk full");
    }

    #[test]
    fn not_implemented_names_the_command_in_its_message() {
        let err = AppError::NotImplemented {
            command: "some_future_command",
        };
        assert_eq!(
            to_json(&err)["message"],
            "not implemented: some_future_command"
        );
    }
}
