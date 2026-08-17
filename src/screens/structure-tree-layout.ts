// Ported from the approved prototype's connectorLinesForParent
// (ui-prototype-v2.html) — trunk stem down from the parent, a horizontal bar
// spanning its children (widened to include the parent's own stem when the
// parent isn't centred over them, since each row centres independently),
// then a drop into each child. `tree-children-row` is flex-wrap:nowrap, so a
// parent's children are always a single row — the prototype's multi-row
// "channel" routing never triggers and isn't ported.
//
// Coordinates are plain canvas-local units (already zoom-divided by the
// caller) so a shared ancestor `transform: scale(zoom)` scales lines and
// nodes together instead of scaling lines twice.
export interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ConnectorLine {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

const TRUNK_OFFSET = 14;

export function computeParentChildConnectorLines(
  parentBox: Box,
  childBoxes: Box[],
): ConnectorLine[] {
  if (childBoxes.length === 0) return [];

  const parentCenterX = parentBox.x + parentBox.width / 2;
  const parentBottomY = parentBox.y + parentBox.height;
  const childTopY = childBoxes[0].y;
  const trunkY = childTopY - TRUNK_OFFSET;
  const childCenterXs = childBoxes.map((b) => b.x + b.width / 2);
  const barMinX = Math.min(parentCenterX, ...childCenterXs);
  const barMaxX = Math.max(parentCenterX, ...childCenterXs);

  const lines: ConnectorLine[] = [
    { x1: parentCenterX, y1: parentBottomY, x2: parentCenterX, y2: trunkY },
  ];
  if (barMaxX > barMinX) {
    lines.push({ x1: barMinX, y1: trunkY, x2: barMaxX, y2: trunkY });
  }
  childCenterXs.forEach((x) => {
    lines.push({ x1: x, y1: trunkY, x2: x, y2: childTopY });
  });
  return lines;
}
