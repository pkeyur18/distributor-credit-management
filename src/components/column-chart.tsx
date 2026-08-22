import { cn } from "@/lib/utils";
import { rowProgress, useElapsedSinceActive, useRevealOnView } from "@/lib/use-chart-reveal";

// Slab-chart redesign, option 2 (client-approved mockup) — used for "Members
// by slab". Each column grows from zero and its value counts up together,
// staggered per column, the first time the chart scrolls into view.
export interface SlabDatum {
  id: string | number;
  label: string;
  target: number;
  tint: string;
}

interface ColumnChartProps {
  rows: SlabDatum[];
  format: (value: number) => string;
  className?: string;
}

const DURATION = 650;
const STAGGER = 50;

function ColumnChart({ rows, format, className }: ColumnChartProps) {
  const { ref, revealed } = useRevealOnView<HTMLDivElement>();
  const totalMs = Math.max(0, rows.length - 1) * STAGGER + DURATION;
  const elapsed = useElapsedSinceActive(revealed, totalMs);
  const max = Math.max(1, ...rows.map((row) => row.target));

  return (
    <div ref={ref} className={cn("relative flex items-end gap-2 pt-2", className)} style={{ height: 180 }}>
      <div className="pointer-events-none absolute inset-x-0 top-2 bottom-6.5 flex flex-col justify-between">
        <span className="border-t border-dashed border-border" />
        <span className="border-t border-dashed border-border" />
        <span className="border-t border-dashed border-border" />
      </div>
      {rows.map((row, i) => {
        const p = rowProgress(elapsed, i, STAGGER, DURATION);
        const heightPct = Math.max(2, (row.target / max) * 100) * p;
        return (
          <div key={row.id} className="relative flex h-full flex-1 flex-col items-center justify-end">
            <span className="num mb-1 text-[10.5px] font-bold">{format(row.target * p)}</span>
            <div
              className="min-h-[3px] w-full rounded-t-lg rounded-b-xs transition-[transform,filter] duration-150 hover:-translate-y-0.75 hover:brightness-110"
              style={{
                height: `${heightPct}%`,
                background: `linear-gradient(180deg, color-mix(in oklch, ${row.tint} 55%, var(--surface)), ${row.tint})`,
              }}
            />
            <span className="mt-1.75 text-[10.5px] font-semibold text-muted-text">{row.label}</span>
          </div>
        );
      })}
    </div>
  );
}

export { ColumnChart };
