import type { ReactNode } from "react";

import { Modal, ModalBody, ModalCancel, ModalFooter, ModalHeader } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

// A small reusable confirm/refuse pattern (deactivate, reactivate, and
// whatever else needs a one-step "are you sure" later) — Cancel first,
// Cancel takes focus, matching every other modal in the system rather than
// inventing a second confirmation language.
interface ConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  body: ReactNode;
  confirmLabel: string;
  danger?: boolean;
  busy?: boolean;
  onConfirm: () => void;
}

function ConfirmDialog({
  open,
  onOpenChange,
  title,
  body,
  confirmLabel,
  danger,
  busy,
  onConfirm,
}: ConfirmDialogProps) {
  return (
    <Modal open={open} onOpenChange={onOpenChange}>
      <ModalHeader title={title} />
      <ModalBody>
        <div className="text-body">{body}</div>
      </ModalBody>
      <ModalFooter>
        <ModalCancel />
        <Button variant={danger ? "danger" : "primary"} disabled={busy} onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </ModalFooter>
    </Modal>
  );
}

export { ConfirmDialog };
