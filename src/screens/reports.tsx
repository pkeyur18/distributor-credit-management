import { useEffect, useState } from "react";
import { ArrowDown, ArrowUp, Download } from "lucide-react";
import { save as saveFileDialog } from "@tauri-apps/plugin-dialog";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Pill } from "@/components/ui/pill";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  TableWrap,
} from "@/components/ui/table";
import { EmptyState } from "@/components/empty-state";
import { MonthSwitcher } from "@/components/month-switcher";
import { PageHeader } from "@/components/page-header";
import { useToast } from "@/components/ui/toast";
import { getPeriodLockStatus, type PeriodLockStatus } from "@/lib/ipc/m2-entries";
import {
  exportMonthly,
  exportYearlyAverage,
  exportLowContribution,
  listBackups,
  redownloadBackup,
  previewMonthlyData,
  previewYearlyAverage,
  type ClosedMonthBackup,
  type MonthlyPreviewRow,
  type MonthlySortField,
  type SortDirection,
  type YearlyAveragePreviewRow,
  type YearlySortField,
} from "@/lib/ipc/m6-reports";
import { getSettings } from "@/lib/ipc/m7-settings";
import { MANDATORY_EXPORT_COLUMNS, OPTIONAL_EXPORT_COLUMNS } from "@/lib/export-columns";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { centsToDisplay, cn, displayToCents, monthLabel } from "@/lib/utils";

const MONTHLY_PREVIEW_LIMIT = 6;
const YEARLY_PREVIEW_LIMIT = 6;
const LOW_CONTRIBUTION_PREVIEW_LIMIT = 8;

const MONTHLY_SORT_OPTIONS: { value: MonthlySortField; label: string }[] = [
  { value: "name", label: "Name" },
  { value: "businessVolume", label: "Business Volume" },
  { value: "totalBusinessVolume", label: "Total Business Volume" },
  { value: "slabPct", label: "Slab %" },
  { value: "rewards", label: "Rewards" },
];

const YEARLY_SORT_OPTIONS: { value: YearlySortField; label: string }[] = [
  { value: "name", label: "Name" },
  { value: "avgBusinessVolume", label: "Average Business Volume" },
  { value: "avgTotalBusinessVolume", label: "Average Total Business Volume" },
];

function compareMonthlyPreview(
  a: MonthlyPreviewRow,
  b: MonthlyPreviewRow,
  field: MonthlySortField,
  direction: SortDirection,
): number {
  // Rewards isn't a column this preview table shows (mod.rs's
  // preview_monthly_data doc comment) — reordering by it here would look
  // arbitrary, so a Rewards sort choice only ever affects the exported file.
  if (field === "rewards") return b.totalBusinessVolume - a.totalBusinessVolume;
  let cmp: number;
  switch (field) {
    case "name":
      cmp = a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
      break;
    case "businessVolume":
      cmp = a.businessVolume - b.businessVolume;
      break;
    case "totalBusinessVolume":
      cmp = a.totalBusinessVolume - b.totalBusinessVolume;
      break;
    case "slabPct":
      cmp = a.slabPct - b.slabPct;
      break;
  }
  if (direction === "desc") cmp = -cmp;
  return cmp !== 0 ? cmp : a.id - b.id;
}

function compareYearlyPreview(
  a: YearlyAveragePreviewRow,
  b: YearlyAveragePreviewRow,
  field: YearlySortField,
  direction: SortDirection,
  // The Low-Contribution card's own table never shows Average Total
  // Business Volume — same reasoning as Rewards above, that field only
  // ever affects the exported file for that card.
  hasTotalBvColumn: boolean,
): number {
  if (field === "avgTotalBusinessVolume" && !hasTotalBvColumn) {
    return a.avgBusinessVolume - b.avgBusinessVolume;
  }
  let cmp: number;
  switch (field) {
    case "name":
      cmp = a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
      break;
    case "avgBusinessVolume":
      cmp = a.avgBusinessVolume - b.avgBusinessVolume;
      break;
    case "avgTotalBusinessVolume":
      cmp = a.avgTotalBusinessVolume - b.avgTotalBusinessVolume;
      break;
  }
  if (direction === "desc") cmp = -cmp;
  return cmp !== 0 ? cmp : a.id - b.id;
}

