// Session gate — two managed `bool`s, not a full authentication system.
// `authenticated` distinguishes "requires auth" from "doesn't" for the S4
// command surface (all 40 registered, per the roadmap's S4 exit gate — see
// PI/02-roadmap.md). `locked` was added in S7 (US-M8.3): API-29's own
// column in `04-technical-architecture.md` §6 gates `unlock_session` on
// "Locked-session state" — a third category, distinct from both the normal
// `Auth` gate and the seven genuinely-unauthenticated commands. Without it,
// `unlock_session` would have to call `require_session`, which `lock_session`
// has just made impossible to satisfy — a deadlock a locked user could never
// escape short of signing out, defeating the point of "resume where you left
// off". `locked` is `true` only between a real `lock_session` and the next
// successful `unlock_session`/`login` — never on a fresh launch, so
// `unlock_session` stays uncallable until something has actually locked.
use std::sync::Mutex;

use crate::error::AppError;

pub struct SessionState {
    authenticated: Mutex<bool>,
    locked: Mutex<bool>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            authenticated: Mutex::new(false),
            locked: Mutex::new(false),
        }
    }

    /// `login`/`unlock_session` call this on success.
    pub fn mark_authenticated(&self) {
        *self.authenticated.lock().unwrap() = true;
        *self.locked.lock().unwrap() = false;
    }

    /// `lock_session` (US-M8.3, S7) calls this — the session stops being
    /// authenticated but is remembered as locked rather than signed out.
    pub fn mark_locked(&self) {
        *self.authenticated.lock().unwrap() = false;
        *self.locked.lock().unwrap() = true;
    }

    /// A full sign-out: neither authenticated nor locked.
    pub fn clear(&self) {
        *self.authenticated.lock().unwrap() = false;
        *self.locked.lock().unwrap() = false;
    }

    pub fn is_authenticated(&self) -> bool {
        *self.authenticated.lock().unwrap()
    }

    pub fn is_locked(&self) -> bool {
        *self.locked.lock().unwrap()
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Every "Auth"-gated command (per `04-api-specification.md`) calls this
/// first. The 7 unauthenticated commands never call it, and
/// `unlock_session` calls `require_locked` instead (see below).
pub fn require_session(state: &SessionState) -> Result<(), AppError> {
    if state.is_authenticated() {
        Ok(())
    } else {
        Err(AppError::AuthRequired)
    }
}

/// API-29's own gate: `unlock_session` is callable only while the session
/// is actually locked — not on a fresh, never-logged-in launch, and not
/// while already authenticated.
pub fn require_locked(state: &SessionState) -> Result<(), AppError> {
    if state.is_locked() {
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

    #[test]
    fn a_fresh_never_logged_in_state_is_not_locked() {
        let state = SessionState::new();
        assert!(require_locked(&state).is_err());
    }

    #[test]
    fn locking_an_authenticated_session_fails_the_auth_gate_and_passes_the_locked_gate() {
        let state = SessionState::new();
        state.mark_authenticated();
        state.mark_locked();
        assert!(require_session(&state).is_err());
        assert!(require_locked(&state).is_ok());
    }

    #[test]
    fn unlocking_restores_the_auth_gate_and_closes_the_locked_gate() {
        let state = SessionState::new();
        state.mark_authenticated();
        state.mark_locked();
        state.mark_authenticated();
        assert!(require_session(&state).is_ok());
        assert!(require_locked(&state).is_err());
    }

    #[test]
    fn a_full_sign_out_closes_both_gates() {
        let state = SessionState::new();
        state.mark_authenticated();
        state.mark_locked();
        state.clear();
        assert!(require_session(&state).is_err());
        assert!(require_locked(&state).is_err());
    }
}
