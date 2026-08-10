import { RadioGroup } from "@base-ui/react/radio-group";
import { Radio } from "@base-ui/react/radio";

import { cn } from "@/lib/utils";

// 07-design-system.md §5/§6 — a mutually-exclusive value picker (PIN vs
// Password, backup schedule), not tabbed content, so it's built on
// RadioGroup/Radio rather than repurposing Tabs without panels.
// Control-lift shadow on the active segment (the system's only other
// shadow besides the modal one) and the Nested-Radius Rule: 5px inner
// radius for the 6px track with 2px padding.

interface SegmentedControlOption<Value extends string> {
  value: Value;
  label: string;
}

interface SegmentedControlProps<Value extends string> {
  value: Value;
  onValueChange: (value: Value) => void;
  options: SegmentedControlOption<Value>[];
  className?: string;
}

function SegmentedControl<Value extends string>({
  value,
  onValueChange,
  options,
  className,
}: SegmentedControlProps<Value>) {
  return (
    <RadioGroup
      value={value}
      onValueChange={(next) => onValueChange(next as Value)}
      className={cn("inline-flex rounded-sm border border-border bg-bg p-0.5", className)}
    >
      {options.map((option) => (
        <Radio.Root
          key={option.value}
          value={option.value}
          className={cn(
            "cursor-pointer rounded-[5px] px-3.25 py-1.5 text-[12.5px] font-[550] text-muted-text",
            "data-checked:bg-surface data-checked:text-ink data-checked:shadow-(--shadow-control-lift)",
          )}
        >
          {option.label}
        </Radio.Root>
      ))}
    </RadioGroup>
  );
}

export { SegmentedControl };
