import { invokeCommand } from "./client";
import type { AuditLogEntry } from "./entities";

export interface AuditLogFilter {
  memberQuery?: string;
}

// API-32 — read-only; reading the log produces no entry of its own.
export function getAuditLog(filter: AuditLogFilter = {}): Promise<AuditLogEntry[]> {
  return invokeCommand("get_audit_log", { ...filter });
}
