import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useLocation, useNavigate, useNavigationType } from "react-router";

interface StackEntry {
  path: string;
  label: string;
}

interface NavigationHistoryContextValue {
  registerLabel: (path: string, label: string) => void;
  stack: StackEntry[];
}

const NavigationHistoryContext = createContext<NavigationHistoryContextValue | null>(null);

// Mirrors the prototype's ROUTE_LABELS (ui-prototype-v2.html:1309) — the
// fallback for any screen that never calls useRouteLabel itself.
const STATIC_ROUTE_LABELS: Record<string, string> = {
  "/": "Home",
  "/structure": "Structure",
  "/entry": "Volume entry",
  "/entry/correct": "Volume entry",
  "/close": "Monthly close",
  "/reports": "Reports",
  "/audit": "Audit log",
  "/settings": "Settings",
};

function fallbackLabel(path: string): string {
  return STATIC_ROUTE_LABELS[path] ?? "Home";
}

/**
 * Observes every route transition and keeps a small stack of {path, label}
 * for whatever screen was left behind on each PUSH — this is what lets
 * `useBackTarget()` on the *next* screen say "Back to <real previous
 * screen>" without that screen having to pass anything explicitly. REPLACE
 * (Structure's ancestor re-root) leaves the stack untouched, matching the
 * prototype's own `{replace:true}` behaviour. POP (the back link itself,
 * or a real browser-back) consumes one level.
 */
export function NavigationHistoryProvider({ children }: { children: ReactNode }) {
  const location = useLocation();
  const navigationType = useNavigationType();
  const [stack, setStack] = useState<StackEntry[]>([]);
  const labelsRef = useRef<Map<string, string>>(new Map());
  const prevPathRef = useRef<string | null>(null);

  useEffect(() => {
    const path = location.pathname;
    const prevPath = prevPathRef.current;
    if (prevPath !== null && prevPath !== path) {
      if (navigationType === "PUSH") {
        const label = labelsRef.current.get(prevPath) ?? fallbackLabel(prevPath);
        setStack((s) => [...s, { path: prevPath, label }]);
      } else if (navigationType === "POP") {
        setStack((s) => s.slice(0, -1));
      }
      // REPLACE: stack stays as-is.
    }
    prevPathRef.current = path;
  }, [location.pathname, navigationType]);

  function registerLabel(path: string, label: string) {
    labelsRef.current.set(path, label);
  }

  return (
    <NavigationHistoryContext.Provider value={{ registerLabel, stack }}>
      {children}
    </NavigationHistoryContext.Provider>
  );
}

function useNavigationHistory(): NavigationHistoryContextValue {
  const ctx = useContext(NavigationHistoryContext);
  if (!ctx) {
    throw new Error("useNavigationHistory must be used within a NavigationHistoryProvider");
  }
  return ctx;
}

/** Screens call this with their own current display identity (a member's
 *  name, "Structure (<root name>)", a static string) so that whatever
 *  screen the user navigates to next can show an accurate "Back to X".
 *  `undefined` (still loading) leaves whatever was registered before. */
export function useRouteLabel(label: string | undefined): void {
  const { registerLabel } = useNavigationHistory();
  const location = useLocation();
  useEffect(() => {
    if (label) registerLabel(location.pathname, label);
  }, [label, location.pathname, registerLabel]);
}

/** `hasHistory` is false only when nothing has been pushed onto the stack
 *  this session (a fresh load / deep link) — Structure hides its back link
 *  in that case; Member Detail and Volume Entry show it anyway, since
 *  `label` already defaults to "Home", a real, always-valid destination. */
export function useBackTarget(): { label: string; hasHistory: boolean; go: () => void } {
  const { stack } = useNavigationHistory();
  const navigate = useNavigate();
  const top = stack[stack.length - 1];
  return {
    label: top?.label ?? "Home",
    hasHistory: !!top,
    go: () => (top ? navigate(-1) : navigate("/")),
  };
}
