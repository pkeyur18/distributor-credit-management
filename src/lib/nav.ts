import type { LucideIcon } from "lucide-react";
import { Home, GitBranch, PlusCircle, Settings, FileBarChart, History } from "lucide-react";

export interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
}

// 07-design-system.md §6.5 / 03-functional-specification.md §5.11 — the six
// sidebar entries. Member Detail, Correction Panel and Monthly Close are
// reached from elsewhere (search results, BV Entry, the outstanding banner),
// not from the sidebar directly.
export const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "Home", icon: Home },
  { to: "/structure", label: "Structure", icon: GitBranch },
  { to: "/entry", label: "Business Volume Entry", icon: PlusCircle },
  { to: "/settings", label: "Settings", icon: Settings },
  { to: "/reports", label: "Reports", icon: FileBarChart },
  { to: "/audit", label: "Audit", icon: History },
];
