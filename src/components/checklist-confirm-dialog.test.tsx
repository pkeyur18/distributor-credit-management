import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ChecklistConfirmDialog } from "./checklist-confirm-dialog";

function TestDialog({ onConfirm }: { onConfirm: () => void }) {
  const [open, setOpen] = useState(true);
  return (
    <ChecklistConfirmDialog
      open={open}
      onOpenChange={setOpen}
      title="Restore from backup"
      warning="This replaces everything currently in the console."
      checklistLabel="I understand this overwrites all current data and cannot be undone."
      confirmLabel="Restore"
      onConfirm={onConfirm}
    />
  );
}

describe("ChecklistConfirmDialog", () => {
  it("the danger action starts disabled until the checkbox is ticked", async () => {
    render(<TestDialog onConfirm={vi.fn()} />);
    const confirmButton = screen.getByRole("button", { name: "Restore" });
    expect(confirmButton).toBeDisabled();

    await userEvent.click(screen.getByRole("checkbox"));
    expect(confirmButton).toBeEnabled();
  });

  it("Cancel takes focus on open, not the danger action (07-design-system.md §6.6)", async () => {
    render(<TestDialog onConfirm={vi.fn()} />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus());
  });

  it("confirming calls onConfirm only once the checkbox is ticked", async () => {
    const onConfirm = vi.fn();
    render(<TestDialog onConfirm={onConfirm} />);

    await userEvent.click(screen.getByRole("checkbox"));
    await userEvent.click(screen.getByRole("button", { name: "Restore" }));

    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("unchecking the box after ticking it disables the action again", async () => {
    render(<TestDialog onConfirm={vi.fn()} />);
    const checkbox = screen.getByRole("checkbox");
    const confirmButton = screen.getByRole("button", { name: "Restore" });

    await userEvent.click(checkbox);
    expect(confirmButton).toBeEnabled();
    await userEvent.click(checkbox);
    expect(confirmButton).toBeDisabled();
  });
});
