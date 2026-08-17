import { useCallback, useEffect, useState } from "react";
import { applyTheme, readStoredTheme, setTheme, type ThemePreference } from "./theme";

/** Wires theme.ts's pure logic to the real DOM/localStorage, applied once on mount. */
export function useTheme(): [ThemePreference, (preference: ThemePreference) => void] {
  const [preference, setPreference] = useState<ThemePreference>(() =>
    readStoredTheme(window.localStorage),
  );

  useEffect(() => {
    applyTheme(document.documentElement, preference);
  }, [preference]);

  const update = useCallback((next: ThemePreference) => {
    setTheme(window.localStorage, document.documentElement, next);
    setPreference(next);
  }, []);

  return [preference, update];
}
