import { useEffect, useRef, useState } from "react";
import { ArrowRight } from "lucide-react";
import { Link, useSearchParams } from "react-router";

import { Button } from "@/components/ui/button";
import { Input, InputHint } from "@/components/ui/input";
import { Pill } from "@/components/ui/pill";
import { AlertNote } from "@/components/ui/alert-note";
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
import { MonthSwitcher } from "@/components/month-switcher";
import { PageHeader } from "@/components/page-header";
import { EmptyState } from "@/components/empty-state";
import { usePagination, TablePagination } from "@/components/ui/pagination";
import { useMemberSearch } from "@/lib/use-member-search";
import { searchMembers } from "@/lib/ipc/m1-members";
import {
  recordEntry,
  getPeriodLockStatus,
  listPeriodEntries,
  type PeriodLockStatus,
} from "@/lib/ipc/m2-entries";
import type { PeriodEntryRecord, SearchResult } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { useToast } from "@/components/ui/toast";
import {
  centsToDisplay,
  currentYm,
  displayToCents,
  isoDate,
  monthBounds,
  monthLabel,
  stripNonNumeric,
} from "@/lib/utils";
import { Breadcrumb } from "@/components/breadcrumb";
import { useBackTarget, useRouteLabel } from "@/lib/navigation-history";

