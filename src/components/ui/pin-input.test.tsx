import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";

import { PinDots } from "./pin-input";

describe("PinDots", () => {
  it("fills exactly the entered count, leaving the rest empty", () => {
    const { container } = render(<PinDots length={6} filledCount={3} />);
    const dots = container.querySelectorAll("[data-slot='pin-dots'] > div");
    expect(dots).toHaveLength(6);
    expect(
      Array.from(dots).filter((d) => d.className.includes("bg-accent")),
    ).toHaveLength(3);
  });

  it("marks every dot with the error border when a login attempt failed", () => {
    const { container } = render(<PinDots length={6} filledCount={0} error />);
    const dots = container.querySelectorAll("[data-slot='pin-dots'] > div");
    expect(Array.from(dots).every((d) => d.className.includes("border-danger"))).toBe(true);
  });
});
