import { createBrowserRouter, RouterProvider } from "react-router";
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
import { Setup } from "@/screens/auth/setup";
import { Login } from "@/screens/auth/login";
import { Locked } from "@/screens/auth/locked";
import { Recovery } from "@/screens/auth/recovery";
import { DataRecovery } from "@/screens/auth/data-recovery";

// T-UI.2-3: routing across the nine primary views plus the five auth
// phases (03-functional-specification.md §5.1–5.10). The auth phases render
// standalone — there's nothing to navigate to before signing in, so they
// don't go through AppShell's sidebar/banner.
const router = createBrowserRouter([
  {
    element: <AppShell />,
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
]);

function App() {
  return <RouterProvider router={router} />;
}

export default App;
