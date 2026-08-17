import { cn } from "@/lib/utils";

// 07-design-system.md §5/§6 — the PIN dots indicator only. The keypad and
// buffer that drive `filledCount` belong to the actual auth screens
// (US-M8.1/US-M8.2, a later sprint), not this component.
interface PinDotsProps {
  length?: number;
  filledCount: number;
  error?: boolean;
  className?: string;
}

function PinDots({ length = 6, filledCount, error, className }: PinDotsProps) {
  return (
    <div
      data-slot="pin-dots"
      role="status"
      aria-label={`${filledCount} of ${length} digits entered`}
      className={cn("mb-5.5 flex justify-center gap-3", className)}
    >
      {Array.from({ length }, (_, i) => (
        <div
          key={i}
          className={cn(
            "size-3.25 rounded-full border-[1.5px]",
            i < filledCount ? "border-accent bg-accent" : "border-border",
            error && "border-danger",
          )}
        />
      ))}
    </div>
  );
}

export { PinDots };
