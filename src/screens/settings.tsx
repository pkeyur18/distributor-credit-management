import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { Plus } from "lucide-react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input, InputHint } from "@/components/ui/input";
import { AlertNote } from "@/components/ui/alert-note";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { RestoreOptionList } from "@/components/restore-option-list";
import { ChecklistConfirmDialog } from "@/components/checklist-confirm-dialog";
import { RecalcWarningDialog } from "@/components/recalc-warning-dialog";
import { PageHeader } from "@/components/page-header";
import { useToast } from "@/components/ui/toast";
import { centsToDisplay, cn, displayToCents } from "@/lib/utils";
import { getDirectChildrenChart } from "@/lib/ipc/m4-search";
import {
  addSlabRow,
  getConsoleBackupSettings,
  getSettings,
  removeSlabRow,
  updateConsoleBackupSettings,
  updateSettings,
  updateSlabRow,
  type ConsoleBackupSettings,
} from "@/lib/ipc/m7-settings";
import {
  previewSettingsImpact,
  type CandidateSettings,
  type SettingsImpactPreview,
} from "@/lib/ipc/m3-calc";
import { runConsoleBackupNow } from "@/lib/ipc/m8-auth";
import { listRestorePoints, restoreFromBackup, restoreFromBackupFile } from "@/lib/ipc/preflight";
import type { BackupRecord, Settings as SettingsData, SlabRow } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { backupPrimaryLabel, backupProvenanceText } from "@/lib/backup-labels";
import { useAuth } from "@/lib/auth-context";
import { MANDATORY_EXPORT_COLUMNS, OPTIONAL_EXPORT_COLUMNS } from "@/lib/export-columns";

function SectionCard({
  id,
  title,
  description,
  action,
  children,
}: {
  id: string;
  title: string;
  description?: string;
  action?: ReactNode;
  children: ReactNode;
}) {
  return (
    <Card id={id}>
      <CardHeader className={action ? undefined : "block"}>
        <div>
          <CardTitle>{title}</CardTitle>
          {description && <CardDescription>{description}</CardDescription>}
        </div>
        {action}
      </CardHeader>
      {children}
    </Card>
  );
}

function errorMessage(raw: unknown): string {
  return toErrorPresentation(raw).message;
}

// --- Mid-period recalculation warning (RQ-18/V7.6, US-M7.3) ---
// Fires only on a Slab table or Royalty save (T-M7.3-4) — every other
// settings section keeps saving silently. `request` previews the candidate
// values, opens the warning, and only runs `action` (the real save) once
// the operator confirms; Cancel is a true no-op (T-M7.3-5).
function useRecalcWarning() {
  const toast = useToast();
  const [pending, setPending] = useState<{
    kind: "slab" | "royalty";
    preview: SettingsImpactPreview | null;
    action: () => Promise<void>;
  } | null>(null);
  const [busy, setBusy] = useState(false);

  async function request(
    kind: "slab" | "royalty",
    candidate: CandidateSettings,
    action: () => Promise<void>,
  ) {
    setPending({ kind, preview: null, action });
    try {
      const preview = await previewSettingsImpact(candidate);
      setPending((prev) => (prev ? { ...prev, preview } : prev));
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
      setPending(null);
    }
  }

  async function confirm() {
    if (!pending) return;
    setBusy(true);
    try {
      await pending.action();
      setPending(null);
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setBusy(false);
    }
  }

  const monthName = new Date().toLocaleDateString(undefined, { month: "long", year: "numeric" });

  const dialog = pending && (
    <RecalcWarningDialog
      open
      onOpenChange={(next) => {
        if (!next) setPending(null);
      }}
      kind={pending.kind}
      monthName={monthName}
      preview={pending.preview}
      busy={busy}
      onConfirm={confirm}
    />
  );

  return { request, dialog };
}

// --- Slab table (US-M7.1) ---

function candidateFromSlabRows(
  pairs: { threshold: number; percentage: number }[],
): CandidateSettings {
  return {
    slabThresholds: pairs.map((p) => p.threshold),
    slabPercentages: pairs.map((p) => p.percentage),
  };
}

