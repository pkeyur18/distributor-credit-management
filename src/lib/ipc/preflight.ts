import { invokeCommand } from "./client";
import type { BackupRecord } from "./entities";

// These four are unauthenticated of necessity — the encrypted database may
// not even be openable yet. See §8.6: check_data_readable and
// list_restore_points reveal only that backups exist and roughly when.

// API-34 — decides sign-in vs. the data-recovery screen.
export function checkDataReadable(): Promise<boolean> {
  return invokeCommand("check_data_readable");
}

// API-35 — every backup kind, newest first.
export function listRestorePoints(): Promise<BackupRecord[]> {
  return invokeCommand("list_restore_points");
}

// API-36 — checksum must verify before overwriting; writes a
// pre_restore_safety backup of the current file first.
export function restoreFromBackup(backupId: number): Promise<void> {
  return invokeCommand("restore_from_backup", { backupId });
}

// API-40 — same checksum-verify-first requirement, for an admin-picked file
// rather than a retained backup (first-run "Restore instead" / Settings' restore-from-file).
export function restoreFromBackupFile(filePath: string): Promise<void> {
  return invokeCommand("restore_from_backup_file", { filePath });
}
