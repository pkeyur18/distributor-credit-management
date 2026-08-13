import { cn, monthLabel } from "@/lib/utils";

interface MonthSwitcherProps {
  /** Oldest first, matching `PeriodLockStatus.recordablePeriodMonths`. */
  months: string[];
  value: string;
  onChange: (month: string) => void;
  className?: string;
}

// T-M2.5-1: rendered only when there is genuinely a choice to make — with
// a single recordable month (the ordinary case), nothing new appears on
// screen anywhere. This is the client's explicit preference (CR-2), and
// the reference behaviour this mirrors is the prototype's own
// `monthSwitcherHtml()` (documents/design/ui-prototype-v2.html), which
// applies the same guard. A plain native <select>, not a custom dropdown
// component — the same "no Field context, no need for the base-ui
// primitive" reasoning src/components/ui/input.tsx already documents.
export function MonthSwitcher({ months, value, onChange, className }: MonthSwitcherProps) {
  if (months.length < 2) return null;
  return (
    <div
      className={cn(
        "mt-3.5 flex items-center gap-2.5 rounded-lg border border-border bg-surface px-3.5 py-3",
        className,
      )}
    >
      <span className="text-caption text-muted-text">Showing figures for</span>
      <select
        className="h-7.5 w-auto rounded-sm border border-border bg-surface px-2 text-body text-ink outline-none focus:border-accent focus:ring-3 focus:ring-accent-weak"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        {months.map((m) => (
          <option key={m} value={m}>
            {monthLabel(m)}
          </option>
        ))}
      </select>
      <span className="text-caption text-muted-text">
        {months.length} months are open for entry until the oldest is closed.
      </span>
    </div>
  );
}
