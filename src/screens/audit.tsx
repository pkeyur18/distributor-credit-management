import { useEffect, useState } from "react";
import { FileText, Search } from "lucide-react";

import { Input } from "@/components/ui/input";
import { EmptyState } from "@/components/empty-state";
import { getAuditLog } from "@/lib/ipc/m9-audit";
import type { AuditCause, AuditEntityType, AuditLogEntry } from "@/lib/ipc/entities";

const CAUSE_LABELS: Record<AuditCause, string> = {
  entry: "Recorded",
  edit: "Edited",
  correction: "Corrected",
  settings_change: "Setting changed",
  period_close: "Month closed",
  manual_backup: "Manual backup",
  console_backup: "Console backup",
};

const ENTITY_LABELS: Record<AuditEntityType, string> = {
  member: "Member",
  entry: "Business Volume entry",
  setting: "Setting",
  period: "Period",
  backup: "Backup",
  auth: "Credential",
};

function formatChangedAt(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
}

// US-M9.1 (§5.9, T-M9.1-4). Chronological, read-only, filterable by member
// name, ID or phone — the filter is `get_audit_log`'s own `memberQuery`,
// which reuses Rule-44's shared search server-side rather than this screen
// re-implementing the matching rules. No entry is ever edited or removed
// (append-only, T-M9.1-1), so this screen has no write path of any kind.
export function Audit() {
  const [query, setQuery] = useState("");
  const [entries, setEntries] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    const trimmed = query.trim();
    const timer = setTimeout(() => {
      setLoading(true);
      getAuditLog(trimmed ? { memberQuery: trimmed } : {})
        .then((found) => {
          if (!cancelled) setEntries(found);
        })
        .finally(() => {
          if (!cancelled) setLoading(false);
        });
    }, 200);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [query]);

  return (
    <>
      <h1 className="text-headline">Audit log</h1>
      <p className="text-caption mt-1">
        {entries.length} recorded {entries.length === 1 ? "change" : "changes"} — no entry is
        ever edited or removed
      </p>

      <div className="mt-4 max-w-85">
        <div className="relative">
          <Search className="pointer-events-none absolute left-2.75 top-1/2 size-3.75 -translate-y-1/2 text-muted-text" />
          <Input
            id="audit-filter"
            className="pl-8"
            placeholder="Filter by member name, ID or phone"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
      </div>

      <div className="mt-4">
        {loading ? null : entries.length === 0 ? (
          <EmptyState icon={<FileText className="size-8" />} title="No matching entries" />
        ) : (
          <div className="divide-y divide-border rounded-sm border border-border bg-surface">
            {entries.map((entry) => (
              <div key={entry.id} className="flex flex-col gap-0.5 px-3 py-2.5">
                <div className="flex items-center justify-between gap-3">
                  <span className="text-body font-semibold text-ink">
                    {ENTITY_LABELS[entry.entityType]} · {entry.field}
                  </span>
                  <span className="shrink-0 text-caption text-muted-text">
                    {formatChangedAt(entry.changedAt)}
                  </span>
                </div>
                <div className="text-caption text-muted-text">
                  {entry.oldValue ?? "—"} → {entry.newValue ?? "—"}
                  <span className="ml-2">{CAUSE_LABELS[entry.cause]}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
