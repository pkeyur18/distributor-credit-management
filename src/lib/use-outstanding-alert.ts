import { useState } from "react";
import type { OutstandingAlert } from "@/lib/ipc/m8-auth";

/**
 * Stub until M8's get_outstanding_alert (API-31) lands, S5 — mirrors
 * db::open_encrypted's own layering in Sprint 1: the shell that consumes
 * this data is built ahead of the command that supplies it. Always null for
 * now, so the banner/notification-list correctly render nothing.
 */
export function useOutstandingAlert(): OutstandingAlert | null {
  const [alert] = useState<OutstandingAlert | null>(null);
  return alert;
}
