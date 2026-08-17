import { invokeCommand } from "./client";
import type { BackupRecord, Period } from "./entities";

// API-12
export function getOutstandingPeriods(): Promise<Period[]> {
  return invokeCommand("get_outstanding_periods");
}

export interface BeginCloseResult {
  periodId: number;
  memberCount: number;
  withEntryCount: number;
  topSlabCount: number;
}

// API-13 — only the oldest outstanding period may begin; there is no id
// parameter to request a different one.
export function beginClose(): Promise<BeginCloseResult> {
  return invokeCommand("begin_close");
}

export interface ConfirmBackupAndCloseInput {
  periodId: number;
  externalMediumPath?: string;
}

export interface CloseOutcome {
  // Rule-31: an external-medium path was given and the copy to it failed —
  // never blocks the close, but the caller should remind the operator.
  externalMediumCopyFailed: boolean;
}

// API-14 — backup-gated: write+verify backup → snapshots v1 → zero live
// figures → mark closed. Verify failure never begins the zeroing phase.
export function confirmBackupAndClose(
  input: ConfirmBackupAndCloseInput,
): Promise<CloseOutcome> {
  return invokeCommand("confirm_backup_and_close", { input });
}

// API-15 — on-demand backup of the in-progress month, no zeroing.
export function manualBackupCurrentPeriod(): Promise<BackupRecord> {
  return invokeCommand("manual_backup_current_period");
}
