import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { CalendarCheck, ShieldCheck, CheckCircle2 } from "lucide-react";
import { save as saveFileDialog } from "@tauri-apps/plugin-dialog";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/ui/pill";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow, TableWrap } from "@/components/ui/table";
import { LoadingState } from "@/components/loading-state";
import { useToast } from "@/components/ui/toast";
import { toErrorPresentation } from "@/lib/ipc/errors";
import {
  beginClose,
  confirmBackupAndClose,
  getOutstandingPeriods,
  manualBackupCurrentPeriod,
  type BeginCloseResult,
} from "@/lib/ipc/m5-close";
import type { Period } from "@/lib/ipc/entities";
import { monthLabel } from "@/lib/utils";
import { useOutstandingAlert } from "@/lib/outstanding-alert-context";

function errorMessage(raw: unknown): string {
  return toErrorPresentation(raw).message;
}

// --- close wizard (T-M5.1-9, US-M5.1, S11) ---
// Full-screen takeover matching the approved prototype's `wizard-shell` —
// no sidebar, no banner, while a close is in progress.

type WizardStep = "confirm" | "backup" | "closing" | "done";

function WizardShell({ children }: { children: ReactNode }) {
  return (
    <div className="fixed inset-0 z-50 flex min-h-screen items-center justify-center bg-bg px-6 py-10">
      <div className="w-full max-w-140">{children}</div>
    </div>
  );
}

function WizardIconWrap({
  variant,
  children,
}: {
  variant?: "success";
  children: ReactNode;
}) {
  return (
    <div
      className={`mx-auto mb-4.5 flex h-13 w-13 items-center justify-center rounded-2xl ${
        variant === "success" ? "bg-success-weak text-success" : "bg-accent-weak text-accent"
      }`}
    >
      {children}
    </div>
  );
}

function CloseWizard({
  period,
  begun,
  onExit,
  onClosed,
}: {
  period: Period;
  begun: BeginCloseResult;
  onExit: () => void;
  onClosed: () => void;
}) {
  const toast = useToast();
  const [step, setStep] = useState<WizardStep>("confirm");
  const [externalPath, setExternalPath] = useState<string | null>(null);
  const [backupConfirmed, setBackupConfirmed] = useState(false);
  const [choosingPath, setChoosingPath] = useState(false);
  const month = monthLabel(period.periodMonth);

  async function chooseBackupLocation() {
    setChoosingPath(true);
    try {
      // Rule-31/RQ-19: the external-medium copy is prompted at the same
      // time as the internal one but never blocks — cancelling this dialog
      // (no path chosen) still lets the close proceed.
      const path = await saveFileDialog({
        defaultPath: `${period.periodMonth}-close-backup.db`,
      });
      setExternalPath(path ?? null);
    } finally {
      setChoosingPath(false);
      setBackupConfirmed(true);
    }
  }

  async function commitClose() {
    setStep("closing");
    try {
      const outcome = await confirmBackupAndClose({
        periodId: period.id,
        externalMediumPath: externalPath ?? undefined,
      });
      if (outcome.externalMediumCopyFailed) {
        toast.add({
          title: "Closed — the external copy could not be written, back it up separately",
          type: "danger",
        });
      }
      setStep("done");
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
      setStep("backup");
    }
  }

  if (step === "confirm") {
    return (
      <WizardShell>
        <div className="text-center">
          <WizardIconWrap>
            <CalendarCheck className="h-6 w-6" />
          </WizardIconWrap>
          <h1 className="text-headline">Close {month}</h1>
          <p className="mx-auto mt-1.5 max-w-105 text-caption text-muted-text">
            This writes a permanent record for every member, then clears live Business Volume,
            Total Business Volume and Rewards to zero for the new period.{" "}
            <strong>This step cannot be undone.</strong>
          </p>
        </div>
        <Card className="mt-5.5">
          <div className="grid grid-cols-3 gap-3 p-4 text-center">
            <div>
              <div className="text-label text-muted-text">Members</div>
              <div className="text-numeric mt-1">{begun.memberCount}</div>
            </div>
            <div>
              <div className="text-label text-muted-text">With an entry</div>
              <div className="text-numeric mt-1">{begun.withEntryCount}</div>
            </div>
            <div>
              <div className="text-label text-muted-text">On top slab</div>
              <div className="text-numeric mt-1">{begun.topSlabCount}</div>
            </div>
          </div>
        </Card>
        <div className="mt-5 flex justify-center gap-2">
          <Button variant="secondary" onClick={onExit}>
            Cancel
          </Button>
          <Button onClick={() => setStep("backup")}>Continue</Button>
        </div>
      </WizardShell>
    );
  }

  if (step === "backup") {
    return (
      <WizardShell>
        <div className="text-center">
          <WizardIconWrap>
            <ShieldCheck className="h-6 w-6" />
          </WizardIconWrap>
          <h1 className="text-headline">Confirm a backup</h1>
          <p className="mx-auto mt-1.5 max-w-105 text-caption text-muted-text">
            A close never proceeds without a confirmed backup. The copy is retained permanently
            inside the console, and you may additionally choose a separate medium to copy it to.
          </p>
        </div>
        <Card className="mt-5.5">
          <div className="flex items-center gap-2.5 p-4 text-body">
            <span
              className={`flex h-4.5 w-4.5 shrink-0 items-center justify-center rounded-full ${
                backupConfirmed ? "bg-success-weak text-success" : "border border-border"
              }`}
            >
              {backupConfirmed && <CheckCircle2 className="h-3.25 w-3.25" />}
            </span>
            <span>Backup copy generated and confirmed</span>
          </div>
          {backupConfirmed && (
            <p className="px-4 pb-4 text-label text-muted-text">
              Closing writes the permanent record and clears live figures immediately — there is
              no way to reopen {month} after this.
            </p>
          )}
        </Card>
        <div className="mt-5 flex justify-center gap-2">
          {backupConfirmed ? (
            <>
              <Button variant="secondary" onClick={() => setStep("confirm")}>
                Back
              </Button>
              <Button variant="commit" onClick={commitClose}>
                Close {month} — cannot be undone
              </Button>
            </>
          ) : (
            <Button disabled={choosingPath} onClick={chooseBackupLocation}>
              {choosingPath ? "Generating…" : "Generate backup"}
            </Button>
          )}
        </div>
      </WizardShell>
    );
  }

  if (step === "closing") {
    return (
      <WizardShell>
        <div className="text-center">
          <LoadingState label="Writing the permanent record…" />
        </div>
      </WizardShell>
    );
  }

  return (
    <WizardShell>
      <div className="text-center">
        <WizardIconWrap variant="success">
          <CheckCircle2 className="h-6 w-6" />
        </WizardIconWrap>
        <h1 className="text-headline">{month} closed</h1>
        <p className="mt-1.5 text-caption text-muted-text">
          The permanent record is written and live figures are cleared.
        </p>
      </div>
      <div className="mt-5 flex justify-center">
        <Button onClick={onClosed}>Return to console</Button>
      </div>
    </WizardShell>
  );
}

