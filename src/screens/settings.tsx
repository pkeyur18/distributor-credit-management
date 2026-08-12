import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input, InputHint } from "@/components/ui/input";
import { AlertNote } from "@/components/ui/alert-note";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { RestoreOptionList } from "@/components/restore-option-list";
import { ChecklistConfirmDialog } from "@/components/checklist-confirm-dialog";
import { RecalcWarningDialog } from "@/components/recalc-warning-dialog";
import { useToast } from "@/components/ui/toast";
import { centsToDisplay, displayToCents } from "@/lib/utils";
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
import { previewSettingsImpact, type CandidateSettings, type SettingsImpactPreview } from "@/lib/ipc/m3-calc";
import { runConsoleBackupNow } from "@/lib/ipc/m8-auth";
import { listRestorePoints, restoreFromBackup, restoreFromBackupFile } from "@/lib/ipc/preflight";
import type { BackupRecord, Settings as SettingsData, SlabRow } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";

// T-M7.2-4's "default export columns" has no picker anywhere in the
// approved prototype (Reports/M6 doesn't exist until S13) and no key
// convention has been established yet — this list and its keys are this
// sprint's own naming, following the seeded mandatory five's style
// (`db/seed.rs`). M6 should align with these rather than inventing a
// second set.
const MANDATORY_EXPORT_COLUMNS: { key: string; label: string }[] = [
  { key: "name", label: "Name" },
  { key: "member_number", label: "Member number" },
  { key: "phone", label: "Phone" },
  { key: "business_volume", label: "Business Volume" },
  { key: "total_business_volume", label: "Total Business Volume" },
];

// Rule-33's optional list, minus Total Business Volume — D-1 already moved
// that into the mandatory five above.
const OPTIONAL_EXPORT_COLUMNS: { key: string; label: string }[] = [
  { key: "email", label: "Email" },
  { key: "address", label: "Address" },
  { key: "reference_number", label: "Reference number" },
  { key: "introducer_name", label: "Introducer name" },
  { key: "hierarchy_level", label: "Hierarchy level" },
  { key: "direct_legs_count", label: "Direct legs count" },
  { key: "slab_pct", label: "Slab %" },
  { key: "rewards", label: "Rewards" },
  { key: "royalty_earned", label: "Royalty earned" },
  { key: "joining_date", label: "Joining date" },
  { key: "active_status", label: "Active/inactive status" },
];

