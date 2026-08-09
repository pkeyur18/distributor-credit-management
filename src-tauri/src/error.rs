use serde::ser::SerializeStruct;
use serde::Serialize;

// Variants are added as the story that first needs them lands (e.g.
// Validation with M1's member checks, S4) — not pre-built speculatively.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
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
        };
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", kind)?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
