import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

// 07-design-system.md §5 Large-Icon-Container Rule (~25% of box width) —
// the auth screens' single icon container, sized either as the plain
// 40px logo mark or the 52px semantic wizard icon (shield/lock/alert).
// Both concrete radii are the approved prototype's own values, not a
// live-computed formula: only these two sizes are ever used.
const SIZE = {
  40: "size-10 rounded-[11px]",
  52: "size-13 rounded-[14px]",
} as const;

const TONE = {
  accent: "bg-accent-weak text-accent",
  success: "bg-success-weak text-success",
  danger: "bg-danger-weak text-danger",
} as const;

interface AuthBrandMarkProps {
  size?: keyof typeof SIZE;
  tone?: keyof typeof TONE;
  className?: string;
  children: ReactNode;
}

function AuthBrandMark({ size = 52, tone = "accent", className, children }: AuthBrandMarkProps) {
  return (
    <div
      data-slot="auth-brand-mark"
      className={cn(
        "mx-auto flex items-center justify-center",
        SIZE[size],
        TONE[tone],
        className,
      )}
    >
      {children}
    </div>
  );
}

export { AuthBrandMark };
