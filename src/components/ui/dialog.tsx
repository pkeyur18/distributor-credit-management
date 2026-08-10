import { createContext, useContext, useRef, type ComponentProps, type ReactNode } from "react";
import { Dialog as DialogPrimitive } from "@base-ui/react/dialog";
import { X } from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "./button";

// 07-design-system.md §6.6. Built on @base-ui/react's Dialog so focus-trap,
// role="dialog"/aria-modal/aria-labelledby and the escape/outside-press
// event plumbing come from the primitive, not hand-rolled.

const CancelFocusContext = createContext<React.RefObject<HTMLElement | null> | null>(null);

interface ModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Non-dismissable modals (add/edit member) opt out of Escape and
   * backdrop-click; only Cancel or ✕ can close them. */
  dismissable?: boolean;
  wide?: boolean;
  children: ReactNode;
}

function Modal({ open, onOpenChange, dismissable = true, wide, children }: ModalProps) {
  const cancelRef = useRef<HTMLElement>(null);

  return (
    <DialogPrimitive.Root
      open={open}
      disablePointerDismissal={!dismissable}
      onOpenChange={(next, details) => {
        // Runtime reason values are kebab-case string literals (e.g.
        // `escape-key`), not the TS enum member names (`escapeKey`) —
        // confirmed both empirically and by the Dialog Root's own change
        // -reason union, which doesn't include `close-watcher` here.
        if (!dismissable && details.reason === "escape-key") {
          details.cancel();
          return;
        }
        onOpenChange(next);
      }}
    >
      <DialogPrimitive.Portal>
        <DialogPrimitive.Backdrop className="fixed inset-0 z-50 bg-ink/50 backdrop-blur-[1px] data-starting-style:opacity-0 data-ending-style:opacity-0 transition-opacity duration-(--motion-modal-duration)" />
        <DialogPrimitive.Popup
          initialFocus={cancelRef}
          className={cn(
            "fixed top-1/2 left-1/2 z-50 flex max-h-[88vh] w-[calc(100%-32px)] -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-lg border border-border bg-surface",
            "shadow-(--shadow-modal)",
            "data-starting-style:translate-y-[calc(-50%+6px)] data-starting-style:scale-98 data-starting-style:opacity-0",
            "data-ending-style:translate-y-[calc(-50%+6px)] data-ending-style:scale-98 data-ending-style:opacity-0",
            "transition-[transform,opacity] duration-(--motion-modal-duration)",
            wide ? "max-w-160" : "max-w-120",
          )}
        >
          <CancelFocusContext.Provider value={cancelRef}>{children}</CancelFocusContext.Provider>
        </DialogPrimitive.Popup>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}

function ModalHeader({ title }: { title: string }) {
  return (
    <div className="flex items-center justify-between gap-3 border-b border-border px-4.5 py-3.5">
      <DialogPrimitive.Title className="text-title">{title}</DialogPrimitive.Title>
      <DialogPrimitive.Close
        aria-label="Close"
        className="text-muted-text hover:text-ink flex size-6 items-center justify-center rounded-sm hover:bg-bg"
      >
        <X className="size-4" />
      </DialogPrimitive.Close>
    </div>
  );
}

function ModalBody({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      data-slot="modal-body"
      className={cn("overflow-y-auto px-4.5 py-4", className)}
      {...props}
    />
  );
}

function ModalFooter({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      data-slot="modal-footer"
      className={cn(
        "flex items-center justify-end gap-2 border-t border-border px-4.5 py-3.5",
        className,
      )}
      {...props}
    />
  );
}

// Cancel-first, always present, and always the modal's initial focus target
// — never the confirming action (07-design-system.md §6.6).
function ModalCancel({ children = "Cancel", ...props }: ComponentProps<typeof Button>) {
  const ref = useContext(CancelFocusContext);
  return (
    <DialogPrimitive.Close
      render={
        <Button ref={ref as React.Ref<HTMLButtonElement>} variant="secondary" {...props}>
          {children}
        </Button>
      }
    />
  );
}

export { Modal, ModalHeader, ModalBody, ModalFooter, ModalCancel };
