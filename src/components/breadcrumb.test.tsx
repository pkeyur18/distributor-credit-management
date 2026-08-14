import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";

import { Breadcrumb } from "./breadcrumb";

describe("Breadcrumb", () => {
  it("renders nothing when there is no back label and no crumbs", () => {
    const { container } = render(
      <MemoryRouter>
        <Breadcrumb backLabel={undefined} onBack={() => {}} crumbs={[]} />
      </MemoryRouter>,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("renders the back link and calls onBack when clicked", () => {
    const onBack = vi.fn();
    render(
      <MemoryRouter>
        <Breadcrumb backLabel="Home" onBack={onBack} crumbs={[]} />
      </MemoryRouter>,
    );
    screen.getByText("Back to Home").click();
    expect(onBack).toHaveBeenCalled();
  });

  it("renders every crumb, only the last one non-clickable", () => {
    render(
      <MemoryRouter>
        <Breadcrumb
          backLabel={undefined}
          onBack={() => {}}
          crumbs={[
            { label: "Root", to: "/structure/1" },
            { label: "Child", to: "/structure/2" },
            { label: "Grandchild" },
          ]}
        />
      </MemoryRouter>,
    );
    expect(screen.getAllByRole("link")).toHaveLength(2);
    expect(screen.getByText("Grandchild").closest("a")).toBeNull();
  });
});
