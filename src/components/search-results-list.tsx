import type { SearchResult } from "@/lib/ipc/entities";
import { Pill } from "@/components/ui/pill";

// T-M1.4-5. One dropdown-style component behind every search box's results
// — name, member number, phone (Rule-44/DESIGN.md "Do show the phone
// number in member search results"), slab and status pill. `slabPct`/
// `totalBusinessVolume` read 0 for every member until M3 (S6)/M2 (S7)
// exist — this component doesn't special-case that, it just displays
// whatever `search_members` returns.
interface SearchResultsListProps {
  results: SearchResult[];
  query: string;
  emptyLabel?: string;
  onSelect: (result: SearchResult) => void;
}

function SearchResultsList({
  results,
  query,
  emptyLabel = "No members match",
  onSelect,
}: SearchResultsListProps) {
  if (!query.trim()) return null;

  if (results.length === 0) {
    return (
      <div className="rounded-sm border border-border bg-surface px-3 py-2.5 text-caption">
        {emptyLabel}
      </div>
    );
  }

  return (
    <div className="max-h-70 overflow-y-auto rounded-sm border border-border bg-surface shadow-(--shadow-modal)">
      {results.map((r) => (
        <button
          key={r.id}
          type="button"
          onClick={() => onSelect(r)}
          className="flex w-full items-center justify-between gap-3 border-b border-border px-3 py-2.25 text-left last:border-b-0 hover:bg-bg"
        >
          <div className="min-w-0">
            <div className="flex items-center gap-1.5 text-title-sm">
              <span className="truncate">{r.name}</span>
              {!r.isActive && <Pill variant="inactive">Inactive</Pill>}
            </div>
            <div className="mono text-[11px] text-muted-text">
              #{r.id} · {r.phone}
            </div>
          </div>
          <Pill variant="slab" className="shrink-0">
            {r.slabPct}%
          </Pill>
        </button>
      ))}
    </div>
  );
}

export { SearchResultsList };