function SortControl<F extends string>({
  field,
  onFieldChange,
  direction,
  onDirectionChange,
  options,
}: {
  field: F;
  onFieldChange: (field: F) => void;
  direction: SortDirection;
  onDirectionChange: (direction: SortDirection) => void;
  options: { value: F; label: string }[];
}) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="text-label">Sort</span>
      <select
        className="h-6.75 rounded-sm border border-border bg-surface px-2 text-[12.5px] font-[550] text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
        value={field}
        onChange={(e) => onFieldChange(e.target.value as F)}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
      <Button
        variant="secondary"
        size="sm"
        className="w-6.75 px-0"
        aria-label={direction === "asc" ? "Sorted ascending, click for descending" : "Sorted descending, click for ascending"}
        onClick={() => onDirectionChange(direction === "asc" ? "desc" : "asc")}
      >
        {direction === "asc" ? <ArrowUp /> : <ArrowDown />}
      </Button>
    </div>
  );
}

// US-M6.1 (§5.8). Rule-19/D-1's five mandatory columns are always included
// (T-M6.1-2) — the picker below only ever offers Rule-33's optional list,
// reusing settings.tsx's own MANDATORY_EXPORT_COLUMNS/OPTIONAL_EXPORT_COLUMNS
// so the two screens can never name a column differently. ADR-007: the
// destination path comes from the same native save dialog the restore flow
// already uses — this screen never touches raw file content. The on-screen
// preview tables (API-43/44) are a separate read-only path from the
// exports themselves, matching the approved prototype.
export function Reports() {
  const [lockStatus, setLockStatus] = useState<PeriodLockStatus | null>(null);
  const [selectedMonth, setSelectedMonth] = useState<string | null>(null);
  const viewMonth = selectedMonth ?? lockStatus?.recordablePeriodMonths[0];
  const [columns, setColumns] = useState<Set<string>>(new Set());
  const [exporting, setExporting] = useState(false);
  const [exportingYearly, setExportingYearly] = useState(false);
  const [thresholdInput, setThresholdInput] = useState("100.00");
  const [exportingLow, setExportingLow] = useState(false);
  const [backups, setBackups] = useState<ClosedMonthBackup[] | null>(null);
  const [selectedBackupPeriodId, setSelectedBackupPeriodId] = useState<number | null>(null);
  const [exportingBackup, setExportingBackup] = useState(false);
  const [monthlyPreview, setMonthlyPreview] = useState<MonthlyPreviewRow[]>([]);
  const [yearlyAverages, setYearlyAverages] = useState<YearlyAveragePreviewRow[] | null>(null);
  const [monthlySortField, setMonthlySortField] = useState<MonthlySortField>("name");
  const [monthlySortDirection, setMonthlySortDirection] = useState<SortDirection>("asc");
  const [yearlySortField, setYearlySortField] = useState<YearlySortField>("name");
  const [yearlySortDirection, setYearlySortDirection] = useState<SortDirection>("asc");
  const [lowContribSortField, setLowContribSortField] = useState<YearlySortField>("name");
  const [lowContribSortDirection, setLowContribSortDirection] = useState<SortDirection>("asc");
  const toast = useToast();

  useEffect(() => {
    getPeriodLockStatus().then(setLockStatus);
    getSettings().then((s) => setThresholdInput(centsToDisplay(s.lowContributionThreshold)));
    listBackups().then(setBackups);
    previewYearlyAverage().then(setYearlyAverages);
  }, []);

  useEffect(() => {
    if (!viewMonth) return;
    let cancelled = false;
    previewMonthlyData(viewMonth).then((rows) => {
      if (!cancelled) setMonthlyPreview(rows);
    });
    return () => {
      cancelled = true;
    };
  }, [viewMonth]);

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
        sortField: monthlySortField,
        sortDirection: monthlySortDirection,
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
      await exportYearlyAverage(outputPath, yearlySortField, yearlySortDirection);
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
        sortField: lowContribSortField,
        sortDirection: lowContribSortDirection,
        outputPath,
      });
      toast.add({ title: "Low-contribution report exported", type: "success" });
    } catch (raw) {
      toast.add({ title: toErrorPresentation(raw).message, type: "danger" });
    } finally {
      setExportingLow(false);
    }
  }

  const selectedBackup =
    backups?.find((b) => b.periodId === selectedBackupPeriodId) ?? backups?.[0] ?? null;

  async function handleRedownloadBackup() {
    if (!selectedBackup) return;
    const outputPath = await saveFileDialog({
      defaultPath: `member-rewards-closed-${selectedBackup.periodMonth}-v${selectedBackup.latestVersion}.xlsx`,
      filters: [{ name: "Excel Workbook", extensions: ["xlsx"] }],
    });
    if (!outputPath) return;
    setExportingBackup(true);
    try {
      await redownloadBackup(selectedBackup.periodId, outputPath);
      toast.add({
        title: `${monthLabel(selectedBackup.periodMonth)} snapshot exported (version ${selectedBackup.latestVersion})`,
        type: "success",
      });
    } catch (raw) {
      toast.add({ title: toErrorPresentation(raw).message, type: "danger" });
    } finally {
      setExportingBackup(false);
    }
  }

  // Recomputed on every keystroke rather than refetched (prototype's
  // `oninput` behaviour) — the authoritative own-BV-not-Total-BV filter
  // (Rule-24) stays solely in export_low_contribution; this is presentation.
  const thresholdCents = displayToCents(thresholdInput) ?? 0;
  const belowThreshold = (yearlyAverages ?? [])
    .filter((d) => d.avgBusinessVolume < thresholdCents)
    .sort((a, b) => compareYearlyPreview(a, b, lowContribSortField, lowContribSortDirection, false));

  const yearlyTop = (yearlyAverages ?? [])
    .slice()
    .sort((a, b) => compareYearlyPreview(a, b, yearlySortField, yearlySortDirection, true))
    .slice(0, YEARLY_PREVIEW_LIMIT);

  return (
    <>
      <PageHeader
        title="Reports"
        subtitle="Spreadsheet extracts only — no on-screen history of past months"
      />

      <Card className="mt-4">
        <CardHeader>
          <div>
            <CardTitle>Monthly data</CardTitle>
            <CardDescription>
              {viewMonth ? monthLabel(viewMonth) : ""} · current live figures
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <SortControl
              field={monthlySortField}
              onFieldChange={setMonthlySortField}
              direction={monthlySortDirection}
              onDirectionChange={setMonthlySortDirection}
              options={MONTHLY_SORT_OPTIONS}
            />
            <Button
              variant="primary"
              size="sm"
              disabled={!viewMonth || exporting}
              onClick={handleExportMonthly}
            >
              <Download />
              Export .xlsx
            </Button>
          </div>
        </CardHeader>

        {lockStatus && (
          <MonthSwitcher
            className="mb-3.5 border-0 bg-transparent px-0 py-0"
            months={lockStatus.recordablePeriodMonths}
            value={viewMonth ?? lockStatus.recordablePeriodMonths[0]}
            onChange={setSelectedMonth}
          />
        )}

        <div className="grid gap-4 lg:grid-cols-[1fr_1.5fr]">
          <div>
            <div className="text-caption text-muted-text mb-3">
              Always included: {MANDATORY_EXPORT_COLUMNS.map((c) => c.label).join(", ")},
              Active/inactive status (a deactivated row&apos;s colour is never shown without this
              label)
            </div>
            <div className="flex flex-col gap-1.5">
              {OPTIONAL_EXPORT_COLUMNS.filter((c) => c.key !== "active_status").map((c) => (
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
          </div>
          <TableWrap>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>#</TableHead>
                  <TableHead numeric>BV</TableHead>
                  <TableHead numeric>Total BV</TableHead>
                  <TableHead>Slab</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {monthlyPreview
                  .slice()
                  .sort((a, b) => compareMonthlyPreview(a, b, monthlySortField, monthlySortDirection))
                  .slice(0, MONTHLY_PREVIEW_LIMIT)
                  .map((row) => (
                  <TableRow key={row.id}>
                    <TableCell primary>{row.name}</TableCell>
                    <TableCell className="mono">{row.id}</TableCell>
                    <TableCell numeric>{centsToDisplay(row.businessVolume)}</TableCell>
                    <TableCell numeric>{centsToDisplay(row.totalBusinessVolume)}</TableCell>
                    <TableCell>
                      <Pill variant="slab">{row.slabPct}% slab</Pill>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableWrap>
        </div>
      </Card>

      <Card className="mt-4">
        <CardHeader>
          <div>
            <CardTitle>Yearly average</CardTitle>
            <CardDescription>
              Based on {backups?.length ?? 0} closed month{backups?.length === 1 ? "" : "s"} so
              far, never a fixed twelve
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <SortControl
              field={yearlySortField}
              onFieldChange={setYearlySortField}
              direction={yearlySortDirection}
              onDirectionChange={setYearlySortDirection}
              options={YEARLY_SORT_OPTIONS}
            />
            <Button
              variant="primary"
              size="sm"
              disabled={exportingYearly}
              onClick={handleExportYearlyAverage}
            >
              <Download />
              Export .xlsx
            </Button>
          </div>
        </CardHeader>
        <TableWrap>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>#</TableHead>
                <TableHead numeric>Average Business Volume</TableHead>
                <TableHead numeric>Average Total Business Volume</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {yearlyTop.map((row) => (
                <TableRow key={row.id}>
                  <TableCell primary>{row.name}</TableCell>
                  <TableCell className="mono">{row.id}</TableCell>
                  <TableCell numeric>{centsToDisplay(row.avgBusinessVolume)}</TableCell>
                  <TableCell numeric>{centsToDisplay(row.avgTotalBusinessVolume)}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableWrap>
      </Card>

      <Card className="mt-4">
        <CardHeader>
          <div>
            <CardTitle>Low-contribution report</CardTitle>
            <CardDescription>
              Yearly average of own Business Volume, below a threshold
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <SortControl
              field={lowContribSortField}
              onFieldChange={setLowContribSortField}
              direction={lowContribSortDirection}
              onDirectionChange={setLowContribSortDirection}
              options={YEARLY_SORT_OPTIONS}
            />
            <Button
              variant="primary"
              size="sm"
              disabled={exportingLow}
              onClick={handleExportLowContribution}
            >
              <Download />
              Export .xlsx
            </Button>
          </div>
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

        <StatCard
          className="mt-3"
          label={`Below ${centsToDisplay(thresholdCents)}`}
          value={String(belowThreshold.length)}
          footer={`of ${yearlyAverages?.length ?? 0} members, by yearly average own Business Volume`}
        />

        {belowThreshold.length > 0 ? (
          <TableWrap className="mt-3">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Member #</TableHead>
                  <TableHead numeric>Average Business Volume</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {belowThreshold.slice(0, LOW_CONTRIBUTION_PREVIEW_LIMIT).map((row) => (
                  <TableRow key={row.id}>
                    <TableCell primary>{row.name}</TableCell>
                    <TableCell className="mono">{row.id}</TableCell>
                    <TableCell numeric>{centsToDisplay(row.avgBusinessVolume)}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableWrap>
        ) : (
          <EmptyState title="No members below this threshold" />
        )}
      </Card>

      {backups && backups.length > 0 && (
        <Card className="mt-4">
          <CardHeader>
            <div>
              <CardTitle>Closed month snapshot</CardTitle>
              <CardDescription>Every field as recorded when that month closed</CardDescription>
            </div>
            <Button
              variant="primary"
              size="sm"
              disabled={!selectedBackup || exportingBackup}
              onClick={handleRedownloadBackup}
            >
              <Download />
              Export .xlsx
            </Button>
          </CardHeader>
          <label htmlFor="closed-snapshot-month" className="text-label mb-1 block">
            Month
          </label>
          <select
            id="closed-snapshot-month"
            className="h-8.5 max-w-70 rounded-sm border border-border bg-surface px-2.5 text-body text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
            value={selectedBackup?.periodId ?? ""}
            onChange={(e) => setSelectedBackupPeriodId(Number(e.target.value))}
          >
            {backups.map((b) => (
              <option key={b.periodId} value={b.periodId}>
                {monthLabel(b.periodMonth)}
                {b.isCorrected ? ` — corrected, v${b.latestVersion}` : ""}
              </option>
            ))}
          </select>
          <p className="text-caption text-muted-text mt-1.5">
            Always exports the latest version. If this month has since been corrected, the export
            reflects the correction — the original stays in the audit trail, not the file.
          </p>
        </Card>
      )}
    </>
  );
}

function StatCard({
  label,
  value,
  footer,
  className,
}: {
  label: string;
  value: string;
  footer: string;
  className?: string;
}) {
  return (
    <div className={cn("rounded-lg border border-border bg-surface p-3.5", className)}>
      <div className="text-label text-muted-text">{label}</div>
      <div className="num mt-1 text-numeric-lg">{value}</div>
      <div className="text-caption mt-0.5">{footer}</div>
    </div>
  );
}
