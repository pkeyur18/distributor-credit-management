// US-M8.1/M8.2 (S5) — envelope key-wrapping, an architect decision recorded
// here because no source document resolves it (same defect class as
// D-9/D-11 in PI/05-decisions-and-gaps.md).
//
// The problem: Rule-29 requires PIN and password to be independently
// configurable, either authenticating — but `04-technical-architecture.md`
// also says the SQLCipher key is "derived via Argon2id from the PIN and/or
// password." Both cannot be literally true against one encrypted file:
// Argon2id(pin) and Argon2id(password) are two unrelated 32-byte outputs,
// and SQLCipher takes exactly one key.
//
// Resolution: the credential never *is* the database key. A random 256-bit
// master key is generated once, at `setup_first_run`, and is the only thing
// that ever opens SQLCipher. Each configured credential (PIN, password,
// and — forward-compatibly, for US-M8.4/S8 — each recovery code) wraps an
// independent copy of that same master key: an Argon2id-derived key
// encrypts it with AES-256-GCM. `auth.pin_hash`/`auth.password_hash` keep
// their existing column names and TEXT type (no migration) but hold this
// envelope, not a bare hash. Login derives the wrapping key from whatever
// credential was submitted and attempts to decrypt its stored envelope —
// success both proves the credential correct (GCM's tag check) and
// recovers the master key in the same step, so there is no separate
// hash-verify pass to keep in sync.
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;
use rand::RngCore;

use crate::error::AppError;

pub const MASTER_KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

// TR-2: tuned against a deliberately modest baseline, not a fast dev
// machine — OWASP's minimum recommended Argon2id baseline (19 MiB, 2
// iterations, 1 lane). Embedded in every envelope so a future retune
// doesn't strand credentials wrapped under the old parameters.
const DEFAULT_MEM_KIB: u32 = 19_456;
const DEFAULT_TIME_COST: u32 = 2;
const DEFAULT_PARALLELISM: u32 = 1;

pub type MasterKey = [u8; MASTER_KEY_LEN];

pub fn generate_master_key() -> MasterKey {
    let mut key = [0u8; MASTER_KEY_LEN];
    rand::rng().fill_bytes(&mut key);
    key
}

fn derive_wrapping_key(
    credential: &str,
    salt: &[u8],
    mem_kib: u32,
    time_cost: u32,
    parallelism: u32,
) -> Result<Key<Aes256Gcm>, AppError> {
    let params = argon2::Params::new(mem_kib, time_cost, parallelism, Some(MASTER_KEY_LEN))
        .map_err(|e| AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string())))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut out = [0u8; MASTER_KEY_LEN];
    argon2
        .hash_password_into(credential.as_bytes(), salt, &mut out)
        .map_err(|e| AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string())))?;
    Ok(*Key::<Aes256Gcm>::from_slice(&out))
}

/// Wraps `master_key` under `credential`, returning a self-describing
/// envelope string to store as-is in `auth.pin_hash` / `auth.password_hash`
/// / one entry of the `recovery_codes` JSON array.
pub fn wrap_master_key(credential: &str, master_key: &MasterKey) -> Result<String, AppError> {
    let mut salt = [0u8; SALT_LEN];
    rand::rng().fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);

    let wrapping_key = derive_wrapping_key(
        credential,
        &salt,
        DEFAULT_MEM_KIB,
        DEFAULT_TIME_COST,
        DEFAULT_PARALLELISM,
    )?;
    let cipher = Aes256Gcm::new(&wrapping_key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, master_key.as_slice()).map_err(|_| {
        AppError::Database(rusqlite::Error::InvalidParameterName("wrap failed".into()))
    })?;

    Ok(format!(
        "argon2id$m={},t={},p={}${}${}${}",
        DEFAULT_MEM_KIB,
        DEFAULT_TIME_COST,
        DEFAULT_PARALLELISM,
        hex_encode(&salt),
        hex_encode(&nonce_bytes),
        hex_encode(&ciphertext),
    ))
}

