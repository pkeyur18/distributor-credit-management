import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

// 07-design-system.md §6.3. Flat by default (no shadow) — see the
// Flat-By-Default Rule. `variant="stat"` is the tighter 14/16px stat-card
// padding; everything else uses the standard 18px.
function Card({
  className,
  variant = "default",
  ...props
}: ComponentProps<"div"> & { variant?: "default" | "stat" }) {
  return (
    <div
      data-slot="card"
      className={cn(
        "rounded-lg border border-border bg-surface",
        variant === "stat" ? "px-4 py-3.5" : "p-[18px]",
        className,
      )}
      {...props}
    />
  );
}

function CardHeader({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      data-slot="card-header"
      className={cn("mb-3.5 flex items-center justify-between gap-3", className)}
      {...props}
    />
  );
}

function CardTitle({ className, ...props }: ComponentProps<"h3">) {
  return <h3 data-slot="card-title" className={cn("text-title-sm mb-0.5", className)} {...props} />;
}

function CardDescription({ className, ...props }: ComponentProps<"p">) {
  return <p data-slot="card-description" className={cn("text-caption", className)} {...props} />;
}

export { Card, CardHeader, CardTitle, CardDescription };
