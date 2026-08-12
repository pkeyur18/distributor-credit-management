import type { ChartNode } from "@/lib/ipc/entities";

// T-M4.3-5 (V4.5): above this many descendants, Structure's "View full
// hierarchy" action gates behind a confirmation naming the exact count.
export const FULL_TREE_GATE = 60;

// T-M4.3-4 — ported from the approved prototype's `FT` constants
// (ui-prototype-v2.html, "FULL HIERARCHY WINDOW"). NODE_W/NODE_H match
// StructureTreeNode's non-root card size (w-43 = 172px); the root card
// renders wider (w-47.5 = 190px) but the prototype's own layout math never
// special-cases it, so neither does this port.
export const FT_NODE_W = 172;
export const FT_NODE_H = 96;
export const FT_GAP_X = 18;
export const FT_GAP_Y = 46;

export interface FullTreePosition {
  x: number;
  y: number;
}

export interface FullTreeLine {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

export interface FullTreeLayout {
  positions: Map<number, FullTreePosition>;
  lines: FullTreeLine[];
  width: number;
  height: number;
}

interface Frame {
  id: number;
  depth: number;
  kids: ChartNode[];
  idx: number;
  childXs: number[];
}

// T-M4.3-4: a single post-order pass over the already-fetched node list,
// computing every position up front — never measured back out of the
// rendered DOM (that's the main Structure screen's approach, forbidden
// here specifically because this screen's node count is unbounded). An
// explicit stack, not recursion: a 25,000-node single chain (TEST-R45)
// would blow the JS call stack at that depth if this recursed.
export function computeFullTreeLayout(nodes: ChartNode[]): FullTreeLayout {
  const positions = new Map<number, FullTreePosition>();
  if (nodes.length === 0) {
    return { positions, lines: [], width: FT_NODE_W, height: FT_NODE_H };
  }

  const childrenByParent = new Map<number, ChartNode[]>();
  for (const n of nodes) {
    if (n.introducerMemberId == null) continue;
    const kids = childrenByParent.get(n.introducerMemberId);
    if (kids) kids.push(n);
    else childrenByParent.set(n.introducerMemberId, [n]);
  }

  const root = nodes[0];
  let cursor = 0;
  let maxDepth = 0;
  const stack: Frame[] = [
    { id: root.memberId, depth: 0, kids: childrenByParent.get(root.memberId) ?? [], idx: 0, childXs: [] },
  ];

  while (stack.length > 0) {
    const frame = stack[stack.length - 1];
    if (frame.depth > maxDepth) maxDepth = frame.depth;

    if (frame.idx < frame.kids.length) {
      const child = frame.kids[frame.idx];
      frame.idx++;
      stack.push({
        id: child.memberId,
        depth: frame.depth + 1,
        kids: childrenByParent.get(child.memberId) ?? [],
        idx: 0,
        childXs: [],
      });
      continue;
    }

    const x =
      frame.childXs.length === 0
        ? (() => {
            const leafX = cursor;
            cursor += FT_NODE_W + FT_GAP_X;
            return leafX;
          })()
        : (frame.childXs[0] + frame.childXs[frame.childXs.length - 1]) / 2;
    const y = frame.depth * (FT_NODE_H + FT_GAP_Y);
    positions.set(frame.id, { x, y });

    stack.pop();
    const parent = stack[stack.length - 1];
    if (parent) parent.childXs.push(x);
  }

  const lines: FullTreeLine[] = [];
  for (const n of nodes) {
    if (n.introducerMemberId == null) continue;
    const parentPos = positions.get(n.introducerMemberId);
    const childPos = positions.get(n.memberId);
    if (!parentPos || !childPos) continue;
    lines.push({
      x1: parentPos.x + FT_NODE_W / 2,
      y1: parentPos.y + FT_NODE_H,
      x2: childPos.x + FT_NODE_W / 2,
      y2: childPos.y,
    });
  }

  return {
    positions,
    lines,
    width: Math.max(cursor, FT_NODE_W),
    height: (maxDepth + 1) * (FT_NODE_H + FT_GAP_Y),
  };
}
