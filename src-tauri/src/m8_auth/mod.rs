// M8 — Setup, login, lockout (04-technical-architecture.md §3.1).
// US-M8.1/US-M8.2 (S5). Session lock (US-M8.3, S7), recovery-code redemption
// (US-M8.4, S8) and console backup/restore (US-M8.5/M8.6, S14) are later
// sprints — this module must not grow those concerns speculatively.
pub mod crypto;
mod lockout;
pub mod store;

use std::path::Path;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use store::{AuthStore, RecoveryCodeEntry};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupFirstRunInput {
    pub pin: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupFirstRunResult {
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialInput {
    pub pin: Option<String>,
    pub password: Option<String>,
}

// Rule-29.
fn validate_pin(pin: &str) -> bool {
    pin.len() == 6 && pin.chars().all(|c| c.is_ascii_digit())
}

fn validate_password(password: &str) -> bool {
    password.len() >= 8
        && password.chars().any(|c| c.is_ascii_alphabetic())
        && password.chars().any(|c| c.is_ascii_digit())
}

const RECOVERY_CODE_COUNT: usize = 10;
// Excludes 0/O and 1/I/L — the characters a tired operator most often
// misreads copying a code down by hand.
const RECOVERY_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

fn generate_recovery_code() -> String {
    let mut rng = rand::rng();
    let raw: String = (0..15)
        .map(|_| RECOVERY_ALPHABET[rng.random_range(0..RECOVERY_ALPHABET.len())] as char)
        .collect();
    format!("{}-{}-{}", &raw[0..5], &raw[5..10], &raw[10..15])
}

fn generate_recovery_codes(
    master_key: &crypto::MasterKey,
) -> Result<(Vec<String>, Vec<RecoveryCodeEntry>), AppError> {
    let mut plaintext = Vec::with_capacity(RECOVERY_CODE_COUNT);
    let mut entries = Vec::with_capacity(RECOVERY_CODE_COUNT);
    for _ in 0..RECOVERY_CODE_COUNT {
        let code = generate_recovery_code();
        let envelope = crypto::wrap_master_key(&code, master_key)?;
        entries.push(RecoveryCodeEntry {
            envelope,
            used: false,
        });
        plaintext.push(code);
    }
    Ok((plaintext, entries))
}

/// API-26. `auth_path` not existing yet is exactly what "no `auth` row
/// exists" (the architecture doc's unauthenticated-callable condition)
/// means now that credential state lives in the sidecar, not a DB row.
pub fn setup_first_run(
    auth_path: &Path,
    input: SetupFirstRunInput,
) -> Result<(SetupFirstRunResult, crypto::MasterKey), AppError> {
    if auth_path.exists() {
        return Err(AppError::Conflict {
            message: "Auth is already configured.".into(),
        });
    }
    if input.pin.is_none() && input.password.is_none() {
        return Err(AppError::Validation {
            field: "pin".into(),
            message: "Set a PIN or a password.".into(),
        });
    }
    if let Some(pin) = input.pin.as_deref() {
        if !validate_pin(pin) {
            return Err(AppError::Validation {
                field: "pin".into(),
                message: "PIN must be exactly 6 digits.".into(),
            });
        }
    }
    if let Some(password) = input.password.as_deref() {
        if !validate_password(password) {
            return Err(AppError::Validation {
                field: "password".into(),
                message: "Password must be at least 8 characters with a letter and a number."
                    .into(),
            });
        }
    }

    let master_key = crypto::generate_master_key();
    let pin_envelope = input
        .pin
        .as_deref()
        .map(|p| crypto::wrap_master_key(p, &master_key))
        .transpose()?;
    let password_envelope = input
        .password
        .as_deref()
        .map(|p| crypto::wrap_master_key(p, &master_key))
        .transpose()?;
    let (plaintext_codes, recovery_codes) = generate_recovery_codes(&master_key)?;

    let store = AuthStore {
        pin_envelope,
        password_envelope,
        recovery_codes,
        failed_attempts: 0,
        locked_until: None,
    };
    store.save(auth_path)?;

    Ok((
        SetupFirstRunResult {
            recovery_codes: plaintext_codes,
        },
        master_key,
    ))
}

/// API-27. Returns the recovered master key on success so the caller can
/// open the SQLCipher connection with it. Every branch — wrong credential,
/// no credential configured for the submitted type, malformed envelope —
/// collapses to the same generic error (Rule-29: never reveal which part
/// was wrong), except an already-active lock, which the login screen's
/// countdown genuinely needs to distinguish.
pub fn login(auth_path: &Path, input: CredentialInput) -> Result<crypto::MasterKey, AppError> {
    if !auth_path.exists() {
        return Err(AppError::NotFound {
            message: "No console is set up on this machine yet.".into(),
        });
    }
    let mut store = AuthStore::load(auth_path)?;

    if let Some(retry_after_seconds) = lockout::seconds_remaining(&store.locked_until) {
        return Err(AppError::AccountLocked {
            retry_after_seconds,
        });
    }

    let attempt = match (input.pin.as_deref(), input.password.as_deref()) {
        (Some(pin), _) => store.pin_envelope.as_deref().map(|env| (pin, env)),
        (_, Some(password)) => store
            .password_envelope
            .as_deref()
            .map(|env| (password, env)),
        (None, None) => None,
    };

    let recovered =
        attempt.and_then(|(credential, envelope)| crypto::unwrap_master_key(credential, envelope));

    match recovered {
        Some(master_key) => {
            store.failed_attempts = 0;
            store.locked_until = None;
            store.save(auth_path)?;
            Ok(master_key)
        }
        None => {
            store.failed_attempts += 1;
            let newly_locked = lockout::tier_duration_seconds(store.failed_attempts);
            if let Some(seconds) = newly_locked {
                store.locked_until = Some(lockout::locked_until_from_now(seconds));
            }
            store.save(auth_path)?;
            match newly_locked {
                Some(retry_after_seconds) => Err(AppError::AccountLocked {
                    retry_after_seconds,
                }),
                // Never a multiple of 5 here — that case took the branch
                // above — so this is always in 1..=4 (T-M8.2-5).
                None => Err(AppError::InvalidCredential {
                    attempts_remaining: 5 - (store.failed_attempts % 5),
                }),
            }
        }
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
            Self(std::env::temp_dir().join(format!("bvconsole-m8-{label}-{nanos}.json")))
        }
    }
    impl Drop for TempPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn credential_pin(pin: &str) -> CredentialInput {
        CredentialInput {
            pin: Some(pin.into()),
            password: None,
        }
    }

    #[test]
    fn setup_then_login_with_the_pin_recovers_the_same_master_key() {
        let path = TempPath::new("setup-login");
        let (result, master_key) = setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: None,
            },
        )
        .unwrap();
        assert_eq!(result.recovery_codes.len(), RECOVERY_CODE_COUNT);

        let recovered = login(&path.0, credential_pin("482913")).unwrap();
        assert_eq!(recovered, master_key);
    }

    #[test]
    fn either_credential_logs_in_when_both_are_configured() {
        let path = TempPath::new("dual-credential");
        let (_, master_key) = setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: Some("Harvest99!".into()),
            },
        )
        .unwrap();

        assert_eq!(
            login(&path.0, credential_pin("482913")).unwrap(),
            master_key
        );
        assert_eq!(
            login(
                &path.0,
                CredentialInput {
                    pin: None,
                    password: Some("Harvest99!".into()),
                },
            )
            .unwrap(),
            master_key
        );
    }

    #[test]
    fn setup_refuses_a_second_call() {
        let path = TempPath::new("no-second-setup");
        setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: None,
            },
        )
        .unwrap();

        let err = setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("111111".into()),
                password: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Conflict { .. }));
    }

    #[test]
    fn setup_requires_at_least_one_credential() {
        let path = TempPath::new("no-credential");
        let err = setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: None,
                password: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn setup_rejects_a_malformed_pin() {
        let path = TempPath::new("bad-pin");
        let err = setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("12345".into()),
                password: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn setup_rejects_a_password_without_a_digit() {
        let path = TempPath::new("bad-password");
        let err = setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: None,
                password: Some("noDigitsHere".into()),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn wrong_pin_is_refused_generically_and_counts_as_a_failure() {
        let path = TempPath::new("wrong-pin");
        setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: None,
            },
        )
        .unwrap();

        let err = login(&path.0, credential_pin("000000")).unwrap_err();
        assert!(matches!(err, AppError::InvalidCredential { .. }));
        assert_eq!(AuthStore::load(&path.0).unwrap().failed_attempts, 1);
    }

    #[test]
    fn attempts_remaining_counts_down_to_the_next_lockout_threshold() {
        // T-M8.2-5: the login screen shows this before the account locks.
        let path = TempPath::new("attempts-remaining");
        setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: None,
            },
        )
        .unwrap();

        for expected_remaining in [4, 3, 2, 1] {
            let err = login(&path.0, credential_pin("000000")).unwrap_err();
            match err {
                AppError::InvalidCredential { attempts_remaining } => {
                    assert_eq!(attempts_remaining, expected_remaining)
                }
                other => panic!("expected InvalidCredential, got {other:?}"),
            }
        }
        // The 5th failure locks instead of reporting a remaining count.
        let fifth = login(&path.0, credential_pin("000000")).unwrap_err();
        assert!(matches!(fifth, AppError::AccountLocked { .. }));
    }

    #[test]
    fn a_successful_login_resets_the_failure_counter() {
        let path = TempPath::new("reset-on-success");
        setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: None,
            },
        )
        .unwrap();
        let _ = login(&path.0, credential_pin("000000"));
        let _ = login(&path.0, credential_pin("000000"));
        login(&path.0, credential_pin("482913")).unwrap();

        assert_eq!(AuthStore::load(&path.0).unwrap().failed_attempts, 0);
    }

    #[test]
    fn exactly_five_consecutive_failures_triggers_lockout() {
        let path = TempPath::new("fifth-failure-locks");
        setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: None,
            },
        )
        .unwrap();

        for _ in 0..4 {
            let err = login(&path.0, credential_pin("000000")).unwrap_err();
            assert!(matches!(err, AppError::InvalidCredential { .. }));
        }
        let fifth = login(&path.0, credential_pin("000000")).unwrap_err();
        assert!(matches!(
            fifth,
            AppError::AccountLocked {
                retry_after_seconds: 30
            }
        ));
    }

    #[test]
    fn a_locked_account_refuses_even_the_correct_credential() {
        let path = TempPath::new("locked-refuses-correct");
        setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: None,
            },
        )
        .unwrap();
        for _ in 0..5 {
            let _ = login(&path.0, credential_pin("000000"));
        }

        let err = login(&path.0, credential_pin("482913")).unwrap_err();
        assert!(matches!(err, AppError::AccountLocked { .. }));
    }

    #[test]
    fn login_before_any_setup_is_refused() {
        let path = TempPath::new("no-setup-yet");
        let err = login(&path.0, credential_pin("482913")).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn submitting_a_password_when_only_a_pin_is_configured_is_refused_generically() {
        let path = TempPath::new("wrong-credential-type");
        setup_first_run(
            &path.0,
            SetupFirstRunInput {
                pin: Some("482913".into()),
                password: None,
            },
        )
        .unwrap();

        let err = login(
            &path.0,
            CredentialInput {
                pin: None,
                password: Some("Harvest99!".into()),
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::InvalidCredential { .. }));
    }
}
