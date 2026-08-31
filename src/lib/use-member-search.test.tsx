// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";

import { useMemberSearch } from "./use-member-search";
import * as m1Members from "@/lib/ipc/m1-members";
import type { SearchResult } from "@/lib/ipc/entities";

const RESULT: SearchResult = {
  id: 284913,
  name: "Asha Patel",
  phone: "9876543210",
  totalBusinessVolume: 0,
  slabPct: 0,
  isActive: true,
  email: null,
  address: "1 Main Street",
  introducerMemberId: null,
};

// Fake-timer advances that trigger a state update need to run inside
// act() for renderHook's `result.current` to reflect the committed
// render — plain `await vi.advanceTimersByTimeAsync(...)` leaves it stale.
async function advance(ms: number) {
  await act(async () => {
    await vi.advanceTimersByTimeAsync(ms);
  });
}

afterEach(() => {
  vi.restoreAllMocks();
  vi.useRealTimers();
});

describe("useMemberSearch — T-M1.4-1: the one hook behind every search box", () => {
  it("returns no results and makes no call for an empty query", () => {
    const searchSpy = vi.spyOn(m1Members, "searchMembers");
    const { result } = renderHook(() => useMemberSearch(""));

    expect(result.current.results).toEqual([]);
    expect(result.current.loading).toBe(false);
    expect(searchSpy).not.toHaveBeenCalled();
  });

  it("treats a whitespace-only query the same as empty", () => {
    const searchSpy = vi.spyOn(m1Members, "searchMembers");
    const { result } = renderHook(() => useMemberSearch("   "));

    expect(result.current.results).toEqual([]);
    expect(searchSpy).not.toHaveBeenCalled();
  });

  it("debounces 200ms before calling search_members with the trimmed query", async () => {
    const searchSpy = vi.spyOn(m1Members, "searchMembers").mockResolvedValue([RESULT]);
    vi.useFakeTimers();

    const { result, rerender } = renderHook(({ q }) => useMemberSearch(q), {
      initialProps: { q: "" },
    });
    rerender({ q: "  Asha  " });

    // Not yet — the debounce hasn't elapsed.
    await advance(199);
    expect(searchSpy).not.toHaveBeenCalled();

    await advance(1);
    expect(searchSpy).toHaveBeenCalledWith("Asha", false);

    await advance(0);
    expect(result.current.results).toEqual([RESULT]);
    expect(result.current.loading).toBe(false);
  });

  it("passes activeOnly through to search_members", async () => {
    const searchSpy = vi.spyOn(m1Members, "searchMembers").mockResolvedValue([]);
    vi.useFakeTimers();

    renderHook(({ q }) => useMemberSearch(q, true), { initialProps: { q: "Asha" } });
    await advance(200);

    expect(searchSpy).toHaveBeenCalledWith("Asha", true);
  });

  it("cancels a stale in-flight query when the input changes again before it resolves", async () => {
    const searchSpy = vi.spyOn(m1Members, "searchMembers").mockImplementation((query) =>
      query === "As"
        ? new Promise(() => {}) // never resolves — must not clobber the later result
        : Promise.resolve([RESULT]),
    );
    vi.useFakeTimers();

    const { result, rerender } = renderHook(({ q }) => useMemberSearch(q), {
      initialProps: { q: "As" },
    });
    await advance(200);
    expect(searchSpy).toHaveBeenCalledWith("As", false);

    rerender({ q: "Asha" });
    await advance(200);
    expect(searchSpy).toHaveBeenCalledWith("Asha", false);

    await advance(0);
    expect(result.current.results).toEqual([RESULT]);
  });

  it("only fires one search for rapid keystrokes within the debounce window", async () => {
    const searchSpy = vi.spyOn(m1Members, "searchMembers").mockResolvedValue([RESULT]);
    vi.useFakeTimers();

    const { rerender } = renderHook(({ q }) => useMemberSearch(q), { initialProps: { q: "A" } });
    await advance(50);
    rerender({ q: "As" });
    await advance(50);
    rerender({ q: "Ash" });
    await advance(50);
    rerender({ q: "Asha" });
    await advance(200);

    expect(searchSpy).toHaveBeenCalledTimes(1);
    expect(searchSpy).toHaveBeenCalledWith("Asha", false);
  });

  it("reports loading true while the request is in flight, false once it settles", async () => {
    let resolveSearch!: (value: SearchResult[]) => void;
    vi.spyOn(m1Members, "searchMembers").mockReturnValue(
      new Promise((resolve) => {
        resolveSearch = resolve;
      }),
    );
    vi.useFakeTimers();

    const { result, rerender } = renderHook(({ q }) => useMemberSearch(q), {
      initialProps: { q: "" },
    });
    rerender({ q: "Asha" });
    await advance(200);

    expect(result.current.loading).toBe(true);

    await act(async () => {
      resolveSearch([RESULT]);
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(result.current.loading).toBe(false);
    expect(result.current.results).toEqual([RESULT]);
  });

  it("returns [] immediately once the query is cleared back to empty, without waiting on a pending request", async () => {
    vi.spyOn(m1Members, "searchMembers").mockReturnValue(new Promise(() => {}));
    vi.useFakeTimers();

    const { result, rerender } = renderHook(({ q }) => useMemberSearch(q), {
      initialProps: { q: "" },
    });
    rerender({ q: "Asha" });
    await advance(200);
    expect(result.current.loading).toBe(true);

    act(() => {
      rerender({ q: "" });
    });
    expect(result.current.results).toEqual([]);
    expect(result.current.loading).toBe(false);
  });
});
