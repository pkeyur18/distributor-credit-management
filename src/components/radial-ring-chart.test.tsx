import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

import { RadialRingChart } from "./radial-ring-chart";

// See column-chart.test.tsx — reduced motion resolves the chart to its
// final state synchronously, no rAF wait needed.
beforeEach(() => {
  vi.stubGlobal(
    "matchMedia",
    vi.fn().mockReturnValue({ matches: true, addEventListener: vi.fn(), removeEventListener: vi.fn() }),
  );
});
afterEach(() => vi.unstubAllGlobals());

describe("RadialRingChart", () => {
  it("renders the grand total in the center and each slab in the legend", () => {
    render(
      <RadialRingChart
        rows={[
          { id: 0, label: "0%", target: 100, tint: "red" },
          { id: 2, label: "2%", target: 2530, tint: "blue" },
        ]}
        format={(v) => String(v)}
        totalLabel="this period"
      />,
    );
    expect(screen.getByText("2630")).toBeInTheDocument(); // center total, distinct from any single row
    expect(screen.getByText("this period")).toBeInTheDocument();
    expect(screen.getByText("0%")).toBeInTheDocument();
    expect(screen.getByText("2%")).toBeInTheDocument();
    expect(screen.getByText("2530")).toBeInTheDocument();
  });

  it("gives every slab's arc a share of the ring proportional to its value", () => {
    const { container } = render(
      <RadialRingChart
        rows={[
          { id: 0, label: "0%", target: 25, tint: "red" },
          { id: 2, label: "2%", target: 75, tint: "blue" },
        ]}
        format={(v) => String(v)}
        totalLabel="members"
      />,
    );
    const arcs = container.querySelectorAll("circle[stroke='red'], circle[stroke='blue']");
    const [first, second] = [...arcs].map((c) => parseFloat(c.getAttribute("stroke-dasharray") ?? "0"));
    expect(second).toBeGreaterThan(first);
  });
});
