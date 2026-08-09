import { NavLink } from "react-router";
import { Moon, Sun, Monitor, Lock, LogOut } from "lucide-react";
import { NAV_ITEMS } from "@/lib/nav";
import { useTheme } from "@/lib/use-theme";

const THEME_CYCLE = { light: "dark", dark: "system", system: "light" } as const;
const THEME_ICON = { light: Sun, dark: Moon, system: Monitor } as const;

export function Sidebar() {
  const [theme, setTheme] = useTheme();
  const ThemeIcon = THEME_ICON[theme];

  return (
    <aside className="flex h-full w-[236px] flex-col border-r border-border bg-surface">
      <div className="px-4 py-5">
        <span className="text-title-sm">Business Volume Console</span>
      </div>

      <nav className="flex flex-1 flex-col gap-0.5 px-2">
        {NAV_ITEMS.map(({ to, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              `flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-[13.5px] ${
                isActive ? "bg-accent-weak font-semibold text-accent" : "text-ink hover:bg-bg"
              }`
            }
          >
            {({ isActive }) => (
              <>
                <Icon className="h-4 w-4" style={{ opacity: isActive ? 1 : 0.75 }} />
                {label}
              </>
            )}
          </NavLink>
        ))}
      </nav>

      <div className="flex flex-col gap-0.5 border-t border-border px-2 py-2">
        <button
          type="button"
          onClick={() => setTheme(THEME_CYCLE[theme])}
          className="flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-[13.5px] text-ink hover:bg-bg"
          aria-label={`Theme: ${theme}. Click to change.`}
        >
          <ThemeIcon className="h-4 w-4 opacity-75" />
          Theme: {theme}
        </button>
        <button
          type="button"
          className="flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-[13.5px] text-ink hover:bg-bg"
        >
          <Lock className="h-4 w-4 opacity-75" />
          Lock session
        </button>
        <button
          type="button"
          className="flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-[13.5px] text-ink hover:bg-bg"
        >
          <LogOut className="h-4 w-4 opacity-75" />
          Sign out
        </button>
      </div>
    </aside>
  );
}
