import type { BackupRecord } from "@/lib/ipc/entities";

// Rule-43/§7 Design system: "reused, not duplicated" — the Settings
// Restore card and the data-recovery screen (US-M8.6, T-M8.6-5/6) list the
// exact same backups through the same `RestoreOptionList` component, so
// they share these two label functions rather than each growing its own
// copy that could drift.

/** A scheduled/manual/pre_restore_safety row names *when* it was taken;
 * a period_close row names the month it holds. */
export function backupPrimaryLabel(record: BackupRecord): string {
  const date = new Date(record.createdAt).toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
  if (record.kind === "period_close") return `Closed month — ${date}`;
  if (record.kind === "pre_restore_safety") return `Safety copy — ${date}`;
  if (record.kind === "scheduled") {
    const cadence = record.scheduleKind
      ? record.scheduleKind[0].toUpperCase() + record.scheduleKind.slice(1)
      : "Scheduled";
    return `${cadence} — ${date}`;
  }
  return `Manual — ${date}`;
}

/** T-M8.6-5: retained backups are listed "marking corrected months" —
 * only `period_close` rows carry a meaningful correction history
 * (Rule-39's versioning); every other kind is always a single-shot
 * snapshot, so the "corrected" qualifier is scoped to that kind alone. */
export function backupProvenanceText(record: BackupRecord): string {
  if (record.kind === "period_close" && !record.isOriginal) {
    return `Version ${record.version}, corrected`;
  }
  return `Version ${record.version}`;
}
