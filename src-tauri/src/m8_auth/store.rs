// The unencrypted sidecar (`auth.json`, next to the encrypted `.db` file —
// see `crate::paths`). Holds only AES-GCM-wrapped copies of the master
// key, salts and nonces, plus lockout bookkeeping — never the master key
// or a credential in the clear. Safe to leave unencrypted for the same
// reason a disk-encryption tool's keyslot header is: ciphertext without the
// wrapping credential reveals nothing. See migration 0002's comment for why
// this can't live inside the encrypted database instead.
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryCodeEntry {
    pub envelope: String,
    pub used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStore {
    pub pin_envelope: Option<String>,
    pub password_envelope: Option<String>,
    pub recovery_codes: Vec<RecoveryCodeEntry>,
    pub failed_attempts: i64,
    /// RFC3339. `None` when not currently locked.
    pub locked_until: Option<String>,
}

fn io_err(e: impl std::fmt::Display) -> AppError {
    AppError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        e.to_string(),
    ))
}

impl AuthStore {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        let bytes = std::fs::read(path)?;
        serde_json::from_slice(&bytes).map_err(io_err)
    }

    /// Atomic write (temp file + rename) so a crash mid-write never leaves
    /// a half-written sidecar the next launch can't parse.
    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        let tmp = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(self).map_err(io_err)?;
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempPath(std::path::PathBuf);
    impl TempPath {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(std::env::temp_dir().join(format!("bvconsole-auth-{label}-{nanos}.json")))
        }
    }
    impl Drop for TempPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
            let _ = std::fs::remove_file(self.0.with_extension("json.tmp"));
        }
    }

    #[test]
    fn round_trips_through_save_and_load() {
        let path = TempPath::new("roundtrip");
        let store = AuthStore {
            pin_envelope: Some("envelope-a".into()),
            password_envelope: None,
            recovery_codes: vec![RecoveryCodeEntry {
                envelope: "envelope-b".into(),
                used: false,
            }],
            failed_attempts: 3,
            locked_until: None,
        };
        store.save(&path.0).unwrap();
        let loaded = AuthStore::load(&path.0).unwrap();
        assert_eq!(loaded.pin_envelope, store.pin_envelope);
        assert_eq!(loaded.failed_attempts, 3);
        assert_eq!(loaded.recovery_codes.len(), 1);
    }

    #[test]
    fn save_leaves_no_tmp_file_behind() {
        let path = TempPath::new("notmp");
        let store = AuthStore {
            pin_envelope: None,
            password_envelope: Some("x".into()),
            recovery_codes: vec![],
            failed_attempts: 0,
            locked_until: None,
        };
        store.save(&path.0).unwrap();
        assert!(!path.0.with_extension("json.tmp").exists());
    }
}
