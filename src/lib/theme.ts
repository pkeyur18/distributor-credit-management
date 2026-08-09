export type ThemePreference = "light" | "dark" | "system";

const STORAGE_KEY = "bvconsole-theme";

type ReadableStorage = Pick<Storage, "getItem">;
type WritableStorage = Pick<Storage, "setItem" | "removeItem">;
type ThemedRoot = { dataset: DOMStringMap };

export function readStoredTheme(storage: ReadableStorage): ThemePreference {
  const value = storage.getItem(STORAGE_KEY);
  return value === "light" || value === "dark" ? value : "system";
}

// 07-design-system.md §7: an explicit choice stamps data-theme on the root;
// "system" removes it so the prefers-color-scheme media query decides.
export function applyTheme(root: ThemedRoot, preference: ThemePreference): void {
  if (preference === "system") {
    delete root.dataset.theme;
  } else {
    root.dataset.theme = preference;
  }
}

export function setTheme(
  storage: WritableStorage,
  root: ThemedRoot,
  preference: ThemePreference,
): void {
  if (preference === "system") {
    storage.removeItem(STORAGE_KEY);
  } else {
    storage.setItem(STORAGE_KEY, preference);
  }
  applyTheme(root, preference);
}
