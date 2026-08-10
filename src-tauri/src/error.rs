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
        };
        let mut state = serializer.serialize_struct("AppError", 3)?;
        state.serialize_field("kind", kind)?;
        state.serialize_field("message", &self.to_string())?;
        if let AppError::Validation { field, .. } = self {
            state.serialize_field("field", field)?;
        } else {
            state.serialize_field("field", &None::<String>)?;
        }
        state.end()
    }
}
