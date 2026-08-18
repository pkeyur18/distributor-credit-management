import { Link } from "react-router";
import { AlertTriangle } from "lucide-react";
import type { OutstandingAlert } from "@/lib/ipc/m8-auth";
import { buttonVariants } from "@/components/ui/button";
import { cn, monthLabel } from "@/lib/utils";

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
  const oldestLabel = monthLabel(oldest);
  const moreCount = alert.outstandingMonths.length - 1;
  const moreClause =
    moreCount > 0
      ? ` ${moreCount} more month${moreCount > 1 ? "s are" : " is"} outstanding after that.`
      : "";

  return (
    <div className="px-8 pt-5">
      <div
        role="status"
        className="flex items-center justify-between gap-4 rounded-lg border px-4 py-2.75"
        style={{
          backgroundColor: "var(--warning-weak)",
          borderColor: "color-mix(in srgb, var(--warning) 35%, var(--border))",
        }}
      >
        <div className="flex items-center gap-2.5 text-[13px]" style={{ color: "var(--ink)" }}>
          <AlertTriangle
            className="h-4.25 w-4.25 shrink-0"
            style={{ color: "var(--warning)" }}
          />
          <span>
            <span className="font-[650]">{oldestLabel} has ended and is awaiting close.</span>
            {moreClause} You can still record entries dated in {oldestLabel}.{" "}
            {monthLabel(alert.currentMonth)} entries unlock once {oldestLabel} is closed.
          </span>
        </div>
        <Link
          to="/close"
          state={{ autoStart: true }}
          className={cn(buttonVariants({ variant: "primary", size: "sm" }), "shrink-0")}
        >
          Close {oldestLabel}
        </Link>
      </div>
    </div>
  );
}
