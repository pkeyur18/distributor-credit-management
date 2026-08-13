import { useEffect, useState } from "react";
import { save as saveFileDialog } from "@tauri-apps/plugin-dialog";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { MonthSwitcher } from "@/components/month-switcher";
import { useToast } from "@/components/ui/toast";
import { getPeriodLockStatus, type PeriodLockStatus } from "@/lib/ipc/m2-entries";
import { exportMonthly, exportYearlyAverage, exportLowContribution } from "@/lib/ipc/m6-reports";
import { getSettings } from "@/lib/ipc/m7-settings";
import { MANDATORY_EXPORT_COLUMNS, OPTIONAL_EXPORT_COLUMNS } from "@/lib/export-columns";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { centsToDisplay, displayToCents, monthLabel } from "@/lib/utils";

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
  const [exportingYearly, setExportingYearly] = useState(false);
  const [thresholdInput, setThresholdInput] = useState("100.00");
  const [exportingLow, setExportingLow] = useState(false);
  const toast = useToast();

  useEffect(() => {
    getPeriodLockStatus().then(setLockStatus);
    getSettings().then((s) => setThresholdInput(centsToDisplay(s.lowContributionThreshold)));
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

  async function handleExportYearlyAverage() {
    const outputPath = await saveFileDialog({
      defaultPath: "member-rewards-yearly-average.xlsx",
      filters: [{ name: "Excel Workbook", extensions: ["xlsx"] }],
    });
    if (!outputPath) return;
    setExportingYearly(true);
    try {
      await exportYearlyAverage(outputPath);
      toast.add({ title: "Yearly average exported", type: "success" });
    } catch (raw) {
      toast.add({ title: toErrorPresentation(raw).message, type: "danger" });
    } finally {
      setExportingYearly(false);
    }
  }

  async function handleExportLowContribution() {
    const outputPath = await saveFileDialog({
      defaultPath: "member-rewards-low-contribution.xlsx",
      filters: [{ name: "Excel Workbook", extensions: ["xlsx"] }],
    });
    if (!outputPath) return;
    const threshold = displayToCents(thresholdInput);
    setExportingLow(true);
    try {
      await exportLowContribution({
        threshold: threshold ?? undefined,
        outputPath,
      });
      toast.add({ title: "Low-contribution report exported", type: "success" });
    } catch (raw) {
      toast.add({ title: toErrorPresentation(raw).message, type: "danger" });
    } finally {
      setExportingLow(false);
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

      <Card className="mt-4">
        <CardHeader>
          <div>
            <CardTitle>Yearly average</CardTitle>
            <CardDescription>
              Divides by each member&apos;s own count of closed months — never a fixed twelve
              (Rule-23)
            </CardDescription>
          </div>
          <Button
            variant="primary"
            size="sm"
            disabled={exportingYearly}
            onClick={handleExportYearlyAverage}
          >
            Export .xlsx
          </Button>
        </CardHeader>
      </Card>

      {/* No on-screen preview table, unlike the prototype's client-side
          mockup — the 40-command API surface has no read-only command for
          this data separate from the export itself (ADR-007 keeps the
          WebView from computing it locally), so the reduction is the
          threshold field and export action only. */}
      <Card className="mt-4">
        <CardHeader>
          <div>
            <CardTitle>Low-contribution report</CardTitle>
            <CardDescription>
              Yearly average of own Business Volume, below a threshold
            </CardDescription>
          </div>
          <Button
            variant="primary"
            size="sm"
            disabled={exportingLow}
            onClick={handleExportLowContribution}
          >
            Export .xlsx
          </Button>
        </CardHeader>
        <label htmlFor="low-threshold" className="text-label mb-1 block">
          Threshold
        </label>
        <Input
          id="low-threshold"
          className="num max-w-50"
          type="text"
          inputMode="decimal"
          value={thresholdInput}
          onChange={(e) => setThresholdInput(e.target.value)}
        />
      </Card>
    </>
  );
}
