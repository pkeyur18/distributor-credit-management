import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router";

import { Home } from "./home";
import { ToastProvider, Toaster } from "@/components/ui/toast";
import * as m1Members from "@/lib/ipc/m1-members";
import * as m2Entries from "@/lib/ipc/m2-entries";
import * as m4Search from "@/lib/ipc/m4-search";
import type { ChartNode, SearchResult, SlabRow } from "@/lib/ipc/entities";
import type { DirectChildrenChartResult } from "@/lib/ipc/m4-search";

function node(overrides: Partial<ChartNode> = {}): ChartNode {
  return {
    memberId: 1,
    name: "Root Member",
    ownBusinessVolume: 0,
    isActive: true,
    introducerMemberId: null,
    slabPct: 0,
    rewards: 0,
    legCount: 0,
    ...overrides,
  };
}

const SLAB_TABLE: SlabRow[] = [
  { id: 1, threshold: 100000, percentage: 6, sortOrder: 0 },
  { id: 2, threshold: 500000, percentage: 14, sortOrder: 1 },
];

function chartResult(nodes: ChartNode[], slabTable: SlabRow[] = SLAB_TABLE): DirectChildrenChartResult {
  return { nodes, slabTable };
}

const SEARCH_RESULT: SearchResult = {
  id: 284913,
  name: "Asha Patel",
  phone: "9876543210",
  totalBusinessVolume: 5000,
  slabPct: 6,
  isActive: true,
  email: null,
  address: "1 Main Street",
  introducerMemberId: 100001,
};

function renderHome() {
  return render(
    <MemoryRouter initialEntries={["/"]}>
      <ToastProvider>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/member/:memberId" element={<div>Member Detail screen</div>} />
        </Routes>
        <Toaster />
      </ToastProvider>
    </MemoryRouter>,
  );
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("Home — empty console", () => {
  it("shows the no-members-yet empty state when no root member exists", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getDirectChildrenChart").mockRejectedValue({ kind: "not_found" });
    renderHome();

    expect(await screen.findByText("No members yet")).toBeInTheDocument();
  });
});

describe("Home — today's standing", () => {
  it("computes the three stat cards from the chart nodes and slab table", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getDirectChildrenChart").mockResolvedValue(
      chartResult([
        node({ memberId: 1, isActive: true, ownBusinessVolume: 1000, slabPct: 14 }),
        node({ memberId: 2, isActive: false, ownBusinessVolume: 0, slabPct: 0 }),
        node({ memberId: 3, isActive: true, ownBusinessVolume: 500, slabPct: 6 }),
      ]),
    );
    renderHome();

    await screen.findByText("Today's standing");
    expect(screen.getByText("3")).toBeInTheDocument(); // Members
    expect(screen.getByText("1 inactive")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument(); // Entries this period (2 with own BV > 0)
    expect(screen.getByText("1")).toBeInTheDocument(); // On top slab (14%)
    expect(screen.getByText("14% and above")).toBeInTheDocument();
  });

  it("renders the slab distribution totals for members and rewards", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getDirectChildrenChart").mockResolvedValue(
      chartResult([
        node({ memberId: 1, slabPct: 6, rewards: 1000 }),
        node({ memberId: 2, slabPct: 14, rewards: 2000 }),
      ]),
    );
    renderHome();

    expect(await screen.findByText("Members by slab")).toBeInTheDocument();
    expect(screen.getByText("2 members total, across 3 slabs")).toBeInTheDocument();
    expect(screen.getByText("Rewards by slab")).toBeInTheDocument();
    expect(screen.getByText("30.00 total this period, across 3 slabs")).toBeInTheDocument();
  });
});

describe("Home — search", () => {
  it("shows the pre-search prompt until a query is typed", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getDirectChildrenChart").mockResolvedValue(chartResult([node()]));
    renderHome();

    expect(
      await screen.findByText("Search for a member to see their details"),
    ).toBeInTheDocument();
  });

  it("navigates to Member Detail when a search result is selected", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getDirectChildrenChart").mockResolvedValue(chartResult([node()]));
    vi.spyOn(m1Members, "searchMembers").mockResolvedValue([SEARCH_RESULT]);
    const user = userEvent.setup();
    renderHome();

    await user.type(screen.getByPlaceholderText(/Search by name/), "Asha");
    const option = await screen.findByText("Asha Patel");
    await user.click(option);

    expect(await screen.findByText("Member Detail screen")).toBeInTheDocument();
  });
});

describe("Home — month switcher", () => {
  it("hides the switcher with exactly one recordable month", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getDirectChildrenChart").mockResolvedValue(chartResult([node()]));
    renderHome();

    await screen.findByText("Today's standing");
    expect(screen.queryByText("Showing figures for")).not.toBeInTheDocument();
  });

  it("shows the switcher with two or more outstanding recordable months", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-05", "2026-06"],
      blockingMonth: null,
    });
    const chartSpy = vi.spyOn(m4Search, "getDirectChildrenChart").mockResolvedValue(
      chartResult([node()]),
    );
    renderHome();

    expect(await screen.findByText("Showing figures for")).toBeInTheDocument();
    await waitFor(() =>
      expect(chartSpy).toHaveBeenCalledWith({ fullTree: true, periodMonth: "2026-05" }),
    );
  });
});

describe("Home — add member", () => {
  it("opens the Add Member modal", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getDirectChildrenChart").mockResolvedValue(chartResult([node()]));
    const user = userEvent.setup();
    renderHome();

    await screen.findByText("Today's standing");
    await user.click(screen.getByRole("button", { name: /Add member/ }));

    expect(await screen.findByRole("heading", { name: "Add member" })).toBeInTheDocument();
  });
});
