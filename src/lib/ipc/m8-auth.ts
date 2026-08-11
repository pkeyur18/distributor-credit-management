import { invokeCommand } from "./client";
import type { BackupRecord } from "./entities";

export interface SetupFirstRunInput {
  pin?: string;
  password?: string;
}

export interface SetupFirstRunResult {
  recoveryCodes: string[];
}

// API-26 — unauthenticated, only when no auth row exists yet.
export function setupFirstRun(input: SetupFirstRunInput): Promise<SetupFirstRunResult> {
  return invokeCommand("setup_first_run", { input });
}

export interface CredentialInput {
  pin?: string;
  password?: string;
}

// API-27 — unauthenticated entry point. Generic failure message regardless
// of which credential type or part was wrong (Rule-29).
export function login(credential: CredentialInput): Promise<void> {
  return invokeCommand("login", { input: credential });
}

// API-28 — idempotent, not audited.
export function lockSession(): Promise<void> {
  return invokeCommand("lock_session");
}

// API-29 — same verification/lockout semantics as login.
export function unlockSession(credential: CredentialInput): Promise<void> {
  return invokeCommand("unlock_session", { input: credential });
}

export interface UseRecoveryCodeInput {
  code: string;
  newPin?: string;
  newPassword?: string;
}

export interface UseRecoveryCodeResult {
  recoveryCodes: string[];
}

// API-30 — unauthenticated. Invalidates every prior code, issues a fresh set.
export function useRecoveryCode(input: UseRecoveryCodeInput): Promise<UseRecoveryCodeResult> {
  return invokeCommand("use_recovery_code", { input });
}

export interface OutstandingAlert {
  outstandingMonths: string[];
  currentMonth: string;
}

// API-31
export function getOutstandingAlert(): Promise<OutstandingAlert> {
  return invokeCommand("get_outstanding_alert");
}

// API-39 — immediate whole-console backup; kind is inferred server-side
// (manual when user-triggered, scheduled when the login check triggers it).
export function runConsoleBackupNow(): Promise<BackupRecord> {
  return invokeCommand("run_console_backup_now");
}
