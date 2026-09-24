import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { RecalcWarningDialog } from "./recalc-warning-dialog";
import type { MemberImpact, SettingsImpactPreview } from "@/lib/ipc/m3-calc";

function impact(overrides: Partial<MemberImpact>): MemberImpact {
  return {
    memberId: 1,
    memberName: "Asha",
    rewardsBefore: 0,
    rewardsAfter: 0,
    slabPctBefore: 14,
    slabPctAfter: 14,
    royaltyBefore: 0,
    royaltyAfter: 0,
    membershipTierBefore: 0,
    membershipTierAfter: 0,
    ...overrides,
  };
}

function preview(members: MemberImpact[]): SettingsImpactPreview {
  return {
    rewardsBefore: 0,
    rewardsAfter: 1,
    royaltyEarnerCountBefore: 1,
    royaltyEarnerCountAfter: 1,
    affectedMembers: members,
  };
}

function renderRoyaltyDialog(members: MemberImpact[]) {
  render(
    <RecalcWarningDialog
      open
      onOpenChange={() => {}}
      kind="royalty"
      monthName="September 2026"
      preview={preview(members)}
      onConfirm={() => {}}
    />,
  );
}

describe("RecalcWarningDialog — royalty changes", () => {
  it("shows a membership level move by name", () => {
    renderRoyaltyDialog([impact({ membershipTierBefore: 2, membershipTierAfter: 1 })]);
    expect(screen.getByText("Platinum → Gold")).toBeInTheDocument();
  });

  it("shows royalty before → after when only a rate moved", () => {
    renderRoyaltyDialog([
      impact({
        membershipTierBefore: 2,
        membershipTierAfter: 2,
        royaltyBefore: 180000,
        royaltyAfter: 270000,
      }),
    ]);
    expect(screen.getByText("1800.00 → 2700.00")).toBeInTheDocument();
  });
});
