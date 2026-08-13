import { invokeCommand } from "./client";
import type { BackupRecord } from "./entities";

export interface ExportResult {
  filePath: string;
}

export interface ExportMonthlyInput {
  periodMonth: string;
  /** Optional columns beyond the five mandatory ones (D-1). */
  optionalColumns?: string[];
  /** Destination path chosen through the native save dialog (ADR-007) — the
   *  WebView never handles raw file content, only the path string itself. */
  outputPath: string;
}

// API-16 — the five mandatory columns (D-1) are always included regardless of selection.
export function exportMonthly(input: ExportMonthlyInput): Promise<ExportResult> {
  return invokeCommand("export_monthly", { ...input });
}

// API-17 — snapshot-count denominator, displayed alongside the average
// (Rule-23), divided per member so a late joiner's average isn't dragged
// down by periods before they existed (T-M6.2-1).
export function exportYearlyAverage(outputPath: string): Promise<ExportResult> {
  return invokeCommand("export_yearly_average", { outputPath });
}

export interface ExportLowContributionInput {
  threshold?: number;
}

// API-18 — filters on own Business Volume, not Total Business Volume (Rule-24).
export function exportLowContribution(
  input: ExportLowContributionInput = {},
): Promise<ExportResult> {
  return invokeCommand("export_low_contribution", { ...input });
}

// API-19
export function listBackups(): Promise<BackupRecord[]> {
  return invokeCommand("list_backups");
}

// API-20 — always the latest version of that period's backup.
export function redownloadBackup(periodId: number): Promise<ExportResult> {
  return invokeCommand("redownload_backup", { periodId });
}
