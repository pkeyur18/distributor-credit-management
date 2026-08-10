import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Modal, ModalBody, ModalCancel, ModalFooter, ModalHeader } from "./dialog";
import { Button } from "./button";

function TestModal({ dismissable = true, onClose }: { dismissable?: boolean; onClose?: () => void }) {
  const [open, setOpen] = useState(true);
  return (
    <Modal
      open={open}
      dismissable={dismissable}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) onClose?.();
      }}
    >
      <ModalHeader title="Add member" />
      <ModalBody>Body content</ModalBody>
      <ModalFooter>
        <ModalCancel />
        <Button variant="primary">Save</Button>
      </ModalFooter>
    </Modal>
  );
}

describe("Modal", () => {
  it("focuses Cancel on open, never the confirming action", async () => {
    render(<TestModal />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus());
  });

  it("a dismissable modal closes on Escape", async () => {
    const onClose = vi.fn();
    render(<TestModal onClose={onClose} />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus());
    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(onClose).toHaveBeenCalled());
  });

  it("a non-dismissable modal (add/edit member) ignores Escape", async () => {
    const onClose = vi.fn();
    render(<TestModal dismissable={false} onClose={onClose} />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus());
    await userEvent.keyboard("{Escape}");
    expect(onClose).not.toHaveBeenCalled();
  });

  it("Cancel still closes a non-dismissable modal", async () => {
    const onClose = vi.fn();
    render(<TestModal dismissable={false} onClose={onClose} />);
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
  });
});
