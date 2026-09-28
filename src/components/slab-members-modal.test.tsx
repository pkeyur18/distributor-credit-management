import { describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SlabMembersModal } from "./slab-members-modal";
import type { ChartNode } from "@/lib/ipc/entities";

function node(overrides: Partial<ChartNode>): ChartNode {
  return {
    memberId: 1,
    name: "Member",
    ownBusinessVolume: 0,
    isActive: true,
    introducerMemberId: null,
    slabPct: 0,
    rewards: 0,
    legCount: 0,
    ...overrides,
  };
}

const NODES: ChartNode[] = [
  node({ memberId: 100001, name: "Zara Shah", slabPct: 14, ownBusinessVolume: 5000, rewards: 700 }),
  node({
    memberId: 100002,
    name: "Asha Patel",
    slabPct: 14,
    ownBusinessVolume: 2000,
    rewards: 2100,
    isActive: false,
  }),
  node({ memberId: 100003, name: "Kiran Mehta", slabPct: 6, ownBusinessVolume: 1000, rewards: 60 }),
];

function renderModal(props: Partial<Parameters<typeof SlabMembersModal>[0]> = {}) {
  const onSelectMember = vi.fn();
  render(
    <SlabMembersModal
      nodes={NODES}
      slabs={[0, 6, 14]}
      initialPct={14}
      metric="count"
      month="June 2026"
      onClose={vi.fn()}
      onSelectMember={onSelectMember}
      {...props}
    />,
  );
  return { onSelectMember };
}

function memberNames() {
  return within(screen.getByRole("table"))
    .getAllByRole("row")
    .slice(1)
    .map((row) => within(row).getAllByRole("cell")[0].textContent);
}

describe("SlabMembersModal", () => {
  it("lists only the chosen slab's members, by name, when opened from Members by slab", () => {
    renderModal();
    expect(screen.getByRole("heading", { name: "Members on the 14% slab" })).toBeInTheDocument();
    expect(memberNames()).toEqual(["Asha PatelInactive#100002", "Zara Shah#100001"]);
    expect(screen.queryByText("Share of slab")).not.toBeInTheDocument();
  });

  it("sorts by Rewards, highest first, with a share column when opened from Rewards by slab", () => {
    renderModal({ metric: "rewards" });
    expect(memberNames()).toEqual(["Asha PatelInactive#100002", "Zara Shah#100001"]);
    expect(screen.getByText("Share of slab")).toBeInTheDocument();
    expect(screen.getByText("75%")).toBeInTheDocument(); // 2100 of 2800
  });

  it("re-sorts by Business Volume", async () => {
    renderModal();
    await userEvent.click(screen.getByRole("radio", { name: "Business Volume" }));
    expect(memberNames()).toEqual(["Zara Shah#100001", "Asha PatelInactive#100002"]);
  });

  it("filters inside the slab by name or member number", async () => {
    renderModal();
    await userEvent.type(screen.getByPlaceholderText(/Search this slab/), "100001");
    expect(memberNames()).toEqual(["Zara Shah#100001"]);
  });

  it("steps to the previous non-empty slab, skipping empty ones", async () => {
    renderModal();
    expect(screen.getByRole("button", { name: "Next slab" })).toBeDisabled();
    await userEvent.click(screen.getByRole("button", { name: "Previous slab" }));
    expect(screen.getByRole("heading", { name: "Members on the 6% slab" })).toBeInTheDocument();
    expect(memberNames()).toEqual(["Kiran Mehta#100003"]);
    // 0% has no members, so there is nowhere further back to go.
    expect(screen.getByRole("button", { name: "Previous slab" })).toBeDisabled();
  });

  it("reports the member when a row is clicked", async () => {
    const { onSelectMember } = renderModal();
    await userEvent.click(screen.getByText("Zara Shah"));
    expect(onSelectMember).toHaveBeenCalledWith(100001);
  });
});
