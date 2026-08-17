import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { AlertTriangle, ShieldCheck } from "lucide-react";

import { AuthBrandMark } from "@/components/auth-brand-mark";
import { Button } from "@/components/ui/button";
import { RestoreOptionList } from "@/components/restore-option-list";
import { listRestorePoints, restoreFromBackup, restoreFromBackupFile } from "@/lib/ipc/preflight";
import { backupPrimaryLabel, backupProvenanceText } from "@/lib/backup-labels";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { useAuth } from "@/lib/auth-context";
import type { BackupRecord } from "@/lib/ipc/entities";

// US-M8.6 (§9.5, T-M8.6-5/6, ui-prototype-v2.html's renderDbRecovery). One
// screen, reworded, for two entry points — not a duplicate:
//   - forced (`?from` absent): this console's own data couldn't be opened
//     (`checkDataReadable` rejected, or `login` hit `data_unreadable`).
//     Lists retained backups, marking corrected months.
//   - voluntary (`?from=setup`, T-M8.1-5's plain link on the setup
//     screen): a brand-new machine has no local backups of its own, so it
//     skips straight to the file picker.
// `listRestorePoints`/`restoreFromBackup`/`restoreFromBackupFile` are
// unauthenticated-capable since S14 (see `backup.rs`'s manifest doc
// comment) — this screen is the reason they had to become so.
export function DataRecovery() {
  const [searchParams] = useSearchParams();
  const isVoluntary = searchParams.get("from") === "setup";
  const navigate = useNavigate();
  const { markSignedOut } = useAuth();

  const [points, setPoints] = useState<BackupRecord[]>([]);
  const [loading, setLoading] = useState(!isVoluntary);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (isVoluntary) return;
    listRestorePoints()
      .then((records) => {
        setPoints(records);
        setSelectedId(records[0] ? String(records[0].id) : null);
      })
      .catch(() => setPoints([]))
      .finally(() => setLoading(false));
  }, [isVoluntary]);

  function afterRestore() {
    // The restored file supplies its own credential regardless of why this
    // screen was reached — land on ordinary sign-in, which still requires it.
    markSignedOut();
    navigate("/auth/login", { replace: true });
  }

  async function chooseFile() {
    const path = await openFileDialog({ multiple: false, directory: false });
    if (!path || Array.isArray(path)) return;
    setBusy(true);
    setError(null);
    try {
      await restoreFromBackupFile(path);
      afterRestore();
    } catch (raw) {
      setError(toErrorPresentation(raw).message);
    } finally {
      setBusy(false);
    }
  }

  async function restoreSelected() {
    if (!selectedId) return;
    setBusy(true);
    setError(null);
    try {
      await restoreFromBackup(Number(selectedId));
      afterRestore();
    } catch (raw) {
      setError(toErrorPresentation(raw).message);
    } finally {
      setBusy(false);
    }
  }

  const selected = points.find((p) => String(p.id) === selectedId) ?? null;

  const heading = isVoluntary
    ? {
        icon: <ShieldCheck className="size-6.5" />,
        tone: "accent" as const,
        title: "Restore from a backup file",
        sub: "Bringing this console over from another computer? Choose the backup file you brought with you.",
      }
    : {
        icon: <AlertTriangle className="size-6.5" />,
        tone: "danger" as const,
        title: "This console could not open its data",
        sub: "Nothing has been lost. Every backup this console has taken is still here — choose one to go back to.",
      };

  return (
    <div className="flex min-h-screen items-center justify-center bg-bg px-4">
      <div className="w-full max-w-95 rounded-lg border border-border bg-surface p-6">
        <AuthBrandMark tone={heading.tone}>{heading.icon}</AuthBrandMark>
        <h1 className="mt-3.5 text-center text-title">{heading.title}</h1>
        <p className="mt-1 text-center text-caption">{heading.sub}</p>

        <div className="mt-4">
          {isVoluntary ? (
            <>
              <Button variant="primary" className="w-full" disabled={busy} onClick={chooseFile}>
                Choose backup file…
              </Button>
              <p className="mt-2.5 text-center text-caption">
                This console will come up in exactly the state that backup holds — the PIN or
                password from that backup will unlock it.
              </p>
            </>
          ) : loading ? null : points.length > 0 ? (
            <>
              <RestoreOptionList
                value={selectedId}
                onValueChange={setSelectedId}
                options={points.map((p) => ({
                  value: String(p.id),
                  primary: backupPrimaryLabel(p),
                  provenance: backupProvenanceText(p),
                }))}
              />
              <Button
                variant="primary"
                className="mt-3 w-full"
                disabled={!selected || busy}
                onClick={restoreSelected}
              >
                Restore from {selected ? backupPrimaryLabel(selected) : "selected backup"}
              </Button>
              <p className="mt-2.5 text-center text-caption">
                Anything recorded after that backup will need entering again.
              </p>
            </>
          ) : (
            <p className="text-center text-caption">
              No backup has been taken yet, so there is nothing to restore from.
            </p>
          )}
          {error && <p className="mt-3 text-center text-[11.5px] text-danger">{error}</p>}
        </div>

        <p className="mt-3.5 text-center text-caption">
          {isVoluntary ? (
            <a href="/auth/setup" className="text-accent">
              Back
            </a>
          ) : (
            <>
              <button
                type="button"
                className="text-accent"
                onClick={() => navigate("/auth/login", { replace: true })}
              >
                Try opening the data again
              </button>
              {" · "}
              <button type="button" className="text-accent" onClick={chooseFile} disabled={busy}>
                Browse a different backup file
              </button>
            </>
          )}
        </p>
      </div>
    </div>
  );
}
