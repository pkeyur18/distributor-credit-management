import type { ReactNode } from "react";

// Pins title/subtitle/actions to the top of the scrolling `<main>` (T-UI.2-1's
// scroll container in app-shell.tsx) — mirrors the prototype's sticky
// `.topbar`. The -mx-8/-mt-5 + px-8/pt-5 pair cancels then re-applies main's
// own padding so the header still sits flush at rest, but its bg-background
// fully covers scrolled content once stuck. `-top-5` matches main's pt-5:
// sticky offsets are measured from the scroll container's padding edge (i.e.
// below its own pt-5), so top-0 alone leaves a 20px gap once stuck where
// scrolled content peeks through above the header — -top-5 cancels it.
// mb-6 collapses against each screen's own mt-* on its first child
// (07-design-system.md §3: section breaks at 18/24/32px) — one consistent
// 24px gap, no per-screen edits.
export function PageHeader({
  title,
  subtitle,
  actions,
  breadcrumb,
}: {
  title: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  breadcrumb?: ReactNode;
}) {
  return (
    <div className="sticky -top-5 z-10 -mx-8 -mt-5 mb-10 bg-background px-8 pb-2 pt-8">
      {breadcrumb}
      <div className="flex items-center justify-between">
        <h1 className="text-headline">{title}</h1>
        {actions}
      </div>
      {subtitle && <p className="text-caption mt-1">{subtitle}</p>}
    </div>
  );
}
