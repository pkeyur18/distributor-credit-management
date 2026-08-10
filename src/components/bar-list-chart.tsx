import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

// 07-design-system.md §5 Half-Height-Bar Rule + §Reused-not-duplicated
// (8 Aug 2026) — built once, reused verbatim by "Members by slab" and
// "Rewards by slab" (CR-5): same track/fill/label shapes, the consuming
// screen only changes what's summed. `fraction` (0–1, relative to the
// largest row) and any per-row tint are the caller's data concern, not
// this component's.
interface BarListRow {
  id: string | number;
  label: ReactNode;
  value: ReactNode;
  fraction: number;
  tint?: string;
}

interface BarListChartProps {
  rows: BarListRow[];
  size?: "default" | "lg";
  className?: string;
}

function BarListChart({ rows, size = "default", className }: BarListChartProps) {
  const lg = size === "lg";
  return (
    <div className={cn("flex flex-col gap-2.25", className)}>
      {rows.map((row) => (
        <div
          key={row.id}
          className="grid items-center"
          style={{
            gridTemplateColumns: lg ? "minmax(60px, auto) 1fr 90px" : "46px 1fr 34px",
            gap: lg ? 14 : 10,
          }}
        >
          <span
            className={cn(
              "truncate",
              lg ? "text-[13.5px] font-[650] text-ink" : "text-xs font-semibold text-muted-text",
            )}
          >
            {row.label}
          </span>
          <span
            className={cn(
              "overflow-hidden border border-border bg-bg",
              lg ? "h-4 rounded-lg" : "h-2.25 rounded-[5px]",
            )}
          >
            <span
              className={cn("block h-full origin-left transition-transform duration-300", lg ? "rounded-lg" : "rounded-[5px]")}
              style={{
                transform: `scaleX(${Math.max(0, Math.min(1, row.fraction))})`,
                background: row.tint ?? "var(--accent)",
              }}
            />
          </span>
          <span
            className={cn(
              "num text-right",
              lg ? "text-[13.5px] font-[650] text-ink" : "text-xs text-muted-text",
            )}
          >
            {row.value}
          </span>
        </div>
      ))}
    </div>
  );
}

export { BarListChart };
export type { BarListRow };
