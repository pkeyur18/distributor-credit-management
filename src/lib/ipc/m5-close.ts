import { invokeCommand } from "./client";
import type { BackupRecord, Period } from "./entities";

// API-12
export function getOutstandingPeriods(): Promise<Period[]> {
  return invokeCommand("get_outstanding_periods");
}

// API-13 — only the oldest outstanding period may begin.
export function beginClose(): Promise<{ periodId: number }> {
  return invokeCommand("begin_close");
}

export interface ConfirmBackupAndCloseInput {
  periodId: number;
  externalMediumPath?: string;
}

// API-14 — backup-gated: write+verify backup → snapshots v1 → zero live
// figures → mark closed. Verify failure never begins the zeroing phase.
export function confirmBackupAndClose(input: ConfirmBackupAndCloseInput): Promise<void> {
  return invokeCommand("confirm_backup_and_close", { ...input });
}

// API-15 — on-demand backup of the in-progress month, no zeroing.
export function manualBackupCurrentPeriod(): Promise<BackupRecord> {
  return invokeCommand("manual_backup_current_period");
}