// US-M2.1 (§5.4). API-41's `list_period_entries` backs the table below and
// the two summary nodes above the lock-status banner — all three read the
// same fetch, scoped to `recordingMonth` (the outstanding month while one
// exists, otherwise the current month; T-M2.3-2/T-M2.3-4's existing rule,
// reused rather than re-derived).
export function BusinessVolumeEntry() {
  const [query, setQuery] = useState("");
  const { results } = useMemberSearch(query);
  const [selected, setSelected] = useState<SearchResult | null>(null);
  const [lockStatus, setLockStatus] = useState<PeriodLockStatus | null>(null);
  // T-M2.5-2: selectable when more than one month is recordable — null
  // until the operator picks one, which defers to the oldest (index 0).
  const [selectedMonth, setSelectedMonth] = useState<string | null>(null);
  const recordingMonth = selectedMonth ?? lockStatus?.recordablePeriodMonths[0] ?? currentYm();
  const [date, setDate] = useState(() => monthBounds(recordingMonth).max);
  const [amountInput, setAmountInput] = useState("");
  const [amountError, setAmountError] = useState<string | null>(null);
  const [dateError, setDateError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  // Starts empty rather than behind a loading gate (both summary nodes
  // read 0 until the first fetch resolves, then replace with real
  // figures — an explicit product decision, not an oversight).
  const [periodEntries, setPeriodEntries] = useState<PeriodEntryRecord[]>([]);
  const pagination = usePagination(periodEntries);
  const toast = useToast();
  const bounds = monthBounds(recordingMonth);
  const [searchParams] = useSearchParams();
  const backTarget = useBackTarget();
  useRouteLabel("Volume entry");
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Recording BV back-to-back for different members is the product's
  // declared most-frequent action; without this, "Change" cleared the
  // selection but left the operator to click into the search box before
  // they could type — an extra click on every entry after the first.
  useEffect(() => {
    if (!selected) searchInputRef.current?.focus();
  }, [selected]);

  // T-M4.1-5: Member Detail's "Record volume" action opens this screen
  // pre-selected on that member (?member=<id>) rather than empty.
  useEffect(() => {
    const prefillId = searchParams.get("member");
    if (!prefillId) return;
    searchMembers(prefillId, false).then((found) => {
      const match = found.find((r) => String(r.id) === prefillId);
      if (match) setSelected(match);
    });
  }, [searchParams]);

  // T-M2.3-2/T-M2.3-4: the recording month defaults to whichever month is
  // oldest recordable — an outstanding month while one exists, otherwise
  // the current month. The date field re-bounds to it here, at the source
  // of the change, rather than reactively in a second effect.
  useEffect(() => {
    getPeriodLockStatus().then((status) => {
      setLockStatus(status);
      if (selectedMonth === null) setDate(monthBounds(status.recordablePeriodMonths[0]).max);
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // T-M2.5-2: switching months re-bounds the date field to the newly
  // selected one.
  function handleMonthChange(month: string) {
    setSelectedMonth(month);
    setDate(monthBounds(month).max);
  }

  // API-41: refetches whenever the recording month changes (including the
  // one-time correction once `getPeriodLockStatus` resolves and moves
  // `recordingMonth` off its `currentYm()` fallback).
  useEffect(() => {
    listPeriodEntries(recordingMonth).then((result) => {
      setPeriodEntries(result.entries);
      pagination.setPage(0);
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [recordingMonth]);

  const cents = displayToCents(amountInput);
  const canSave = !!selected && cents !== null && !saving;

  async function handleSave() {
    if (!selected || cents === null) return;
    setSaving(true);
    setAmountError(null);
    setDateError(null);
    try {
      const entry = await recordEntry({ memberId: selected.id, amount: cents, entryDate: date });
      setAmountInput("");
      toast.add({
        title: `Recorded ${centsToDisplay(entry.amount)} for ${selected.name}`,
        type: "success",
      });

      // T-M2.1-5: immediate on-screen update of the affected figures, no
      // recalculate control anywhere (Rule-26). `record_entry`'s own
      // response is just the raw entry — `search_members` (M1's shared
      // search, already reading `member_period_totals`) is the only
      // command today that also carries totalBusinessVolume/slabPct, so
      // this re-fetches by ID rather than waiting on get_member_detail
      // (S8). The period table/summary nodes get the same in-place
      // refresh via `list_period_entries` (API-41).
      const refreshed = await searchMembers(String(selected.id), false);
      const match = refreshed.find((r) => r.id === selected.id);
      if (match) setSelected(match);
      const refreshedEntries = await listPeriodEntries(recordingMonth);
      setPeriodEntries(refreshedEntries.entries);
      pagination.setPage(0);
    } catch (raw) {
      const presented = toErrorPresentation(raw);
      // T-M2.4-2: a period-eligibility refusal is about the date, not the
      // amount — only that field is rejected, the rest of the form stays
      // available.
      if (
        presented.kind === "period_not_accepting_entries" ||
        presented.kind === "period_closed" ||
        presented.field === "entryDate"
      ) {
        setDateError(presented.message);
      } else {
        setAmountError(presented.message);
      }
    } finally {
      setSaving(false);
    }
  }

  // Raw entry-record counts (not distinct members), per the confirmed
  // definition — a member with two entries this month counts as two here.
  const entriesThisMonthCount = periodEntries.length;
  const todayIso = isoDate(new Date());
  const recordedTodayCount = periodEntries.filter((e) => e.entryDate === todayIso).length;

  return (
    <>
      <PageHeader
        title="Volume Entry"
        breadcrumb={<Breadcrumb backLabel={backTarget.label} onBack={backTarget.go} crumbs={[]} />}
      />

      <div className="mx-auto max-w-200">
        <div className="grid grid-cols-2 gap-3">
          <div className="rounded-lg border border-border bg-surface p-3.5">
            <div className="text-label text-muted-text">Entries recorded</div>
            <div className="num mt-1 text-numeric-lg">{entriesThisMonthCount}</div>
            <div className="text-caption mt-0.5">{monthLabel(recordingMonth)}</div>
          </div>
          <div className="rounded-lg border border-border bg-surface p-3.5">
            <div className="text-label text-muted-text">Recorded today</div>
            <div className="num mt-1 text-numeric-lg">{recordedTodayCount}</div>
          </div>
        </div>

        {lockStatus && (
          <AlertNote variant="warn" className="mt-3.5">
            Recording into <strong>{monthLabel(recordingMonth)}</strong>.{" "}
            {lockStatus.blockingMonth
              ? `${monthLabel(currentYm())} entries can be recorded once ${monthLabel(
                  lockStatus.blockingMonth,
                )} is closed.`
              : "Dates are limited to this month."}
          </AlertNote>
        )}

        {lockStatus && (
          <MonthSwitcher
            months={lockStatus.recordablePeriodMonths}
            value={recordingMonth}
            onChange={handleMonthChange}
          />
        )}

        <div className="mt-5 rounded-lg border border-border bg-surface p-4.5">
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
                <div className="mt-1 flex items-center gap-2 text-caption">
                  <span>
                    {/* SearchResult.totalBusinessVolume is already a real-unit
                      decimal (search_members converts server-side), unlike
                      BusinessVolumeEntry.amount/MemberPeriodFigures below,
                      which stay ×100 integers per ADR-004 — two different,
                      pre-existing conventions live on the wire today. */}
                    Total Business Volume{" "}
                    <span className="num">{selected.totalBusinessVolume.toFixed(2)}</span>
                  </span>
                  <Pill variant="slab">{selected.slabPct}%</Pill>
                </div>
              </div>
              <Button variant="ghost" size="sm" onClick={() => setSelected(null)}>
                Change
              </Button>
            </div>
          ) : (
            <div>
              <label htmlFor="entry-search" className="text-label mb-1 block">
                Member
              </label>
              <Input
                ref={searchInputRef}
                id="entry-search"
                placeholder="Search by name, 6-digit member number or phone"
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

          <div className="mt-3.5">
            <label htmlFor="entry-date" className="text-label mb-1 block">
              Date <span className="text-danger">*</span>
            </label>
            <Input
              id="entry-date"
              type="date"
              min={bounds.min}
              max={bounds.max}
              value={date}
              disabled={!selected}
              aria-invalid={!!dateError}
              onChange={(e) => {
                setDate(e.target.value);
                setDateError(null);
              }}
            />
            {dateError && <InputHint error>{dateError}</InputHint>}
          </div>

          <div className="mt-3.5">
            <label htmlFor="entry-amount" className="text-label mb-1 block">
              Business Volume <span className="text-danger">*</span>
            </label>
            <Input
              id="entry-amount"
              className="num text-numeric-lg h-14.5"
              type="text"
              inputMode="decimal"
              placeholder="0.00"
              value={amountInput}
              disabled={!selected}
              aria-invalid={!!amountError}
              onChange={(e) => {
                setAmountInput(stripNonNumeric(e.target.value));
                setAmountError(null);
              }}
            />
            <InputHint error={!!amountError}>
              {amountError ?? "Numbers only · up to two decimals"}
            </InputHint>
          </div>

          <Button
            variant="primary"
            className="mt-3.5 w-full"
            disabled={!canSave}
            onClick={handleSave}
          >
            Save entry
          </Button>
        </div>

        <p className="mt-3.5 text-caption">
          <Link to="/entry/correct" className="inline-flex items-center gap-1 text-accent">
            Correct a closed month instead <ArrowRight className="size-3.5" />
          </Link>
        </p>

        <div className="mt-4.5">
          <div className="text-title-sm mb-1.5">{monthLabel(recordingMonth)} entries</div>
          {periodEntries.length === 0 ? (
            <div className="rounded-lg border border-border bg-surface">
              <EmptyState title={`No entries yet in ${monthLabel(recordingMonth)}`} />
            </div>
          ) : (
            <>
              <TableWrap>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Name</TableHead>
                      <TableHead>Member #</TableHead>
                      <TableHead>Recorded Date</TableHead>
                      <TableHead numeric>Business Volume</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {pagination.pageItems.map((e) => (
                      <TableRow key={`${e.id}-${e.updatedAt ?? e.createdAt}`}>
                        <TableCell primary>{e.memberName}</TableCell>
                        <TableCell className="mono">{e.memberId}</TableCell>
                        <TableCell>{e.entryDate}</TableCell>
                        <TableCell numeric>
                          <span className="num">{centsToDisplay(e.amount)}</span>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableWrap>
              <TablePagination idPrefix="entries" pagination={pagination} />
            </>
          )}
        </div>
      </div>
    </>
  );
}