/// Attempts to recover the master key wrapped in `envelope` using
/// `credential`. `None` covers every failure mode alike — malformed
/// envelope, wrong credential — deliberately: this is a low-level primitive
/// with no notion of "attempts remaining," that's `m8_auth::login`'s policy
/// to attach (Rule-29: never reveal *which part* was wrong, but the caller
/// may still say *how many tries are left*).
pub fn unwrap_master_key(credential: &str, envelope: &str) -> Option<MasterKey> {
    let parsed = parse_envelope(envelope)?;
    let wrapping_key = derive_wrapping_key(
        credential,
        &parsed.salt,
        parsed.mem_kib,
        parsed.time_cost,
        parsed.parallelism,
    )
    .ok()?;
    let cipher = Aes256Gcm::new(&wrapping_key);
    let nonce = Nonce::from_slice(&parsed.nonce);
    let plaintext = cipher.decrypt(nonce, parsed.ciphertext.as_slice()).ok()?;
    MasterKey::try_from(plaintext.as_slice()).ok()
}

struct ParsedEnvelope {
    mem_kib: u32,
    time_cost: u32,
    parallelism: u32,
    salt: Vec<u8>,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

fn parse_envelope(envelope: &str) -> Option<ParsedEnvelope> {
    let mut parts = envelope.split('$');
    let algo = parts.next()?;
    if algo != "argon2id" {
        return None;
    }
    let params = parts.next()?;
    let salt = hex_decode(parts.next()?)?;
    let nonce = hex_decode(parts.next()?)?;
    let ciphertext = hex_decode(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }

    let mut mem_kib = None;
    let mut time_cost = None;
    let mut parallelism = None;
    for kv in params.split(',') {
        let (k, v) = kv.split_once('=')?;
        let v: u32 = v.parse().ok()?;
        match k {
            "m" => mem_kib = Some(v),
            "t" => time_cost = Some(v),
            "p" => parallelism = Some(v),
            _ => return None,
        }
    }

    Some(ParsedEnvelope {
        mem_kib: mem_kib?,
        time_cost: time_cost?,
        parallelism: parallelism?,
        salt,
        nonce,
        ciphertext,
    })
}

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// SQLCipher raw-key PRAGMA syntax: `x'<64 hex chars>'`. Using the raw form
/// (rather than a passphrase) skips SQLCipher's own PBKDF2 derivation —
/// Argon2id already did the slow, tunable part.
pub fn sqlcipher_raw_key_pragma(master_key: &MasterKey) -> String {
    format!("x'{}'", hex_encode(master_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_then_unwrap_recovers_the_same_master_key() {
        let master_key = generate_master_key();
        let envelope = wrap_master_key("482913", &master_key).unwrap();
        let recovered = unwrap_master_key("482913", &envelope).unwrap();
        assert_eq!(recovered, master_key);
    }

    #[test]
    fn wrong_credential_is_refused_generically() {
        let master_key = generate_master_key();
        let envelope = wrap_master_key("482913", &master_key).unwrap();
        assert!(unwrap_master_key("000000", &envelope).is_none());
    }

    #[test]
    fn two_credentials_independently_unwrap_the_same_master_key() {
        // The scenario Rule-29 requires: a PIN and a password, either
        // authenticating, against the one encrypted file.
        let master_key = generate_master_key();
        let pin_envelope = wrap_master_key("482913", &master_key).unwrap();
        let password_envelope = wrap_master_key("Harvest99!", &master_key).unwrap();

        assert_eq!(
            unwrap_master_key("482913", &pin_envelope).unwrap(),
            master_key
        );
        assert_eq!(
            unwrap_master_key("Harvest99!", &password_envelope).unwrap(),
            master_key
        );
    }

    #[test]
    fn envelope_never_contains_the_plaintext_credential_or_master_key() {
        let master_key = generate_master_key();
        let envelope = wrap_master_key("482913", &master_key).unwrap();
        assert!(!envelope.contains("482913"));
        assert!(!envelope.contains(&hex_encode(&master_key)));
    }

    #[test]
    fn malformed_envelope_is_refused_not_panicked_on() {
        assert!(unwrap_master_key("482913", "not-a-real-envelope").is_none());
    }

    #[test]
    fn sqlcipher_pragma_is_64_hex_chars_wrapped_in_x_quotes() {
        let master_key = generate_master_key();
        let pragma = sqlcipher_raw_key_pragma(&master_key);
        assert_eq!(pragma.len(), 2 + 64 + 1);
        assert!(pragma.starts_with("x'"));
        assert!(pragma.ends_with('\''));
    }
}
