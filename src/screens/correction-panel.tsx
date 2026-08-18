import { useEffect, useState } from "react";
import { ArrowLeft, Pencil, Plus } from "lucide-react";
import { Link } from "react-router";

import { Button } from "@/components/ui/button";
import { Input, InputHint } from "@/components/ui/input";
import { AlertNote } from "@/components/ui/alert-note";
import { Pill } from "@/components/ui/pill";
import { Modal, ModalHeader, ModalBody, ModalFooter, ModalCancel } from "@/components/ui/dialog";
import {
  TableWrap,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import { SearchResultsList } from "@/components/search-results-list";
import { PageHeader } from "@/components/page-header";
import { EmptyState } from "@/components/empty-state";
import { usePagination, TablePagination } from "@/components/ui/pagination";
import { useMemberSearch } from "@/lib/use-member-search";
import { listBackups, type ClosedMonthBackup } from "@/lib/ipc/m6-reports";
import { getMemberDetail } from "@/lib/ipc/m4-search";
import { addClosedMonthEntry, editEntry, listPeriodEntries } from "@/lib/ipc/m2-entries";
import type { BusinessVolumeEntry, PeriodEntryRecord, SearchResult } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { useToast } from "@/components/ui/toast";
import {
  centsToDisplay,
  displayToCents,
  monthBounds,
  monthLabel,
  stripNonNumeric,
} from "@/lib/utils";
import { Breadcrumb } from "@/components/breadcrumb";
import { useBackTarget, useRouteLabel } from "@/lib/navigation-history";

// US-M2.2 (§5.5), Rule-39 (amended 15 Aug 2026 to extend to creation) —
// `edit_entry`/`add_closed_month_entry` are the two correction mechanisms.
// Matches ui-prototype-v2.html's `renderCorrectionPanel()`: pick the closed
// month, pick the member, then edit or add one of that member's entries in
// that month from the listed table — never a bare "Entry ID" lookup.
export function CorrectionPanel() {
  const [months, setMonths] = useState<ClosedMonthBackup[] | null>(null);
  const [selectedMonth, setSelectedMonth] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const { results } = useMemberSearch(query);
  const [selected, setSelected] = useState<SearchResult | null>(null);
  const [historicalSlabPct, setHistoricalSlabPct] = useState<number | null>(null);
  const [entries, setEntries] = useState<PeriodEntryRecord[]>([]);
  const pagination = usePagination(entries);
  const [modal, setModal] = useState<{ entry: PeriodEntryRecord | null } | null>(null);
  const toast = useToast();
  const backTarget = useBackTarget();
  useRouteLabel("Correct a closed month");

  const ym = selectedMonth ?? months?.[0]?.periodMonth ?? null;

  useEffect(() => {
    listBackups().then((rows) => {
      setMonths(rows);
    });
  }, []);

  // The historical TBV/slab shown on the selected-member card reads that
  // month's own snapshot (get_member_detail's periodMonth arg), not the
  // member's live figures — this is a past, closed month.
  // Both effects stay no-ops until a member and month are picked; the two
  // pieces of state they fill only ever render inside a `selected && ym`
  // guard below, so there's nothing to reset when either clears back out.
  useEffect(() => {
    if (!selected || !ym) return;
    getMemberDetail(selected.id, ym).then((detail) => setHistoricalSlabPct(detail.slabPct));
  }, [selected, ym]);

  useEffect(() => {
    if (!selected || !ym) return;
    listPeriodEntries(ym).then((result) => {
      setEntries(result.entries.filter((e) => e.memberId === selected.id));
      pagination.setPage(0);
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selected, ym]);

  function handleMonthChange(month: string) {
    setSelectedMonth(month);
  }

  async function refreshEntries() {
    if (!selected || !ym) return;
    const result = await listPeriodEntries(ym);
    setEntries(result.entries.filter((e) => e.memberId === selected.id));
  }

  return (
    <>
      <PageHeader
        title="Correct a closed month"
        breadcrumb={<Breadcrumb backLabel={backTarget.label} onBack={backTarget.go} crumbs={[]} />}
      />

      <div className="mx-auto max-w-200">
        <AlertNote variant="warn" className="mt-3.5">
          Editing a record recalculates the affected chain and writes a{" "}
          <strong>new snapshot version</strong> — the original record is never overwritten.
        </AlertNote>

        <div className="mt-3.5 rounded-lg border border-border bg-surface p-4.5">
          {months && months.length === 0 ? (
            <EmptyState
              title="No closed months yet"
              description="Nothing has been closed to correct."
            />
          ) : (
            <>
              <div>
                <label htmlFor="correction-ym" className="text-label mb-1 block">
                  Month
                </label>
                <select
                  id="correction-ym"
                  className="h-9 w-full rounded-sm border border-border bg-surface px-2.5 text-body text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
                  value={ym ?? ""}
                  onChange={(e) => handleMonthChange(e.target.value)}
                >
                  {(months ?? []).map((m) => (
                    <option key={m.periodMonth} value={m.periodMonth}>
                      {monthLabel(m.periodMonth)}
                    </option>
                  ))}
                </select>
              </div>

              <div className="mt-3.5">
                {selected ? (
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="flex items-center gap-1.5 text-title-sm">
                        {selected.name}
                        {!selected.isActive && <Pill variant="inactive">Inactive</Pill>}
                      </div>
                      <div className="mono text-[11px] text-muted-text">
                        #{selected.id} · {selected.phone}
                      </div>
                      {historicalSlabPct !== null && (
                        <div className="mt-1 text-caption">
                          <Pill variant="slab">{historicalSlabPct}% slab</Pill>
                        </div>
                      )}
                    </div>
                    <Button variant="ghost" size="sm" onClick={() => setSelected(null)}>
                      Change
                    </Button>
                  </div>
                ) : (
                  <div>
                    <label htmlFor="correction-search" className="text-label mb-1 block">
                      Member
                    </label>
                    <Input
                      id="correction-search"
                      placeholder="Search by name, member number or phone"
                      value={query}
                      onChange={(e) => setQuery(e.target.value)}
                    />
                    <div className="mt-1.5">
                      <SearchResultsList
                        results={results}
                        query={query}
                        onSelect={(r) => {
                          setSelected(r);
                          setQuery("");
                        }}
                      />
                    </div>
                  </div>
                )}
              </div>
            </>
          )}
        </div>

        {selected && ym && (
          <div className="mt-4.5">
            <div className="mb-1.5 flex items-center justify-between">
              <div className="text-title-sm">Records — {monthLabel(ym)}</div>
              <Button variant="secondary" size="sm" onClick={() => setModal({ entry: null })}>
                <Plus className="size-3.5" />
                Add record
              </Button>
            </div>
            {entries.length === 0 ? (
              <div className="rounded-lg border border-border bg-surface">
                <EmptyState title={`No records for ${selected.name} in ${monthLabel(ym)}`} />
              </div>
            ) : (
              <>
                <TableWrap>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Date</TableHead>
                        <TableHead numeric>Business Volume</TableHead>
                        <TableHead />
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {pagination.pageItems.map((e) => (
                        <TableRow key={`${e.id}-${e.updatedAt ?? e.createdAt}`}>
                          <TableCell>{e.entryDate}</TableCell>
                          <TableCell numeric>
                            <span className="num">{centsToDisplay(e.amount)}</span>
                          </TableCell>
                          <TableCell className="w-px">
                            <Button
                              variant="ghost"
                              size="sm"
                              aria-label="Edit record"
                              onClick={() => setModal({ entry: e })}
                            >
                              <Pencil className="size-3.5" />
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableWrap>
                <TablePagination idPrefix="correction" pagination={pagination} />
              </>
            )}
          </div>
        )}

        <p className="mt-3.5 text-caption">
          <Link to="/entry" className="inline-flex items-center gap-1 text-accent">
            <ArrowLeft className="size-3.5" /> Back to volume entry
          </Link>
        </p>
      </div>

      {modal && selected && ym && (
        <EntryModal
          entry={modal.entry}
          memberId={selected.id}
          ym={ym}
          onOpenChange={(open) => !open && setModal(null)}
          onSaved={async (saved) => {
            setModal(null);
            toast.add({
              title: modal.entry
                ? `Entry #${saved.id} corrected to ${centsToDisplay(saved.amount)}`
                : `Entry #${saved.id} added — ${centsToDisplay(saved.amount)}`,
              type: "success",
            });
            await refreshEntries();
          }}
        />
      )}
    </>
  );
}

interface EntryModalProps {
  /** `null` adds a new entry into the closed month; set, edits that one. */
  entry: PeriodEntryRecord | null;
  memberId: number;
  ym: string;
  onOpenChange: (open: boolean) => void;
  onSaved: (saved: BusinessVolumeEntry) => void;
}

function EntryModal({ entry, memberId, ym, onOpenChange, onSaved }: EntryModalProps) {
  const bounds = monthBounds(ym);
  const isNew = entry === null;
  const [date, setDate] = useState(entry?.entryDate ?? bounds.max);
  const [amountInput, setAmountInput] = useState(entry ? centsToDisplay(entry.amount) : "");
  const [amountError, setAmountError] = useState<string | null>(null);
  const [dateError, setDateError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const cents = displayToCents(amountInput);
  const canSave = cents !== null && !!date && !saving;

  async function handleSave() {
    if (!canSave || cents === null) return;
    setSaving(true);
    setAmountError(null);
    setDateError(null);
    try {
      const saved = entry
        ? await editEntry({ id: entry.id, amount: cents, entryDate: date })
        : await addClosedMonthEntry({ memberId, amount: cents, entryDate: date });
      onSaved(saved);
    } catch (raw) {
      const presented = toErrorPresentation(raw);
      if (presented.field === "entryDate") {
        setDateError(presented.message);
      } else {
        setAmountError(presented.message);
      }
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal open onOpenChange={onOpenChange} dismissable>
      <ModalHeader title={isNew ? "Add record" : "Edit record"} />
      <ModalBody>
        <label htmlFor="closed-entry-date" className="text-label mb-1 block">
          Date
        </label>
        <Input
          id="closed-entry-date"
          type="date"
          min={bounds.min}
          max={bounds.max}
          value={date}
          aria-invalid={!!dateError}
          onChange={(e) => {
            setDate(e.target.value);
            setDateError(null);
          }}
        />
        <InputHint error={!!dateError}>
          {dateError ?? `Must stay within ${monthLabel(ym)}.`}
        </InputHint>

        <div className="mt-3.5">
          <label htmlFor="closed-entry-amount" className="text-label mb-1 block">
            Business Volume
          </label>
          <Input
            id="closed-entry-amount"
            className="num"
            type="text"
            inputMode="decimal"
            placeholder="0.00"
            value={amountInput}
            aria-invalid={!!amountError}
            onChange={(e) => {
              setAmountInput(stripNonNumeric(e.target.value));
              setAmountError(null);
            }}
          />
          <InputHint error={!!amountError}>{amountError}</InputHint>
        </div>
      </ModalBody>
      <ModalFooter>
        <ModalCancel />
        <Button variant="primary" disabled={!canSave} onClick={handleSave}>
          Save
        </Button>
      </ModalFooter>
    </Modal>
  );
}
