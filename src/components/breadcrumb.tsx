import { ChevronLeft } from "lucide-react";
import { Link } from "react-router";

export interface BreadcrumbCrumb {
  label: string;
  /** Omit for the current, non-clickable crumb. */
  to?: string;
  /** Structure's ancestor-trail crumbs re-root via `replace` so the back
   *  target doesn't change as the user moves within the same trail. */
  replace?: boolean;
}

export function Breadcrumb({
  backLabel,
  onBack,
  crumbs,
}: {
  backLabel?: string;
  onBack: () => void;
  crumbs: BreadcrumbCrumb[];
}) {
  if (!backLabel && crumbs.length === 0) return null;

  return (
    <div className="mb-4 flex flex-wrap items-center gap-1.5 text-caption text-muted-text">
      {backLabel && (
        <>
          <button
            type="button"
            onClick={onBack}
            className="inline-flex items-center gap-0.5 hover:text-accent"
          >
            <ChevronLeft className="size-3.5" />
            Back to {backLabel}
          </button>
          {crumbs.length > 0 && <span className="opacity-50">/</span>}
        </>
      )}
      {crumbs.map((c, i) => (
        <span key={i} className="inline-flex items-center gap-1.5">
          {i > 0 && <span className="opacity-50">/</span>}
          {c.to ? (
            <Link to={c.to} replace={c.replace} className="hover:text-accent">
              {c.label}
            </Link>
          ) : (
            <span className="font-semibold text-ink">{c.label}</span>
          )}
        </span>
      ))}
    </div>
  );
}
