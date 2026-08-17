import { invokeCommand } from "./client";
import type { BusinessVolumeEntry, PeriodEntries } from "./entities";

export interface PeriodLockStatus {
  /** Recordable periods, oldest first — never a plain boolean (CR-2). */
  recordablePeriodMonths: string[];
  /** The current month, when it's refused because an earlier one is outstanding. */
  blockingMonth: string | null;
}

// API-07
export function getPeriodLockStatus(): Promise<PeriodLockStatus> {
  return invokeCommand("get_period_lock_status");
}

export interface RecordEntryInput {
  memberId: number;
  amount: number;
  entryDate: string;
}

// API-08 — refused when the entry's own period is closed, or when it's the
// current month while an earlier period is awaiting_close (Rule-36/V2.7).
export function recordEntry(input: RecordEntryInput): Promise<BusinessVolumeEntry> {
  return invokeCommand("record_entry", { input });
}

// API-41 — the period table and its two summary nodes on Volume Entry all
// read off this one list; `periodMonth` is caller-supplied (already
// resolved via `getPeriodLockStatus`), never re-derived server-side.
export function listPeriodEntries(periodMonth: string): Promise<PeriodEntries> {
  return invokeCommand("list_period_entries", { periodMonth });
}

export interface EditEntryInput {
  id: number;
  amount: number;
  entryDate: string;
}

// API-09 — an open-period edit or a closed-month correction; closed months
// write a new snapshot version, version 1 is never touched (Rule-39).
export function editEntry(input: EditEntryInput): Promise<BusinessVolumeEntry> {
  return invokeCommand("edit_entry", { input });
}

export interface AddClosedMonthEntryInput {
  memberId: number;
  amount: number;
  entryDate: string;
}

// API-45 — correction panel's "Add record": a brand-new entry into an
// already-closed month, writing a new snapshot version the same way
// editEntry's closed-month path does (Rule-39, extended to creation).
export function addClosedMonthEntry(input: AddClosedMonthEntryInput): Promise<BusinessVolumeEntry> {
  return invokeCommand("add_closed_month_entry", { input });
}
