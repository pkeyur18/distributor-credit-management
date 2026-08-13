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
// Rule-43/S14: an unencrypted mirror of `backups` row metadata (id/kind/
// version/checksum/path/created_at — never member data or figures), kept in
// lockstep by every write in `backup.rs`. Exists because `backups` itself
// lives inside the SQLCipher file: the pre-auth commands (`list_restore_points`,
// `restore_from_backup`, `restore_from_backup_file`) have no key to read that
// table with. See `backup::manifest`'s doc comment for the full reasoning.
pub(crate) const BACKUPS_MANIFEST_FILE_NAME: &str = "backups-manifest.json";

pub struct AppPaths {
    pub db_path: PathBuf,
    pub auth_path: PathBuf,
    pub backups_manifest_path: PathBuf,
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
            backups_manifest_path: dir.join(BACKUPS_MANIFEST_FILE_NAME),
            app_data_dir: dir,
        })
    }
}
