import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Button } from "./button";

describe("Button", () => {
  it("defaults to the primary variant", () => {
    render(<Button>Save</Button>);
    expect(screen.getByRole("button", { name: "Save" })).toHaveClass("bg-accent");
  });

  it("does not fire onClick when disabled", async () => {
    const onClick = vi.fn();
    render(
      <Button disabled onClick={onClick}>
        Close month
      </Button>,
    );
    await userEvent.click(screen.getByRole("button", { name: "Close month" }));
    expect(onClick).not.toHaveBeenCalled();
  });

  it("commit variant is taller and bolder than a routine primary", () => {
    render(<Button variant="commit">Close month</Button>);
    expect(screen.getByRole("button", { name: "Close month" })).toHaveClass("h-9", "font-bold");
  });
});
