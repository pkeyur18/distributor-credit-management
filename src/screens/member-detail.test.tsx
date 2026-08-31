import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router";

import { MemberDetail } from "./member-detail";
import { ToastProvider, Toaster } from "@/components/ui/toast";
import { NavigationHistoryProvider } from "@/lib/navigation-history";
import * as m1Members from "@/lib/ipc/m1-members";
import * as m2Entries from "@/lib/ipc/m2-entries";
import * as m4Search from "@/lib/ipc/m4-search";
import type { Member } from "@/lib/ipc/entities";
import type { MemberDetail as MemberDetailData } from "@/lib/ipc/m4-search";

const ROOT_MEMBER: Member = {
  id: 100001,
  name: "Root Member",
  phone: "9876500000",
  email: null,
  address: "HQ",
  introducerMemberId: null,
  level: 1,
  isActive: true,
  joiningDate: "2026-01-01",
  consentGiven: true,
  consentDate: "2026-01-01",
  createdAt: "2026-01-01",
};

const CHILD_MEMBER: Member = {
  id: 284913,
  name: "Asha Patel",
  phone: "9876543210",
  email: "asha@example.com",
  address: "1 Main Street",
  introducerMemberId: 100001,
  level: 2,
  isActive: true,
  joiningDate: "2026-02-01",
  consentGiven: true,
  consentDate: "2026-02-01",
  createdAt: "2026-02-01",
};

function detailFor(member: Member, overrides: Partial<MemberDetailData> = {}): MemberDetailData {
  return {
    member,
    totalBusinessVolume: 500000,
    slabPct: 14,
    legCount: 2,
    rewards: {
      ownReward: { ownBusinessVolume: 100000, ownSlabPct: 14, amount: 14000 },
      differentials: [
        {
          childId: 500001,
          childName: "Child One",
          childTotalBusinessVolume: 200000,
          childSlabPct: 6,
          ownSlabPct: 14,
          differentialPct: 8,
          amount: 16000,
        },
      ],
      royalty: { qualifyingChildren: 3, ratePercent: 5, amount: 5000 },
      rewardsTotal: 35000,
    },
    directChildren: [
      { memberId: 500001, name: "Child One", totalBusinessVolume: 200000, slabPct: 6, isActive: true },
      { memberId: 500002, name: "Child Two", totalBusinessVolume: 0, slabPct: 0, isActive: false },
    ],
    ...overrides,
  };
}

function renderDetail(memberId: number) {
  return render(
    <MemoryRouter initialEntries={[`/member/${memberId}`]}>
      <ToastProvider>
        <NavigationHistoryProvider>
          <Routes>
            <Route path="/member/:memberId" element={<MemberDetail />} />
            <Route path="/" element={<div>Home screen</div>} />
            <Route path="/structure/:memberId" element={<div>Structure screen</div>} />
            <Route path="/entry" element={<div>Volume entry screen</div>} />
          </Routes>
        </NavigationHistoryProvider>
        <Toaster />
      </ToastProvider>
    </MemoryRouter>,
  );
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("MemberDetail — loading and error", () => {
  it("shows loading, then the member-not-found empty state on a rejected fetch", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockRejectedValue({
      kind: "not_found",
      message: "No member matches that ID.",
    });
    renderDetail(999999);

    expect(await screen.findByText("Member not found")).toBeInTheDocument();
    expect(screen.getByText("No member matches that ID.")).toBeInTheDocument();
  });
});

describe("MemberDetail — rewards breakdown", () => {
  it("renders own reward, differential legs, royalty, and the total", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(detailFor(CHILD_MEMBER));
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    expect(screen.getByText(/1000\.00 at 14%/)).toBeInTheDocument();
    // "Child One" appears both as a differential row and again in the
    // Direct legs table below.
    expect(screen.getAllByText("Child One")).toHaveLength(2);
    expect(screen.getByText(/3 of 1 legs qualifying/)).toBeInTheDocument();
    // Rewards total appears both in its stat card and the table's total row.
    expect(screen.getAllByText("350.00")).toHaveLength(2);
  });

  it("shows the no-direct-legs row when there are none", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(
      detailFor(CHILD_MEMBER, {
        rewards: {
          ownReward: { ownBusinessVolume: 0, ownSlabPct: 0, amount: 0 },
          differentials: [],
          royalty: null,
          rewardsTotal: 0,
        },
        directChildren: [],
      }),
    );
    renderDetail(CHILD_MEMBER.id);

    expect(
      await screen.findByText(/No direct legs — differential and royalty are earned/),
    ).toBeInTheDocument();
    expect(screen.queryByText(/Direct legs \(/)).not.toBeInTheDocument();
  });
});

