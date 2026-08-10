import { useState } from "react";
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SegmentedControl } from "./segmented-control";

function Fixture() {
  const [mode, setMode] = useState<"pin" | "password">("pin");
  return (
    <SegmentedControl
      value={mode}
      onValueChange={setMode}
      options={[
        { value: "pin", label: "PIN" },
        { value: "password", label: "Password" },
      ]}
    />
  );
}

describe("SegmentedControl", () => {
  it("switches the checked segment on click", async () => {
    render(<Fixture />);
    expect(screen.getByRole("radio", { name: "PIN" })).toHaveAttribute("aria-checked", "true");

    await userEvent.click(screen.getByRole("radio", { name: "Password" }));

    expect(screen.getByRole("radio", { name: "Password" })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: "PIN" })).toHaveAttribute("aria-checked", "false");
  });
});
