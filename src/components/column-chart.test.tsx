import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ColumnChart } from "./column-chart";

// prefers-reduced-motion:true short-circuits useElapsedSinceActive to the
// fully-revealed state synchronously (see use-chart-reveal.ts), so these
// assertions don't need to wait on requestAnimationFrame at all.
beforeEach(() => {
  vi.stubGlobal(
    "matchMedia",
    vi.fn().mockReturnValue({ matches: true, addEventListener: vi.fn(), removeEventListener: vi.fn() }),
  );
});
afterEach(() => vi.unstubAllGlobals());

describe("ColumnChart", () => {
  it("renders each row's slab label and formatted value", () => {
    render(
      <ColumnChart
        rows={[
          { id: 0, label: "0%", target: 34, tint: "red" },
          { id: 2, label: "2%", target: 58, tint: "blue" },
        ]}
        format={(v) => String(v)}
      />,
    );
    expect(screen.getByText("0%")).toBeInTheDocument();
    expect(screen.getByText("2%")).toBeInTheDocument();
    expect(screen.getByText("34")).toBeInTheDocument();
    expect(screen.getByText("58")).toBeInTheDocument();
  });

  it("gives every column at least a sliver of height, even a zero-value one", () => {
    const { container } = render(
      <ColumnChart
        rows={[
          { id: 0, label: "0%", target: 0, tint: "red" },
          { id: 2, label: "2%", target: 100, tint: "blue" },
        ]}
        format={(v) => String(v)}
      />,
    );
    const bars = container.querySelectorAll("[style*='linear-gradient']");
    expect((bars[0] as HTMLElement).style.height).not.toBe("0%");
  });

  it("makes each non-empty column a button that reports its slab when onSelect is given", async () => {
    const onSelect = vi.fn();
    render(
      <ColumnChart
        rows={[
          { id: 0, label: "0%", target: 0, tint: "red" },
          { id: 14, label: "14%", target: 5, tint: "blue" },
        ]}
        format={(v) => String(v)}
        onSelect={onSelect}
      />,
    );
    expect(screen.getByRole("button", { name: /0% slab/ })).toBeDisabled();
    await userEvent.click(screen.getByRole("button", { name: /14% slab/ }));
    expect(onSelect).toHaveBeenCalledWith(14);
  });
});
