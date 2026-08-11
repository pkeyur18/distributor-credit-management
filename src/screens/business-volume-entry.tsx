import { useEffect, useState } from "react";
import { ArrowRight } from "lucide-react";
import { Link, useSearchParams } from "react-router";

import { Button } from "@/components/ui/button";
import { Input, InputHint } from "@/components/ui/input";
import { Pill } from "@/components/ui/pill";
import { SearchResultsList } from "@/components/search-results-list";
import { useMemberSearch } from "@/lib/use-member-search";
import { searchMembers } from "@/lib/ipc/m1-members";
import { recordEntry } from "@/lib/ipc/m2-entries";
import type { BusinessVolumeEntry as Entry } from "@/lib/ipc/entities";
import type { SearchResult } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { useToast } from "@/components/ui/toast";
import { centsToDisplay, displayToCents } from "@/lib/utils";

function isoDate(d: Date) {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

// T-M2.1-4: bounded to the recording month, defaulting to today. US-M2.3's
// outstanding-month recording (S12) doesn't exist yet, so "the recording
// month" is always the current calendar month for now.
function currentMonthBounds() {
  const now = new Date();
  const first = new Date(now.getFullYear(), now.getMonth(), 1);
  return { min: isoDate(first), max: isoDate(now) };
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
  const [date, setDate] = useState(() => currentMonthBounds().max);
  const [amountInput, setAmountInput] = useState("");
  const [amountError, setAmountError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [sessionEntries, setSessionEntries] = useState<Entry[]>([]);
  const toast = useToast();
  const bounds = currentMonthBounds();
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

  const cents = displayToCents(amountInput);
  const canSave = !!selected && cents !== null && !saving;

  async function handleSave() {
    if (!selected || cents === null) return;
    setSaving(true);
    setAmountError(null);
    try {
      const entry = await recordEntry({ memberId: selected.id, amount: cents, entryDate: date });
      setSessionEntries((prev) => [entry, ...prev]);
      setAmountInput("");
      toast.add({ title: `Recorded ${centsToDisplay(entry.amount)} for ${selected.name}`, type: "success" });

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
      setAmountError(presented.message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <>
      <h1 className="text-headline">Business Volume Entry</h1>

      <div className="mt-5 max-w-md rounded-lg border border-border bg-surface p-4.5">
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
            Date *
          </label>
          <Input
            id="entry-date"
            type="date"
            min={bounds.min}
            max={bounds.max}
            value={date}
            disabled={!selected}
            onChange={(e) => setDate(e.target.value)}
          />
        </div>

        <div className="mt-3.5">
          <label htmlFor="entry-amount" className="text-label mb-1 block">
            Business Volume *
          </label>
          <Input
            id="entry-amount"
            className="num"
            type="text"
            inputMode="decimal"
            placeholder="0.00"
            value={amountInput}
            disabled={!selected}
            aria-invalid={!!amountError}
            onChange={(e) => {
              setAmountInput(e.target.value);
              setAmountError(null);
            }}
          />
          <InputHint error={!!amountError}>
            {amountError ?? "Up to two decimals · no currency field"}
          </InputHint>
        </div>

        <Button variant="primary" className="mt-3.5 w-full" disabled={!canSave} onClick={handleSave}>
          Save entry
        </Button>
      </div>

      <p className="mt-3.5 max-w-md text-caption">
        <Link to="/entry/correct" className="inline-flex items-center gap-1 text-accent">
          Correct a closed month instead <ArrowRight className="size-3.5" />
        </Link>
      </p>

      {sessionEntries.length > 0 && (
        <div className="mt-4.5 max-w-md">
          <div className="text-title-sm mb-1.5">Recorded this session</div>
          {sessionEntries.map((e) => (
            <div
              key={`${e.id}-${e.updatedAt ?? e.createdAt}`}
              className="flex items-center justify-between border-b border-border py-2 text-body last:border-b-0"
            >
              <span>{e.entryDate}</span>
              <span className="num">{centsToDisplay(e.amount)}</span>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
