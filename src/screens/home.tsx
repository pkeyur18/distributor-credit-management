import { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { PlusCircle, Search } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { MemberModal } from "@/components/member-modal";
import { PageHeader } from "@/components/page-header";
import { SearchResultsList } from "@/components/search-results-list";
import { BarListChart, type BarListRow } from "@/components/bar-list-chart";
import { EmptyState } from "@/components/empty-state";
import { MonthSwitcher } from "@/components/month-switcher";
import { useMemberSearch } from "@/lib/use-member-search";
import { getDirectChildrenChart } from "@/lib/ipc/m4-search";
import { getPeriodLockStatus, type PeriodLockStatus } from "@/lib/ipc/m2-entries";
import type { ChartNode, SlabRow } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { centsToDisplay, monthLabel } from "@/lib/utils";

// US-M1.1 (T-M1.1-7), US-M4.4 (§5.1). Search now navigates straight to
// Member Detail (US-M4.1, S8) rather than the inline edit/deactivate card
// this screen carried before that view existed — same as the prototype's
// own `homeResultsHtml`.
export function Home() {
  const [addMemberOpen, setAddMemberOpen] = useState(false);
  const [query, setQuery] = useState("");
  const { results } = useMemberSearch(query);
  const navigate = useNavigate();

  const [nodes, setNodes] = useState<ChartNode[] | null>(null);
  const [slabTable, setSlabTable] = useState<SlabRow[] | null>(null);
  // No `create_root_member` route existed anywhere in the UI before this
  // sprint — `get_direct_children_chart` refusing with "not found" against
  // an empty directory is the one signal that the console has no root
  // member yet, which is what the Add Member modal needs to skip the
  // (otherwise-mandatory) introducer field.
  const [noRootYet, setNoRootYet] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);

  // T-M2.5-3: this figure screen defaults to the oldest recordable month,
  // switchable when more than one is outstanding (CR-2).
  const [lockStatus, setLockStatus] = useState<PeriodLockStatus | null>(null);
  const [selectedMonth, setSelectedMonth] = useState<string | null>(null);
  const viewMonth = selectedMonth ?? lockStatus?.recordablePeriodMonths[0];

  useEffect(() => {
    getPeriodLockStatus().then(setLockStatus);
  }, []);

  useEffect(() => {
    getDirectChildrenChart({ fullTree: true, periodMonth: viewMonth })
      .then((result) => {
        setNodes(result.nodes);
        setSlabTable(result.slabTable);
        setNoRootYet(false);
      })
      .catch((raw) => {
        if (toErrorPresentation(raw).kind === "not_found") setNoRootYet(true);
      });
  }, [refreshKey, viewMonth]);

  return (
    <>
      <PageHeader
        title="Home"
        subtitle="Search any member, or scan today's standing"
        actions={
          <Button variant="primary" onClick={() => setAddMemberOpen(true)}>
            <PlusCircle className="size-4" />
            Add member
          </Button>
        }
      />

      {noRootYet && (
        <div className="mt-4">
          <EmptyState
            title="No members yet"
            description="Add the first (root) member to start building the structure."
          />
        </div>
      )}

      <div className="mt-4 rounded-lg border border-border bg-surface p-4.5">
        <div className="relative">
          <Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-text" />
          <Input
            id="home-search"
            placeholder="Search by name, 6-digit member number or phone"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="h-11 pl-9 text-[15px]"
          />
        </div>
        <div className="mt-3">
          {query.trim() ? (
            <SearchResultsList
              results={results}
              query={query}
              onSelect={(r) => navigate(`/member/${r.id}`)}
            />
          ) : (
            <EmptyState
              icon={<Search className="size-8" />}
              title="Search for a member to see their details"
              description="Type a name, 6-digit member number or phone number above — nothing is shown until you search."
              descriptionClassName="text-[11px]"
            />
          )}
        </div>
      </div>

      {lockStatus && (
        <MonthSwitcher
          months={lockStatus.recordablePeriodMonths}
          value={viewMonth ?? lockStatus.recordablePeriodMonths[0]}
          onChange={setSelectedMonth}
        />
      )}

      {nodes && slabTable && viewMonth && (
        <>
          <div className="text-label text-muted-text mt-4">Today's standing</div>
          <StatRow nodes={nodes} slabTable={slabTable} viewMonth={viewMonth} />
        </>
      )}

      {nodes && slabTable && (
        <div className="mt-4 grid gap-4 sm:grid-cols-2">
          <SlabDistributionChart
            title="Members by slab"
            nodes={nodes}
            slabTable={slabTable}
            metric="count"
          />
          <SlabDistributionChart
            title="Rewards by slab"
            nodes={nodes}
            slabTable={slabTable}
            metric="rewards"
          />
        </div>
      )}

      <MemberModal
        open={addMemberOpen}
        onOpenChange={setAddMemberOpen}
        mode="add"
        noMembersYet={noRootYet}
        onSaved={() => setRefreshKey((k) => k + 1)}
      />
    </>
  );
}

