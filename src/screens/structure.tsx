import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { EmptyState } from "@/components/empty-state";
import { LoadingState } from "@/components/loading-state";
import { PageHeader } from "@/components/page-header";
import { SearchResultsList } from "@/components/search-results-list";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { MonthSwitcher } from "@/components/month-switcher";
import { useToast } from "@/components/ui/toast";
import {
  StructureTreeNode,
  TreeConnectorLayer,
  TreeConnectorLine,
} from "@/components/structure-tree-node";
import { useMemberSearch } from "@/lib/use-member-search";
import { getDirectChildrenChart, getAncestorChain, type AncestorNode } from "@/lib/ipc/m4-search";
import { getPeriodLockStatus, type PeriodLockStatus } from "@/lib/ipc/m2-entries";
import type { ChartNode } from "@/lib/ipc/entities";
import { toErrorPresentation } from "@/lib/ipc/errors";
import { centsToDisplay } from "@/lib/utils";
import { FULL_TREE_GATE } from "@/windows/full-hierarchy-layout";
import { Breadcrumb, type BreadcrumbCrumb } from "@/components/breadcrumb";
import { useBackTarget, useRouteLabel } from "@/lib/navigation-history";
import {
  computeParentChildConnectorLines,
  type Box,
  type ConnectorLine,
} from "./structure-tree-layout";

const ZOOM_MIN = 0.5;
const ZOOM_MAX = 1.5;
const ZOOM_STEP = 0.1;

