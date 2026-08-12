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

pub struct AppPaths {
    pub db_path: PathBuf,
    pub auth_path: PathBuf,
    /// Base app-data directory. The backups folder is a name stored in the
    /// `console_backup_folder` setting (row 16, Rule-43) inside the
    /// encrypted DB, so unlike `db_path`/`auth_path` it can't be resolved
    /// here — the DB isn't even open yet at startup. Every backup call site
    /// joins this with the live setting value at the point of use (see
    /// `backup::resolve_backups_dir`).
    pub app_data_dir: PathBuf,
}

impl AppPaths {
    pub fn resolve(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = app.path().app_data_dir()?;
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            db_path: dir.join(DB_FILE_NAME),
            auth_path: dir.join(AUTH_FILE_NAME),
            app_data_dir: dir,
        })
    }
}
