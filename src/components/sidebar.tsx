import { Fragment } from "react";
import { NavLink, useNavigate } from "react-router";
import { Moon, Sun, Monitor, Lock, LogOut } from "lucide-react";
import { NAV_ITEMS } from "@/lib/nav";
import { useTheme } from "@/lib/use-theme";
import { useAuth } from "@/lib/auth-context";
import { lockSession } from "@/lib/ipc/m8-auth";

const THEME_CYCLE = { light: "dark", dark: "system", system: "light" } as const;
const THEME_ICON = { light: Sun, dark: Moon, system: Monitor } as const;

// US-M8.3 (S7): both buttons call the same `lock_session` command — there
// is no separate "log out" primitive in the backend, only locked vs.
// authenticated (`session.rs`). They differ only in which screen the
// frontend routes to afterwards: Lock offers the PIN/password "resume"
// screen, Sign out goes straight back to a full Login.
export function Sidebar() {
  const [theme, setTheme] = useTheme();
  const ThemeIcon = THEME_ICON[theme];
  const { markLocked, markSignedOut } = useAuth();
  const navigate = useNavigate();

  async function handleLock() {
    await lockSession();
    markLocked();
    navigate("/auth/locked", { replace: true });
  }

  async function handleSignOut() {
    await lockSession();
    markSignedOut();
    navigate("/auth/login", { replace: true });
  }

  return (
    <aside className="flex h-full w-[236px] flex-col border-r border-border bg-surface">
      <div className="px-4 py-5">
        <span className="text-title-sm">Member Rewards Console</span>
      </div>

      <nav className="flex flex-1 flex-col gap-0.5 px-2">
        {NAV_ITEMS.map(({ to, label, icon: Icon, group }) => (
          <Fragment key={to}>
            {group ? (
              <div className="px-2.5 pb-1 pt-3.5 text-[10.5px] font-semibold tracking-[0.06em] text-muted-text uppercase">
                {group}
              </div>
            ) : null}
            <NavLink
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                `flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-[13.5px] transition-[background,color] duration-100 ${
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
          </Fragment>
        ))}
      </nav>

      <div className="flex flex-col gap-0.5 border-t border-border px-2 py-2">
        <button
          type="button"
          onClick={() => setTheme(THEME_CYCLE[theme])}
          className="flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-[13.5px] text-ink transition-[background] duration-100 hover:bg-bg"
          aria-label={`Theme: ${theme}. Click to change.`}
        >
          <ThemeIcon className="h-4 w-4 opacity-75" />
          Theme: {theme}
        </button>
        <button
          type="button"
          onClick={handleLock}
          className="flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-[13.5px] text-ink transition-[background] duration-100 hover:bg-bg"
        >
          <Lock className="h-4 w-4 opacity-75" />
          Lock session
        </button>
        <button
          type="button"
          onClick={handleSignOut}
          className="flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-[13.5px] text-ink transition-[background] duration-100 hover:bg-bg"
        >
          <LogOut className="h-4 w-4 opacity-75" />
          Sign out
        </button>
      </div>
    </aside>
  );
}
