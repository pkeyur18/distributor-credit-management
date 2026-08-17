import type { LucideIcon } from "lucide-react";
import {
  Home,
  GitBranch,
  PlusCircle,
  CalendarCheck,
  Settings,
  FileBarChart,
  History,
} from "lucide-react";

export interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
  group?: string;
}

// documents/design/ui-prototype-v2.html §SHELL NAV_ITEMS — sidebar entries
// grouped into an ungrouped top section, "Period", and "Admin". Member
// Detail and Correction Panel are reached from elsewhere (search results,
// BV Entry), not from the sidebar directly.
export const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "Home", icon: Home },
  { to: "/structure", label: "Structure", icon: GitBranch },
  { to: "/entry", label: "Volume Entry", icon: PlusCircle },
  { to: "/close", label: "Monthly Close", icon: CalendarCheck, group: "Period" },
  { to: "/reports", label: "Reports", icon: FileBarChart },
  { to: "/settings", label: "Settings", icon: Settings, group: "Admin" },
  { to: "/audit", label: "Audit", icon: History },
];
