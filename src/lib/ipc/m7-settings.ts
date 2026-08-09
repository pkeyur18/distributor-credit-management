import { invokeCommand } from "./client";
import type { BackupScheduleKind, Settings, SlabRow } from "./entities";

// API-21
export function getSettings(): Promise<Settings> {
  return invokeCommand("get_settings");
}

// API-22 — caller must show the pre-save warning first, sourced from API-33.
export function updateSettings(patch: Partial<Settings>): Promise<Settings> {
  return invokeCommand("update_settings", { patch });
}

export interface SlabRowInput {
  threshold: number;
  percentage: number;
}

// API-23 — duplicate threshold rejected; no monotonicity check (Rule-41).
export function addSlabRow(input: SlabRowInput): Promise<SlabRow> {
  return invokeCommand("add_slab_row", { ...input });
}

// API-24 — refused when it's the last remaining row.
export function removeSlabRow(id: number): Promise<void> {
  return invokeCommand("remove_slab_row", { id });
}

// API-25
export function updateSlabRow(id: number, input: SlabRowInput): Promise<SlabRow> {
  return invokeCommand("update_slab_row", { id, ...input });
}

export interface ConsoleBackupSettings {
  schedule: "off" | BackupScheduleKind;
  retentionCount: number;
  folder: string;
}

// API-37
export function getConsoleBackupSettings(): Promise<ConsoleBackupSettings> {
  return invokeCommand("get_console_backup_settings");
}

// API-38
export function updateConsoleBackupSettings(
  input: ConsoleBackupSettings,
): Promise<ConsoleBackupSettings> {
  return invokeCommand("update_console_backup_settings", { ...input });
}
