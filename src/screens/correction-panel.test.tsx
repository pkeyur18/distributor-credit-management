import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router";

import { CorrectionPanel } from "./correction-panel";
import { ToastProvider, Toaster } from "@/components/ui/toast";
import { NavigationHistoryProvider } from "@/lib/navigation-history";
import * as m1Members from "@/lib/ipc/m1-members";
import * as m2Entries from "@/lib/ipc/m2-entries";
import * as m4Search from "@/lib/ipc/m4-search";
import * as m6Reports from "@/lib/ipc/m6-reports";
import type { ClosedMonthBackup } from "@/lib/ipc/m6-reports";
import type { MemberDetail } from "@/lib/ipc/m4-search";
import type { Member, PeriodEntryRecord, SearchResult } from "@/lib/ipc/entities";

const CLOSED_MONTH: ClosedMonthBackup = {
  periodId: 1,
  periodMonth: "2026-06",
  closedAt: "2026-07-01",
  latestVersion: 1,
  isCorrected: false,
};

const MEMBER: Member = {
  id: 284913,
  name: "Asha Patel",
  phone: "9876543210",
  email: null,
  address: "1 Main Street",
  introducerMemberId: 100001,
  level: 2,
  isActive: true,
  joiningDate: "2026-01-01",
  consentGiven: true,
  consentDate: "2026-01-01",
  createdAt: "2026-01-01",
};

const SEARCH_RESULT: SearchResult = {
  id: MEMBER.id,
  name: MEMBER.name,
  phone: MEMBER.phone,
  totalBusinessVolume: 5000,
  slabPct: 6,
  isActive: true,
  email: null,
  address: MEMBER.address,
  introducerMemberId: MEMBER.introducerMemberId,
};

function memberDetail(overrides: Partial<MemberDetail> = {}): MemberDetail {
  return {
    member: MEMBER,
    totalBusinessVolume: 5000,
    slabPct: 6,
    legCount: 0,
    rewards: {
      ownReward: { ownBusinessVolume: 0, ownSlabPct: 0, amount: 0 },
      differentials: [],
      royalty: null,
      rewardsTotal: 0,
    },
    directChildren: [],
    ...overrides,
  };
}

function entryRecord(overrides: Partial<PeriodEntryRecord> = {}): PeriodEntryRecord {
  return {
    id: 501,
    memberId: MEMBER.id,
    memberName: MEMBER.name,
    amount: 100000,
    entryDate: "2026-06-15",
    createdAt: "2026-06-15T00:00:00Z",
    updatedAt: null,
    ...overrides,
  };
}

function renderPanel() {
  return render(
    <MemoryRouter initialEntries={["/entry/correct"]}>
      <ToastProvider>
        <NavigationHistoryProvider>
          <CorrectionPanel />
        </NavigationHistoryProvider>
        <Toaster />
      </ToastProvider>
    </MemoryRouter>,
  );
}

