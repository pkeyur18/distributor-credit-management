import { useEffect, useMemo, useRef, useState } from "react";
import { Printer } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { EmptyState } from "@/components/empty-state";
import { LoadingState } from "@/components/loading-state";
import {
  StructureTreeNode,
  TreeConnectorLayer,
  TreeConnectorLine,
} from "@/components/structure-tree-node";
import { getDirectChildrenChart } from "@/lib/ipc/m4-search";
import type { ChartNode } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { centsToDisplay, cn } from "@/lib/utils";
import { computeFullTreeLayout } from "./full-hierarchy-layout";

const ZOOM_MIN = 0.1;
const ZOOM_MAX = 1.5;
const ZOOM_STEP = 0.1;

// US-M4.3 (§5.3a/§6.13). Its own top-level route, mounted outside
// AppShell/RequireAuth (see App.tsx) — a separate Tauri window (structure.tsx
// opens it via `new WebviewWindow`) is a genuinely separate webview, so this
// component shares no live state with the main console by construction.
// It fetches once on mount and never refetches (Rule-45: a point-in-time
// draw that does not update, does not poll, does not follow later theme or
// data changes).
export function FullHierarchy() {
  const [nodes, setNodes] = useState<ChartNode[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [zoom, setZoom] = useState(1);
  const [query, setQuery] = useState("");
  const [highlightId, setHighlightId] = useState<number | null>(null);

  const wrapRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLDivElement>(null);
  const nodeRefs = useRef<Map<number, HTMLDivElement>>(new Map());

  // "as at <date, time>" — captured once, at draw time, not re-derived on
  // every render (T-M4.3-6).
  const [stamp] = useState(() =>
    new Date().toLocaleString("en-GB", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }),
  );

  useEffect(() => {
    let cancelled = false;
    getDirectChildrenChart({ fullTree: true })
      .then((result) => {
        if (!cancelled) setNodes(result.nodes);
      })
      .catch((raw) => {
        if (!cancelled) setError(toErrorPresentation(raw).message);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const layout = useMemo(() => (nodes ? computeFullTreeLayout(nodes) : null), [nodes]);

  function fitWidth() {
    const wrap = wrapRef.current;
    if (!wrap || !layout) return;
    const next = Math.max(ZOOM_MIN, Math.min(1, (wrap.clientWidth - 48) / layout.width));
    setZoom(Math.round(next * 100) / 100);
  }

  function find(q: string) {
    setQuery(q);
    const trimmed = q.trim().toLowerCase();
    if (!trimmed || !nodes) {
      setHighlightId(null);
      return;
    }
    const hit = nodes.find(
      (n) => n.name.toLowerCase().includes(trimmed) || String(n.memberId).includes(trimmed),
    );
    setHighlightId(hit?.memberId ?? null);
    if (hit) {
      nodeRefs.current.get(hit.memberId)?.scrollIntoView({ block: "center", inline: "center" });
    }
  }

  if (error) {
    return <EmptyState title="Couldn't open the full hierarchy" description={error} />;
  }
  if (!nodes || !layout) return <LoadingState />;

  const root = nodes[0];

  return (
    <div className="min-h-screen bg-bg text-ink">
      <div className="sticky top-0 z-10 flex flex-wrap items-center gap-3 border-b border-border bg-surface px-8 py-3 print:hidden">
        <div>
          <h1 className="text-headline">Full hierarchy — {root.name}</h1>
          <p className="text-caption mt-0.5">
            {nodes.length.toLocaleString()} members · as at {stamp}
          </p>
        </div>
        <div className="ml-auto flex items-center gap-1.5">
          <Input
            placeholder="Find a member by name or number"
            value={query}
            onChange={(e) => find(e.target.value)}
            className="w-64"
          />
          <Button
            variant="secondary"
            size="sm"
            disabled={zoom <= ZOOM_MIN + 1e-6}
            aria-label="Zoom out"
            onClick={() => setZoom((z) => Math.max(ZOOM_MIN, Math.round((z - ZOOM_STEP) * 100) / 100))}
          >
            −
          </Button>
          <span className="num w-11 text-center text-caption">{Math.round(zoom * 100)}%</span>
          <Button
            variant="secondary"
            size="sm"
            disabled={zoom >= ZOOM_MAX - 1e-6}
            aria-label="Zoom in"
            onClick={() => setZoom((z) => Math.min(ZOOM_MAX, Math.round((z + ZOOM_STEP) * 100) / 100))}
          >
            +
          </Button>
          <Button variant="secondary" size="sm" onClick={fitWidth}>
            Fit width
          </Button>
          <Button variant="secondary" size="sm" onClick={() => window.print()}>
            <Printer className="size-3.5" />
            Print
          </Button>
        </div>
      </div>

      <div
        ref={wrapRef}
        className="overflow-auto p-6 print:overflow-visible print:p-0"
      >
        <div
          ref={canvasRef}
          className="relative origin-top-left"
          style={{ width: layout.width, height: layout.height, transform: `scale(${zoom})` }}
        >
          <TreeConnectorLayer>
            {layout.lines.map((line, i) => (
              <TreeConnectorLine key={i} x1={line.x1} y1={line.y1} x2={line.x2} y2={line.y2} />
            ))}
          </TreeConnectorLayer>

          {nodes.map((node) => {
            const pos = layout.positions.get(node.memberId);
            if (!pos) return null;
            return (
              <div
                key={node.memberId}
                ref={(el) => {
                  if (el) nodeRefs.current.set(node.memberId, el);
                }}
                className={cn(
                  "absolute",
                  highlightId === node.memberId && "rounded-lg outline-2 outline-offset-2 outline-accent",
                )}
                style={{ left: pos.x, top: pos.y }}
              >
                <StructureTreeNode
                  root={node.memberId === root.memberId}
                  interactive={false}
                  name={node.name}
                  memberNumber={String(node.memberId)}
                  ownBusinessVolume={centsToDisplay(node.ownBusinessVolume)}
                  inactive={!node.isActive}
                  legCount={node.legCount}
                />
              </div>
            );
          })}
        </div>

        {nodes.length === 1 && (
          <div className="mt-6">
            <EmptyState
              title="No direct legs"
              description="This member has no one introduced beneath them."
            />
          </div>
        )}
      </div>
    </div>
  );
}
