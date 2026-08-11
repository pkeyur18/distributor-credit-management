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
