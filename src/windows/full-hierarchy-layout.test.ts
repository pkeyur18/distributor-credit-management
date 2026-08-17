import { describe, expect, it } from "vitest";
import type { ChartNode } from "@/lib/ipc/entities";
import { computeFullTreeLayout, FT_GAP_X, FT_GAP_Y, FT_NODE_H, FT_NODE_W } from "./full-hierarchy-layout";

function node(memberId: number, introducerMemberId: number | null): ChartNode {
  return {
    memberId,
    name: `Member ${memberId}`,
    ownBusinessVolume: 0,
    isActive: true,
    introducerMemberId,
    slabPct: 0,
    rewards: 0,
    legCount: 0,
  };
}

// Root + N direct children (a flat, wide tree).
function flatTree(childCount: number): ChartNode[] {
  const nodes = [node(0, null)];
  for (let i = 1; i <= childCount; i++) nodes.push(node(i, 0));
  return nodes;
}

// A single unbranched chain of the given length beneath the root.
function chain(length: number): ChartNode[] {
  const nodes = [node(0, null)];
  for (let i = 1; i <= length; i++) nodes.push(node(i, i - 1));
  return nodes;
}

describe("computeFullTreeLayout", () => {
  it("is deterministic across runs on identical input", () => {
    const nodes = flatTree(5);
    const a = computeFullTreeLayout(nodes);
    const b = computeFullTreeLayout(nodes);
    expect([...a.positions]).toEqual([...b.positions]);
    expect(a.lines).toEqual(b.lines);
    expect(a.width).toBe(b.width);
    expect(a.height).toBe(b.height);
  });

  it("lays out a single node with nobody beneath it", () => {
    const layout = computeFullTreeLayout([node(0, null)]);
    expect(layout.positions.get(0)).toEqual({ x: 0, y: 0 });
    expect(layout.lines).toEqual([]);
    expect(layout.height).toBe(FT_NODE_H + FT_GAP_Y);
  });

  it("lays out a deep single chain without recursing (no stack overflow)", () => {
    const length = 5000;
    const layout = computeFullTreeLayout(chain(length));
    expect(layout.positions.size).toBe(length + 1);
    // Every node sits directly above the next: three connector segments per
    // edge (down, across, down), all vertical since parent and child share
    // the same x (the elbow's horizontal segment is zero-length).
    expect(layout.lines).toHaveLength(length * 3);
    for (const line of layout.lines) expect(line.x1).toBe(line.x2);
    expect(layout.positions.get(length)?.y).toBe(length * (FT_NODE_H + FT_GAP_Y));
  });

  it("centres a parent over its children", () => {
    const nodes = flatTree(3);
    const layout = computeFullTreeLayout(nodes);
    const c1 = layout.positions.get(1)!.x;
    const c3 = layout.positions.get(3)!.x;
    expect(layout.positions.get(0)!.x).toBe((c1 + c3) / 2);
    // Three leaves side by side, each NODE_W + GAP_X apart.
    expect(c3 - c1).toBe(2 * (FT_NODE_W + FT_GAP_X));
  });

  it("connects every non-root node to its introducer", () => {
    const nodes = flatTree(60);
    const layout = computeFullTreeLayout(nodes);
    // Three elbow segments (down, across, down) per edge.
    expect(layout.lines).toHaveLength(60 * 3);
  });

  it("routes an edge to an off-centre child as an elbow through the gap's midpoint", () => {
    const layout = computeFullTreeLayout(flatTree(3));
    const parentPos = layout.positions.get(0)!;
    const childPos = layout.positions.get(1)!; // leftmost leaf, off-centre from the parent
    const [down, across, drop] = layout.lines.slice(0, 3);
    const midY = parentPos.y + FT_NODE_H + FT_GAP_Y / 2;
    expect(down).toEqual({
      x1: parentPos.x + FT_NODE_W / 2,
      y1: parentPos.y + FT_NODE_H,
      x2: parentPos.x + FT_NODE_W / 2,
      y2: midY,
    });
    expect(across).toEqual({
      x1: parentPos.x + FT_NODE_W / 2,
      y1: midY,
      x2: childPos.x + FT_NODE_W / 2,
      y2: midY,
    });
    expect(drop).toEqual({
      x1: childPos.x + FT_NODE_W / 2,
      y1: midY,
      x2: childPos.x + FT_NODE_W / 2,
      y2: childPos.y,
    });
  });

  it("handles 25,000 members (NFR-2 ceiling) without error, wide or deep", () => {
    const wide = computeFullTreeLayout(flatTree(25000));
    expect(wide.positions.size).toBe(25001);

    const deep = computeFullTreeLayout(chain(25000));
    expect(deep.positions.size).toBe(25001);
  });
});
