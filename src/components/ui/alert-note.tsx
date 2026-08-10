import type { ReactNode } from "react";
import { AlertCircle, AlertTriangle } from "lucide-react";

import { cn } from "@/lib/utils";

// 07-design-system.md §6.8 — the Blended Alert Border Rule: never a solid
// status-coloured border, always 35% status colour mixed into the neutral
// border token.
const VARIANT = {
  warn: {
    Icon: AlertTriangle,
    className: "bg-warning-weak",
    borderColor: "color-mix(in srgb, var(--warning) 35%, var(--border))",
    iconColor: "text-warning",
    // The amber fill measures ≈3.2:1 on white — copy uses the darker
    // warning-text step, which is the one place it applies (§1).
    textColor: "text-warning-text",
  },
  danger: {
    Icon: AlertCircle,
    className: "bg-danger-weak",
    borderColor: "color-mix(in srgb, var(--danger) 35%, var(--border))",
    iconColor: "text-danger",
    textColor: "text-danger",
  },
} as const;

interface AlertNoteProps {
  variant: "warn" | "danger";
  className?: string;
  children: ReactNode;
}

function AlertNote({ variant, className, children }: AlertNoteProps) {
  const v = VARIANT[variant];
  return (
    <div
      data-slot="alert-note"
      className={cn("rounded-sm border px-3 py-2.5 text-[12.5px]", v.className, className)}
      style={{ borderColor: v.borderColor }}
    >
      <div className="flex items-start gap-2.25">
        <v.Icon aria-hidden="true" className={cn("mt-px size-3.75 shrink-0", v.iconColor)} />
        <div className={v.textColor}>{children}</div>
      </div>
    </div>
  );
}

export { AlertNote };
