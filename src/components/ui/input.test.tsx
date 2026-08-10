import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Input, InputHint } from "./input";

describe("Input", () => {
  it("accepts typed text", async () => {
    render(<Input aria-label="Business Volume" />);
    const field = screen.getByLabelText("Business Volume");
    await userEvent.type(field, "1250.50");
    expect(field).toHaveValue("1250.50");
  });

  it("disabled field rejects input", async () => {
    render(<Input aria-label="Business Volume" disabled />);
    const field = screen.getByLabelText("Business Volume");
    await userEvent.type(field, "5");
    expect(field).toHaveValue("");
  });
});

describe("InputHint", () => {
  it("switches to danger colour on error, muted otherwise", () => {
    const { rerender } = render(<InputHint>Name is required</InputHint>);
    expect(screen.getByText("Name is required")).toHaveClass("text-muted-text");

    rerender(<InputHint error>Name is required</InputHint>);
    expect(screen.getByText("Name is required")).toHaveClass("text-danger");
  });
});
