import { type ComponentProps } from "react";

import { cn } from "@/lib/utils";

// 07-design-system.md §6.4. A plain native <input> covers everything the
// spec asks for — no need for @base-ui/react's Field-bound Input primitive
// when there's no Field context here.
function Input({ className, ...props }: ComponentProps<"input">) {
  return (
    <input
      data-slot="input"
      className={cn(
        "h-[34px] w-full rounded-sm border border-border bg-surface px-[11px] text-body text-ink outline-none placeholder:text-[12.5px] placeholder:text-muted-text/60",
        "focus:border-accent focus:ring-3 focus:ring-accent-weak",
        "aria-invalid:border-danger",
        "disabled:cursor-not-allowed disabled:bg-bg disabled:text-muted-text",
        className,
      )}
      {...props}
    />
  );
}

// The 11.5px hint line below a field — danger-coloured when it's a
// validation error, muted otherwise.
function InputHint({
  className,
  error,
  ...props
}: ComponentProps<"p"> & { error?: boolean }) {
  return (
    <p
      data-slot="input-hint"
      className={cn("text-[11.5px]", error ? "text-danger" : "text-muted-text", className)}
      {...props}
    />
  );
}

export { Input, InputHint };
