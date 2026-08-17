import { Delete } from "lucide-react";

import { cn } from "@/lib/utils";

// The onscreen numeric keypad `PinDots` deliberately excludes (its own doc
// comment: "the keypad and buffer... belong to the actual auth screens").
// US-M8.1/M8.2's Login/Locked screens are those screens.
const KEYS = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "", "0", "⌫"] as const;

interface PinKeypadProps {
  onPress: (digit: string) => void;
  onBackspace: () => void;
  disabled?: boolean;
  className?: string;
}

function PinKeypad({ onPress, onBackspace, disabled, className }: PinKeypadProps) {
  return (
    <div className={cn("grid grid-cols-3 gap-2", className)}>
      {KEYS.map((key, i) => {
        if (key === "") return <div key={i} aria-hidden="true" />;
        if (key === "⌫") {
          return (
            <button
              key={i}
              type="button"
              disabled={disabled}
              onClick={onBackspace}
              aria-label="Backspace"
              className="flex h-11 items-center justify-center rounded-sm border border-border bg-surface text-ink hover:not-disabled:bg-bg disabled:opacity-45"
            >
              <Delete className="size-4" />
            </button>
          );
        }
        return (
          <button
            key={i}
            type="button"
            disabled={disabled}
            onClick={() => onPress(key)}
            className="h-11 rounded-sm border border-border bg-surface text-title-sm hover:not-disabled:bg-bg disabled:opacity-45"
          >
            {key}
          </button>
        );
      })}
    </div>
  );
}

export { PinKeypad };
