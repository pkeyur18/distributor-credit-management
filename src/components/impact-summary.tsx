import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

// 07-design-system.md §6.9 — shows what a pending settings change would do
// before it's committed (RQ-18/V7.6). `ImpactValue`'s `changed` flag
// structurally enforces the unchanged-state rule: a caller cannot render
// an identical before/after pair either side of an arrow by accident.

function ImpactSummary({ className, children }: { className?: string; children: ReactNode }) {
  return (
    <div
      data-slot="impact-summary"
      className={cn("divide-y divide-border rounded-sm border border-border", className)}
    >
      {children}
    </div>
  );
}

function ImpactRow({ label, children }: { label: ReactNode; children: ReactNode }) {
  return (
    <div className="flex items-baseline justify-between gap-4 px-3 py-2.25 text-[12.5px]">
      <span className="text-muted-text">{label}</span>
      <span className="num font-[650]">{children}</span>
    </div>
  );
}

interface ImpactValueProps {
  before: ReactNode;
  after: ReactNode;
  /** False renders `<after> unchanged` — never an identical before/after
   * pair either side of an arrow, which would read as a change that isn't one. */
  changed: boolean;
}

function ImpactValue({ before, after, changed }: ImpactValueProps) {
  if (!changed) {
    return (
      <>
        {after} <span className="font-normal text-muted-text">unchanged</span>
      </>
    );
  }
  return (
    <>
      <span className="font-normal text-muted-text">{before}</span>
      <span className="px-[3px] font-normal text-muted-text">→</span>
      {after}
    </>
  );
}

export { ImpactSummary, ImpactRow, ImpactValue };
