import { Modal, ModalBody, ModalCancel, ModalFooter, ModalHeader } from "@/components/ui/dialog";
import { AlertNote } from "@/components/ui/alert-note";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import type { ReactNode } from "react";

// 07-design-system.md §6.11 — the month-close wizard's own checklist
// pattern, built here first since M7.4's restore confirmation needs it
// before that wizard exists (S11). A `.modal-warn` note naming what will
// be replaced, one checklist checkbox, Cancel first, then a
// disabled-until-checked danger action. General enough that the wizard can
// reuse this component verbatim rather than inventing a second version of
// the same weight.
interface ChecklistConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  warning: ReactNode;
  checklistLabel: ReactNode;
  confirmLabel: string;
  busy?: boolean;
  onConfirm: () => void;
}

function ChecklistConfirmDialog({
  open,
  onOpenChange,
  title,
  warning,
  checklistLabel,
  confirmLabel,
  busy,
  onConfirm,
}: ChecklistConfirmDialogProps) {
  const [checked, setChecked] = useState(false);

  return (
    <Modal
      open={open}
      onOpenChange={(next) => {
        if (!next) setChecked(false);
        onOpenChange(next);
      }}
    >
      <ModalHeader title={title} />
      <ModalBody>
        <AlertNote variant="warn">{warning}</AlertNote>
        <label className="mt-3 flex items-start gap-2 text-body">
          <input
            type="checkbox"
            className="mt-0.5"
            checked={checked}
            onChange={(e) => setChecked(e.target.checked)}
          />
          <span>{checklistLabel}</span>
        </label>
      </ModalBody>
      <ModalFooter>
        <ModalCancel />
        <Button variant="danger" disabled={!checked || busy} onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </ModalFooter>
    </Modal>
  );
}

export { ChecklistConfirmDialog };
