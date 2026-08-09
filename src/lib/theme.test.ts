import { describe, expect, it } from "vitest";
import { readStoredTheme, applyTheme, setTheme, type ThemePreference } from "./theme";

function fakeStorage(initial: Record<string, string> = {}) {
  const store = { ...initial };
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    _store: store,
  };
}

function fakeRoot() {
  return { dataset: {} as DOMStringMap };
}

describe("readStoredTheme", () => {
  it("returns 'system' when nothing is stored", () => {
    expect(readStoredTheme(fakeStorage())).toBe("system");
  });

  it("returns the stored preference when it is 'light' or 'dark'", () => {
    expect(readStoredTheme(fakeStorage({ "bvconsole-theme": "dark" }))).toBe("dark");
    expect(readStoredTheme(fakeStorage({ "bvconsole-theme": "light" }))).toBe("light");
  });

  it("falls back to 'system' for a garbage stored value", () => {
    expect(readStoredTheme(fakeStorage({ "bvconsole-theme": "blue" }))).toBe("system");
  });
});

describe("applyTheme", () => {
  it("sets data-theme to the explicit preference", () => {
    const root = fakeRoot();
    applyTheme(root, "dark");
    expect(root.dataset.theme).toBe("dark");
  });

  it("removes data-theme entirely for 'system', so the media query decides", () => {
    const root = fakeRoot();
    root.dataset.theme = "dark";
    applyTheme(root, "system");
    expect(root.dataset.theme).toBeUndefined();
  });
});

describe("setTheme", () => {
  it("persists an explicit preference and applies it", () => {
    const storage = fakeStorage();
    const root = fakeRoot();
    setTheme(storage, root, "dark");
    expect(storage.getItem("bvconsole-theme")).toBe("dark");
    expect(root.dataset.theme).toBe("dark");
  });

  it("clears storage for 'system' rather than writing the literal string", () => {
    const storage = fakeStorage({ "bvconsole-theme": "dark" });
    const root = fakeRoot();
    setTheme(storage, root, "system");
    expect(storage.getItem("bvconsole-theme")).toBeNull();
    expect(root.dataset.theme).toBeUndefined();
  });

  it("round-trips every preference value", () => {
    const storage = fakeStorage();
    const root = fakeRoot();
    const values: ThemePreference[] = ["light", "dark", "system"];
    for (const value of values) {
      setTheme(storage, root, value);
      expect(readStoredTheme(storage)).toBe(value === "system" ? "system" : value);
    }
  });
});
