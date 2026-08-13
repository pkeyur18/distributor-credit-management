import { useEffect, useState } from "react";
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
import { useMemberSearch } from "@/lib/use-member-search";
import { searchMembers } from "@/lib/ipc/m1-members";
import { recordEntry, getPeriodLockStatus, type PeriodLockStatus } from "@/lib/ipc/m2-entries";
import type { BusinessVolumeEntry as Entry } from "@/lib/ipc/entities";
import type { SearchResult } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { useToast } from "@/components/ui/toast";
import { centsToDisplay, displayToCents, monthLabel } from "@/lib/utils";

const PAGE_SIZES = [10, 25, 50] as const;

// Frontend-only pairing of a recorded entry with the member name shown
// against it — `BusinessVolumeEntry` itself carries no name (see
// entities.ts), and this session-scoped list is the only feed available
// (no command in the closed 40-command surface lists a member's past
// entries yet — same gap the file's header comment already documents).
type SessionEntry = Entry & { memberName: string };

// Strips anything that isn't a digit or decimal point as the operator
// types, and collapses a second "." rather than let it through — a
// keystroke-level guard only. The actual amount validation stays in
// displayToCents (Rule-16/Rule-16a) below, untouched.
function stripNonNumeric(raw: string): string {
  const digitsAndDots = raw.replace(/[^\d.]/g, "");
  const firstDot = digitsAndDots.indexOf(".");
  if (firstDot === -1) return digitsAndDots;
  return (
    digitsAndDots.slice(0, firstDot + 1) + digitsAndDots.slice(firstDot + 1).replace(/\./g, "")
  );
}

function isoDate(d: Date) {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function currentYm(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

// T-M2.1-4/T-M2.3-4: bounded to the recording month. The current month
// caps at today (can't record ahead of itself); an outstanding earlier
// month is bounded by its own last calendar day.
function monthBounds(ym: string) {
  const [year, month] = ym.split("-").map(Number);
  const first = new Date(year, month - 1, 1);
  const last = new Date(year, month, 0);
  const isCurrent = ym === currentYm();
  return { min: isoDate(first), max: isoDate(isCurrent ? new Date() : last) };
}

// US-M2.1 (§5.4). T-M2.1-6's "this period's entries" list has no backing
// command yet — no API in the closed 40-command surface lists a member's
// past entries (get_member_detail is S8, get_audit_log is S14). Until one
// of those ships, this list is only what's been recorded in the current
// app session, not the month's full history — labelled honestly below
// rather than implying it's complete.
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
  const [sessionEntries, setSessionEntries] = useState<SessionEntry[]>([]);
  const [entriesPage, setEntriesPage] = useState(0);
  const [entriesPageSize, setEntriesPageSize] = useState<number>(PAGE_SIZES[0]);
  const toast = useToast();
  const bounds = monthBounds(recordingMonth);
  const [searchParams] = useSearchParams();

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

  const cents = displayToCents(amountInput);
  const canSave = !!selected && cents !== null && !saving;

  async function handleSave() {
    if (!selected || cents === null) return;
    setSaving(true);
    setAmountError(null);
    setDateError(null);
    try {
      const entry = await recordEntry({ memberId: selected.id, amount: cents, entryDate: date });
      setSessionEntries((prev) => [{ ...entry, memberName: selected.name }, ...prev]);
      setEntriesPage(0);
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
      // (S8).
      const refreshed = await searchMembers(String(selected.id), false);
      const match = refreshed.find((r) => r.id === selected.id);
      if (match) setSelected(match);
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

  const sortedEntries = [...sessionEntries].sort(
    (a, b) => b.entryDate.localeCompare(a.entryDate) || b.id - a.id,
  );
  const totalEntriesPages = Math.max(1, Math.ceil(sortedEntries.length / entriesPageSize));
  const currentEntriesPage = Math.min(entriesPage, totalEntriesPages - 1);
  const entriesRangeStart = currentEntriesPage * entriesPageSize;
  const entriesPageRows = sortedEntries.slice(
    entriesRangeStart,
    entriesRangeStart + entriesPageSize,
  );

  return (
    <>
      <PageHeader title="Volume Entry" />

      <div className="mx-auto max-w-200">
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
                    TBV <span className="num">{selected.totalBusinessVolume.toFixed(2)}</span>
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

        {sessionEntries.length > 0 && (
          <div className="mt-4.5">
            <div className="text-title-sm mb-1.5">Recorded this session</div>
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
                  {entriesPageRows.map((e) => (
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
            <div className="mt-2.5 flex items-center justify-between">
              <span className="text-caption">
                Showing {entriesRangeStart + 1}–
                {Math.min(entriesRangeStart + entriesPageSize, sortedEntries.length)} of{" "}
                {sortedEntries.length}
              </span>
              <div className="flex items-center gap-2">
                <label htmlFor="entries-page-size" className="text-caption">
                  Rows per page
                </label>
                <select
                  id="entries-page-size"
                  className="h-7.5 w-auto rounded-sm border border-border bg-surface px-2 text-body text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
                  value={entriesPageSize}
                  onChange={(e) => {
                    setEntriesPageSize(Number(e.target.value));
                    setEntriesPage(0);
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
                  disabled={currentEntriesPage === 0}
                  onClick={() => setEntriesPage((p) => p - 1)}
                >
                  Prev
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  disabled={currentEntriesPage >= totalEntriesPages - 1}
                  onClick={() => setEntriesPage((p) => p + 1)}
                >
                  Next
                </Button>
              </div>
            </div>
          </div>
        )}
      </div>
    </>
  );
}
