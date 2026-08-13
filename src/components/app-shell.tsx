import { useEffect, useRef } from "react";
import { Outlet, useLocation } from "react-router";
import { Sidebar } from "./sidebar";
import { OutstandingMonthBanner } from "./outstanding-month-banner";
import { OutstandingAlertProvider, useOutstandingAlert } from "@/lib/outstanding-alert-context";
import { useInactivityLock } from "@/lib/use-inactivity-lock";

/**
 * T-UI.2-1/T-UI.2-2 — fixed 236px sidebar + fluid content column, sticky at
 * full viewport height. Wraps every route except the auth phases, which
 * render standalone (there is nothing to navigate to before signing in).
 */
export function AppShell() {
  useInactivityLock();

  return (
    <OutstandingAlertProvider>
      <AppShellLayout />
    </OutstandingAlertProvider>
  );
}

function AppShellLayout() {
  const { alert } = useOutstandingAlert();
  const { pathname } = useLocation();
  const mainRef = useRef<HTMLElement>(null);

  // <main> persists across routes (only <Outlet />'s content swaps), so its
  // scrollTop otherwise carries over from whatever screen was open before —
  // most visible now that each screen's header is sticky and no longer
  // scrolls away to reveal the jump.
  useEffect(() => {
    mainRef.current?.scrollTo(0, 0);
  }, [pathname]);

  return (
    <div className="grid h-screen grid-cols-[236px_1fr]">
      <Sidebar />
      <div className="flex h-screen flex-col overflow-hidden">
        <OutstandingMonthBanner alert={alert} />
        <main ref={mainRef} className="flex-1 overflow-y-auto px-8 pb-10 pt-5">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
