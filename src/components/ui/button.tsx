import { Button as ButtonPrimitive } from "@base-ui/react/button";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

// DESIGN.md §Components / 07-design-system.md §6.1 — five variants only.
// No `outline`/`link`: those aren't in the system's button vocabulary.
const buttonVariants = cva(
  "inline-flex shrink-0 items-center justify-center gap-1.5 rounded-sm border border-transparent px-[13px] text-[13px] font-[550] whitespace-nowrap transition-[background,border-color,opacity] duration-100 outline-none select-none focus-visible:border-accent focus-visible:ring-3 focus-visible:ring-accent-weak disabled:pointer-events-none disabled:opacity-45 [&_svg]:size-3.5 [&_svg]:shrink-0",
  {
    variants: {
      // `size` is concatenated before `variant` deliberately: `commit`'s h-9
      // must win over the default size's h-8 when both apply, and cn()'s
      // tailwind-merge keeps whichever conflicting class comes last.
      size: {
        default: "h-8",
        sm: "h-[27px] px-[9px] text-[12.5px]",
      },
      variant: {
        primary: "bg-accent text-white hover:not-disabled:brightness-[1.08]",
        secondary: "border-border bg-surface text-ink hover:not-disabled:bg-bg",
        ghost: "text-muted-text hover:not-disabled:bg-bg hover:not-disabled:text-ink",
        danger:
          "border-border bg-surface text-danger hover:not-disabled:border-danger hover:not-disabled:bg-danger-weak",
        // The single irreversible action in the system (closing a month) —
        // weight communicates stakes, never a new colour (One Accent Rule).
        commit: "h-9 bg-accent px-4.5 font-bold text-white hover:not-disabled:brightness-[1.08]",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "default",
    },
  },
);

function Button({
  className,
  variant,
  size,
  ...props
}: ButtonPrimitive.Props & VariantProps<typeof buttonVariants>) {
  return (
    <ButtonPrimitive
      data-slot="button"
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  );
}

export { Button, buttonVariants };
