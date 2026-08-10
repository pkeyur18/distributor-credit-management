import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { ImpactRow, ImpactSummary, ImpactValue } from "./impact-summary";

describe("ImpactValue", () => {
  it("unchanged state shows one figure plus a muted 'unchanged', never a before/after pair", () => {
    render(
      <ImpactSummary>
        <ImpactRow label="Rewards this month">
          <ImpactValue before={980} after={980} changed={false} />
        </ImpactRow>
      </ImpactSummary>,
    );
    expect(screen.getByText("unchanged")).toBeInTheDocument();
    expect(screen.queryByText("→")).not.toBeInTheDocument();
    // The figure must appear exactly once, not twice either side of an arrow.
    expect(screen.getAllByText("980")).toHaveLength(1);
  });

  it("changed state shows before → after", () => {
    render(
      <ImpactSummary>
        <ImpactRow label="Rewards this month">
          <ImpactValue before={980} after={1120} changed={true} />
        </ImpactRow>
      </ImpactSummary>,
    );
    expect(screen.getByText("980")).toBeInTheDocument();
    expect(screen.getByText("1120")).toBeInTheDocument();
    expect(screen.getByText("→")).toBeInTheDocument();
  });
});