function SectionCard({
  id,
  title,
  description,
  children,
}: {
  id: string;
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <Card id={id}>
      <CardHeader className="block">
        <CardTitle>{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
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

function candidateFromSlabRows(pairs: { threshold: number; percentage: number }[]): CandidateSettings {
  return {
    slabThresholds: pairs.map((p) => p.threshold),
    slabPercentages: pairs.map((p) => p.percentage),
  };
}

function SlabTableCard({
  rows,
  onRowsChange,
}: {
  rows: SlabRow[];
  onRowsChange: (rows: SlabRow[]) => void;
}) {
  const toast = useToast();
  const recalcWarning = useRecalcWarning();
  const [drafts, setDrafts] = useState<Record<number, { threshold: string; percentage: string }>>(
    {},
  );
  const [newRow, setNewRow] = useState({ threshold: "", percentage: "" });
  const [saving, setSaving] = useState<number | "new" | null>(null);

  function draftFor(row: SlabRow) {
    return drafts[row.id] ?? { threshold: centsToDisplay(row.threshold), percentage: String(row.percentage) };
  }

  function setDraft(id: number, patch: Partial<{ threshold: string; percentage: string }>) {
    setDrafts((prev) => ({ ...prev, [id]: { ...draftFor({ id } as SlabRow), ...prev[id], ...patch } }));
  }

  async function saveRow(row: SlabRow) {
    const draft = draftFor(row);
    const threshold = displayToCents(draft.threshold);
    const percentage = Number(draft.percentage);
    if (threshold === null || !Number.isFinite(percentage)) {
      toast.add({ title: "Enter a valid threshold and percentage", type: "danger" });
      return;
    }
    const candidate = candidateFromSlabRows(
      rows.map((r) => (r.id === row.id ? { threshold, percentage } : { threshold: r.threshold, percentage: r.percentage })),
    );
    await recalcWarning.request("slab", candidate, async () => {
      setSaving(row.id);
      try {
        const updated = await updateSlabRow(row.id, { threshold, percentage });
        onRowsChange(rows.map((r) => (r.id === row.id ? updated : r)));
        setDrafts((prev) => {
          const next = { ...prev };
          delete next[row.id];
          return next;
        });
        toast.add({ title: "Slab row saved", type: "success" });
      } finally {
        setSaving(null);
      }
    });
  }

  async function removeRow(row: SlabRow) {
    const candidate = candidateFromSlabRows(
      rows.filter((r) => r.id !== row.id).map((r) => ({ threshold: r.threshold, percentage: r.percentage })),
    );
    await recalcWarning.request("slab", candidate, async () => {
      setSaving(row.id);
      try {
        await removeSlabRow(row.id);
        onRowsChange(rows.filter((r) => r.id !== row.id));
        toast.add({ title: "Slab row removed", type: "success" });
      } finally {
        setSaving(null);
      }
    });
  }

  async function addRow() {
    const threshold = displayToCents(newRow.threshold);
    const percentage = Number(newRow.percentage);
    if (threshold === null || !Number.isFinite(percentage)) {
      toast.add({ title: "Enter a valid threshold and percentage", type: "danger" });
      return;
    }
    const candidate = candidateFromSlabRows([
      ...rows.map((r) => ({ threshold: r.threshold, percentage: r.percentage })),
      { threshold, percentage },
    ]);
    await recalcWarning.request("slab", candidate, async () => {
      setSaving("new");
      try {
        const created = await addSlabRow({ threshold, percentage });
        onRowsChange([...rows, created]);
        setNewRow({ threshold: "", percentage: "" });
        toast.add({ title: "Slab row added", type: "success" });
      } finally {
        setSaving(null);
      }
    });
  }

  const onlyRow = rows.length <= 1;

  return (
    <SectionCard
      id="settings-card-slab"
      title="Slab table"
      description="The band looked up from Total Business Volume. Top slab recalculates automatically."
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
            {rows.map((row) => {
              const draft = draftFor(row);
              return (
                <tr key={row.id} className="border-t border-border">
                  <td className="py-1.5 pr-2">
                    <Input
                      id={`slab-threshold-${row.id}`}
                      className="num"
                      value={draft.threshold}
                      onChange={(e) => setDraft(row.id, { threshold: e.target.value })}
                    />
                  </td>
                  <td className="py-1.5 pr-2">
                    <Input
                      id={`slab-percentage-${row.id}`}
                      className="num"
                      value={draft.percentage}
                      onChange={(e) => setDraft(row.id, { percentage: e.target.value })}
                    />
                  </td>
                  <td className="py-1.5 pr-2">
                    <Button
                      id={`slab-save-${row.id}`}
                      size="sm"
                      variant="secondary"
                      disabled={saving === row.id}
                      onClick={() => saveRow(row)}
                    >
                      Save
                    </Button>
                  </td>
                  <td className="py-1.5">
                    <Button
                      size="sm"
                      variant="ghost"
                      disabled={onlyRow || saving === row.id}
                      aria-label={
                        onlyRow
                          ? "Remove row — the table must keep at least one slab"
                          : "Remove this slab row"
                      }
                      onClick={() => removeRow(row)}
                    >
                      ✕
                    </Button>
                  </td>
                </tr>
              );
            })}
            <tr className="border-t border-border">
              <td className="py-1.5 pr-2">
                <Input
                  id="slab-new-threshold"
                  className="num"
                  placeholder="0.00"
                  value={newRow.threshold}
                  onChange={(e) => setNewRow((prev) => ({ ...prev, threshold: e.target.value }))}
                />
              </td>
              <td className="py-1.5 pr-2">
                <Input
                  id="slab-new-percentage"
                  className="num"
                  placeholder="0"
                  value={newRow.percentage}
                  onChange={(e) => setNewRow((prev) => ({ ...prev, percentage: e.target.value }))}
                />
              </td>
              <td className="py-1.5" colSpan={2}>
                <Button id="slab-add-row" size="sm" variant="secondary" disabled={saving === "new"} onClick={addRow}>
                  Add row
                </Button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <InputHint className="mt-3">
        Rows are not checked for consistency against each other — this is deliberate.
        Misconfiguring the table can produce unexpected Rewards.
      </InputHint>
      {onlyRow && (
        <InputHint className="mt-1.5">
          The last slab cannot be removed — the console needs at least one to work out a
          member&apos;s slab.
        </InputHint>
      )}
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
        A member with {minChildren || settings.royaltyQualifyingCount}+ direct legs on the top
        slab earns {rate || settings.royaltyRatePercent}% royalty on each qualifying leg&apos;s
        Total Business Volume.
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
    const values = { hierarchyDepth: Number(depth), level2Width: Number(l2), level3Width: Number(l3), level4Width: Number(l4) };
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
          <label htmlFor="depth-guidance" className="text-label mb-1 block">
            Depth guidance
          </label>
          <Input id="depth-guidance" value={depth} onChange={(e) => setDepth(e.target.value)} />
        </div>
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

function ReportingCard({
  settings,
  onSettingsChange,
}: {
  settings: SettingsData;
  onSettingsChange: (next: SettingsData) => void;
}) {
  const toast = useToast();
  const [start, setStart] = useState(settings.yearlyCycle.start);
  const [end, setEnd] = useState(settings.yearlyCycle.end);
  const [threshold, setThreshold] = useState(centsToDisplay(settings.lowContributionThreshold));
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
    const lowContributionThreshold = displayToCents(threshold);
    if (lowContributionThreshold === null) {
      toast.add({ title: "Enter a valid low-contribution threshold", type: "danger" });
      return;
    }
    setSaving(true);
    try {
      const updated = await updateSettings({
        yearlyCycle: { start, end },
        lowContributionThreshold,
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
      description="Controls the yearly cycle and the low-contribution report"
    >
      <div className="grid grid-cols-3 gap-3">
        <div>
          <label htmlFor="cycle-start" className="text-label mb-1 block">
            Yearly cycle starts
          </label>
          <Input id="cycle-start" placeholder="MM-DD" value={start} onChange={(e) => setStart(e.target.value)} />
        </div>
        <div>
          <label htmlFor="cycle-end" className="text-label mb-1 block">
            Yearly cycle ends
          </label>
          <Input id="cycle-end" placeholder="MM-DD" value={end} onChange={(e) => setEnd(e.target.value)} />
        </div>
        <div>
          <label htmlFor="low-threshold-setting" className="text-label mb-1 block">
            Low-contribution threshold
          </label>
          <Input
            id="low-threshold-setting"
            className="num"
            value={threshold}
            onChange={(e) => setThreshold(e.target.value)}
          />
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

function ReferenceUnitValueCard({
  settings,
  onSettingsChange,
}: {
  settings: SettingsData;
  onSettingsChange: (next: SettingsData) => void;
}) {
  const toast = useToast();
  const [value, setValue] = useState(String(settings.referenceUnitValue));
  const [saving, setSaving] = useState(false);

  async function save() {
    const referenceUnitValue = Number(value);
    if (!Number.isFinite(referenceUnitValue)) {
      toast.add({ title: "Enter a valid number", type: "danger" });
      return;
    }
    setSaving(true);
    try {
      const updated = await updateSettings({ referenceUnitValue });
      onSettingsChange(updated);
      toast.add({ title: "Reference unit value saved", type: "success" });
    } catch (raw) {
      toast.add({ title: errorMessage(raw), type: "danger" });
    } finally {
      setSaving(false);
    }
  }

  return (
    <SectionCard
      id="settings-card-reference"
      title="Reference unit value"
      description="Reference only — never shown on any screen, report, or export, and used in no calculation"
    >
      <div className="max-w-55">
        <label htmlFor="reference-unit-value" className="text-label mb-1 block">
          1 unit =
        </label>
        <Input id="reference-unit-value" value={value} onChange={(e) => setValue(e.target.value)} />
      </div>
      <Button className="mt-3.5" disabled={saving} onClick={save}>
        Save
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
        Save
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
}: {
  backupSettings: ConsoleBackupSettings;
  onBackupSettingsChange: (next: ConsoleBackupSettings) => void;
  onBackedUp: (record: BackupRecord) => void;
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
        <Input id="backup-retention" value={retention} onChange={(e) => setRetention(e.target.value)} />
      </div>
      <InputHint className="mt-1.5">
        Older backups beyond this count are removed automatically — closed-month backups are
        never affected.
      </InputHint>
      <div className="mt-3.5 max-w-55">
        <label htmlFor="backup-folder" className="text-label mb-1 block">
          Backup folder name
        </label>
        <Input id="backup-folder" value={folder} onChange={(e) => setFolder(e.target.value)} />
      </div>
      <InputHint className="mt-1.5">
        A single folder name inside the app's data directory — not a path. Changing it only
        affects backups written from now on; existing ones stay where they are.
      </InputHint>
      <div className="mt-3.5 flex gap-2">
        <Button variant="secondary" onClick={saveRetention}>
          Save
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

function backupPrimaryLabel(record: BackupRecord): string {
  const date = new Date(record.createdAt).toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
  if (record.kind === "period_close") return `Closed month — ${date}`;
  if (record.kind === "pre_restore_safety") return `Safety copy — ${date}`;
  if (record.kind === "scheduled") {
    const cadence = record.scheduleKind ? record.scheduleKind[0].toUpperCase() + record.scheduleKind.slice(1) : "Scheduled";
    return `${cadence} — ${date}`;
  }
  return `Manual — ${date}`;
}

function RestoreCard({
  restorePoints,
  onRestored,
}: {
  restorePoints: BackupRecord[];
  onRestored: () => void;
}) {
  const toast = useToast();
  const [selectedId, setSelectedId] = useState<string | null>(
    restorePoints[0] ? String(restorePoints[0].id) : null,
  );
  const [confirmTarget, setConfirmTarget] = useState<{ kind: "retained"; id: number; label: string } | { kind: "file"; path: string } | null>(null);
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
      toast.add({ title: "Restored — sign in again", type: "success" });
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
              provenance: `Version ${r.version}`,
            }))}
          />
          <Button
            variant="secondary"
            className="mt-2"
            disabled={!selected}
            onClick={() =>
              selected &&
              setConfirmTarget({ kind: "retained", id: selected.id, label: backupPrimaryLabel(selected) })
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
            <strong>This replaces everything</strong> currently in the console with the backup
            from <strong>{namedTarget}</strong>. Anything recorded after that moment will be lost
            unless it is in a newer backup.
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
        <h1 className="text-headline">Settings</h1>
        <AlertNote variant="danger" className="mt-4 max-w-md">
          Settings could not be loaded.
        </AlertNote>
      </>
    );
  }

  return (
    <>
      <h1 className="text-headline">Settings</h1>
      <p className="mt-1 text-caption">Each section saves independently</p>

      <div className="mt-5 flex flex-col gap-4">
        <SlabTableCard rows={slabRows} onRowsChange={setSlabRows} />
        <RoyaltyCard settings={settings} onSettingsChange={setSettings} />
        <StructureGuidanceCard settings={settings} onSettingsChange={setSettings} />
        <ReportingCard settings={settings} onSettingsChange={setSettings} />
        <ReferenceUnitValueCard settings={settings} onSettingsChange={setSettings} />
        <SessionCard settings={settings} onSettingsChange={setSettings} />
        <BackupScheduleCard
          backupSettings={backupSettings}
          onBackupSettingsChange={setBackupSettings}
          onBackedUp={(record) => setRestorePoints((prev) => [record, ...prev])}
        />
        <RestoreCard restorePoints={restorePoints} onRestored={refreshRestorePoints} />
      </div>
    </>
  );
}