// US-M4.2 (§5.3). One branch open at a time — `openPath` is the single
// chain of expanded node ids from the screen's root down; opening a
// different node at an already-open depth replaces that branch rather than
// adding a second one.
export function Structure() {
  const { memberId: routeMemberId } = useParams<{ memberId?: string }>();
  const navigate = useNavigate();
  const requestedId = routeMemberId ? Number(routeMemberId) : undefined;

  const [rootNode, setRootNode] = useState<ChartNode | null>(null);
  const [levels, setLevels] = useState<Record<number, ChartNode[]>>({});
  const [openPath, setOpenPath] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [zoom, setZoom] = useState(1);
  const [query, setQuery] = useState("");
  const { results } = useMemberSearch(query);
  const toast = useToast();

  // T-M2.5-3: this figure screen defaults to the oldest recordable month,
  // switchable when more than one is outstanding (CR-2).
  const [lockStatus, setLockStatus] = useState<PeriodLockStatus | null>(null);
  const [selectedMonth, setSelectedMonth] = useState<string | null>(null);
  const viewMonth = selectedMonth ?? lockStatus?.recordablePeriodMonths[0];

  useEffect(() => {
    getPeriodLockStatus().then(setLockStatus);
  }, []);

  const [ancestorChain, setAncestorChain] = useState<AncestorNode[]>([]);
  useEffect(() => {
    if (!rootNode) return;
    let cancelled = false;
    getAncestorChain(rootNode.memberId).then((result) => {
      if (!cancelled) setAncestorChain(result.chain);
    });
    return () => {
      cancelled = true;
    };
  }, [rootNode?.memberId]);

  const backTarget = useBackTarget();
  useRouteLabel(rootNode ? `Structure (${rootNode.name})` : undefined);

  // US-M4.3 (§5.3a/Rule-45). The gate's count and the eventual draw are two
  // separate reads of the same cheap query (04-technical-architecture.md
  // §API-11) — this one just resolves the top member's descendant count.
  const [fullHierarchyBusy, setFullHierarchyBusy] = useState(false);
  const [gateCount, setGateCount] = useState<number | null>(null);

  // True only before the very first fetch resolves — re-navigating to a
  // different member updates in place rather than flashing back to loading.
  const loading = rootNode === null && error === null;

  const canvasRef = useRef<HTMLDivElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const nodeRefs = useRef<Map<number, HTMLDivElement>>(new Map());

  // Re-root the whole view when the route's member or the viewed month
  // changes — a month switch invalidates every cached generation, since
  // each one was fetched against the previous month's figures.
  useEffect(() => {
    let cancelled = false;
    getDirectChildrenChart({ memberId: requestedId, fullTree: false, periodMonth: viewMonth })
      .then((result) => {
        if (cancelled) return;
        // Backend contract: the requested (or resolved-root) member is
        // always the first node, its direct children follow.
        const root = result.nodes[0];
        setRootNode(root);
        setLevels({ [root.memberId]: result.nodes.slice(1) });
        // Expanded by default — root's own direct children are the base
        // view (FR-2); further generations are opened one at a time below.
        setOpenPath([root.memberId]);
        setError(null);
      })
      .catch((raw) => {
        if (!cancelled) setError(toErrorPresentation(raw).message);
      });
    return () => {
      cancelled = true;
    };
  }, [requestedId, viewMonth]);

  // Fetch children for whichever node just opened.
  useEffect(() => {
    const openId = openPath[openPath.length - 1];
    if (openId === undefined || levels[openId]) return;
    getDirectChildrenChart({ memberId: openId, fullTree: false, periodMonth: viewMonth })
      .then((result) => {
        setLevels((prev) => ({ ...prev, [openId]: result.nodes.slice(1) }));
      })
      .catch((raw) => setError(toErrorPresentation(raw).message));
  }, [openPath, levels, viewMonth]);

  // Root has no ancestor in this view (its own introducer, if any, was
  // never fetched), so it toggles as a special full collapse/expand rather
  // than going through the generic parent-lookup path every other node
  // uses.
  function toggleRoot() {
    if (!rootNode) return;
    setOpenPath((prev) => (prev.length === 0 ? [rootNode.memberId] : []));
  }

  function toggle(node: ChartNode) {
    setOpenPath((prev) => {
      const idx = prev.indexOf(node.memberId);
      if (idx !== -1) return prev.slice(0, idx);
      const parentIdx = prev.indexOf(node.introducerMemberId ?? -1);
      if (parentIdx === -1) return prev;
      return [...prev.slice(0, parentIdx + 1), node.memberId];
    });
  }

  const generations = useMemo(
    () => openPath.map((id) => levels[id]).filter((g): g is ChartNode[] => !!g),
    [openPath, levels],
  );

  // Connector geometry — measured after layout, same DOM-measurement
  // approach the prototype's own drawTreeConnectors() uses. Not the Full
  // Hierarchy Window's constraint (T-M4.3-4 forbids it there specifically,
  // for that screen's much larger node count).
  const [lines, setLines] = useState<ConnectorLine[]>([]);
  useLayoutEffect(() => {
    function recomputeLines() {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const canvasBox = canvas.getBoundingClientRect();
      // Rects here are post-transform screen pixels, but this SVG sits inside
      // the same `transform: scale(zoom)` canvas as the nodes it connects — so
      // dividing zoom back out here (matching the prototype's drawTreeConnectors)
      // keeps the ancestor's scale from being applied twice.
      const toLocalBox = (rect: DOMRect): Box => ({
        x: (rect.left - canvasBox.left) / zoom,
        y: (rect.top - canvasBox.top) / zoom,
        width: rect.width / zoom,
        height: rect.height / zoom,
      });
      const next: ConnectorLine[] = [];
      openPath.forEach((parentId) => {
        const children = levels[parentId];
        const parentEl = nodeRefs.current.get(parentId);
        if (!children || !parentEl) return;
        const childBoxes = children
          .map((child) => nodeRefs.current.get(child.memberId))
          .filter((el): el is HTMLDivElement => !!el)
          .map((el) => toLocalBox(el.getBoundingClientRect()));
        next.push(
          ...computeParentChildConnectorLines(toLocalBox(parentEl.getBoundingClientRect()), childBoxes),
        );
      });
      setLines(next);
    }
    recomputeLines();
    // Resizing/maximizing the window reflows the centred rows (node
    // positions shift) without touching any of this effect's own deps —
    // same gap the prototype's own resize listener closes for
    // drawTreeConnectors (ui-prototype-v2.html:1398).
    window.addEventListener("resize", recomputeLines);
    return () => window.removeEventListener("resize", recomputeLines);
  }, [rootNode, openPath, levels, zoom]);

  function fitWidth() {
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    if (!canvas || !wrap) return;
    const contentWidth = canvas.scrollWidth / zoom;
    const next = Math.max(ZOOM_MIN, Math.min(1, wrap.clientWidth / contentWidth));
    setZoom(Math.round(next * 100) / 100);
  }

  // T-M4.3-3: always the top member, regardless of what this screen is
  // currently rooted at — `memberId` omitted resolves server-side to the
  // one true root (get_direct_children_chart contract).
  function openFullHierarchyWindow() {
    const win = new WebviewWindow(`full-hierarchy-${Date.now()}`, {
      url: "/full-hierarchy",
      title: "Full hierarchy",
      width: 1280,
      height: 800,
      minWidth: 1024,
      minHeight: 720,
      resizable: true,
    });
    win.once("tauri://error", () => {
      toast.add({ title: "Couldn't open the full hierarchy window", type: "danger" });
    });
  }

  async function viewFullHierarchy() {
    setFullHierarchyBusy(true);
    try {
      const result = await getDirectChildrenChart({ fullTree: true });
      const descendantCount = result.nodes.length - 1;
      if (descendantCount > FULL_TREE_GATE) {
        setGateCount(descendantCount);
      } else {
        openFullHierarchyWindow();
      }
    } catch (raw) {
      toast.add({ title: toErrorPresentation(raw).message, type: "danger" });
    } finally {
      setFullHierarchyBusy(false);
    }
  }

  if (loading) return <LoadingState />;
  if (error || !rootNode) {
    return (
      <EmptyState
        title="No members yet"
        description={error ?? "Add the first (root) member to start building the structure."}
      />
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <PageHeader
        title="Structure"
        subtitle="Open one branch at a time"
        breadcrumb={
          <Breadcrumb
            backLabel={backTarget.hasHistory ? backTarget.label : undefined}
            onBack={backTarget.go}
            crumbs={ancestorChain.map(
              (a, i): BreadcrumbCrumb =>
                i === ancestorChain.length - 1
                  ? { label: a.name }
                  : { label: a.name, to: `/structure/${a.id}`, replace: true },
            )}
          />
        }
      />

      <div className="mt-4 flex flex-wrap items-center gap-3">
        <div className="relative min-w-70 flex-1">
          <Input
            placeholder="Search any member by name, 6-digit member number or phone"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          {query.trim() && (
            <div className="absolute z-10 mt-1 w-full">
              <SearchResultsList
                results={results}
                query={query}
                onSelect={(r) => {
                  setQuery("");
                  navigate(`/structure/${r.id}`);
                }}
              />
            </div>
          )}
        </div>
        <div className="ml-auto flex items-center gap-1.5">
          <Button
            variant="secondary"
            size="sm"
            disabled={zoom <= ZOOM_MIN + 1e-6}
            aria-label="Zoom out"
            onClick={() =>
              setZoom((z) => Math.max(ZOOM_MIN, Math.round((z - ZOOM_STEP) * 100) / 100))
            }
          >
            −
          </Button>
          <span className="num w-11 text-center text-caption">{Math.round(zoom * 100)}%</span>
          <Button
            variant="secondary"
            size="sm"
            disabled={zoom >= ZOOM_MAX - 1e-6}
            aria-label="Zoom in"
            onClick={() =>
              setZoom((z) => Math.min(ZOOM_MAX, Math.round((z + ZOOM_STEP) * 100) / 100))
            }
          >
            +
          </Button>
          <Button variant="secondary" size="sm" onClick={fitWidth}>
            Fit width
          </Button>
        </div>
        <Button
          variant="secondary"
          size="sm"
          disabled={openPath.length === 0}
          onClick={() => setOpenPath([])}
        >
          Collapse all
        </Button>
        <Button
          variant="secondary"
          size="sm"
          disabled={fullHierarchyBusy}
          onClick={viewFullHierarchy}
        >
          View full hierarchy
        </Button>
      </div>

      {lockStatus && (
        <MonthSwitcher
          months={lockStatus.recordablePeriodMonths}
          value={viewMonth ?? lockStatus.recordablePeriodMonths[0]}
          onChange={setSelectedMonth}
        />
      )}

      <div
        ref={wrapRef}
        className="mt-4 min-h-0 flex-1 overflow-auto rounded-lg border border-border bg-bg p-6"
      >
        <div
          ref={canvasRef}
          className="relative flex origin-top-left flex-col items-center gap-9"
          style={{ transform: `scale(${zoom})` }}
        >
          <TreeConnectorLayer>
            {lines.map((line, i) => (
              <TreeConnectorLine key={i} x1={line.x1} y1={line.y1} x2={line.x2} y2={line.y2} />
            ))}
          </TreeConnectorLayer>

          <div
            ref={(el) => {
              if (el) nodeRefs.current.set(rootNode.memberId, el);
            }}
          >
            <StructureTreeNode
              root
              name={rootNode.name}
              memberNumber={String(rootNode.memberId)}
              ownBusinessVolume={centsToDisplay(rootNode.ownBusinessVolume)}
              inactive={!rootNode.isActive}
              legCount={rootNode.legCount}
              open={openPath.length > 0}
              onOpenToggle={toggleRoot}
              onViewDetail={() => navigate(`/member/${rootNode.memberId}`)}
            />
          </div>

          {rootNode.legCount === 0 ? (
            <EmptyState
              title="No direct legs"
              description="This member has no one introduced beneath them."
            />
          ) : (
            generations.map((gen, i) => (
              <div key={i} className="flex gap-4.5">
                {gen.map((node) => (
                  <div
                    key={node.memberId}
                    ref={(el) => {
                      if (el) nodeRefs.current.set(node.memberId, el);
                    }}
                  >
                    <StructureTreeNode
                      name={node.name}
                      memberNumber={String(node.memberId)}
                      ownBusinessVolume={centsToDisplay(node.ownBusinessVolume)}
                      inactive={!node.isActive}
                      legCount={node.legCount}
                      open={openPath.includes(node.memberId)}
                      onOpenToggle={() => toggle(node)}
                      onViewDetail={() => navigate(`/member/${node.memberId}`)}
                    />
                  </div>
                ))}
              </div>
            ))
          )}
        </div>
      </div>

      <p className="text-caption mx-auto mt-3 max-w-3xl shrink-0 text-center">
        Click a member to open the legs beneath them; opening one closes the branch already open at
        that level. A wide level scrolls sideways — use the zoom to take more of it in. Each card
        shows name, member number and own Business Volume only. <strong>View full hierarchy</strong>{" "}
        opens the whole structure, every branch expanded, in a separate window.
      </p>

      <ConfirmDialog
        open={gateCount !== null}
        onOpenChange={(open) => !open && setGateCount(null)}
        title="Open the full hierarchy?"
        body={
          <>
            <p>
              This will draw {gateCount?.toLocaleString()} members in a new window. It may take a
              moment.
            </p>
            <p className="text-caption mt-2">
              The console stays usable while it draws. The new window is a picture of this moment
              and does not update.
            </p>
          </>
        }
        confirmLabel="Open"
        onConfirm={() => {
          setGateCount(null);
          openFullHierarchyWindow();
        }}
      />
    </div>
  );
}
