// Fixed filenames in the app's data directory — one encrypted database, one
// unencrypted sidecar next to it (see m8_auth::store's doc comment).
// Resolved once at startup into managed state (`AppPaths`) rather than
// re-derived per command from an `AppHandle`, so contract tests can inject
// temp-directory paths the same way they already inject `DbState` — no
// automated test should ever touch the real OS app-data directory.
use std::path::PathBuf;

use tauri::Manager;

const DB_FILE_NAME: &str = "console.db";
const AUTH_FILE_NAME: &str = "auth.json";
const BACKUPS_DIR_NAME: &str = "backups";

pub struct AppPaths {
    pub db_path: PathBuf,
    pub auth_path: PathBuf,
    /// Internal-retained backup copies (`backups.internal_retained_path`,
    /// ADR-012). First needed by US-M2.2's closed-month correction (S7,
    /// `backup::write_backup_copy`); S11's close and S14's console backup
    /// reuse the same directory and helper rather than each deriving it.
    pub backups_dir: PathBuf,
}

impl AppPaths {
    pub fn resolve(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = app.path().app_data_dir()?;
        std::fs::create_dir_all(&dir)?;
        let backups_dir = dir.join(BACKUPS_DIR_NAME);
        std::fs::create_dir_all(&backups_dir)?;
        Ok(Self {
            db_path: dir.join(DB_FILE_NAME),
            auth_path: dir.join(AUTH_FILE_NAME),
            backups_dir,
        })
    }
}
