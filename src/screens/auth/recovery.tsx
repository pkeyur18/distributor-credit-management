import { useState } from "react";
import { useNavigate } from "react-router";
import { KeyRound } from "lucide-react";

import { AuthBrandMark } from "@/components/auth-brand-mark";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { Input, InputHint } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useRecoveryCode } from "@/lib/ipc/m8-auth";
import { toErrorPresentation } from "@/lib/ipc/errors";

// US-M8.4 (§5.10 Recovery, ui-prototype-v2.html's renderRecovery). Three
// steps: enter the code, choose a new credential, save the fresh codes.
// Only one command exists (API-30) and it verifies the code *and* sets the
// new credential in one call, so step 0's "Verify code" is a format check
// only — a wrong/used code surfaces back here once step 1 actually submits.
function validPin(v: string) {
  return /^[0-9]{6}$/.test(v);
}
function validPassword(v: string) {
  return v.length >= 8 && /[a-zA-Z]/.test(v) && /[0-9]/.test(v);
}

export function Recovery() {
  const [step, setStep] = useState<0 | 1 | 2>(0);
  const [code, setCode] = useState("");
  const [mode, setMode] = useState<"pin" | "password">("pin");
  const [pin, setPin] = useState("");
  const [pin2, setPin2] = useState("");
  const [password, setPassword] = useState("");
  const [password2, setPassword2] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([]);
  const [confirmed, setConfirmed] = useState(false);
  const navigate = useNavigate();

  function verifyFormat() {
    setError(null);
    if (!code.trim()) {
      setError("Enter the recovery code you saved when this console was first set up.");
      return;
    }
    setStep(1);
  }

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
      // Not a React hook — this is m8-auth.ts's IPC wrapper for API-30,
      // named to match the command; the lint rule only pattern-matches the
      // "use" prefix.
      // eslint-disable-next-line react-hooks/rules-of-hooks
      const result = await useRecoveryCode({
        code: code.trim(),
        ...(mode === "pin" ? { newPin: pin } : { newPassword: password }),
      });
      setRecoveryCodes(result.recoveryCodes);
      setStep(2);
    } catch (raw) {
      const presented = toErrorPresentation(raw);
      setError(presented.message);
      // A refused code is the one thing that sends the operator back to
      // re-enter it, rather than just re-showing this step's own error.
      if (presented.kind === "validation" && presented.field === "code") {
        setStep(0);
      }
    } finally {
      setSubmitting(false);
    }
  }

  function handleFinish() {
    navigate("/auth/login", { replace: true });
  }

  if (step === 2) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-bg px-4">
        <div className="w-full max-w-95 rounded-lg border border-border bg-surface p-6">
          <AuthBrandMark tone="success">
            <KeyRound className="size-6" />
          </AuthBrandMark>
          <h1 className="mt-3.5 text-center text-title">Save your new recovery codes</h1>
          <p className="mt-1 text-center text-caption">
            These replace your old codes, which no longer work.
          </p>

          <div className="mt-4 grid grid-cols-2 gap-1.5 rounded-sm border border-border bg-bg p-3">
            {recoveryCodes.map((c) => (
              <div key={c} className="mono text-center text-[12.5px] tracking-[0.02em]">
                {c}
              </div>
            ))}
          </div>

          <label htmlFor="recovery-confirm-saved" className="mt-3.5 mb-3 flex items-start gap-2 text-body">
            <input
              id="recovery-confirm-saved"
              type="checkbox"
              checked={confirmed}
              onChange={(e) => setConfirmed(e.target.checked)}
              className="mt-0.5"
            />
            <span>I have saved these recovery codes somewhere safe, outside this console.</span>
          </label>
          <Button variant="primary" className="w-full" disabled={!confirmed} onClick={handleFinish}>
            Sign in with new credential
          </Button>
        </div>
      </div>
    );
  }

  if (step === 1) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-bg px-4">
        <div className="w-full max-w-95 rounded-lg border border-border bg-surface p-6">
          <AuthBrandMark>
            <KeyRound className="size-6" />
          </AuthBrandMark>
          <h1 className="mt-3.5 text-center text-title">Set a new credential</h1>

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
                  <label htmlFor="recovery-pin" className="text-label mb-1 block">
                    New 6-digit PIN
                  </label>
                  <Input
                    id="recovery-pin"
                    type="password"
                    inputMode="numeric"
                    maxLength={6}
                    placeholder="••••••"
                    value={pin}
                    onChange={(e) => setPin(e.target.value.replace(/\D/g, ""))}
                  />
                </div>
                <div>
                  <label htmlFor="recovery-pin2" className="text-label mb-1 block">
                    Confirm PIN
                  </label>
                  <Input
                    id="recovery-pin2"
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
                  <label htmlFor="recovery-pw" className="text-label mb-1 block">
                    New password
                  </label>
                  <Input
                    id="recovery-pw"
                    type="password"
                    placeholder="At least 8 characters"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                  />
                  <InputHint>At least 8 characters, with a letter and a number.</InputHint>
                </div>
                <div>
                  <label htmlFor="recovery-pw2" className="text-label mb-1 block">
                    Confirm password
                  </label>
                  <Input
                    id="recovery-pw2"
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
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-bg px-4">
      <div className="w-full max-w-95 rounded-lg border border-border bg-surface p-6">
        <AuthBrandMark>
          <KeyRound className="size-6" />
        </AuthBrandMark>
        <h1 className="mt-3.5 text-center text-title">Recover access</h1>
        <p className="mt-1 text-center text-caption">
          Enter the recovery code you saved when this console was first set up.
        </p>

        <div className="mt-4 flex flex-col gap-3">
          <div>
            <label htmlFor="recovery-code-input" className="text-label mb-1 block">
              Recovery code
            </label>
            <Input
              id="recovery-code-input"
              className="mono text-center"
              placeholder="XXXXX-XXXXX-XXXXX"
              value={code}
              onChange={(e) => setCode(e.target.value)}
            />
          </div>
          {error && <InputHint error>{error}</InputHint>}
          <Button variant="primary" className="w-full" onClick={verifyFormat}>
            Verify code
          </Button>
        </div>

        <p className="mt-3.5 text-center text-caption">
          <a href="/auth/login" className="text-accent">
            Back to sign in
          </a>
        </p>
      </div>
    </div>
  );
}
