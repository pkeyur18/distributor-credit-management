import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { SearchResultsList } from "./search-results-list";
import type { SearchResult } from "@/lib/ipc/entities";

function result(overrides: Partial<SearchResult> = {}): SearchResult {
  return {
    id: 482913,
    name: "Asha Patel",
    phone: "9876543210",
    totalBusinessVolume: 0,
    slabPct: 0,
    isActive: true,
    email: null,
    address: "1 Main Street",
    introducerMemberId: 100001,
    ...overrides,
  };
}

describe("SearchResultsList", () => {
  it("renders nothing when the query is empty", () => {
    const { container } = render(
      <SearchResultsList results={[]} query="" onSelect={() => {}} />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("shows the empty label when a non-empty query has no matches", () => {
    render(<SearchResultsList results={[]} query="zzz" onSelect={() => {}} />);
    expect(screen.getByText("No members match")).toBeInTheDocument();
  });

  it("renders name, member number and phone for each result", () => {
    render(
      <SearchResultsList results={[result()]} query="asha" onSelect={() => {}} />,
    );
    expect(screen.getByText("Asha Patel")).toBeInTheDocument();
    expect(screen.getByText("#482913 · 9876543210")).toBeInTheDocument();
  });

  it("marks an inactive result with the Inactive pill, colour plus label", () => {
    render(
      <SearchResultsList
        results={[result({ isActive: false })]}
        query="asha"
        onSelect={() => {}}
      />,
    );
    expect(screen.getByText("Inactive")).toBeInTheDocument();
  });

  it("calls onSelect with the clicked result", () => {
    const onSelect = vi.fn();
    render(
      <SearchResultsList results={[result()]} query="asha" onSelect={onSelect} />,
    );
    fireEvent.click(screen.getByText("Asha Patel"));
    expect(onSelect).toHaveBeenCalledWith(result());
  });
});
