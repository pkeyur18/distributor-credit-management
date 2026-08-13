import { useEffect, useState } from "react";
import { save as saveFileDialog } from "@tauri-apps/plugin-dialog";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { MonthSwitcher } from "@/components/month-switcher";
import { useToast } from "@/components/ui/toast";
import { getPeriodLockStatus, type PeriodLockStatus } from "@/lib/ipc/m2-entries";
import { exportMonthly } from "@/lib/ipc/m6-reports";
import { MANDATORY_EXPORT_COLUMNS, OPTIONAL_EXPORT_COLUMNS } from "@/lib/export-columns";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { monthLabel } from "@/lib/utils";

// US-M6.1 (§5.8). Rule-19/D-1's five mandatory columns are always included
// (T-M6.1-2) — the picker below only ever offers Rule-33's optional list,
// reusing settings.tsx's own MANDATORY_EXPORT_COLUMNS/OPTIONAL_EXPORT_COLUMNS
// so the two screens can never name a column differently. ADR-007: the
// destination path comes from the same native save dialog the restore flow
// already uses — this screen never touches raw file content.
export function Reports() {
  const [lockStatus, setLockStatus] = useState<PeriodLockStatus | null>(null);
  const [selectedMonth, setSelectedMonth] = useState<string | null>(null);
  const viewMonth = selectedMonth ?? lockStatus?.recordablePeriodMonths[0];
  const [columns, setColumns] = useState<Set<string>>(new Set());
  const [exporting, setExporting] = useState(false);
  const toast = useToast();

  useEffect(() => {
    getPeriodLockStatus().then(setLockStatus);
  }, []);

  function toggleColumn(key: string) {
    setColumns((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  async function handleExportMonthly() {
    if (!viewMonth) return;
    // T-M6.1-5: the filename names the period, never a member.
    const outputPath = await saveFileDialog({
      defaultPath: `member-rewards-monthly-${viewMonth}.xlsx`,
      filters: [{ name: "Excel Workbook", extensions: ["xlsx"] }],
    });
    if (!outputPath) return;
    setExporting(true);
    try {
      await exportMonthly({
        periodMonth: viewMonth,
        optionalColumns: Array.from(columns),
        outputPath,
      });
      toast.add({ title: "Monthly data exported", type: "success" });
    } catch (raw) {
      toast.add({ title: toErrorPresentation(raw).message, type: "danger" });
    } finally {
      setExporting(false);
    }
  }

  return (
    <>
      <h1 className="text-headline">Reports</h1>
      <p className="text-caption mt-1">
        Spreadsheet extracts only — no on-screen history of past months
      </p>

      <Card className="mt-4">
        <CardHeader>
          <div>
            <CardTitle>Monthly data</CardTitle>
            <CardDescription>
              {viewMonth ? monthLabel(viewMonth) : ""} · current live figures
            </CardDescription>
          </div>
          <Button
            variant="primary"
            size="sm"
            disabled={!viewMonth || exporting}
            onClick={handleExportMonthly}
          >
            Export .xlsx
          </Button>
        </CardHeader>

        {lockStatus && (
          <MonthSwitcher
            className="mb-3.5 border-0 bg-transparent px-0 py-0"
            months={lockStatus.recordablePeriodMonths}
            value={viewMonth ?? lockStatus.recordablePeriodMonths[0]}
            onChange={setSelectedMonth}
          />
        )}

        <div className="text-caption text-muted-text mb-2">
          Always included: {MANDATORY_EXPORT_COLUMNS.map((c) => c.label).join(", ")}
        </div>
        <div className="flex flex-wrap gap-x-4 gap-y-1.5">
          {OPTIONAL_EXPORT_COLUMNS.map((c) => (
            <label key={c.key} className="flex items-center gap-1.5 text-body">
              <input
                type="checkbox"
                checked={columns.has(c.key)}
                onChange={() => toggleColumn(c.key)}
              />
              {c.label}
            </label>
          ))}
        </div>
      </Card>
    </>
  );
}
