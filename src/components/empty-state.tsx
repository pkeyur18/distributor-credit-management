import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  descriptionClassName?: string;
  action?: ReactNode;
}

/**
 * Shared empty-state primitive (T-UI.5-3) — plain, not an error. Used for
 * "no results," "nothing below this member," "no members on this slab," etc.
 */
export function EmptyState({ icon, title, description, descriptionClassName, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center gap-2 py-12 text-center">
      {icon ? <span className="text-muted-text opacity-50">{icon}</span> : null}
      <p className="text-title-sm">{title}</p>
      {description ? (
        <p className={cn("max-w-sm text-caption", descriptionClassName)}>{description}</p>
      ) : null}
      {action ? <div className="mt-2">{action}</div> : null}
    </div>
  );
}
