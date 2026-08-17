import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { getOutstandingAlert, type OutstandingAlert } from "@/lib/ipc/m8-auth";

interface OutstandingAlertContextValue {
  alert: OutstandingAlert | null;
  refresh: () => void;
}

const OutstandingAlertContext = createContext<OutstandingAlertContextValue | null>(null);

/**
 * US-M5.2 (API-31) — mounted once inside `AppShell`, so the banner and the
 * notification-list read the same fetch instead of each rolling their own.
 * `refresh()` is what `MonthlyClose` calls after a completed close — the
 * only moment Rule-20 allows the alert to clear (AC-20: never on
 * navigation, logout or a timer).
 */
export function OutstandingAlertProvider({ children }: { children: ReactNode }) {
  const [alert, setAlert] = useState<OutstandingAlert | null>(null);

  function refresh() {
    getOutstandingAlert()
      .then(setAlert)
      .catch(() => setAlert(null));
  }

  useEffect(() => {
    refresh();
  }, []);

  return (
    <OutstandingAlertContext.Provider value={{ alert, refresh }}>
      {children}
    </OutstandingAlertContext.Provider>
  );
}

export function useOutstandingAlert() {
  const ctx = useContext(OutstandingAlertContext);
  if (!ctx) {
    throw new Error("useOutstandingAlert must be used inside OutstandingAlertProvider");
  }
  return ctx;
}