async function selectMember(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByLabelText("Member"), "Asha");
  const option = await screen.findByText("Asha Patel");
  await user.click(option);
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("CorrectionPanel", () => {
  it("shows the empty state when there are no closed months yet", async () => {
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([]);
    renderPanel();

    expect(await screen.findByText("No closed months yet")).toBeInTheDocument();
  });

  it("defaults to the first closed month and lets the operator search and pick a member", async () => {
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([CLOSED_MONTH]);
    vi.spyOn(m1Members, "searchMembers").mockResolvedValue([SEARCH_RESULT]);
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(memberDetail());
    vi.spyOn(m2Entries, "listPeriodEntries").mockResolvedValue({
      periodMonth: "2026-06",
      entries: [entryRecord()],
    });
    const user = userEvent.setup();
    renderPanel();

    await screen.findByLabelText("Member");
    await selectMember(user);

    expect(await screen.findByText("6% slab")).toBeInTheDocument();
    expect(await screen.findByText("2026-06-15")).toBeInTheDocument();
    expect(screen.getByText("1000.00")).toBeInTheDocument();
  });

  it("shows the no-records state for a member with nothing in the selected month", async () => {
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([CLOSED_MONTH]);
    vi.spyOn(m1Members, "searchMembers").mockResolvedValue([SEARCH_RESULT]);
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(memberDetail());
    vi.spyOn(m2Entries, "listPeriodEntries").mockResolvedValue({
      periodMonth: "2026-06",
      entries: [],
    });
    const user = userEvent.setup();
    renderPanel();

    await screen.findByLabelText("Member");
    await selectMember(user);

    expect(await screen.findByText(/No records for Asha Patel in June 2026/)).toBeInTheDocument();
  });

  it("'Change' clears the selected member back to the search box", async () => {
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([CLOSED_MONTH]);
    vi.spyOn(m1Members, "searchMembers").mockResolvedValue([SEARCH_RESULT]);
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(memberDetail());
    vi.spyOn(m2Entries, "listPeriodEntries").mockResolvedValue({
      periodMonth: "2026-06",
      entries: [],
    });
    const user = userEvent.setup();
    renderPanel();

    await screen.findByLabelText("Member");
    await selectMember(user);
    await screen.findByText("Change");

    await user.click(screen.getByRole("button", { name: "Change" }));
    expect(screen.getByLabelText("Member")).toBeInTheDocument();
  });

  it("adds a new closed-month record and shows the success toast", async () => {
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([CLOSED_MONTH]);
    vi.spyOn(m1Members, "searchMembers").mockResolvedValue([SEARCH_RESULT]);
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(memberDetail());
    const listSpy = vi
      .spyOn(m2Entries, "listPeriodEntries")
      .mockResolvedValue({ periodMonth: "2026-06", entries: [] });
    const addSpy = vi.spyOn(m2Entries, "addClosedMonthEntry").mockResolvedValue({
      id: 999,
      memberId: MEMBER.id,
      amount: 250000,
      entryDate: "2026-06-10",
      periodMonth: "2026-06",
      createdAt: "2026-06-10T00:00:00Z",
      updatedAt: null,
    });
    const user = userEvent.setup();
    renderPanel();

    await screen.findByLabelText("Member");
    await selectMember(user);
    await screen.findByText(/No records for Asha Patel/);

    await user.click(screen.getByRole("button", { name: /Add record/ }));
    expect(await screen.findByRole("heading", { name: "Add record" })).toBeInTheDocument();

    const dateInput = screen.getByLabelText("Date");
    await user.clear(dateInput);
    await user.type(dateInput, "2026-06-10");
    await user.type(screen.getByLabelText("Business Volume"), "2500.00");

    listSpy.mockResolvedValue({ periodMonth: "2026-06", entries: [entryRecord({ id: 999 })] });
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(addSpy).toHaveBeenCalledWith({
        memberId: MEMBER.id,
        amount: 250000,
        entryDate: "2026-06-10",
      }),
    );
    expect(await screen.findByText(/Entry #999 added — 2500\.00/)).toBeInTheDocument();
  });

  it("edits an existing record via the row's Edit button", async () => {
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([CLOSED_MONTH]);
    vi.spyOn(m1Members, "searchMembers").mockResolvedValue([SEARCH_RESULT]);
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(memberDetail());
    const record = entryRecord();
    const listSpy = vi
      .spyOn(m2Entries, "listPeriodEntries")
      .mockResolvedValue({ periodMonth: "2026-06", entries: [record] });
    const editSpy = vi.spyOn(m2Entries, "editEntry").mockResolvedValue({
      id: record.id,
      memberId: MEMBER.id,
      amount: 150000,
      entryDate: "2026-06-15",
      periodMonth: "2026-06",
      createdAt: record.createdAt,
      updatedAt: "2026-06-20T00:00:00Z",
    });
    const user = userEvent.setup();
    renderPanel();

    await screen.findByLabelText("Member");
    await selectMember(user);
    await screen.findByText("2026-06-15");

    await user.click(screen.getByRole("button", { name: "Edit record" }));
    expect(await screen.findByRole("heading", { name: "Edit record" })).toBeInTheDocument();
    expect(screen.getByLabelText("Business Volume")).toHaveValue("1000.00");

    const amountInput = screen.getByLabelText("Business Volume");
    await user.clear(amountInput);
    await user.type(amountInput, "1500.00");

    listSpy.mockResolvedValue({
      periodMonth: "2026-06",
      entries: [entryRecord({ amount: 150000 })],
    });
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(editSpy).toHaveBeenCalledWith({ id: record.id, amount: 150000, entryDate: "2026-06-15" }),
    );
    expect(await screen.findByText(/Entry #501 corrected to 1500\.00/)).toBeInTheDocument();
  });

  it("routes a server field error to the date or amount hint based on the error's field", async () => {
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([CLOSED_MONTH]);
    vi.spyOn(m1Members, "searchMembers").mockResolvedValue([SEARCH_RESULT]);
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(memberDetail());
    vi.spyOn(m2Entries, "listPeriodEntries").mockResolvedValue({
      periodMonth: "2026-06",
      entries: [entryRecord()],
    });
    vi.spyOn(m2Entries, "editEntry").mockRejectedValue({
      kind: "validation",
      field: "entryDate",
      message: "That date falls outside the closed month.",
    });
    const user = userEvent.setup();
    renderPanel();

    await screen.findByLabelText("Member");
    await selectMember(user);
    await screen.findByText("2026-06-15");

    await user.click(screen.getByRole("button", { name: "Edit record" }));
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText("That date falls outside the closed month.")).toBeInTheDocument();
  });

  it("keeps Save disabled until both a valid date and amount are present", async () => {
    vi.spyOn(m6Reports, "listBackups").mockResolvedValue([CLOSED_MONTH]);
    vi.spyOn(m1Members, "searchMembers").mockResolvedValue([SEARCH_RESULT]);
    vi.spyOn(m4Search, "getMemberDetail").mockResolvedValue(memberDetail());
    vi.spyOn(m2Entries, "listPeriodEntries").mockResolvedValue({
      periodMonth: "2026-06",
      entries: [],
    });
    const user = userEvent.setup();
    renderPanel();

    await screen.findByLabelText("Member");
    await selectMember(user);
    await user.click(screen.getByRole("button", { name: /Add record/ }));

    const saveButton = await screen.findByRole("button", { name: "Save" });
    expect(saveButton).toBeDisabled();

    await user.type(screen.getByLabelText("Business Volume"), "0.00");
    expect(saveButton).toBeDisabled();
  });
});
