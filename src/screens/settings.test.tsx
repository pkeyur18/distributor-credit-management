import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Settings } from "./settings";
import { ToastProvider, Toaster } from "@/components/ui/toast";
import * as authContext from "@/lib/auth-context";
import * as m3Calc from "@/lib/ipc/m3-calc";
import * as m4Search from "@/lib/ipc/m4-search";
import * as m7Settings from "@/lib/ipc/m7-settings";
import * as preflight from "@/lib/ipc/preflight";
import type { Settings as SettingsData } from "@/lib/ipc/entities";

const SETTINGS: SettingsData = {
  slabThresholds: [10000],
  slabPercentages: [2],
  referenceUnitValue: 500,
  hierarchyDepth: 4,
  level2Width: 9,
  level3Width: 6,
  level4Width: 3,
  royaltyQualifyingCount: 3,
  royaltyRatePercent: 1,
  royaltyTier2QualifyingCount: 3,
  royaltyTier2RatePercent: 1,
  royaltyTier3QualifyingCount: 3,
  royaltyTier3RatePercent: 1,
  royaltyTier4QualifyingCount: 3,
  royaltyTier4RatePercent: 1,
  yearlyCycle: { start: "01-01", end: "12-31" },
  lowContributionThreshold: 10000,
  defaultExportColumns: [],
  sessionTimeoutMinutes: 15,
  consoleBackupSchedule: "off",
  consoleBackupRetentionCount: 10,
  consoleBackupFolder: "backups",
};

function mockSettingsScreen() {
  vi.spyOn(authContext, "useAuth").mockReturnValue({
    markSignedOut: vi.fn(),
  } as unknown as ReturnType<typeof authContext.useAuth>);
  vi.spyOn(m7Settings, "getSettings").mockResolvedValue(SETTINGS);
  vi.spyOn(m7Settings, "getConsoleBackupSettings").mockResolvedValue({
    schedule: "off",
    retentionCount: 10,
    folder: "backups",
  });
  vi.spyOn(m4Search, "getDirectChildrenChart").mockResolvedValue({
    nodes: [],
    slabTable: [{ id: 1, threshold: 10000, percentage: 2, sortOrder: 1 }],
  });
  vi.spyOn(preflight, "listRestorePoints").mockResolvedValue([]);
  const preview = vi.spyOn(m3Calc, "previewSettingsImpact").mockResolvedValue({
    rewardsBefore: 0,
    rewardsAfter: 0,
    royaltyEarnerCountBefore: 0,
    royaltyEarnerCountAfter: 0,
    affectedMembers: [],
  });
  const update = vi.spyOn(m7Settings, "updateSettings").mockResolvedValue(SETTINGS);
  render(
    <ToastProvider>
      <Settings />
      <Toaster />
    </ToastProvider>,
  );
  return { preview, update };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("Settings — Royalty card", () => {
  it("sends only the membership values that changed", async () => {
    const { preview, update } = mockSettingsScreen();
    const user = userEvent.setup();
    const platinumRate = await screen.findByLabelText("Platinum royalty rate (%)");

    await user.clear(platinumRate);
    await user.type(platinumRate, "1.5");
    await user.click(screen.getByRole("button", { name: "Save royalty settings" }));
    await user.click(await screen.findByRole("button", { name: /Save and re-work/ }));

    expect(preview).toHaveBeenCalledWith({ royaltyTier2RatePercent: 1.5 });
    await waitFor(() => expect(update).toHaveBeenCalledWith({ royaltyTier2RatePercent: 1.5 }));
  });

  it("refuses a qualifying count below 1 before previewing anything", async () => {
    const { preview } = mockSettingsScreen();
    const user = userEvent.setup();
    const diamondCount = await screen.findByLabelText("Diamond minimum qualifying legs");

    await user.clear(diamondCount);
    await user.type(diamondCount, "0");
    await user.click(screen.getByRole("button", { name: "Save royalty settings" }));

    expect(
      await screen.findByText(
        "Qualifying legs must be a whole number of 1 or more, and a rate 0 or more",
      ),
    ).toBeInTheDocument();
    expect(preview).not.toHaveBeenCalled();
  });
});
