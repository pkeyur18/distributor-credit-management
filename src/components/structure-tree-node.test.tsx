import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { StructureTreeNode } from "./structure-tree-node";

describe("StructureTreeNode", () => {
  it("shows exactly the three member-data fields, never Total Business Volume", () => {
    render(
      <StructureTreeNode
        name="Asha Patel"
        memberNumber="284913"
        ownBusinessVolume="12,450"
        legCount={2}
      />,
    );
    expect(screen.getByText("Asha Patel")).toBeInTheDocument();
    expect(screen.getByText("#284913")).toBeInTheDocument();
    expect(screen.getByText("12,450")).toBeInTheDocument();
  });

  it("a leaf node (no direct legs) is not clickable to expand and states so", () => {
    render(<StructureTreeNode name="Leaf" memberNumber="100001" ownBusinessVolume="0" legCount={0} />);
    expect(screen.getByText("No legs beneath")).toBeInTheDocument();
    // The detail-view button is always available (M4.1); only the
    // branch-toggle role is withheld from a leaf.
    expect(screen.queryByRole("button", { name: /branch/ })).not.toBeInTheDocument();
  });

  it("a non-leaf node toggles open on click", async () => {
    const onOpenToggle = vi.fn();
    render(
      <StructureTreeNode
        name="Asha Patel"
        memberNumber="284913"
        ownBusinessVolume="12,450"
        legCount={3}
        onOpenToggle={onOpenToggle}
      />,
    );
    expect(screen.getByText("3 direct legs")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Open Asha Patel's branch" }));
    expect(onOpenToggle).toHaveBeenCalledOnce();
  });

  it("the Full Hierarchy Window's read-only reuse strips every affordance", () => {
    render(
      <StructureTreeNode
        name="Asha Patel"
        memberNumber="284913"
        ownBusinessVolume="12,450"
        legCount={3}
        interactive={false}
      />,
    );
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
    expect(screen.queryByText(/direct leg/)).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/View .* member detail/)).not.toBeInTheDocument();
  });

  it("an inactive member gets the labelled pill, never colour alone", () => {
    render(
      <StructureTreeNode
        name="Rahul Shah"
        memberNumber="512004"
        ownBusinessVolume="0"
        legCount={0}
        inactive
      />,
    );
    expect(screen.getByText("Inactive")).toBeInTheDocument();
  });
});
