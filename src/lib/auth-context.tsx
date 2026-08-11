import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

import { checkDataReadable } from "@/lib/ipc/preflight";

// Client-side mirror of the server-side session gate (`require_session` in
// src-tauri) — the frontend has no other way to know whether the operator
// is signed in, since Tauri commands are the only channel to the backend.
// `loading` -> one `check_data_readable` call decides `needs-setup` vs
// `needs-login` (US-M8.1/M8.2); `setup_first_run`/`login` success moves to
// `authenticated` via `markAuthenticated`. Session lock (US-M8.3, S7) will
// add a fifth state here when it lands — not built speculatively now.
type AuthState = "loading" | "needs-setup" | "needs-login" | "authenticated";

interface AuthContextValue {
  state: AuthState;
  markAuthenticated: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>("loading");

  useEffect(() => {
    checkDataReadable()
      .then((exists) => setState(exists ? "needs-login" : "needs-setup"))
      .catch(() => setState("needs-login"));
  }, []);

  return (
    <AuthContext.Provider value={{ state, markAuthenticated: () => setState("authenticated") }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used inside AuthProvider");
  return ctx;
}
