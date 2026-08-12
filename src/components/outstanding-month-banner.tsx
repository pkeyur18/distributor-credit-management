import { Link } from "react-router";
import { AlertTriangle } from "lucide-react";
import type { OutstandingAlert } from "@/lib/ipc/m8-auth";

interface OutstandingMonthBannerProps {
  alert: OutstandingAlert | null;
}

/**
 * T-UI.2-4 — present on every screen once any month has ended without being
 * closed. Rule-20: no dismiss control of any kind, not even a disguised one
 * — no close icon, no auto-hide, no acknowledge action. Clears only when
 * `alert` itself goes null, which only happens on a completed close.
 */
export function OutstandingMonthBanner({ alert }: OutstandingMonthBannerProps) {
  if (!alert || alert.outstandingMonths.length === 0) return null;

  const oldest = alert.outstandingMonths[0];
  const moreCount = alert.outstandingMonths.length - 1;
  const moreClause =
    moreCount > 0
      ? ` ${moreCount} more month${moreCount > 1 ? "s are" : " is"} outstanding after that.`
      : "";

  return (
    <div
      role="status"
      className="flex items-center gap-2.5 border-b px-8 py-2.5 text-[12.5px]"
      style={{
        backgroundColor: "var(--warning-weak)",
        borderColor: "color-mix(in srgb, var(--warning) 35%, var(--border))",
      }}
    >
      <AlertTriangle
        className="h-[15px] w-[15px] shrink-0"
        style={{ color: "var(--warning-text)" }}
      />
      <p style={{ color: "var(--warning-text)" }}>
        <span className="font-semibold">{oldest} has ended and is awaiting close.</span>
        {moreClause} You can still record entries dated in {oldest}. {alert.currentMonth} entries
        unlock once {oldest} is closed.
      </p>
      <Link
        to="/close"
        className="ml-auto shrink-0 rounded-sm border border-border bg-surface px-3 py-1 text-[13px] font-medium text-ink hover:bg-bg"
      >
        Close {oldest}
      </Link>
    </div>
  );
}
