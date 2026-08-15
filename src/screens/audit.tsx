import { useEffect, useState } from "react";
import { FileText, Search } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { EmptyState } from "@/components/empty-state";
import { PageHeader } from "@/components/page-header";
import {
  TableWrap,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import { getAuditLog } from "@/lib/ipc/m9-audit";
import type { AuditCause, AuditLogEntry } from "@/lib/ipc/entities";

const PAGE_SIZES = [10, 25, 50] as const;

// Fuller than the raw enum key, but kept generic enough to stay accurate
// across every entity type that shares a given cause (e.g. "entry" covers
// volume entries, member onboarding and credential setup alike) — the
// prototype's mock data invents one bespoke sentence per event, which the
// real 7-value enum can't reproduce 1:1 without misdescribing some rows.
const CAUSE_LABELS: Record<AuditCause, string> = {
  entry: "New record recorded",
  edit: "Record edited",
  correction: "Closed-month record corrected — new snapshot version created",
  settings_change: "Setting changed by administrator",
  period_close: "Month closed — permanent record written, live figures cleared",
  manual_backup: "Manual backup created",
  console_backup: "Console backup created",
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
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState<number>(PAGE_SIZES[0]);

  useEffect(() => {
    let cancelled = false;
    const trimmed = query.trim();
    const timer = setTimeout(() => {
      setLoading(true);
      getAuditLog(trimmed ? { memberQuery: trimmed } : {})
        .then((found) => {
          if (!cancelled) {
            setEntries(found);
            setPage(0);
          }
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

  const totalPages = Math.max(1, Math.ceil(entries.length / pageSize));
  const currentPage = Math.min(page, totalPages - 1);
  const rangeStart = currentPage * pageSize;
  const pageRows = entries.slice(rangeStart, rangeStart + pageSize);

  return (
    <>
      <PageHeader
        title="Audit log"
        subtitle={`${entries.length} recorded ${entries.length === 1 ? "change" : "changes"} — no entry is ever edited or removed`}
      />

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
          <div className="rounded-lg border border-border bg-surface">
            <EmptyState icon={<FileText className="size-8" />} title="No matching entries" />
          </div>
        ) : (
          <>
            <TableWrap>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Date</TableHead>
                    <TableHead>Member</TableHead>
                    <TableHead>Field</TableHead>
                    <TableHead>Before</TableHead>
                    <TableHead>After</TableHead>
                    <TableHead>Cause</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {pageRows.map((entry) => (
                    <TableRow key={entry.id}>
                      <TableCell className="mono whitespace-nowrap">
                        {formatChangedAt(entry.changedAt)}
                      </TableCell>
                      <TableCell>{entry.memberName ?? "—"}</TableCell>
                      <TableCell>{entry.field}</TableCell>
                      <TableCell className="text-muted-text">{entry.oldValue ?? "—"}</TableCell>
                      <TableCell>{entry.newValue ?? "—"}</TableCell>
                      <TableCell className="text-muted-text">
                        {CAUSE_LABELS[entry.cause]}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableWrap>
            <div className="mt-2.5 flex items-center justify-between">
              <span className="text-caption">
                Showing {rangeStart + 1}–{Math.min(rangeStart + pageSize, entries.length)} of{" "}
                {entries.length}
              </span>
              <div className="flex items-center gap-2">
                <label htmlFor="audit-page-size" className="text-caption">
                  Rows per page
                </label>
                <select
                  id="audit-page-size"
                  className="h-7.5 w-auto rounded-sm border border-border bg-surface px-2 text-body text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
                  value={pageSize}
                  onChange={(e) => {
                    setPageSize(Number(e.target.value));
                    setPage(0);
                  }}
                >
                  {PAGE_SIZES.map((size) => (
                    <option key={size} value={size}>
                      {size}
                    </option>
                  ))}
                </select>
                <Button
                  variant="secondary"
                  size="sm"
                  disabled={currentPage === 0}
                  onClick={() => setPage((p) => p - 1)}
                >
                  Prev
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  disabled={currentPage >= totalPages - 1}
                  onClick={() => setPage((p) => p + 1)}
                >
                  Next
                </Button>
              </div>
            </div>
          </>
        )}
      </div>
    </>
  );
}
