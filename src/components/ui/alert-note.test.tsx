import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { AlertNote } from "./alert-note";

describe("AlertNote", () => {
  it("warn copy uses the AA-contrast warning-text step, never the plain warning colour", () => {
    render(<AlertNote variant="warn">Closed months are never affected.</AlertNote>);
    expect(screen.getByText("Closed months are never affected.")).toHaveClass("text-warning-text");
  });

  it("danger copy uses the plain danger colour", () => {
    render(<AlertNote variant="danger">Enter an amount greater than zero.</AlertNote>);
    expect(screen.getByText("Enter an amount greater than zero.")).toHaveClass("text-danger");
  });
});
