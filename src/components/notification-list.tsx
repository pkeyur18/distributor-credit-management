import { useState } from "react";
import { Bell } from "lucide-react";
import type { OutstandingAlert } from "@/lib/ipc/m8-auth";

interface NotificationListProps {
  alert: OutstandingAlert | null;
}

/**
 * T-UI.2-5 — mirrors the outstanding-month alert as a persistent list entry,
 * not a dismissable toast (Rule-20). Entries have no dismiss/acknowledge
 * action; the list reflects live state, it doesn't accumulate history.
 */
export function NotificationList({ alert }: NotificationListProps) {
  const [open, setOpen] = useState(false);
  const entries = alert?.outstandingMonths ?? [];

  return (
    <div className="relative">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-label={`Notifications${entries.length > 0 ? ` (${entries.length})` : ""}`}
        aria-expanded={open}
        className="relative flex h-8 w-8 items-center justify-center rounded-sm text-ink hover:bg-bg"
      >
        <Bell className="h-4 w-4 opacity-75" />
        {entries.length > 0 ? (
          <span
            className="num text-caption absolute -right-1 -top-1 flex h-[18px] min-w-[18px] items-center justify-center rounded-full px-1 font-semibold text-white"
            style={{ backgroundColor: "var(--warning)" }}
          >
            {entries.length}
          </span>
        ) : null}
      </button>

      {open ? (
        <div
          className="absolute right-0 z-10 mt-1 w-72 rounded-lg border border-border bg-surface p-2"
          style={{ boxShadow: "var(--shadow-modal)" }}
        >
          {entries.length === 0 ? (
            <p className="p-2 text-caption">Nothing outstanding.</p>
          ) : (
            <ul className="flex flex-col gap-1">
              {entries.map((month) => (
                <li
                  key={month}
                  className="rounded-sm p-2 text-[12.5px]"
                  style={{ color: "var(--warning-text)" }}
                >
                  {month} has ended and is awaiting close.
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : null}
    </div>
  );
}
