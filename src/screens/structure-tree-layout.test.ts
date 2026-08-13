import { describe, expect, it } from "vitest";
import { computeParentChildConnectorLines, type Box } from "./structure-tree-layout";

function box(x: number, y: number, width = 172, height = 96): Box {
  return { x, y, width, height };
}

describe("computeParentChildConnectorLines", () => {
  it("returns nothing for a leaf (no children)", () => {
    expect(computeParentChildConnectorLines(box(0, 0), [])).toEqual([]);
  });

  it("draws stem, bar and one drop per child when the parent centres over its children", () => {
    const parent = box(100, 0);
    const children = [box(0, 150), box(100, 150), box(200, 150)];
    const lines = computeParentChildConnectorLines(parent, children);

    // stem: parent bottom-centre down to the trunk
    expect(lines[0]).toEqual({ x1: 186, y1: 96, x2: 186, y2: 136 });
    // bar spans the outermost children's centres, at the trunk
    expect(lines[1]).toEqual({ x1: 86, y1: 136, x2: 286, y2: 136 });
    // one drop per child, from the trunk to that child's top
    expect(lines.slice(2)).toEqual([
      { x1: 86, y1: 136, x2: 86, y2: 150 },
      { x1: 186, y1: 136, x2: 186, y2: 150 },
      { x1: 286, y1: 136, x2: 286, y2: 150 },
    ]);
  });

  it("widens the bar to reach the parent's stem when the parent sits outside its children's span", () => {
    // Rows centre independently, so a parent can land to one side of its own children.
    const parent = box(400, 0);
    const children = [box(0, 150), box(100, 150)];
    const lines = computeParentChildConnectorLines(parent, children);
    const bar = lines[1];
    expect(bar.x1).toBe(86); // leftmost child centre
    expect(bar.x2).toBe(486); // parent centre, not a child centre
  });

  it("skips the bar for a single child (nothing to span)", () => {
    const lines = computeParentChildConnectorLines(box(100, 0), [box(100, 150)]);
    // stem + single drop only, no zero-length bar
    expect(lines).toHaveLength(2);
  });
});
