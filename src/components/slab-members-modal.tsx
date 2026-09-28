import { useState } from "react";
import { ChevronLeft, ChevronRight, Search } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Modal, ModalBody, ModalHeader } from "@/components/ui/dialog";
import { Pill } from "@/components/ui/pill";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { TablePagination, usePagination } from "@/components/ui/pagination";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  TableWrap,
} from "@/components/ui/table";
import type { ChartNode } from "@/lib/ipc/entities";
import { centsToDisplay } from "@/lib/utils";

// Home slab drill-down (client-approved prototype, 28 Sep 2026): one panel
// behind both slab charts. Opened from "Members by slab" it reads as a
// roster (by name); from "Rewards by slab" it reads as a ranking (by
// Rewards, with each member's share of the slab's total).
type Sort = "name" | "volume" | "rewards";

const SORTS: Record<Sort, (a: ChartNode, b: ChartNode) => number> = {
  name: (a, b) => a.name.localeCompare(b.name),
  volume: (a, b) => b.ownBusinessVolume - a.ownBusinessVolume,
  rewards: (a, b) => b.rewards - a.rewards,
};

const pct = (part: number, whole: number) => (whole ? Math.round((part / whole) * 100) : 0);

interface SlabMembersModalProps {
  nodes: ChartNode[];
  /** Every slab the chart shows, lowest first — the ‹ › arrows walk this. */
  slabs: number[];
  initialPct: number;
  metric: "count" | "rewards";
  month: string;
  onClose: () => void;
  onSelectMember: (memberId: number) => void;
}

function SlabMembersModal({
  nodes,
  slabs,
  initialPct,
  metric,
  month,
  onClose,
  onSelectMember,
}: SlabMembersModalProps) {
  const [slabPct, setSlabPct] = useState(initialPct);
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<Sort>(metric === "count" ? "name" : "rewards");

  const inSlab = nodes.filter((n) => n.slabPct === slabPct);
  const slabRewards = inSlab.reduce((sum, n) => sum + n.rewards, 0);
  const allRewards = nodes.reduce((sum, n) => sum + n.rewards, 0);
  const inactive = inSlab.filter((n) => !n.isActive).length;

  const q = query.trim().toLowerCase();
  const rows = inSlab
    .filter((n) => !q || n.name.toLowerCase().includes(q) || String(n.memberId).includes(q))
    .sort(SORTS[sort]);
  const pagination = usePagination(rows);

  // Empty slabs aren't clickable on the charts, so the arrows skip them too.
  const nonEmpty = slabs.filter((s) => nodes.some((n) => n.slabPct === s));
  const at = nonEmpty.indexOf(slabPct);
  const goTo = (next: number | undefined) => {
    if (next === undefined) return;
    setSlabPct(next);
    setQuery("");
    pagination.setPage(0);
  };

  return (
    <Modal open onOpenChange={(open) => !open && onClose()} wide>
      <ModalHeader
        title={`Members on the ${slabPct}% slab`}
        subtitle={`${month} · from ${metric === "count" ? "Members by slab" : "Rewards by slab"}`}
      >
        <Button
          variant="secondary"
          size="sm"
          aria-label="Previous slab"
          disabled={at <= 0}
          onClick={() => goTo(nonEmpty[at - 1])}
        >
          <ChevronLeft />
        </Button>
        <Button
          variant="secondary"
          size="sm"
          aria-label="Next slab"
          disabled={at === -1 || at >= nonEmpty.length - 1}
          onClick={() => goTo(nonEmpty[at + 1])}
        >
          <ChevronRight />
        </Button>
      </ModalHeader>
      <ModalBody className="flex flex-col gap-3.5">
        <div className="grid grid-cols-3 gap-2.5">
          <Stat
            label="Members"
            value={String(inSlab.length)}
            footer={`${inactive} inactive · ${pct(inSlab.length, nodes.length)}% of all members`}
          />
          <Stat
            label="Rewards"
            value={centsToDisplay(slabRewards)}
            footer={`${pct(slabRewards, allRewards)}% of all Rewards`}
          />
          <Stat
            label="Business Volume"
            value={centsToDisplay(inSlab.reduce((sum, n) => sum + n.ownBusinessVolume, 0))}
            footer="own, this slab"
          />
        </div>

        <div className="flex flex-wrap items-center gap-2.5">
          <div className="relative min-w-45 flex-1">
            <Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-text" />
            <Input
              id="slab-members-search"
              placeholder="Search this slab by name or member number"
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                pagination.setPage(0);
              }}
              className="pl-8"
            />
          </div>
          <SegmentedControl
            value={sort}
            onValueChange={setSort}
            options={[
              { value: "name", label: "Name" },
              { value: "volume", label: "Business Volume" },
              { value: "rewards", label: "Rewards" },
            ]}
          />
        </div>

        <div>
          <TableWrap>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Member</TableHead>
                  <TableHead numeric>Own Business Volume</TableHead>
                  <TableHead numeric>Rewards</TableHead>
                  {metric === "rewards" && <TableHead numeric>Share of slab</TableHead>}
                </TableRow>
              </TableHeader>
              <TableBody>
                {pagination.pageItems.map((n) => (
                  <TableRow
                    key={n.memberId}
                    clickable
                    tabIndex={0}
                    onClick={() => onSelectMember(n.memberId)}
                    onKeyDown={(e) => e.key === "Enter" && onSelectMember(n.memberId)}
                  >
                    <TableCell>
                      <div className="flex items-center gap-1.5 text-title-sm">
                        <span className="truncate">{n.name}</span>
                        {!n.isActive && <Pill variant="inactive">Inactive</Pill>}
                      </div>
                      <div className="mono text-[11px] text-muted-text">#{n.memberId}</div>
                    </TableCell>
                    <TableCell numeric>{centsToDisplay(n.ownBusinessVolume)}</TableCell>
                    <TableCell numeric>{centsToDisplay(n.rewards)}</TableCell>
                    {metric === "rewards" && (
                      <TableCell numeric>
                        <div className="flex items-center justify-end gap-2">
                          <span className="text-caption">{pct(n.rewards, slabRewards)}%</span>
                          <span className="h-1.75 w-16 overflow-hidden rounded-[4px] border border-border bg-bg">
                            <span
                              className="block h-full bg-accent"
                              style={{
                                width: `${slabRewards ? (n.rewards / slabRewards) * 100 : 0}%`,
                              }}
                            />
                          </span>
                        </div>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
                {rows.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center text-caption">
                      No members on this slab match “{query}”
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </TableWrap>
          {rows.length > 0 && <TablePagination idPrefix="slab-members" pagination={pagination} />}
        </div>
      </ModalBody>
    </Modal>
  );
}

function Stat({ label, value, footer }: { label: string; value: string; footer: string }) {
  return (
    <div className="rounded-lg border border-border p-3">
      <div className="text-label text-muted-text">{label}</div>
      <div className="num mt-0.5 text-numeric">{value}</div>
      <div className="text-caption mt-0.5">{footer}</div>
    </div>
  );
}

export { SlabMembersModal };
