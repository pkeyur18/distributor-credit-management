import type { ReactNode } from "react";
import { Toast as ToastPrimitive } from "@base-ui/react/toast";
import { CheckCircle2, Info, XCircle } from "lucide-react";

import { cn } from "@/lib/utils";

// 07-design-system.md §6.7. Toasts confirm; they never carry anything the
// operator must act on — anything actionable belongs in a banner or a
// modal, which don't disappear on their own. Built on @base-ui/react's
// Toast so the ~3.4s auto-dismiss, swipe-to-dismiss and aria-live plumbing
// come from the primitive.

// `@base-ui/react/toast` only exports the `Toast` namespace at runtime
// (its top-level `useToastManager` re-export is type-only) — pull the hook
// off the namespace instead of importing it directly.
const { useToastManager } = ToastPrimitive;

// The DESIGN.md-specified lifetime — the primitive's own default is 5000ms.
const TOAST_TIMEOUT_MS = 3400;

function ToastProvider({ children }: { children: ReactNode }) {
  return <ToastPrimitive.Provider timeout={TOAST_TIMEOUT_MS}>{children}</ToastPrimitive.Provider>;
}

const TOAST_ICON: Record<string, typeof Info> = {
  success: CheckCircle2,
  danger: XCircle,
};

const TOAST_TONE: Record<string, string> = {
  success: "bg-success text-white",
  danger: "bg-danger text-white",
};

function Toaster() {
  const { toasts } = useToastManager();
  return (
    <ToastPrimitive.Portal>
      <ToastPrimitive.Viewport className="fixed right-5 bottom-5 z-50 flex flex-col items-end gap-2">
        {toasts.map((toast) => {
          const Icon = TOAST_ICON[toast.type ?? ""] ?? Info;
          return (
            <ToastPrimitive.Root
              key={toast.id}
              toast={toast}
              className={cn(
                "flex max-w-85 items-center gap-2 rounded-sm px-3.5 py-2.5 text-[12.5px]",
                "shadow-(--shadow-modal)",
                "data-starting-style:translate-y-1.5 data-starting-style:opacity-0",
                "data-ending-style:opacity-0",
                "transition-[transform,opacity] duration-150",
                TOAST_TONE[toast.type ?? ""] ?? "bg-ink text-bg",
              )}
            >
              <Icon aria-hidden="true" className="size-3.75 shrink-0" />
              <ToastPrimitive.Title className="flex-1" />
            </ToastPrimitive.Root>
          );
        })}
      </ToastPrimitive.Viewport>
    </ToastPrimitive.Portal>
  );
}

export { ToastProvider, Toaster, useToastManager as useToast };
