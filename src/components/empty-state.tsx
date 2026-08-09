import type { ReactNode } from "react";

interface EmptyStateProps {
  title: string;
  description?: string;
  action?: ReactNode;
}

/**
 * Shared empty-state primitive (T-UI.5-3) — plain, not an error. Used for
 * "no results," "nothing below this member," "no members on this slab," etc.
 */
export function EmptyState({ title, description, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center gap-2 py-12 text-center">
      <p className="text-title-sm">{title}</p>
      {description ? <p className="max-w-sm text-caption">{description}</p> : null}
      {action ? <div className="mt-2">{action}</div> : null}
    </div>
  );
}
