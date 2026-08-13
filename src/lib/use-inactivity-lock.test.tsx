// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderHook } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";

import { useInactivityLock } from "./use-inactivity-lock";

const markLocked = vi.fn();
const navigate = vi.fn();

vi.mock("@/lib/auth-context", () => ({
  useAuth: () => ({ markLocked }),
}));
vi.mock("react-router", () => ({
  useNavigate: () => navigate,
}));

afterEach(() => {
  clearMocks();
  vi.useRealTimers();
  vi.clearAllMocks();
});

describe("useInactivityLock", () => {
  it("locks after the configured session_timeout_minutes, not the hardcoded default", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_settings") return { sessionTimeoutMinutes: 1 };
      if (cmd === "lock_session") return null;
      throw new Error(`unexpected command ${cmd}`);
    });

    vi.useFakeTimers();
    renderHook(() => useInactivityLock());

    // flush the get_settings microtask so the timer re-arms at 1 minute
    await vi.advanceTimersByTimeAsync(0);

    await vi.advanceTimersByTimeAsync(60_000);

    expect(markLocked).toHaveBeenCalled();
    expect(navigate).toHaveBeenCalledWith("/auth/locked", { replace: true });
  });
});
