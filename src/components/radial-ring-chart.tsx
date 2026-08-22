import { cn } from "@/lib/utils";
import { rowProgress, useElapsedSinceActive, useRevealOnView } from "@/lib/use-chart-reveal";
import type { SlabDatum } from "./column-chart";

// Slab-chart redesign, option 4 (client-approved mockup) — used for "Rewards
// by slab". Arcs sweep in from zero length (fixed at their final rotational
// offset) and the center total + legend values count up alongside, staggered
// per slab, the first time the chart scrolls into view.
interface RadialRingChartProps {
  rows: SlabDatum[];
  format: (value: number) => string;
  totalLabel: string;
  className?: string;
}

const SIZE = 148;
const STROKE = 20;
const DURATION = 700;
const STAGGER = 70;

function RadialRingChart({ rows, format, totalLabel, className }: RadialRingChartProps) {
  const { ref, revealed } = useRevealOnView<HTMLDivElement>();
  const totalMs = Math.max(0, rows.length - 1) * STAGGER + DURATION;
  const elapsed = useElapsedSinceActive(revealed, totalMs);

  const r = SIZE / 2 - STROKE / 2;
  const circumference = 2 * Math.PI * r;
  const total = Math.max(1, rows.reduce((sum, row) => sum + row.target, 0));

  const offsets = rows.map((_, i) =>
    rows.slice(0, i).reduce((sum, row) => sum + (row.target / total) * circumference, 0),
  );
  const totalProgress = rowProgress(elapsed, 0, STAGGER, DURATION);

  return (
    <div ref={ref} className={cn("flex items-center gap-5.5", className)}>
      <svg width={SIZE} height={SIZE} viewBox={`0 0 ${SIZE} ${SIZE}`} className="shrink-0">
        <circle cx={SIZE / 2} cy={SIZE / 2} r={r} fill="none" stroke="var(--border)" strokeWidth={STROKE} />
        {rows.map((row, i) => {
          const p = rowProgress(elapsed, i, STAGGER, DURATION);
          const share = row.target / total;
          const dash = share * circumference * p;
          return (
            <circle
              key={row.id}
              cx={SIZE / 2}
              cy={SIZE / 2}
              r={r}
              fill="none"
              stroke={row.tint}
              strokeWidth={STROKE}
              strokeDasharray={`${dash} ${circumference - dash}`}
              strokeDashoffset={-offsets[i]}
              transform={`rotate(-90 ${SIZE / 2} ${SIZE / 2})`}
              className="transition-[filter] duration-150 hover:brightness-110"
            />
          );
        })}
        <text x="50%" y="46%" textAnchor="middle" className="text-numeric fill-ink">
          {format(total * totalProgress)}
        </text>
        <text x="50%" y="60%" textAnchor="middle" className="text-label fill-muted-text">
          {totalLabel}
        </text>
      </svg>
      <div className="flex min-w-0 flex-1 flex-col gap-1.75">
        {rows.map((row, i) => {
          const p = rowProgress(elapsed, i, STAGGER, DURATION);
          return (
            <div key={row.id} className="grid grid-cols-[9px_30px_1fr] items-center gap-1.75 text-xs">
              <span className="size-2.25 rounded-full" style={{ background: row.tint }} />
              <span className="font-[650]">{row.label}</span>
              <span className="num text-right text-muted-text">{format(row.target * p)}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export { RadialRingChart };
