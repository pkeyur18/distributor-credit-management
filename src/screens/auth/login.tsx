import { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { AlertTriangle, Lock } from "lucide-react";

import { AuthBrandMark } from "@/components/auth-brand-mark";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { PinDots } from "@/components/ui/pin-input";
import { PinKeypad } from "@/components/ui/pin-keypad";
import { login } from "@/lib/ipc/m8-auth";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { useAuth } from "@/lib/auth-context";

// US-M8.2 — login with the D-2 lockout ladder (ui-prototype-v2.html's
// renderLogin, mechanics replaced: the ladder and the countdown are the
// server's `AccountLocked { retryAfterSeconds }`, never a client-side
// counter — the prototype's flat 20s-with-reset is demo pacing, not the
// real security behaviour, and is deliberately not ported. PIN mode
// auto-submits at the 6th digit, exactly as the prototype does.
export function Login() {
  const [mode, setMode] = useState<"pin" | "password">("pin");
  const [pinBuffer, setPinBuffer] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [lockedSeconds, setLockedSeconds] = useState<number | null>(null);
  const { markAuthenticated } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    if (lockedSeconds === null || lockedSeconds <= 0) return;
    const timer = setTimeout(() => setLockedSeconds((s) => (s !== null ? s - 1 : null)), 1000);
    return () => clearTimeout(timer);
  }, [lockedSeconds]);

  async function attempt(credential: { pin?: string } | { password?: string }) {
    setSubmitting(true);
    try {
      await login(credential);
      markAuthenticated();
      navigate("/", { replace: true });
    } catch (raw) {
      const presented = toErrorPresentation(raw);
      if (presented.kind === "account_locked") {
        setLockedSeconds(presented.retryAfterSeconds ?? 0);
        setError(null);
      } else if (presented.kind === "data_unreadable") {
        // US-M8.6: Argon2 succeeded but the database itself won't open —
        // never a credential problem (see error.rs's own doc comment on
        // `AppError::DataUnreadable`). Route to recovery, not the login
        // error state.
        navigate("/auth/data-recovery", { replace: true });
        return;
      } else {
        setError(presented.message);
      }
      setPinBuffer("");
    } finally {
      setSubmitting(false);
    }
  }

  function pinPress(digit: string) {
    if (pinBuffer.length >= 6 || submitting) return;
    const next = pinBuffer + digit;
    setPinBuffer(next);
    if (next.length === 6) {
      setTimeout(() => attempt({ pin: next }), 150);
    }
  }

  const lockedOut = lockedSeconds !== null && lockedSeconds > 0;

  return (
    <div className="flex min-h-screen items-center justify-center bg-bg px-4">
      <div className="w-full max-w-95 rounded-lg border border-border bg-surface p-6">
        <AuthBrandMark>
          <Lock className="size-6" />
        </AuthBrandMark>
        <h1 className="mt-3.5 text-center text-title">Member Rewards Console</h1>
        <p className="mt-1 text-center text-caption">Sign in to continue</p>

        <div className="mt-4">
          {lockedOut ? (
            <div className="flex flex-col items-center gap-1.5 py-4 text-center">
              <AlertTriangle className="size-7 text-danger" />
              <p className="text-title-sm">Too many attempts</p>
              <p className="num text-numeric-lg text-danger">{lockedSeconds}s</p>
              <p className="text-caption">Sign-in is temporarily locked for your security.</p>
            </div>
          ) : (
            <>
              <div className="flex justify-center">
                <SegmentedControl
                  value={mode}
                  onValueChange={(next) => {
                    setMode(next);
                    setPinBuffer("");
                    setError(null);
                  }}
                  options={[
                    { value: "pin", label: "PIN" },
                    { value: "password", label: "Password" },
                  ]}
                  className="mb-3.5"
                />
              </div>
              {mode === "pin" ? (
                <>
                  <PinDots filledCount={pinBuffer.length} error={!!error} />
                  <PinKeypad
                    onPress={pinPress}
                    onBackspace={() => setPinBuffer((b) => b.slice(0, -1))}
                    disabled={submitting}
                  />
                </>
              ) : (
                <div className="flex flex-col gap-3">
                  <div>
                    <label htmlFor="login-pw" className="text-label mb-1 block">
                      Password
                    </label>
                    <Input
                      id="login-pw"
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") attempt({ password });
                      }}
                    />
                  </div>
                  <Button
                    variant="primary"
                    className="w-full"
                    disabled={submitting || password.length === 0}
                    onClick={() => attempt({ password })}
                  >
                    Unlock
                  </Button>
                </div>
              )}
              {error && <p className="mt-3 text-center text-[11.5px] text-danger">{error}</p>}
            </>
          )}
        </div>

        {!lockedOut && (
          <p className="mt-3.5 text-center text-caption">
            <a href="/auth/recovery" className="text-accent">
              Forgot your PIN or password?
            </a>
          </p>
        )}
      </div>
    </div>
  );
}