// --- status page ---

export function MonthlyClose() {
  const toast = useToast();
  const { refresh: refreshAlert } = useOutstandingAlert();
  const [outstanding, setOutstanding] = useState<Period[] | null>(null);
  const [wizard, setWizard] = useState<{ period: Period; begun: BeginCloseResult } | null>(null);
  const [backingUp, setBackingUp] = useState(false);

  async function refresh() {
    try {
      setOutstanding(await getOutstandingPeriods());
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    }
    refreshAlert();
  }

  useEffect(() => {
    getOutstandingPeriods()
      .then(setOutstanding)
      .catch((raw) => toast.add({ title: errorMessage(raw), type: "danger" }));
    refreshAlert();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function startClose(period: Period) {
    try {
      const begun = await beginClose();
      setWizard({ period, begun });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    }
  }

  async function backUpNow() {
    setBackingUp(true);
    try {
      await manualBackupCurrentPeriod();
      toast.add({ title: "This month backed up", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setBackingUp(false);
    }
  }

  if (wizard) {
    return (
      <CloseWizard
        period={wizard.period}
        begun={wizard.begun}
        onExit={() => setWizard(null)}
        onClosed={() => {
          setWizard(null);
          refresh();
        }}
      />
    );
  }

  return (
    <>
      <h1 className="text-headline">Monthly close</h1>
      <p className="mt-1 text-caption">Close months oldest first — each close writes a permanent record</p>

      <Card className="mt-5">
        <CardHeader className="flex items-center justify-between">
          <div>
            <CardTitle>Outstanding months</CardTitle>
            <CardDescription>Only the oldest can be closed</CardDescription>
          </div>
          <Button variant="secondary" size="sm" disabled={backingUp} onClick={backUpNow}>
            {backingUp ? "Backing up…" : "Back up this month"}
          </Button>
        </CardHeader>

        {outstanding === null ? null : outstanding.length === 0 ? (
          <p className="p-4 text-body text-muted-text">No months are waiting to close.</p>
        ) : (
          <TableWrap>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Month</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {outstanding.map((period, i) => (
                  <TableRow key={period.id}>
                    <TableCell primary>{monthLabel(period.periodMonth)}</TableCell>
                    <TableCell>
                      {i === 0 ? (
                        <Pill variant="locked">Oldest — closable now</Pill>
                      ) : (
                        <Pill variant="neutral">Waiting</Pill>
                      )}
                    </TableCell>
                    <TableCell>
                      {i === 0 && (
                        <Button size="sm" onClick={() => startClose(period)}>
                          Close
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableWrap>
        )}
      </Card>
    </>
  );
}
