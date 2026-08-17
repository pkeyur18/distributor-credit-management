import { useState } from "react";
import { useNavigate } from "react-router";
import { ShieldCheck } from "lucide-react";

import { AuthBrandMark } from "@/components/auth-brand-mark";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Input, InputHint } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AlertNote } from "@/components/ui/alert-note";
import { setupFirstRun } from "@/lib/ipc/m8-auth";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { useAuth } from "@/lib/auth-context";

// US-M8.1 — first-run setup wizard (03-functional-specification.md §5.10;
// ui-prototype-v2.html's renderSetup). Step 0 chooses PIN or password and
// captures it; step 1 reveals the recovery codes once, gated by the
// mandatory "I have saved this" checkbox. Non-dismissable by construction —
// there is nowhere else to go before a credential exists.
//
// One deliberate visual departure from the prototype: it shows a single
// recovery code in one field; T-M8.1-3 generates ten independent one-time
// codes (confirmed behaviour, not a visual detail the prototype was free to
// decide) — shown here as a list in the same bordered, mono, read-only
// treatment the prototype gives its one field.
function validPin(v: string) {
  return /^[0-9]{6}$/.test(v);
}
function validPassword(v: string) {
  return v.length >= 8 && /[a-zA-Z]/.test(v) && /[0-9]/.test(v);
}

export function Setup() {
  const [mode, setMode] = useState<"pin" | "password">("pin");
  const [pin, setPin] = useState("");
  const [pin2, setPin2] = useState("");
  const [password, setPassword] = useState("");
  const [password2, setPassword2] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [step, setStep] = useState<0 | 1>(0);
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([]);
  const [confirmed, setConfirmed] = useState(false);
  const { markAuthenticated } = useAuth();
  const navigate = useNavigate();

  async function handleContinue() {
    setError(null);
    if (mode === "pin") {
      if (!validPin(pin)) {
        setError("PIN must be exactly 6 digits.");
        return;
      }
      if (pin !== pin2) {
        setError("PINs do not match.");
        return;
      }
    } else {
      if (!validPassword(password)) {
        setError("Password must be at least 8 characters, with a letter and a number.");
        return;
      }
      if (password !== password2) {
        setError("Passwords do not match.");
        return;
      }
    }

    setSubmitting(true);
    try {
      const result = await setupFirstRun(
        mode === "pin" ? { pin } : { password },
      );
      setRecoveryCodes(result.recoveryCodes);
      setStep(1);
    } catch (raw) {
      setError(toErrorPresentation(raw).message);
    } finally {
      setSubmitting(false);
    }
  }

  function handleFinish() {
    markAuthenticated();
    navigate("/", { replace: true });
  }

  if (step === 1) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-bg px-4">
        <div className="w-full max-w-95 rounded-lg border border-border bg-surface p-6">
          <AuthBrandMark tone="success">
            <ShieldCheck className="size-6" />
          </AuthBrandMark>
          <h1 className="mt-3.5 text-center text-title">Save your recovery codes</h1>
          <p className="mt-1 text-center text-caption">
            If you ever forget your {mode === "pin" ? "PIN" : "password"}, these are the only way
            back in. Each code works once. They are shown only here.
          </p>

          {/* T-M8.4-3 (US-M8.4, S8) / ADR-008: the permanently-unrecoverable
              consequence, stated plainly at setup — not buried in Settings. */}
          <AlertNote variant="danger" className="mt-3">
            If you lose both your {mode === "pin" ? "PIN" : "password"} and these codes, there is
            no way back in. No vendor backdoor, no email reset — nobody but you can ever recover
            this console.
          </AlertNote>

          <div className="mt-4 grid grid-cols-2 gap-1.5 rounded-sm border border-border bg-bg p-3">
            {recoveryCodes.map((code) => (
              <div key={code} className="mono text-center text-[12.5px] tracking-[0.02em]">
                {code}
              </div>
            ))}
          </div>

          <label htmlFor="setup-confirm-saved" className="mt-3.5 mb-3 flex items-start gap-2 text-body">
            <input
              id="setup-confirm-saved"
              type="checkbox"
              checked={confirmed}
              onChange={(e) => setConfirmed(e.target.checked)}
              className="mt-0.5"
            />
            <span>I have saved these recovery codes somewhere safe, outside this console.</span>
          </label>
          <Button variant="primary" className="w-full" disabled={!confirmed} onClick={handleFinish}>
            Enter the console
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-bg px-4">
      <div className="w-full max-w-95 rounded-lg border border-border bg-surface p-6">
        <AuthBrandMark>
          <ShieldCheck className="size-6" />
        </AuthBrandMark>
        <h1 className="mt-3.5 text-center text-title">Set up the console</h1>
        <p className="mt-1 text-center text-caption">
          Choose how you will unlock this console. This runs once, before the first sign-in.
        </p>

        <div className="mt-4 flex justify-center">
          <SegmentedControl
            value={mode}
            onValueChange={setMode}
            options={[
              { value: "pin", label: "PIN" },
              { value: "password", label: "Password" },
            ]}
          />
        </div>

        <div className="mt-3.5 flex flex-col gap-3">
          {mode === "pin" ? (
            <>
              <div>
                <label htmlFor="setup-pin" className="text-label mb-1 block">
                  Choose a 6-digit PIN
                </label>
                <Input
                  id="setup-pin"
                  type="password"
                  inputMode="numeric"
                  maxLength={6}
                  placeholder="••••••"
                  value={pin}
                  onChange={(e) => setPin(e.target.value.replace(/\D/g, ""))}
                />
                <InputHint>Numbers only, exactly 6 digits.</InputHint>
              </div>
              <div>
                <label htmlFor="setup-pin2" className="text-label mb-1 block">
                  Confirm PIN
                </label>
                <Input
                  id="setup-pin2"
                  type="password"
                  inputMode="numeric"
                  maxLength={6}
                  placeholder="••••••"
                  value={pin2}
                  onChange={(e) => setPin2(e.target.value.replace(/\D/g, ""))}
                />
              </div>
            </>
          ) : (
            <>
              <div>
                <label htmlFor="setup-pw" className="text-label mb-1 block">
                  Choose a password
                </label>
                <Input
                  id="setup-pw"
                  type="password"
                  placeholder="At least 8 characters"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
                <InputHint>At least 8 characters, with a letter and a number.</InputHint>
              </div>
              <div>
                <label htmlFor="setup-pw2" className="text-label mb-1 block">
                  Confirm password
                </label>
                <Input
                  id="setup-pw2"
                  type="password"
                  placeholder="Re-type your password"
                  value={password2}
                  onChange={(e) => setPassword2(e.target.value)}
                />
              </div>
            </>
          )}
          {error && <InputHint error>{error}</InputHint>}
          <Button variant="primary" className="w-full" disabled={submitting} onClick={handleContinue}>
            Continue
          </Button>
        </div>

        <p className="mt-3.5 text-center text-caption">
          <a href="/auth/data-recovery?from=setup" className="text-accent">
            Restore from a backup file instead
          </a>
        </p>
      </div>
    </div>
  );
}
