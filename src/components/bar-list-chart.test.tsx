import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { BarListChart } from "./bar-list-chart";

describe("BarListChart", () => {
  it("renders each row's label and value", () => {
    render(
      <BarListChart
        rows={[
          { id: 1, label: "8%", value: 42, fraction: 1 },
          { id: 2, label: "12%", value: 17, fraction: 0.4 },
        ]}
      />,
    );
    expect(screen.getByText("8%")).toBeInTheDocument();
    expect(screen.getByText("42")).toBeInTheDocument();
    expect(screen.getByText("12%")).toBeInTheDocument();
    expect(screen.getByText("17")).toBeInTheDocument();
  });

  it("clamps an out-of-range fraction so a bar never overflows or inverts its track", () => {
    const { container } = render(
      <BarListChart rows={[{ id: 1, label: "8%", value: 42, fraction: 1.4 }]} />,
    );
    const fill = container.querySelector("[style*='scaleX']") as HTMLElement;
    expect(fill.style.transform).toBe("scaleX(1)");
  });

  it("falls back to the accent colour when a row carries no tint", () => {
    const { container } = render(
      <BarListChart rows={[{ id: 1, label: "8%", value: 42, fraction: 0.5 }]} />,
    );
    const fill = container.querySelector("[style*='scaleX']") as HTMLElement;
    expect(fill.style.background).toBe("var(--accent)");
  });
});
