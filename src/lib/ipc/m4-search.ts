import { invokeCommand } from "./client";
import type { ChartNode, Member, SlabRow } from "./entities";
import type { ExportResult } from "./m6-reports";

export interface RewardBreakdown {
  ownReward: { ownBusinessVolume: number; ownSlabPct: number; amount: number };
  differentials: Array<{
    childId: number;
    childName: string;
    childTotalBusinessVolume: number;
    childSlabPct: number;
    ownSlabPct: number;
    differentialPct: number;
    amount: number;
  }>;
  royalty: { qualifyingChildren: number; ratePercent: number; amount: number } | null;
  rewardsTotal: number;
}

// One depth of direct legs for the Member Detail table (T-M4.1-4) — unlike
// the chart's `ChartNode`, this carries Total Business Volume: the
// "never TBV" rule is specific to the Structure/Full-Hierarchy tree node
// display, not to this screen's plain table.
export interface MemberDetailChild {
  memberId: number;
  name: string;
  totalBusinessVolume: number;
  slabPct: number;
  isActive: boolean;
}

export interface MemberDetail {
  member: Member;
  totalBusinessVolume: number;
  slabPct: number;
  legCount: number;
  rewards: RewardBreakdown;
  directChildren: MemberDetailChild[];
}

// API-10. `periodMonth`: T-M2.5-3's month switcher — omit to default to the
// oldest recordable period, never "whatever's newest."
export function getMemberDetail(memberId: number, periodMonth?: string): Promise<MemberDetail> {
  return invokeCommand("get_member_detail", { memberId, periodMonth });
}

export interface DirectChildrenChartRequest {
  /** Omit to resolve to the root member — there is always at most one. */
  memberId?: number;
  /** false: member + direct children (FR-2). true: entire subtree (FR-10). */
  fullTree: boolean;
  /** T-M2.5-3's month switcher — omit to default to the oldest recordable
   *  period, same default as `getMemberDetail`. */
  periodMonth?: string;
}

export interface DirectChildrenChartResult {
  /** The requested member first, then its descendants. */
  nodes: ChartNode[];
  /** Sprint 8's resolved gap: get_settings (S10) doesn't exist yet, so this
   *  is the only IPC path carrying the configured slab rows — needed by
   *  Home's slab-distribution charts to draw one bar per row. */
  slabTable: SlabRow[];
}

// API-11 — node shape is the same either way: name, ID, own Business
// Volume, active flag, introducer link, slab/rewards (the latter two for
// Home's aggregation, not for the tree node's own display).
export function getDirectChildrenChart(
  request: DirectChildrenChartRequest,
): Promise<DirectChildrenChartResult> {
  return invokeCommand("get_direct_children_chart", { ...request });
}

export interface AncestorNode {
  id: number;
  name: string;
}

export interface AncestorChainResult {
  /** Root-first, the requested member last. */
  chain: AncestorNode[];
}

// API-42 — the Structure screen's breadcrumb trail.
export function getAncestorChain(memberId: number): Promise<AncestorChainResult> {
  return invokeCommand("get_ancestor_chain", { memberId });
}

// API-46 (CR-6, M4.8) — reuses get_member_detail's data unchanged; no new
// calculation logic. `periodMonth`: same default-to-oldest-recordable
// behaviour as `getMemberDetail`.
export function exportMemberDetailPdf(
  memberId: number,
  periodMonth: string | undefined,
  outputPath: string,
): Promise<ExportResult> {
  return invokeCommand("export_member_detail_pdf", { memberId, periodMonth, outputPath });
}
