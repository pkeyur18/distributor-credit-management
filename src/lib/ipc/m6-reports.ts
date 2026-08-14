import { invokeCommand } from "./client";

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
  return invokeCommand("export_monthly", { input });
}

// API-17 — snapshot-count denominator, displayed alongside the average
// (Rule-23), divided per member so a late joiner's average isn't dragged
// down by periods before they existed (T-M6.2-1).
export function exportYearlyAverage(outputPath: string): Promise<ExportResult> {
  return invokeCommand("export_yearly_average", { outputPath });
}

export interface ExportLowContributionInput {
  /** Cents (ADR-004). Omit to read settings.lowContributionThreshold. */
  threshold?: number;
  outputPath: string;
}

// API-18 — filters on own Business Volume, not Total Business Volume (Rule-24).
export function exportLowContribution(input: ExportLowContributionInput): Promise<ExportResult> {
  return invokeCommand("export_low_contribution", { input });
}

// Closed periods that have a snapshot (T-M5.4-2 — an empty-month close is
// never listed here) — distinct from `preflight.ts`'s `BackupRecord`,
// which lists every whole-console backup for Settings' restore card.
export interface ClosedMonthBackup {
  periodId: number;
  periodMonth: string;
  latestVersion: number;
  isCorrected: boolean;
}

// API-19
export function listBackups(): Promise<ClosedMonthBackup[]> {
  return invokeCommand("list_backups");
}

// API-20 — always the latest version of that period's backup (T-M6.4-2).
export function redownloadBackup(periodId: number, outputPath: string): Promise<ExportResult> {
  return invokeCommand("redownload_backup", { periodId, outputPath });
}

export interface MonthlyPreviewRow {
  id: number;
  name: string;
  businessVolume: number;
  totalBusinessVolume: number;
  slabPct: number;
}

// API-43 — read-only, sorted by Total Business Volume descending. Backs the
// Reports screen's on-screen "Monthly data" preview table; never touches
// the filesystem (ADR-007's boundary is unaffected by a plain data return).
export function previewMonthlyData(periodMonth: string): Promise<MonthlyPreviewRow[]> {
  return invokeCommand("preview_monthly_data", { periodMonth });
}

export interface YearlyAveragePreviewRow {
  id: number;
  name: string;
  avgBusinessVolume: number;
  avgTotalBusinessVolume: number;
  periodCount: number;
}

// API-44 — read-only, sorted by average Total Business Volume descending.
// Backs both the "Yearly average" preview table and the "Low-contribution
// report" stat-card/table, which filter this same list on own Business
// Volume client-side as the threshold input changes (no round-trip per
// keystroke) — Rule-24's "own BV, not Total BV" filter itself stays
// authoritative only in `export_low_contribution`.
export function previewYearlyAverage(): Promise<YearlyAveragePreviewRow[]> {
  return invokeCommand("preview_yearly_average");
}
