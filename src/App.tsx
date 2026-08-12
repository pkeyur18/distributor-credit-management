import type { ReactNode } from "react";
import { createBrowserRouter, Navigate, RouterProvider } from "react-router";
import { AppShell } from "@/components/app-shell";
import { Home } from "@/screens/home";
import { MemberDetail } from "@/screens/member-detail";
import { Structure } from "@/screens/structure";
import { BusinessVolumeEntry } from "@/screens/business-volume-entry";
import { CorrectionPanel } from "@/screens/correction-panel";
import { MonthlyClose } from "@/screens/monthly-close";
import { Settings } from "@/screens/settings";
import { Reports } from "@/screens/reports";
import { Audit } from "@/screens/audit";
import { FullHierarchy } from "@/windows/full-hierarchy";
import { Setup } from "@/screens/auth/setup";
import { Login } from "@/screens/auth/login";
import { Locked } from "@/screens/auth/locked";
import { Recovery } from "@/screens/auth/recovery";
import { DataRecovery } from "@/screens/auth/data-recovery";
import { AuthProvider, useAuth } from "@/lib/auth-context";
import { ToastProvider, Toaster } from "@/components/ui/toast";

// US-M8.1/M8.2 (S5): the frontend's only signal for "is someone signed in"
// — gates the AppShell route group behind `checkDataReadable` + a
// successful setup/login. `loading` renders nothing rather than flashing
// Home before the one check resolves.
function RequireAuth({ children }: { children: ReactNode }) {
  const { state } = useAuth();
  if (state === "loading") return null;
  if (state === "needs-setup") return <Navigate to="/auth/setup" replace />;
  if (state === "needs-login") return <Navigate to="/auth/login" replace />;
  if (state === "locked") return <Navigate to="/auth/locked" replace />;
  return <>{children}</>;
}

// T-UI.2-3: routing across the nine primary views plus the five auth
// phases (03-functional-specification.md §5.1–5.10). The auth phases render
// standalone — there's nothing to navigate to before signing in, so they
// don't go through AppShell's sidebar/banner.
const router = createBrowserRouter([
  {
    element: (
      <RequireAuth>
        <AppShell />
      </RequireAuth>
    ),
    children: [
      { path: "/", element: <Home /> },
      { path: "/member/:memberId", element: <MemberDetail /> },
      { path: "/structure", element: <Structure /> },
      { path: "/structure/:memberId", element: <Structure /> },
      { path: "/entry", element: <BusinessVolumeEntry /> },
      { path: "/entry/correct", element: <CorrectionPanel /> },
      { path: "/close", element: <MonthlyClose /> },
      { path: "/settings", element: <Settings /> },
      { path: "/reports", element: <Reports /> },
      { path: "/audit", element: <Audit /> },
    ],
  },
  { path: "/auth/setup", element: <Setup /> },
  { path: "/auth/login", element: <Login /> },
  { path: "/auth/locked", element: <Locked /> },
  { path: "/auth/recovery", element: <Recovery /> },
  { path: "/auth/data-recovery", element: <DataRecovery /> },
  // US-M4.3 (§5.3a). Own top-level route, deliberately outside RequireAuth
  // and AppShell — opened only in a separate `full-hierarchy-*` Tauri
  // window (see structure.tsx), never navigated to inside the main window.
  { path: "/full-hierarchy", element: <FullHierarchy /> },
  // Sprint 3 (US-UI.3/US-UI.4) DoD item 13 verification aid — never linked
  // from the sidebar, stripped from production builds.
  ...(import.meta.env.DEV
    ? [
        {
          path: "/dev/components",
          lazy: async () => {
            const { ComponentGallery } = await import("./dev/component-gallery");
            return { Component: ComponentGallery };
          },
        },
      ]
    : []),
]);

function App() {
  return (
    <AuthProvider>
      <ToastProvider>
        <RouterProvider router={router} />
        <Toaster />
      </ToastProvider>
    </AuthProvider>
  );
}

export default App;
