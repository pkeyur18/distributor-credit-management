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
    /// `record_entry`, only via `edit_entry`'s correction path (Rule-39).
    #[error("{month} is closed — use the correction panel instead")]
    PeriodClosed { month: String },
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
        if let AppError::PeriodNotAcceptingEntries { month, .. } | AppError::PeriodClosed { month } =
            self
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
