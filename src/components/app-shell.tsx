import { Outlet } from "react-router";
import { Sidebar } from "./sidebar";
import { OutstandingMonthBanner } from "./outstanding-month-banner";
import { NotificationList } from "./notification-list";
import { useOutstandingAlert } from "@/lib/use-outstanding-alert";

/**
 * T-UI.2-1/T-UI.2-2 — fixed 236px sidebar + fluid content column, sticky at
 * full viewport height. Wraps every route except the auth phases, which
 * render standalone (there is nothing to navigate to before signing in).
 */
export function AppShell() {
  const alert = useOutstandingAlert();

  return (
    <div className="grid h-screen grid-cols-[236px_1fr]">
      <Sidebar />
      <div className="flex h-screen flex-col overflow-hidden">
        <OutstandingMonthBanner alert={alert} />
        <header className="sticky top-0 z-10 flex h-14 shrink-0 items-center justify-end border-b border-border bg-surface px-8">
          <NotificationList alert={alert} />
        </header>
        <main className="flex-1 overflow-y-auto px-8 pb-10 pt-5">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
