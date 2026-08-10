import { useState } from "react";
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { RestoreOptionList } from "./restore-option-list";

function Fixture() {
  const [value, setValue] = useState<string | null>("2026-06");
  return (
    <RestoreOptionList
      value={value}
      onValueChange={setValue}
      options={[
        { value: "2026-06", primary: "June 2026", provenance: "Version 1" },
        { value: "2026-05", primary: "May 2026 (corrected)", provenance: "Version 2" },
      ]}
    />
  );
}

describe("RestoreOptionList", () => {
  it("selects exactly one option at a time, provenance shown for each", () => {
    render(<Fixture />);
    expect(screen.getByRole("radio", { name: /^June 2026/ })).toHaveAttribute(
      "aria-checked",
      "true",
    );
    expect(screen.getByText("Version 1")).toBeInTheDocument();
  });

  it("clicking another option moves the selection", async () => {
    render(<Fixture />);
    await userEvent.click(screen.getByRole("radio", { name: /^May 2026 \(corrected\)/ }));
    expect(screen.getByRole("radio", { name: /^May 2026 \(corrected\)/ })).toHaveAttribute(
      "aria-checked",
      "true",
    );
    expect(screen.getByRole("radio", { name: /^June 2026/ })).toHaveAttribute(
      "aria-checked",
      "false",
    );
  });
});
