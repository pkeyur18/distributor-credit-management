import type { ReactNode } from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

// 07-design-system.md §6.2 — the Colour-Plus-Label Rule: the dot reinforces
// colour, it never substitutes for the word. `children` is required, not
// optional, so a pill can't be rendered dot-only by accident.
const pillVariants = cva(
  "inline-flex h-[21px] items-center gap-[5px] rounded-full px-[9px] text-[11px] font-[650] whitespace-nowrap",
  {
    variants: {
      variant: {
        active: "bg-success-weak text-success",
        inactive: "bg-danger-weak text-danger",
        slab: "bg-accent-weak text-accent",
        locked: "bg-warning-weak text-warning",
        neutral: "border border-border bg-bg text-muted-text",
      },
    },
    defaultVariants: {
      variant: "neutral",
    },
  },
);

// Slab/Neutral carry no implied state, so they never show the status dot.
const DOTLESS_VARIANTS = new Set(["slab", "neutral"]);

type PillVariant = NonNullable<VariantProps<typeof pillVariants>["variant"]>;

const dotColor: Record<PillVariant, string> = {
  active: "bg-success",
  inactive: "bg-danger",
  slab: "",
  locked: "bg-warning",
  neutral: "",
};

interface PillProps {
  variant?: PillVariant;
  className?: string;
  children: ReactNode;
}

function Pill({ className, variant = "neutral", children }: PillProps) {
  return (
    <span data-slot="pill" className={cn(pillVariants({ variant, className }))}>
      {!DOTLESS_VARIANTS.has(variant) && (
        <span aria-hidden="true" className={cn("size-1.5 rounded-full", dotColor[variant])} />
      )}
      {children}
    </span>
  );
}

export { Pill, pillVariants };
