// Minimal session gate — a managed `bool`, not an authentication system.
// It exists only so the S4 command surface (all 40 registered, per the
// roadmap's S4 exit gate — see PI/02-roadmap.md) can honestly distinguish
// "requires auth" from "doesn't" before real login exists. Real credential
// verification, Argon2id hashing, and the lockout ladder are US-M8.1/M8.2,
// Sprint 5 — this file must not grow those concerns speculatively.
use std::sync::Mutex;

use crate::error::AppError;

pub struct SessionState {
    authenticated: Mutex<bool>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            authenticated: Mutex::new(false),
        }
    }

    /// S5's `login`/`unlock_session` call this on success. Nothing in S4
    /// calls it outside tests — there is no real credential check yet.
    pub fn mark_authenticated(&self) {
        *self.authenticated.lock().unwrap() = true;
    }

    pub fn clear(&self) {
        *self.authenticated.lock().unwrap() = false;
    }

    pub fn is_authenticated(&self) -> bool {
        *self.authenticated.lock().unwrap()
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Every "Auth"-gated command (33 of the 40, per `04-api-specification.md`)
/// calls this first. The 7 unauthenticated commands never call it.
pub fn require_session(state: &SessionState) -> Result<(), AppError> {
    if state.is_authenticated() {
        Ok(())
    } else {
        Err(AppError::AuthRequired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_state_is_unauthenticated() {
        let state = SessionState::new();
        assert!(require_session(&state).is_err());
    }

    #[test]
    fn authenticated_state_passes_the_gate() {
        let state = SessionState::new();
        state.mark_authenticated();
        assert!(require_session(&state).is_ok());
    }

    #[test]
    fn clear_revokes_a_session() {
        let state = SessionState::new();
        state.mark_authenticated();
        state.clear();
        assert!(require_session(&state).is_err());
    }
}