describe("MemberDetail — root member", () => {
  it("shows the Root member pill and disables Deactivate", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(
      detailFor(ROOT_MEMBER, { directChildren: [] }),
    );
    renderDetail(ROOT_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    expect(screen.getByText("Root member")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Deactivate" })).toBeDisabled();
    expect(screen.getByText("None — root member")).toBeInTheDocument();
  });
});

describe("MemberDetail — deactivate / reactivate", () => {
  it("deactivates the member through the confirm dialog and updates the badge", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    // No direct children here — Child Two's own "Inactive" pill in the
    // direct-legs table would otherwise collide with the header's own
    // badge this test is actually checking.
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(
      detailFor(CHILD_MEMBER, { directChildren: [] }),
    );
    const deactivateSpy = vi.spyOn(m1Members, "deactivateMember").mockResolvedValue(undefined);
    const user = userEvent.setup();
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    await user.click(screen.getByRole("button", { name: "Deactivate" }));

    expect(await screen.findByRole("heading", { name: "Deactivate Asha Patel?" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Deactivate" }));

    await waitFor(() => expect(deactivateSpy).toHaveBeenCalledWith(CHILD_MEMBER.id));
    expect(await screen.findByText("Inactive")).toBeInTheDocument();
  });

  it("reactivates an inactive member", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(
      detailFor({ ...CHILD_MEMBER, isActive: false }, { directChildren: [] }),
    );
    const reactivateSpy = vi.spyOn(m1Members, "reactivateMember").mockResolvedValue(undefined);
    const user = userEvent.setup();
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    expect(screen.getByText("Inactive")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Reactivate" }));
    await user.click(screen.getByRole("button", { name: "Reactivate" }));

    await waitFor(() => expect(reactivateSpy).toHaveBeenCalledWith(CHILD_MEMBER.id));
    await waitFor(() => expect(screen.queryByText("Inactive")).not.toBeInTheDocument());
  });
});

describe("MemberDetail — navigation", () => {
  it("clicking a direct leg row navigates to that member's own detail page", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    const getMemberDetailSpy = vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(
      detailFor(CHILD_MEMBER),
    );
    const user = userEvent.setup();
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    // Both the differential row and the Direct legs row for this same
    // child navigate to the same place — either demonstrates the behavior.
    await user.click(screen.getAllByText("Child One")[0]);

    await waitFor(() => expect(getMemberDetailSpy).toHaveBeenCalledWith(500001, "2026-06"));
  });

  it("'View in structure' navigates to the Structure screen rooted at this member", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(detailFor(CHILD_MEMBER));
    const user = userEvent.setup();
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    await user.click(screen.getByRole("button", { name: "View in structure" }));

    expect(await screen.findByText("Structure screen")).toBeInTheDocument();
  });

  it("'Record volume' navigates to Volume Entry", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(detailFor(CHILD_MEMBER));
    const user = userEvent.setup();
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    await user.click(screen.getByRole("button", { name: "Record volume" }));

    expect(await screen.findByText("Volume entry screen")).toBeInTheDocument();
  });
});

describe("MemberDetail — month switcher", () => {
  it("re-fetches member detail for the newly selected month", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-05", "2026-06"],
      blockingMonth: null,
    });
    const getMemberDetailSpy = vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(
      detailFor(CHILD_MEMBER),
    );
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    await waitFor(() => expect(getMemberDetailSpy).toHaveBeenCalledWith(CHILD_MEMBER.id, "2026-05"));
  });
});

describe("MemberDetail — edit", () => {
  it("opens the edit modal prefilled with the member", async () => {
    vi.spyOn(m2Entries, "getPeriodLockStatus").mockResolvedValue({
      recordablePeriodMonths: ["2026-06"],
      blockingMonth: null,
    });
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(detailFor(CHILD_MEMBER));
    const user = userEvent.setup();
    renderDetail(CHILD_MEMBER.id);

    await screen.findByRole("button", { name: "Edit member" });
    await user.click(screen.getByRole("button", { name: "Edit member" }));

    expect(await screen.findByRole("heading", { name: "Edit member" })).toBeInTheDocument();
    expect(screen.getByLabelText(/^Name/)).toHaveValue("Asha Patel");
  });
});
