import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { MemberLevelTag, MembershipBadge } from "./membership-badge";

describe("MembershipBadge", () => {
  it("draws the rank's badge image at the requested size, as decoration", () => {
    render(<MembershipBadge rank={4} size={40} />);
    const img = screen.getByRole("presentation");
    expect(img.getAttribute("src")).toMatch(/elite/);
    expect(img).toHaveAttribute("alt", "");
    expect(img).toHaveAttribute("title", "Elite membership");
    expect(img).toHaveAttribute("width", "40");
  });

  it("draws nothing without a level", () => {
    const { container } = render(<MembershipBadge rank={0} size={40} />);
    expect(container).toBeEmptyDOMElement();
  });
});

describe("MemberLevelTag", () => {
  it("shows the badge and the level name", () => {
    render(<MemberLevelTag rank={1} />);
    expect(screen.getByRole("presentation").getAttribute("src")).toMatch(/gold/);
    expect(screen.getByText("Gold")).toBeInTheDocument();
  });

  it("shows nothing without a level", () => {
    const { container } = render(<MemberLevelTag rank={0} />);
    expect(container).toBeEmptyDOMElement();
  });
});
