import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// ADR-004: every volume/reward figure crosses the IPC boundary as an ×100
// integer (see src/lib/ipc/entities.ts's own header note) — conversion to
// a two-decimal display string happens only here, at the UI boundary.
export function centsToDisplay(cents: number): string {
  return (cents / 100).toFixed(2);
}

/** Rule-16/Rule-16a: `> 0`, at most two decimal places. `null` on anything
 * else — the caller decides how to surface that as a field error. */
export function displayToCents(value: string): number | null {
  const trimmed = value.trim();
  if (!/^\d+(\.\d{1,2})?$/.test(trimmed)) return null;
  const cents = Math.round(parseFloat(trimmed) * 100);
  return cents > 0 ? cents : null;
}

// "2026-06" -> "June 2026" — every screen that names a period month uses
// this, not its own formatting (monthly-close.tsx, business-volume-entry.tsx).
export function monthLabel(periodMonth: string): string {
  const [year, month] = periodMonth.split("-").map(Number);
  return new Date(year, month - 1, 1).toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  });
}

export function isoDate(d: Date) {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export function currentYm(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

// T-M2.1-4/T-M2.3-4: bounded to the given month. The current month caps at
// today (can't record ahead of itself); any other month is bounded by its
// own first/last calendar day — true for both a not-yet-closed month
// (business-volume-entry.tsx) and a closed one being corrected
// (correction-panel.tsx).
export function monthBounds(ym: string) {
  const [year, month] = ym.split("-").map(Number);
  const first = new Date(year, month - 1, 1);
  const last = new Date(year, month, 0);
  const isCurrent = ym === currentYm();
  return { min: isoDate(first), max: isoDate(isCurrent ? new Date() : last) };
}

// Strips anything that isn't a digit or decimal point as the operator
// types, and collapses a second "." rather than let it through — a
// keystroke-level guard only. The actual amount validation stays in
// displayToCents (Rule-16/Rule-16a) below, untouched.
export function stripNonNumeric(raw: string): string {
  const digitsAndDots = raw.replace(/[^\d.]/g, "");
  const firstDot = digitsAndDots.indexOf(".");
  if (firstDot === -1) return digitsAndDots;
  return (
    digitsAndDots.slice(0, firstDot + 1) + digitsAndDots.slice(firstDot + 1).replace(/\./g, "")
  );
}