function StatRow({
  nodes,
  slabTable,
  viewMonth,
}: {
  nodes: ChartNode[];
  slabTable: SlabRow[];
  viewMonth: string;
}) {
  const inactiveCount = nodes.filter((n) => !n.isActive).length;
  const entriesThisPeriod = nodes.filter((n) => n.ownBusinessVolume > 0).length;
  const topSlabPct = Math.max(0, ...slabTable.map((s) => s.percentage));
  const topSlabCount = nodes.filter((n) => n.slabPct === topSlabPct).length;

  return (
    <div className="mt-4 grid grid-cols-3 gap-3">
      <StatCard label="Members" value={String(nodes.length)} footer={`${inactiveCount} inactive`} />
      <StatCard
        label="Entries this period"
        value={String(entriesThisPeriod)}
        footer={monthLabel(viewMonth)}
      />
      <StatCard
        label="On top slab"
        value={String(topSlabCount)}
        footer={`${topSlabPct}% and above`}
      />
    </div>
  );
}

function StatCard({ label, value, footer }: { label: string; value: string; footer: string }) {
  return (
    <div className="rounded-lg border border-border bg-surface p-3.5">
      <div className="text-label text-muted-text">{label}</div>
      <div className="num mt-1 text-numeric-lg">{value}</div>
      <div className="text-caption mt-0.5">{footer}</div>
    </div>
  );
}

// The One Accent Rule (07-design-system.md §Colors): a lightness gradient
// of the single accent colour, never a second hue — ported verbatim from
// the prototype's slabTint().
function slabTint(i: number, n: number) {
  const p = n > 1 ? Math.round(28 + (i / (n - 1)) * 72) : 100;
  return `color-mix(in oklch, var(--accent) ${p}%, var(--bg) ${100 - p}%)`;
}

function SlabDistributionChart({
  title,
  nodes,
  slabTable,
  metric,
}: {
  title: string;
  nodes: ChartNode[];
  slabTable: SlabRow[];
  metric: "count" | "rewards";
}) {
  const buckets = slabTable.map((row) => ({
    pct: row.percentage,
    total:
      metric === "count"
        ? nodes.filter((n) => n.slabPct === row.percentage).length
        : nodes.filter((n) => n.slabPct === row.percentage).reduce((sum, n) => sum + n.rewards, 0),
  }));
  const max = Math.max(1, ...buckets.map((b) => b.total));
  const grandTotal = buckets.reduce((sum, b) => sum + b.total, 0);

  const rows: BarListRow[] = buckets.map((b, i) => ({
    id: b.pct,
    label: `${b.pct}%`,
    value: metric === "count" ? b.total : centsToDisplay(b.total),
    fraction: b.total / max,
    tint: slabTint(i, buckets.length),
  }));

  return (
    <div className="rounded-lg border border-border bg-surface p-4.5">
      <div className="text-title-sm">{title}</div>
      <div className="text-caption mb-5">
        {metric === "count"
          ? `${nodes.length} members total, across ${buckets.length} slabs`
          : `${centsToDisplay(grandTotal)} total this period, across ${buckets.length} slabs`}
      </div>
      {buckets.length === 0 ? (
        <EmptyState title="No slabs configured" />
      ) : (
        <BarListChart size="lg" rows={rows} />
      )}
    </div>
  );
}
