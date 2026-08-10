import type { ReactNode } from "react";
import { RadioGroup } from "@base-ui/react/radio-group";
import { Radio } from "@base-ui/react/radio";

import { cn } from "@/lib/utils";

// 07-design-system.md §6.10. One component for every kind of backup —
// the month-close list and the whole-console backup list both use this
// exact shape, only the primary/provenance text differs (§Reused, not
// duplicated, 7 Aug 2026). Selection reuses the input-focus treatment
// (border-accent + 3px accent-weak ring) rather than inventing a second
// selection language.
interface RestoreOption<Value extends string> {
  value: Value;
  /** Names the thing in the operator's own terms — the month a backup
   * holds, or when it was taken — never a filename. */
  primary: ReactNode;
  /** Provenance: version, whether it was corrected. */
  provenance?: ReactNode;
}

interface RestoreOptionListProps<Value extends string> {
  value: Value | null;
  onValueChange: (value: Value) => void;
  options: RestoreOption<Value>[];
  className?: string;
}

function RestoreOptionList<Value extends string>({
  value,
  onValueChange,
  options,
  className,
}: RestoreOptionListProps<Value>) {
  return (
    <RadioGroup
      value={value}
      onValueChange={(next) => onValueChange(next as Value)}
      className={cn("flex flex-col gap-2", className)}
    >
      {options.map((option) => (
        <Radio.Root
          key={option.value}
          value={option.value}
          className={cn(
            "group flex w-full cursor-pointer items-start gap-2.5 rounded-sm border border-border bg-surface px-3 py-2.5 text-left",
            "hover:bg-bg",
            "data-checked:border-accent data-checked:bg-surface data-checked:ring-3 data-checked:ring-accent-weak",
          )}
        >
          <span className="mt-0.5 flex size-3.75 shrink-0 items-center justify-center rounded-full border border-border group-data-checked:border-accent">
            <Radio.Indicator className="size-1.75 rounded-full bg-accent" />
          </span>
          <span className="min-w-0 flex-1">
            <span className="block text-[13px] font-semibold text-ink">{option.primary}</span>
            {option.provenance && (
              <span className="block text-[11.5px] text-muted-text">{option.provenance}</span>
            )}
          </span>
        </Radio.Root>
      ))}
    </RadioGroup>
  );
}

export { RestoreOptionList };
