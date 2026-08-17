import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";

import { MonthlyClose } from "./monthly-close";
import { OutstandingAlertProvider } from "@/lib/outstanding-alert-context";
import { ToastProvider } from "@/components/ui/toast";
import * as m5Close from "@/lib/ipc/m5-close";
import * as m6Reports from "@/lib/ipc/m6-reports";
import type { Period } from "@/lib/ipc/entities";

const period: Period = {
  id: 1,
  periodMonth: "2026-06",
  status: "awaiting_close",
  endedAt: "2026-07-01",
  closedAt: null,
};

beforeEach(() => {
  vi.spyOn(m6Reports, "listBackups").mockResolvedValue([]);
});

// Banner's Close link (outstanding-month-banner.tsx) navigates here with
// `state: { autoStart: true }` so the wizard opens immediately for the
// oldest outstanding month — same effect as clicking the list row's own
// Close button — instead of landing on the plain list.
describe("MonthlyClose auto-start", () => {
  it("opens the close wizard immediately when navigated with autoStart state", async () => {
    vi.spyOn(m5Close, "getOutstandingPeriods").mockResolvedValue([period]);
    vi.spyOn(m5Close, "beginClose").mockResolvedValue({
      periodId: 1,
      memberCount: 3,
      withEntryCount: 2,
      topSlabCount: 1,
    });

    render(
      <MemoryRouter initialEntries={[{ pathname: "/close", state: { autoStart: true } }]}>
        <ToastProvider>
          <OutstandingAlertProvider>
            <MonthlyClose />
          </OutstandingAlertProvider>
        </ToastProvider>
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: "Close June 2026" })).toBeInTheDocument();
  });

  it("shows the plain list without autoStart", async () => {
    vi.spyOn(m5Close, "getOutstandingPeriods").mockResolvedValue([period]);

    render(
      <MemoryRouter initialEntries={["/close"]}>
        <ToastProvider>
          <OutstandingAlertProvider>
            <MonthlyClose />
          </OutstandingAlertProvider>
        </ToastProvider>
      </MemoryRouter>,
    );

    expect(await screen.findByRole("button", { name: "Close" })).toBeInTheDocument();
  });
});

describe("MonthlyClose status page", () => {
  it("lists closed months alongside outstanding ones", async () => {
    vi.spyOn(m5Close, "getOutstandingPeriods").mockResolvedValue([period]);
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([
      { periodId: 2, periodMonth: "2026-05", closedAt: "2026-06-01", latestVersion: 1, isCorrected: false },
    ]);

    render(
      <MemoryRouter initialEntries={["/close"]}>
        <ToastProvider>
          <OutstandingAlertProvider>
            <MonthlyClose />
          </OutstandingAlertProvider>
        </ToastProvider>
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: "Closed months" })).toBeInTheDocument();
    expect(await screen.findByText("May 2026")).toBeInTheDocument();
  });
});