type SlabDraftRow = { key: string; id: number | null; threshold: string; percentage: string };

function draftFromRows(rows: SlabRow[]): SlabDraftRow[] {
  return rows.map((r) => ({
    key: String(r.id),
    id: r.id,
    threshold: centsToDisplay(r.threshold),
    percentage: String(r.percentage),
  }));
}

// T-M7.1 restyle: the approved prototype stages add/edit/remove locally and
// commits the whole table with one "Save slab table" button, instead of a
// save per row. Same per-row IPC calls (add/update/remove) and the same
// server-side rules (duplicate threshold, last-row refusal) — only the
// moment they fire moves from per-action to the single confirmed save.
function SlabTableCard({
  rows,
  onRowsChange,
}: {
  rows: SlabRow[];
  onRowsChange: (rows: SlabRow[]) => void;
}) {
  const toast = useToast();
  const recalcWarning = useRecalcWarning();
  const [draft, setDraft] = useState<SlabDraftRow[]>(() => draftFromRows(rows));
  const [nextTempId, setNextTempId] = useState(0);
  const [saving, setSaving] = useState(false);

  function updateDraftRow(
    key: string,
    patch: Partial<Pick<SlabDraftRow, "threshold" | "percentage">>,
  ) {
    setDraft((prev) => prev.map((r) => (r.key === key ? { ...r, ...patch } : r)));
  }

  function addRow() {
    setDraft((prev) => [...prev, { key: `new-${nextTempId}`, id: null, threshold: "", percentage: "" }]);
    setNextTempId((n) => n + 1);
  }

  function removeRow(key: string) {
    setDraft((prev) => (prev.length <= 1 ? prev : prev.filter((r) => r.key !== key)));
  }

  async function save() {
    const parsed = draft.map((r) => ({
      key: r.key,
      id: r.id,
      threshold: displayToCents(r.threshold),
      percentage: Number(r.percentage),
    }));
    if (parsed.some((r) => r.threshold === null || !Number.isFinite(r.percentage))) {
      toast.add({ title: "Enter a valid threshold and percentage", type: "danger" });
      return;
    }
    const thresholds = parsed.map((r) => r.threshold as number);
    if (new Set(thresholds).size !== thresholds.length) {
      toast.add({ title: "Two rows share the same threshold — adjust before saving", type: "danger" });
      return;
    }
    const candidate = candidateFromSlabRows(
      parsed.map((r) => ({ threshold: r.threshold as number, percentage: r.percentage })),
    );
    await recalcWarning.request("slab", candidate, async () => {
      setSaving(true);
      try {
        const draftIds = new Set(parsed.filter((r) => r.id !== null).map((r) => r.id));
        for (const row of rows) {
          if (!draftIds.has(row.id)) await removeSlabRow(row.id);
        }
        const saved: SlabRow[] = [];
        for (const r of parsed) {
          const input = { threshold: r.threshold as number, percentage: r.percentage };
          if (r.id === null) {
            saved.push(await addSlabRow(input));
          } else {
            const original = rows.find((row) => row.id === r.id);
            const changed =
              !original ||
              original.threshold !== input.threshold ||
              original.percentage !== input.percentage;
            saved.push(changed ? await updateSlabRow(r.id, input) : (original as SlabRow));
          }
        }
        onRowsChange(saved);
        setDraft(draftFromRows(saved));
        toast.add({ title: "Slab table saved", type: "success" });
      } finally {
        setSaving(false);
      }
    });
  }

  const onlyRow = draft.length <= 1;

  return (
    <SectionCard
      id="settings-card-slab"
      title="Slab table"
      description="The band looked up from Total Business Volume. Top slab recalculates automatically."
      action={
        <Button size="sm" variant="secondary" disabled={saving} onClick={addRow}>
          <Plus />
          Add row
        </Button>
      }
    >
      <div className="overflow-x-auto">
        <table className="w-full text-body">
          <thead>
            <tr className="text-label text-left text-muted-text">
              <th className="pb-1.5 font-normal">Threshold</th>
              <th className="pb-1.5 font-normal">Slab %</th>
              <th className="pb-1.5"></th>
            </tr>
          </thead>
          <tbody>
            {draft.map((row) => (
              <tr key={row.key} className="border-t border-border">
                <td className="py-1.5 pr-2">
                  <Input
                    id={`slab-threshold-${row.key}`}
                    className="num"
                    placeholder="0.00"
                    disabled={saving}
                    value={row.threshold}
                    onChange={(e) => updateDraftRow(row.key, { threshold: e.target.value })}
                  />
                </td>
                <td className="py-1.5 pr-2">
                  <Input
                    id={`slab-percentage-${row.key}`}
                    className="num"
                    placeholder="0"
                    disabled={saving}
                    value={row.percentage}
                    onChange={(e) => updateDraftRow(row.key, { percentage: e.target.value })}
                  />
                </td>
                <td className="py-1.5" style={{ width: "1%" }}>
                  <Button
                    id={`slab-remove-${row.key}`}
                    variant="secondary"
                    className="h-7.5 w-7.5 justify-center p-0 hover:not-disabled:border-danger hover:not-disabled:text-danger"
                    disabled={onlyRow || saving}
                    aria-label={
                      onlyRow
                        ? "Remove row — the table must keep at least one slab"
                        : "Remove this slab row"
                    }
                    onClick={() => removeRow(row.key)}
                  >
                    ✕
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <InputHint className="mt-3">
        Rows are not checked for consistency against each other — this is deliberate. Misconfiguring
        the table can produce unexpected Rewards.
      </InputHint>
      {onlyRow && (
        <InputHint className="mt-1.5">
          The last slab cannot be removed — the console needs at least one to work out a
          member&apos;s slab.
        </InputHint>
      )}
      <Button id="slab-save-table" className="mt-3.5" disabled={saving} onClick={save}>
        Save slab table
      </Button>
      {recalcWarning.dialog}
    </SectionCard>
  );
}

// --- Royalty / structure guidance / reporting / reference / session (US-M7.2) ---

function RoyaltyCard({
  settings,
  onSettingsChange,
}: {
  settings: SettingsData;
  onSettingsChange: (next: SettingsData) => void;
}) {
  const toast = useToast();
  const recalcWarning = useRecalcWarning();
  const [minChildren, setMinChildren] = useState(String(settings.royaltyQualifyingCount));
  const [rate, setRate] = useState(String(settings.royaltyRatePercent));
  const [saving, setSaving] = useState(false);

  async function save() {
    const royaltyQualifyingCount = Number(minChildren);
    const royaltyRatePercent = Number(rate);
    if (!Number.isFinite(royaltyQualifyingCount) || !Number.isFinite(royaltyRatePercent)) {
      toast.add({ title: "Enter valid numbers", type: "danger" });
      return;
    }
    await recalcWarning.request(
      "royalty",
      { royaltyQualifyingCount, royaltyRatePercent },
      async () => {
        setSaving(true);
        try {
          const updated = await updateSettings({ royaltyQualifyingCount, royaltyRatePercent });
          onSettingsChange(updated);
          toast.add({ title: "Royalty settings saved", type: "success" });
        } finally {
          setSaving(false);
        }
      },
    );
  }

  return (
    <SectionCard
      id="settings-card-royalty"
      title="Royalty"
      description="Paid when enough direct legs land on the top slab"
    >
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label htmlFor="royalty-min" className="text-label mb-1 block">
            Minimum qualifying legs
          </label>
          <Input
            id="royalty-min"
            value={minChildren}
            onChange={(e) => setMinChildren(e.target.value)}
          />
        </div>
        <div>
          <label htmlFor="royalty-rate" className="text-label mb-1 block">
            Royalty rate (%)
          </label>
          <Input id="royalty-rate" value={rate} onChange={(e) => setRate(e.target.value)} />
        </div>
      </div>
      <InputHint className="mt-2">
        A member with {minChildren || settings.royaltyQualifyingCount}+ direct legs on the top slab
        earns {rate || settings.royaltyRatePercent}% royalty on each qualifying leg&apos;s Total
        Business Volume.
      </InputHint>
      <Button className="mt-3.5" disabled={saving} onClick={save}>
        Save royalty settings
      </Button>
      {recalcWarning.dialog}
    </SectionCard>
  );
}

function StructureGuidanceCard({
  settings,
  onSettingsChange,
}: {
  settings: SettingsData;
  onSettingsChange: (next: SettingsData) => void;
}) {
  const toast = useToast();
  const [depth, setDepth] = useState(String(settings.hierarchyDepth));
  const [l2, setL2] = useState(String(settings.level2Width));
  const [l3, setL3] = useState(String(settings.level3Width));
  const [l4, setL4] = useState(String(settings.level4Width));
  const [saving, setSaving] = useState(false);

  async function save() {
    const values = {
      hierarchyDepth: Number(depth),
      level2Width: Number(l2),
      level3Width: Number(l3),
      level4Width: Number(l4),
    };
    if (Object.values(values).some((v) => !Number.isFinite(v))) {
      toast.add({ title: "Enter valid numbers", type: "danger" });
      return;
    }
    setSaving(true);
    try {
      const updated = await updateSettings(values);
      onSettingsChange(updated);
      toast.add({ title: "Structure guidance saved", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setSaving(false);
    }
  }

  return (
    <SectionCard
      id="settings-card-structure"
      title="Structure guidance"
      description="Advisory only — never blocks a new member being added"
    >
      <div className="grid grid-cols-4 gap-3">
        <div>
          <label htmlFor="width-l2" className="text-label mb-1 block">
            Level 2 width
          </label>
          <Input id="width-l2" value={l2} onChange={(e) => setL2(e.target.value)} />
        </div>
        <div>
          <label htmlFor="width-l3" className="text-label mb-1 block">
            Level 3 width
          </label>
          <Input id="width-l3" value={l3} onChange={(e) => setL3(e.target.value)} />
        </div>
        <div>
          <label htmlFor="width-l4" className="text-label mb-1 block">
            Level 4 width
          </label>
          <Input id="width-l4" value={l4} onChange={(e) => setL4(e.target.value)} />
        </div>
        <div>
          <label htmlFor="depth-guidance" className="text-label mb-1 block">
            Depth guidance
          </label>
          <Input id="depth-guidance" value={depth} onChange={(e) => setDepth(e.target.value)} />
        </div>
      </div>
      <InputHint className="mt-2">
        These numbers only produce an advisory note on the Structure screen — entry is never
        blocked.
      </InputHint>
      <Button className="mt-3.5" disabled={saving} onClick={save}>
        Save structure guidance
      </Button>
    </SectionCard>
  );
}

const MONTH_START_OPTIONS = Array.from({ length: 12 }, (_, i) => ({
  value: `${String(i + 1).padStart(2, "0")}-01`,
  label: new Date(2000, i, 1).toLocaleDateString(undefined, { month: "long" }),
}));

const MONTH_END_OPTIONS = Array.from({ length: 12 }, (_, i) => {
  const lastDay = new Date(2001, i + 1, 0).getDate();
  return {
    value: `${String(i + 1).padStart(2, "0")}-${String(lastDay).padStart(2, "0")}`,
    label: new Date(2000, i, 1).toLocaleDateString(undefined, { month: "long" }),
  };
});

function monthEndOptionFor(end: string): string {
  const month = end.slice(0, 2);
  return MONTH_END_OPTIONS.find((o) => o.value.startsWith(month))?.value ?? MONTH_END_OPTIONS[11].value;
}

function ReportingCard({
  settings,
  onSettingsChange,
}: {
  settings: SettingsData;
  onSettingsChange: (next: SettingsData) => void;
}) {
  const toast = useToast();
  const [start, setStart] = useState(settings.yearlyCycle.start);
  const [end, setEnd] = useState(monthEndOptionFor(settings.yearlyCycle.end));
  const [columns, setColumns] = useState<Set<string>>(new Set(settings.defaultExportColumns));
  const [saving, setSaving] = useState(false);

  function toggleColumn(key: string) {
    setColumns((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  async function save() {
    setSaving(true);
    try {
      const updated = await updateSettings({
        yearlyCycle: { start, end },
        defaultExportColumns: [
          ...MANDATORY_EXPORT_COLUMNS.map((c) => c.key),
          ...OPTIONAL_EXPORT_COLUMNS.filter((c) => columns.has(c.key)).map((c) => c.key),
        ],
      });
      onSettingsChange(updated);
      toast.add({ title: "Reporting settings saved", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setSaving(false);
    }
  }

  return (
    <SectionCard
      id="settings-card-reporting"
      title="Reporting"
      description="Controls the yearly cycle used for reports and default export columns"
    >
      <div className="grid grid-cols-3 gap-3">
        <div>
          <label htmlFor="cycle-start" className="text-label mb-1 block">
            Yearly cycle starts
          </label>
          <select
            id="cycle-start"
            className="h-8.5 w-full rounded-sm border border-border bg-surface px-2.5 text-body text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
            value={start}
            onChange={(e) => setStart(e.target.value)}
          >
            {MONTH_START_OPTIONS.map((m) => (
              <option key={m.value} value={m.value}>
                {m.label}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label htmlFor="cycle-end" className="text-label mb-1 block">
            Yearly cycle ends
          </label>
          <select
            id="cycle-end"
            className="h-8.5 w-full rounded-sm border border-border bg-surface px-2.5 text-body text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
            value={end}
            onChange={(e) => setEnd(e.target.value)}
          >
            {MONTH_END_OPTIONS.map((m) => (
              <option key={m.value} value={m.value}>
                {m.label}
              </option>
            ))}
          </select>
        </div>
      </div>
      <div className="mt-3.5">
        <div className="text-label mb-1.5">Default export columns</div>
        <div className="flex flex-wrap gap-x-4 gap-y-1.5">
          {MANDATORY_EXPORT_COLUMNS.map((c) => (
            <label key={c.key} className="flex items-center gap-1.5 text-body text-muted-text">
              <input type="checkbox" checked disabled />
              {c.label}
            </label>
          ))}
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
      </div>
      <Button className="mt-3.5" disabled={saving} onClick={save}>
        Save reporting settings
      </Button>
    </SectionCard>
  );
}

function LowContributionThresholdCard({
  settings,
  onSettingsChange,
}: {
  settings: SettingsData;
  onSettingsChange: (next: SettingsData) => void;
}) {
  const toast = useToast();
  const [threshold, setThreshold] = useState(centsToDisplay(settings.lowContributionThreshold));
  const [saving, setSaving] = useState(false);

  async function save() {
    const lowContributionThreshold = displayToCents(threshold);
    if (lowContributionThreshold === null) {
      toast.add({ title: "Enter a valid low-contribution threshold", type: "danger" });
      return;
    }
    setSaving(true);
    try {
      const updated = await updateSettings({ lowContributionThreshold });
      onSettingsChange(updated);
      toast.add({ title: "Threshold saved", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setSaving(false);
    }
  }

  return (
    <SectionCard
      id="settings-card-low-contribution"
      title="Low-contribution threshold"
      description="Default minimum yearly Business Volume used to flag low-contribution members in the Low-contribution report. Override per export on the Reports screen."
    >
      <div className="max-w-40">
        <label htmlFor="low-threshold-setting" className="text-label mb-1 block">
          Threshold
        </label>
        <Input
          id="low-threshold-setting"
          className="num"
          value={threshold}
          onChange={(e) => setThreshold(e.target.value)}
        />
      </div>
      <Button className="mt-3.5" disabled={saving} onClick={save}>
        Save threshold
      </Button>
    </SectionCard>
  );
}

function SessionCard({
  settings,
  onSettingsChange,
}: {
  settings: SettingsData;
  onSettingsChange: (next: SettingsData) => void;
}) {
  const toast = useToast();
  const [minutes, setMinutes] = useState(String(settings.sessionTimeoutMinutes));
  const [saving, setSaving] = useState(false);

  async function save() {
    const sessionTimeoutMinutes = Number(minutes);
    if (!Number.isFinite(sessionTimeoutMinutes)) {
      toast.add({ title: "Enter a valid number", type: "danger" });
      return;
    }
    setSaving(true);
    try {
      const updated = await updateSettings({ sessionTimeoutMinutes });
      onSettingsChange(updated);
      toast.add({ title: "Saved", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setSaving(false);
    }
  }

  return (
    <SectionCard
      id="settings-card-session"
      title="Session"
      description="How long the console waits before locking on its own"
    >
      <div className="max-w-55">
        <label htmlFor="session-timeout" className="text-label mb-1 block">
          Inactivity timeout (minutes)
        </label>
        <Input id="session-timeout" value={minutes} onChange={(e) => setMinutes(e.target.value)} />
      </div>
      <Button className="mt-3.5" disabled={saving} onClick={save}>
        Save timeout
      </Button>
    </SectionCard>
  );
}

// --- Backup schedule / restore (US-M7.4) ---

const BACKUP_SCHEDULES: { value: ConsoleBackupSettings["schedule"]; label: string }[] = [
  { value: "off", label: "Off" },
  { value: "daily", label: "Daily" },
  { value: "weekly", label: "Weekly" },
  { value: "monthly", label: "Monthly" },
];

function BackupScheduleCard({
  backupSettings,
  onBackupSettingsChange,
  onBackedUp,
  lastBackupLabel,
}: {
  backupSettings: ConsoleBackupSettings;
  onBackupSettingsChange: (next: ConsoleBackupSettings) => void;
  onBackedUp: (record: BackupRecord) => void;
  lastBackupLabel: string | null;
}) {
  const toast = useToast();
  const [retention, setRetention] = useState(String(backupSettings.retentionCount));
  const [folder, setFolder] = useState(backupSettings.folder);
  const [runningBackup, setRunningBackup] = useState(false);

  // T-M7.4-2: the segmented control saves immediately, no separate Save step.
  async function setSchedule(schedule: ConsoleBackupSettings["schedule"]) {
    try {
      const updated = await updateConsoleBackupSettings({ ...backupSettings, schedule });
      onBackupSettingsChange(updated);
      toast.add({ title: `Backup schedule set to ${schedule}`, type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    }
  }

  async function saveRetention() {
    const retentionCount = Number(retention);
    if (!Number.isFinite(retentionCount) || retentionCount < 1) {
      toast.add({ title: "Retention count must be at least 1", type: "danger" });
      return;
    }
    try {
      const updated = await updateConsoleBackupSettings({ ...backupSettings, retentionCount });
      onBackupSettingsChange(updated);
      toast.add({ title: "Retention saved", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    }
  }

  async function saveFolder() {
    if (!folder.trim()) {
      toast.add({ title: "Backup folder name cannot be empty", type: "danger" });
      return;
    }
    try {
      const updated = await updateConsoleBackupSettings({ ...backupSettings, folder });
      onBackupSettingsChange(updated);
      toast.add({ title: "Backup folder saved", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    }
  }

  async function backUpNow() {
    setRunningBackup(true);
    try {
      const record = await runConsoleBackupNow();
      onBackedUp(record);
      toast.add({ title: "Console backed up", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setRunningBackup(false);
    }
  }

  return (
    <SectionCard
      id="settings-card-backup"
      title="Backup schedule"
      description="Backs up the whole console — every member, entry, record and setting, not just one month"
    >
      <SegmentedControl
        className="mb-3.5"
        value={backupSettings.schedule}
        onValueChange={setSchedule}
        options={BACKUP_SCHEDULES}
      />
      <div className="max-w-55">
        <label htmlFor="backup-retention" className="text-label mb-1 block">
          Keep the most recent
        </label>
        <Input
          id="backup-retention"
          value={retention}
          onChange={(e) => setRetention(e.target.value)}
        />
      </div>
      <InputHint className="mt-1.5">
        {lastBackupLabel && `Last backup: ${lastBackupLabel}. `}Older backups beyond this count are
        removed automatically — closed-month backups are never affected.
      </InputHint>
      <div className="mt-3.5 max-w-55">
        <label htmlFor="backup-folder" className="text-label mb-1 block">
          Backup folder name
        </label>
        <Input id="backup-folder" value={folder} onChange={(e) => setFolder(e.target.value)} />
      </div>
      <InputHint className="mt-1.5">
        A single folder name inside the app's data directory — not a path. Changing it only affects
        backups written from now on; existing ones stay where they are.
      </InputHint>
      <div className="mt-3.5 flex gap-2">
        <Button variant="secondary" onClick={saveRetention}>
          Save retention
        </Button>
        <Button variant="secondary" onClick={saveFolder}>
          Save folder
        </Button>
        <Button disabled={runningBackup} onClick={backUpNow}>
          Back up now
        </Button>
      </div>
    </SectionCard>
  );
}

function RestoreCard({
  restorePoints,
  onRestored,
}: {
  restorePoints: BackupRecord[];
  onRestored: () => void;
}) {
  const toast = useToast();
  const { markSignedOut } = useAuth();
  const [selectedId, setSelectedId] = useState<string | null>(
    restorePoints[0] ? String(restorePoints[0].id) : null,
  );
  const [confirmTarget, setConfirmTarget] = useState<
    { kind: "retained"; id: number; label: string } | { kind: "file"; path: string } | null
  >(null);
  const [restoring, setRestoring] = useState(false);

  const selected = restorePoints.find((r) => String(r.id) === selectedId) ?? null;

  async function chooseFile() {
    const path = await openFileDialog({ multiple: false, directory: false });
    if (!path || Array.isArray(path)) return;
    setConfirmTarget({ kind: "file", path });
  }

  async function confirmRestore() {
    if (!confirmTarget) return;
    setRestoring(true);
    try {
      if (confirmTarget.kind === "retained") {
        await restoreFromBackup(confirmTarget.id);
      } else {
        await restoreFromBackupFile(confirmTarget.path);
      }
      setConfirmTarget(null);
      onRestored();
      // T-M8.6-4: the backend already dropped the session (a restored file
      // may hold a different credential) — the frontend must follow it to
      // sign-in rather than staying on a screen whose next authenticated
      // call would just fail with `auth_required`. The confirmation rides
      // to Login as a persistent notice, not a toast: a 3.4s toast racing
      // this redirect risks the operator missing it at the exact moment
      // they need to know the restore actually completed.
      markSignedOut("Restore complete — sign in again.");
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setRestoring(false);
    }
  }

  const namedTarget =
    confirmTarget?.kind === "retained" ? confirmTarget.label : "the file you chose";

  return (
    <SectionCard
      id="settings-card-restore"
      title="Restore"
      description="Overwrites everything currently in the console — cannot be undone"
    >
      {restorePoints.length > 0 ? (
        <>
          <RestoreOptionList
            value={selectedId}
            onValueChange={setSelectedId}
            options={restorePoints.map((r) => ({
              value: String(r.id),
              primary: backupPrimaryLabel(r),
              provenance: backupProvenanceText(r),
            }))}
          />
          <Button
            variant="secondary"
            className="mt-2"
            disabled={!selected}
            onClick={() =>
              selected &&
              setConfirmTarget({
                kind: "retained",
                id: selected.id,
                label: backupPrimaryLabel(selected),
              })
            }
          >
            Restore from {selected ? backupPrimaryLabel(selected) : "selected backup"}
          </Button>
        </>
      ) : (
        <InputHint>No whole-console backup has been taken yet.</InputHint>
      )}
      <div className="mt-2">
        <button type="button" className="text-caption text-accent" onClick={chooseFile}>
          Restore from a file…
        </button>
      </div>

      <ChecklistConfirmDialog
        open={confirmTarget !== null}
        onOpenChange={(open) => !open && setConfirmTarget(null)}
        title="Restore from backup"
        warning={
          <span>
            <strong>This replaces everything</strong> currently in the console with the backup from{" "}
            <strong>{namedTarget}</strong>. Anything recorded after that moment will be lost unless
            it is in a newer backup.
          </span>
        }
        checklistLabel="I understand this overwrites all current data and cannot be undone."
        confirmLabel="Restore"
        busy={restoring}
        onConfirm={confirmRestore}
      />
    </SectionCard>
  );
}

// --- Screen ---

const SETTINGS_NAV_GROUPS = [
  {
    label: "Calculation rules",
    items: [
      { id: "settings-card-slab", label: "Slab table" },
      { id: "settings-card-royalty", label: "Royalty" },
      { id: "settings-card-structure", label: "Structure" },
    ],
  },
  {
    label: "Reporting",
    items: [
      { id: "settings-card-reporting", label: "Reporting" },
      { id: "settings-card-low-contribution", label: "Low-contribution threshold" },
    ],
  },
  {
    label: "System",
    items: [
      { id: "settings-card-session", label: "Session" },
      { id: "settings-card-backup", label: "Backup schedule" },
      { id: "settings-card-restore", label: "Restore" },
    ],
  },
];

function SettingsNav() {
  return (
    <nav className="sticky top-20 flex flex-col gap-4">
      {SETTINGS_NAV_GROUPS.map((group, i) => (
        <div
          key={group.label}
          className={cn(
            "flex flex-col gap-0.5",
            i > 0 && "border-t border-border pt-3.5",
          )}
        >
          <div className="text-label text-muted-text px-2.5 pb-1">{group.label}</div>
          {group.items.map((item) => (
            <a
              key={item.id}
              href={`#${item.id}`}
              className="flex h-8 items-center rounded-sm px-2.5 text-[13.5px] text-ink transition-[background] duration-100 hover:bg-bg"
            >
              {item.label}
            </a>
          ))}
        </div>
      ))}
    </nav>
  );
}

export function Settings() {
  const toast = useToast();
  const [settings, setSettings] = useState<SettingsData | null>(null);
  const [slabRows, setSlabRows] = useState<SlabRow[]>([]);
  const [backupSettings, setBackupSettings] = useState<ConsoleBackupSettings | null>(null);
  const [restorePoints, setRestorePoints] = useState<BackupRecord[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      getSettings(),
      getConsoleBackupSettings(),
      getDirectChildrenChart({ fullTree: false }),
      listRestorePoints(),
    ])
      .then(([s, cb, chart, points]) => {
        setSettings(s);
        setBackupSettings(cb);
        setSlabRows(chart.slabTable);
        setRestorePoints(points);
      })
      .catch((raw) => toast.add({ title: errorMessage(raw), type: "danger" }))
      .finally(() => setLoading(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function refreshRestorePoints() {
    try {
      setRestorePoints(await listRestorePoints());
    } catch {
      // The Restore card already reflects the failure via its own toast.
    }
  }

  if (loading) {
    return <h1 className="text-headline">Settings</h1>;
  }

  if (!settings || !backupSettings) {
    return (
      <>
        <PageHeader title="Settings" />
        <AlertNote variant="danger" className="mt-4 max-w-md">
          Settings could not be loaded.
        </AlertNote>
      </>
    );
  }

  const lastBackup = restorePoints
    .filter((r) => r.kind === "scheduled" || r.kind === "manual")
    .reduce<BackupRecord | null>(
      (latest, r) => (!latest || r.createdAt > latest.createdAt ? r : latest),
      null,
    );
  const lastBackupLabel = lastBackup
    ? new Date(lastBackup.createdAt).toLocaleDateString(undefined, {
        day: "numeric",
        month: "short",
        year: "numeric",
      })
    : null;

  return (
    <>
      <PageHeader title="Settings" subtitle="Each section saves independently" />

      <div className="mt-5 grid grid-cols-[190px_1fr] items-start gap-6">
        <SettingsNav />
        <div className="flex flex-col gap-4">
          <SlabTableCard rows={slabRows} onRowsChange={setSlabRows} />
          <RoyaltyCard settings={settings} onSettingsChange={setSettings} />
          <StructureGuidanceCard settings={settings} onSettingsChange={setSettings} />
          <ReportingCard settings={settings} onSettingsChange={setSettings} />
          <LowContributionThresholdCard settings={settings} onSettingsChange={setSettings} />
          <SessionCard settings={settings} onSettingsChange={setSettings} />
          <BackupScheduleCard
            backupSettings={backupSettings}
            onBackupSettingsChange={setBackupSettings}
            onBackedUp={(record) => setRestorePoints((prev) => [record, ...prev])}
            lastBackupLabel={lastBackupLabel}
          />
          <RestoreCard restorePoints={restorePoints} onRestored={refreshRestorePoints} />
        </div>
      </div>
    </>
  );
}
