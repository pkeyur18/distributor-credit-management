import type { ComponentProps, ReactNode } from "react";
import { ChevronRight, Search } from "lucide-react";

import { cn } from "@/lib/utils";
import { Pill } from "@/components/ui/pill";

// 07-design-system.md §6.12 — the signature component. Exactly three
// member-data fields: name, member number, own Business Volume — never
// Total Business Volume. Reused verbatim by the Full Hierarchy Window
// (§6.13), where `interactive={false}` strips every affordance (hover
// lift, expand chevron, the detail-view button) — that window is
// read-only by design, not merely by convention.
interface StructureTreeNodeProps {
  name: string;
  memberNumber: string;
  ownBusinessVolume: string;
  root?: boolean;
  inactive?: boolean;
  /** Structure screen (true, default) vs Full Hierarchy Window (false). */
  interactive?: boolean;
  legCount?: number;
  open?: boolean;
  onOpenToggle?: () => void;
  onViewDetail?: () => void;
}

function StructureTreeNode({
  name,
  memberNumber,
  ownBusinessVolume,
  root,
  inactive,
  interactive = true,
  legCount = 0,
  open,
  onOpenToggle,
  onViewDetail,
}: StructureTreeNodeProps) {
  const isLeaf = legCount === 0;
  const canToggle = interactive && !isLeaf;

  return (
    <div
      data-slot="structure-tree-node"
      role={canToggle ? "button" : undefined}
      tabIndex={canToggle ? 0 : undefined}
      aria-label={canToggle ? `${open ? "Close" : "Open"} ${name}'s branch` : undefined}
      aria-expanded={canToggle ? open : undefined}
      onClick={canToggle ? onOpenToggle : undefined}
      onKeyDown={
        canToggle
          ? (e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onOpenToggle?.();
              }
            }
          : undefined
      }
      className={cn(
        "rounded-lg border-[1.5px] bg-surface px-3.25 py-3 transition-[border-color,transform] duration-100",
        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
        root ? "w-47.5 border-accent bg-accent-weak" : "w-43 border-border",
        canToggle
          ? "cursor-pointer hover:-translate-y-px hover:border-accent"
          : interactive
            ? "cursor-default"
            : undefined,
        interactive && open && "border-accent",
      )}
    >
      <div className="flex items-start justify-between">
        <div>
          <div className="text-title-sm mb-0.5">
            {name}
            {inactive && (
              <Pill variant="inactive" className="ml-1">
                Inactive
              </Pill>
            )}
          </div>
          <div className="mono text-[11px] text-muted-text">#{memberNumber}</div>
        </div>
        {interactive && (
          <button
            type="button"
            aria-label={`View ${name}'s member detail`}
            onClick={(e) => {
              e.stopPropagation();
              onViewDetail?.();
            }}
            className="flex size-6 shrink-0 items-center justify-center rounded-sm text-muted-text hover:bg-bg hover:text-ink"
          >
            <Search className="size-3" />
          </button>
        )}
      </div>

      <div className="mt-2.25 flex items-baseline justify-between border-t border-border pt-2">
        <span className="text-[10.5px] tracking-[0.04em] text-muted-text uppercase">
          Business Volume
        </span>
        <span className="num text-sm font-[650]">{ownBusinessVolume}</span>
      </div>

      {interactive && (
        <div className="mt-2 flex items-center justify-between">
          <span
            className={cn(
              "inline-flex items-center gap-1.25 text-[11.5px] font-[650]",
              isLeaf ? "font-normal text-muted-text" : "text-accent",
            )}
          >
            {!isLeaf && (
              <ChevronRight className={cn("size-2.75 transition-transform", open && "rotate-90")} />
            )}
            {isLeaf ? "No legs beneath" : `${legCount} direct leg${legCount === 1 ? "" : "s"}`}
          </span>
        </div>
      )}
    </div>
  );
}

// The connector-styling rule (§6.12), factored out so no screen can draw a
// tree connector in the accent colour or thicker than the node borders it
// connects — the diagram's data must always outweigh its scaffolding.
// Geometry (x1/y1/x2/y2) is the consuming screen's layout pass, not this
// component's concern.
function TreeConnectorLayer({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <svg
      aria-hidden="true"
      className={cn("pointer-events-none absolute inset-0 size-full", className)}
    >
      {children}
    </svg>
  );
}

function TreeConnectorLine(props: ComponentProps<"line">) {
  return <line stroke="var(--border)" strokeWidth={1.5} {...props} />;
}

export { StructureTreeNode, TreeConnectorLayer, TreeConnectorLine };
