import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

import { checkDataReadable } from "@/lib/ipc/preflight";

// Client-side mirror of the server-side session gate (`require_session`/
// `require_locked` in src-tauri) — the frontend has no other way to know
// whether the operator is signed in, since Tauri commands are the only
// channel to the backend. `loading` -> one `check_data_readable` call
// decides `needs-setup` vs `needs-login` (US-M8.1/M8.2); `setup_first_run`/
// `login`/`unlock_session` success moves to `authenticated`. `locked`
// (US-M8.3, S7) is distinct from `needs-login`: `unlock_session` is only
// reachable server-side from a locked state (`session.rs`'s own
// `require_locked`), so the client must remember it too rather than
// treating "locked" and "never logged in" as the same thing.
//
// `needs-recovery` (US-M8.6, S14): `check_data_readable` now genuinely
// rejects when this machine has been set up before but its data looks
// broken — routing that into `needs-login` (the old `.catch` behaviour)
// would show the ordinary sign-in screen over data that can't be opened.
type AuthState =
  | "loading"
  | "needs-setup"
  | "needs-login"
  | "needs-recovery"
  | "authenticated"
  | "locked";

interface AuthContextValue {
  state: AuthState;
  markAuthenticated: () => void;
  markLocked: () => void;
  /** A full sign-out: unlike `markLocked`, routes straight back to Login
   * rather than the "resume where you left off" Locked screen. `notice`
   * survives the redirect (unlike a toast) so Login can still show it. */
  markSignedOut: (notice?: string) => void;
  signOutNotice: string | null;
  clearSignOutNotice: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>("loading");
  const [signOutNotice, setSignOutNotice] = useState<string | null>(null);

  useEffect(() => {
    checkDataReadable()
      .then((exists) => setState(exists ? "needs-login" : "needs-setup"))
      .catch(() => setState("needs-recovery"));
  }, []);

  return (
    <AuthContext.Provider
      value={{
        state,
        markAuthenticated: () => setState("authenticated"),
        markLocked: () => setState("locked"),
        markSignedOut: (notice) => {
          setSignOutNotice(notice ?? null);
          setState("needs-login");
        },
        signOutNotice,
        clearSignOutNotice: () => setSignOutNotice(null),
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used inside AuthProvider");
  return ctx;
}
